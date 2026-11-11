// The pure core of Mekanism: Generators slice 1: the Heat Generator's fuel,
// lava buffer, heat capacitor and FE buffer.
//
// This exact file is the standalone `horizon-mekanismgenerators` library and,
// injected by horizon-mixins, the `crate::game::mekanismgenerators` module of
// the staged game. It has no game types: the block entity glue passes in the
// ambient temperature and the lava beside the block, and reads back the stored
// FE, lava and heat.
//
// The numbers are Mekanism 10.7.19.85's compiled defaults (read from
// `GeneratorsConfig`/`GeneratorsStorageConfig` at tag `v1.21.1-10.7.19.85`):
// `heatGeneration` 200, `heatGenerationLava` 30, `heatGenerationNether` 100,
// `heatTankCapacity` 24 buckets, `heatGenerationFluidRate` 10 mB/t, the heat
// generator's storage 160,000 FE, and `TileEntityHeatGenerator`'s thermal
// constants (heat capacity 10, inverse conduction 5, inverse insulation 100,
// thermal efficiency 0.5). The heat withdrawn per tick is capped at
// `MAX_PRODUCTION` 510 (30 x 7 lava sides + 100 nether + 200): an upper bound
// the Carnot conversion keeps the generator below (a fuelled one settles near
// 112 FE/t, six lava sides near 96).

/// The FE buffer's capacity, in FE (`storage.heatGenerator`, 160,000).
pub const CAPACITY: i32 = 160_000;

/// The lava tank's capacity, in mB (`heatTankCapacity`, 24 buckets).
pub const LAVA_CAPACITY: i32 = 24_000;

/// Lava the generator burns a tick while it has room (`heatGenerationFluidRate`).
pub const LAVA_PER_TICK: i32 = 10;

/// Ticks of burn time that become one mB of lava (upstream's `burnTime / 20`).
pub const BURN_TICKS_PER_MB: u32 = 20;

/// The most FE one tick of heat may become (`heatGenerationLava` x 7 sides +
/// `heatGenerationNether` + `heatGeneration`).
pub const MAX_PRODUCTION: i32 = 510;

/// The most FE a side may take from the generator (`getMaxOutput` doubles it).
pub const MAX_OUTPUT: i32 = MAX_PRODUCTION * 2;

/// Heat one tick of burning lava adds (`heatGeneration` 200).
pub const HEAT_GENERATION: f64 = 200.0;

/// Heat a tick of heat each lava block beside the generator adds
/// (`heatGenerationLava` 30).
pub const LAVA_SIDE_BONUS: f64 = 30.0;

/// Heat a tick of heat an ultrawarm dimension adds (`heatGenerationNether` 100).
pub const NETHER_BONUS: f64 = 100.0;

/// The capacitor's heat capacity (`HEAT_CAPACITY` 10).
pub const HEAT_CAPACITY: f64 = 10.0;

/// The capacitor's inverse conduction coefficient (`INVERSE_CONDUCTION_COEFFICIENT`
/// 5).
pub const INVERSE_CONDUCTION: f64 = 5.0;

/// The capacitor's inverse insulation coefficient
/// (`INVERSE_INSULATION_COEFFICIENT` 100).
pub const INVERSE_INSULATION: f64 = 100.0;

/// The inverse insulation of the bottom face, which leaks like bare air
/// (`HeatAPI.DEFAULT_INVERSE_INSULATION` 0).
pub const DOWN_INVERSE_INSULATION: f64 = 0.0;

/// The heat transfer coefficient for air (`HeatAPI.AIR_INVERSE_COEFFICIENT`).
pub const AIR_INVERSE_COEFFICIENT: f64 = 10_000.0;

/// How much of the capacitor's heat becomes work (`THERMAL_EFFICIENCY` 0.5).
pub const THERMAL_EFFICIENCY: f64 = 0.5;

/// The ambient temperature this slice assumes (`HeatAPI.AMBIENT_TEMP`). The
/// biome-dependent temperature is a recorded gap.
pub const AMBIENT_TEMPERATURE: f64 = 300.0;

/// What the world offers one tick: the ambient temperature, how many of the six
/// neighbours are lava, and whether the dimension is ultrawarm (the Nether).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    /// The ambient temperature in kelvin.
    pub ambient_temperature: f64,
    /// How many of the six neighbouring blocks are lava (0 to 6).
    pub lava_sides: u8,
    /// Whether the dimension is ultrawarm (`DimensionType.ultraWarm`).
    pub ultrawarm: bool,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            ambient_temperature: AMBIENT_TEMPERATURE,
            lava_sides: 0,
            ultrawarm: false,
        }
    }
}

/// What one tick did: the FE that reached the buffer and whether it burned lava.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tick {
    /// FE added to the buffer this tick (upstream's production rate).
    pub produced: i32,
    /// Whether a tick of lava was burned into heat.
    pub burned_lava: bool,
}

/// The inverse insulation of each of the six faces; only the bottom leaks like
/// bare air, as `getInverseInsulation(Direction.DOWN)` makes it.
const FACE_INVERSE_INSULATION: [f64; 6] = [
    DOWN_INVERSE_INSULATION,
    INVERSE_INSULATION,
    INVERSE_INSULATION,
    INVERSE_INSULATION,
    INVERSE_INSULATION,
    INVERSE_INSULATION,
];

/// A Heat Generator: its fuel as lava, its heat capacitor and its FE buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatGenerator {
    /// Stored FE, the int `energy` in the block entity's saved compound.
    pub energy: i32,
    /// Stored lava in mB (upstream's `lavaTank`).
    pub lava: i32,
    /// The capacitor's stored heat, `temperature * HEAT_CAPACITY`.
    pub heat: f64,
}

impl Default for HeatGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl HeatGenerator {
    /// A generator with no fuel, no FE and an ambient capacitor.
    #[must_use]
    pub fn new() -> Self {
        Self::from_saved(0, 0, AMBIENT_TEMPERATURE * HEAT_CAPACITY)
    }

    /// A generator read back from a saved compound, clamped to valid values.
    #[must_use]
    pub fn from_saved(energy: i32, lava: i32, heat: f64) -> Self {
        let ambient_heat = AMBIENT_TEMPERATURE * HEAT_CAPACITY;
        Self {
            energy: energy.clamp(0, CAPACITY),
            lava: lava.clamp(0, LAVA_CAPACITY),
            heat: if heat.is_finite() {
                heat.max(ambient_heat)
            } else {
                ambient_heat
            },
        }
    }

    /// The capacitor's temperature in kelvin.
    #[must_use]
    pub fn temperature(&self) -> f64 {
        self.heat / HEAT_CAPACITY
    }

    /// Turn one fuel item's burn time into lava, as `FluidFuelInventorySlot`
    /// does: `burnTime / 20` mB, and only when the whole amount fits the tank.
    /// False when nothing was taken.
    pub fn accept_fuel(&mut self, burn_time_ticks: u32) -> bool {
        let lava = burn_time_ticks / BURN_TICKS_PER_MB;
        let room = LAVA_CAPACITY - self.lava;
        if lava == 0 || room <= 0 || lava > room as u32 {
            return false;
        }
        self.lava += lava as i32;
        true
    }

    /// One server tick, following `TileEntityHeatGenerator.onUpdateServer`:
    /// passive heat, a tick of lava burned while the buffer has room, the
    /// Carnot conversion of the heat lost, then the environment's leak.
    pub fn tick(&mut self, environment: Environment) -> Tick {
        let before = self.energy;
        let passive = LAVA_SIDE_BONUS * f64::from(environment.lava_sides)
            + if environment.ultrawarm {
                NETHER_BONUS
            } else {
                0.0
            };
        self.heat += passive;
        let burned_lava = self.energy < CAPACITY && self.lava >= LAVA_PER_TICK;
        if burned_lava {
            self.lava -= LAVA_PER_TICK;
            self.heat += HEAT_GENERATION;
        }
        let ambient = environment.ambient_temperature;
        let temperature = self.temperature();
        let heat_lost = THERMAL_EFFICIENCY * (temperature - ambient);
        self.heat -= heat_lost;
        let from_heat = (heat_lost.abs() * carnot_efficiency(temperature, ambient))
            .clamp(0.0, f64::from(MAX_PRODUCTION));
        let space = CAPACITY - self.energy;
        let produced = (from_heat as i32).min(space).max(0);
        self.energy += produced;
        self.leak(ambient);
        Tick {
            produced: self.energy - before,
            burned_lava,
        }
    }

    /// `ITileHeatHandler.simulateEnvironment`: each face leaks
    /// `(temperature - ambient) / (air + insulation + conduction)` kelvin of
    /// the capacitor's heat.
    fn leak(&mut self, ambient: f64) {
        let delta = self.temperature() - ambient;
        let mut lost = 0.0;
        for inverse_insulation in FACE_INVERSE_INSULATION {
            let inverse = AIR_INVERSE_COEFFICIENT + inverse_insulation + INVERSE_CONDUCTION;
            lost += delta / inverse * HEAT_CAPACITY;
        }
        self.heat -= lost;
    }
}

/// `1 - Qc / Qh` with the colder of the two temperatures below the hotter.
fn carnot_efficiency(temperature: f64, ambient: f64) -> f64 {
    let cold = temperature.min(ambient);
    let hot = temperature.max(ambient);
    if hot <= 0.0 { 0.0 } else { 1.0 - cold / hot }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One tick with no lava beside the block and a 300 K ambient.
    fn plain() -> Environment {
        Environment::default()
    }

    #[test]
    fn a_new_generator_is_empty_and_ambient() {
        let generator = HeatGenerator::new();
        assert_eq!(generator.energy, 0);
        assert_eq!(generator.lava, 0);
        assert_eq!(generator.temperature(), AMBIENT_TEMPERATURE);
        assert_eq!(generator.energy, 0, "a fresh generator stores no FE");
    }

    #[test]
    fn an_unfuelled_generator_produces_nothing() {
        let mut generator = HeatGenerator::new();
        for _ in 0..200 {
            let tick = generator.tick(plain());
            assert_eq!(tick.produced, 0, "no fuel and no lava heat makes no FE");
            assert!(!tick.burned_lava);
        }
        assert_eq!(generator.energy, 0);
        assert_eq!(generator.lava, 0);
        assert_eq!(generator.temperature(), AMBIENT_TEMPERATURE);
    }

    #[test]
    fn one_coal_becomes_eighty_millibuckets_of_lava() {
        let mut generator = HeatGenerator::new();
        assert!(generator.accept_fuel(1_600), "a coal's 1600 ticks fit");
        assert_eq!(generator.lava, 80);
        assert_eq!(generator.energy, 0, "loading fuel makes no FE");
    }

    #[test]
    fn a_lava_bucket_becomes_a_thousand_millibuckets() {
        let mut generator = HeatGenerator::new();
        assert!(generator.accept_fuel(20_000));
        assert_eq!(generator.lava, 1_000);
    }

    #[test]
    fn fuel_the_tank_cannot_hold_is_refused_whole() {
        let mut generator = HeatGenerator::new();
        assert!(generator.accept_fuel(20_000));
        assert!(generator.accept_fuel(20_000));
        assert_eq!(generator.lava, 2_000);
        // 2,000 + 23,000 = 25,000 mB would not fit in the 24,000 mB tank.
        assert!(!generator.accept_fuel(20 * 23_000 + 20));
        assert_eq!(generator.lava, 2_000, "a refused item adds nothing");
    }

    #[test]
    fn a_fuel_the_tank_can_just_hold_is_accepted() {
        let mut generator = HeatGenerator::new();
        generator.lava = LAVA_CAPACITY - 10;
        assert!(generator.accept_fuel(200), "200 ticks are exactly 10 mB");
        assert_eq!(generator.lava, LAVA_CAPACITY);
    }

    #[test]
    fn an_item_with_no_burn_time_is_not_fuel() {
        let mut generator = HeatGenerator::new();
        assert!(!generator.accept_fuel(0));
        assert_eq!(generator.lava, 0);
    }

    #[test]
    fn burning_lava_settles_at_112_fe_a_tick() {
        // 200 heat/t against the 0.5 Carnot conversion and the environment
        // leak settles the capacitor 395.3 K above ambient (695.3 K): the heat
        // lost is 197.6/t, Carnot efficiency 1 - 300/695.3 = 0.5685, so
        // 197.6 x 0.5685 = 112 FE/t.
        let mut generator = HeatGenerator::new();
        // Four buckets: 4,000 mB is 400 burning ticks at 10 mB/t.
        for _ in 0..4 {
            assert!(generator.accept_fuel(20_000));
        }
        let mut last = Tick::default();
        for _ in 0..300 {
            last = generator.tick(plain());
        }
        assert_eq!(last.produced, 112, "one burning tick settles at 112 FE");
        assert!(
            last.produced < MAX_PRODUCTION,
            "the 510 cap is an upper bound"
        );
        assert!(generator.energy > 25_000, "300 ticks make over 25,000 FE");
        assert_eq!(generator.lava, 1_000, "300 ticks burn exactly 3,000 mB");
    }

    #[test]
    fn one_burning_tick_uses_ten_millibuckets() {
        let mut generator = HeatGenerator::new();
        generator.accept_fuel(20_000);
        let tick = generator.tick(plain());
        assert!(tick.burned_lava);
        assert_eq!(generator.lava, 990);
    }

    #[test]
    fn a_full_buffer_burns_no_lava_and_makes_no_fe() {
        let mut generator = HeatGenerator::new();
        generator.energy = CAPACITY;
        generator.lava = 500;
        for _ in 0..100 {
            let tick = generator.tick(plain());
            assert_eq!(tick.produced, 0);
            assert!(!tick.burned_lava, "a full buffer does not burn fuel");
        }
        assert_eq!(generator.energy, CAPACITY);
        assert_eq!(generator.lava, 500, "the fuel is kept");
    }

    #[test]
    fn energy_never_exceeds_the_buffer() {
        let mut generator = HeatGenerator::new();
        for _ in 0..4 {
            assert!(generator.accept_fuel(20_000));
        }
        // Warm the capacitor up first: a cold generator produces nothing on
        // its first tick, so the space test needs a producing one.
        for _ in 0..200 {
            generator.tick(plain());
        }
        generator.energy = CAPACITY - 1;
        let tick = generator.tick(plain());
        assert_eq!(tick.produced, 1);
        assert_eq!(generator.energy, CAPACITY);
        assert_eq!(generator.tick(plain()).produced, 0);
        assert_eq!(generator.energy, CAPACITY);
    }

    #[test]
    fn lava_beside_the_block_heats_it_without_fuel() {
        let mut generator = HeatGenerator::new();
        let hot = Environment {
            lava_sides: 6,
            ..Environment::default()
        };
        let mut last = Tick::default();
        for _ in 0..300 {
            last = generator.tick(hot);
        }
        // 180 heat/t settles 355.8 K above ambient (655.8 K): 177.9 heat/t is
        // lost, Carnot efficiency 1 - 300/655.8 = 0.5425, so 96 FE/t.
        assert_eq!(last.produced, 96);
        assert!(last.produced < MAX_PRODUCTION);
        assert!(generator.energy > 25_000);
        assert_eq!(generator.lava, 0, "passive heat burns no fuel");
        assert!(generator.temperature() > AMBIENT_TEMPERATURE);
    }

    #[test]
    fn no_lava_beside_the_block_and_no_fuel_makes_no_heat() {
        let mut generator = HeatGenerator::new();
        for _ in 0..300 {
            generator.tick(plain());
        }
        assert_eq!(generator.energy, 0);
        assert_eq!(generator.temperature(), AMBIENT_TEMPERATURE);
    }

    #[test]
    fn the_nether_bonus_heats_the_generator_without_fuel() {
        let mut generator = HeatGenerator::new();
        let nether = Environment {
            ultrawarm: true,
            ..Environment::default()
        };
        let mut last = Tick::default();
        for _ in 0..300 {
            last = generator.tick(nether);
        }
        // 100 heat/t alone settles 197.7 K above ambient (497.7 K): 98.8
        // heat/t lost, Carnot efficiency 1 - 300/497.7 = 0.397, so 39 FE/t.
        assert_eq!(last.produced, 39);
        assert!(generator.energy > 10_000);
    }

    #[test]
    fn a_hot_generator_cools_down_without_input() {
        let mut generator = HeatGenerator::new();
        generator.accept_fuel(20_000);
        for _ in 0..100 {
            generator.tick(plain());
        }
        let hot = generator.temperature();
        let mut last = Tick::default();
        for _ in 0..2_000 {
            last = generator.tick(plain());
        }
        assert!(generator.temperature() < hot, "it cools");
        assert!(generator.temperature() < AMBIENT_TEMPERATURE + 0.1);
        assert_eq!(last.produced, 0, "a cold generator makes nothing");
    }

    #[test]
    fn producing_is_the_energy_delta() {
        let mut generator = HeatGenerator::new();
        generator.accept_fuel(20_000);
        for _ in 0..80 {
            let before = generator.energy;
            let tick = generator.tick(plain());
            assert_eq!(tick.produced, generator.energy - before);
        }
    }

    #[test]
    fn saved_state_is_clamped_and_read_back() {
        let generator = HeatGenerator::from_saved(1_234, 567, 3_015.5);
        assert_eq!(generator.energy, 1_234);
        assert_eq!(generator.lava, 567);
        assert_eq!(generator.temperature(), 301.55);
        let clamped = HeatGenerator::from_saved(CAPACITY + 1, LAVA_CAPACITY + 1, -5.0);
        assert_eq!(clamped.energy, CAPACITY);
        assert_eq!(clamped.lava, LAVA_CAPACITY);
        assert!(clamped.heat >= AMBIENT_TEMPERATURE * HEAT_CAPACITY);
    }

    #[test]
    fn temperature_is_stored_heat_over_the_heat_capacity() {
        let mut generator = HeatGenerator::new();
        generator.heat = 3_500.0;
        assert_eq!(generator.temperature(), 350.0);
    }

    #[test]
    fn more_lava_sides_make_more_heat() {
        let mut one = HeatGenerator::new();
        let mut six = HeatGenerator::new();
        for _ in 0..40 {
            one.tick(Environment {
                lava_sides: 1,
                ..Environment::default()
            });
            six.tick(Environment {
                lava_sides: 6,
                ..Environment::default()
            });
        }
        assert!(six.energy > one.energy, "six sides beat one");
        assert!(one.energy > 0, "one side already heats it");
    }
}
