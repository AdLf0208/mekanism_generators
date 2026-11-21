//! The Heat Generator's glue: the block entity Part, the block's container and
//! energy declarations, the saved state and the `/mekanismgenerators` command
//! bodies.
//!
//! Every rule lives in [`crate::runtime`] (the retired bundle's `runtime.rs`,
//! byte-identical); this file reads the game, calls it and writes the result
//! back. It has no rules of its own, so its tests pin what it *declares* (one
//! fuel slot, an output-only buffer), that the state written to the saved
//! compound reads back, and the exact line `/mekanismgenerators info` prints.
use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityTickEvent, BlockEntityTickReply,
    BlockEntityTicksEvent, BlockPart, Compound, Direction, ENERGY, Host, HostTransaction,
    ItemStack, LevelRef, LevelSide, ModContainer, ModEnergyBuffer, ServerHost, Tag,
};

use crate::runtime::{
    AMBIENT_TEMPERATURE, CAPACITY, Environment, HEAT_CAPACITY, HeatGenerator, LAVA_CAPACITY,
    MAX_OUTPUT,
};

/// The slice's one block; the block entity type declared for it carries the
/// same name.
pub(crate) const GENERATOR: &str = "mekanismgenerators:heat_generator";

/// The block entity type's path, in this mod's namespace: `/mekanismgenerators`
/// has no bearing on it.
const GENERATOR_TYPE: &str = "heat_generator";

/// The lava block a neighbour's block key names.
const LAVA_BLOCK: &str = "minecraft:lava";

/// The Nether, the pack's only ultrawarm dimension. The pin has no
/// `DimensionType.ultraWarm` read, so the dimension id is the test (the retired
/// bundle did the same).
const NETHER: &str = "minecraft:the_nether";

/// The fuel slot (upstream's `fuelSlot`; the energy slot is not ported).
const FUEL_SLOT: usize = 0;

/// How many slots the generator's container has.
const SLOTS: usize = 1;

/// The block entity type declared for the generator, built in `Hooks::register`.
pub(crate) fn declare(registrar: &mut dyn mod_api::Registrar) -> mod_api::BlockEntityTypeKey {
    registrar
        .block_entity_type(GENERATOR_TYPE)
        .blocks(&[GENERATOR_TYPE])
        .build()
}

/// The generator's container: one fuel slot every face reaches, no menu
/// (`native_block_entity_container`'s `ModContainer` at the retired pin).
pub(crate) fn container() -> ModContainer {
    ModContainer {
        slots: SLOTS,
        faces: std::array::from_fn(|_| vec![FUEL_SLOT]),
        inputs: vec![FUEL_SLOT],
        outputs: vec![FUEL_SLOT],
        menu: None,
    }
}

/// The generator's FE buffer: 160,000 FE that a side may drain at 1,020 FE/t,
/// and nothing may fill (upstream's buffer is output-only).
pub(crate) fn battery() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: CAPACITY,
        max_insert: 0,
        max_extract: MAX_OUTPUT,
        insert_faces: [false; 6],
        extract_faces: [true; 6],
    }
}

/// The generator saved in a block entity's compound; a fresh one is empty and
/// ambient.
pub(crate) fn read(saved: &Tag) -> HeatGenerator {
    HeatGenerator::from_saved(
        int(saved, "energy"),
        int(saved, "lava"),
        double(saved, "heat", AMBIENT_TEMPERATURE * HEAT_CAPACITY),
    )
}

/// Write the tick's result back: the buffer's amount (`energy`, the int the
/// energy capability reads), the lava, the capacitor's heat, and the two fields
/// `/mekanismgenerators info` reports (`produced`, `burning`).
pub(crate) fn write(saved: &mut Tag, state: &HeatGenerator, produced: i32, burning: bool) {
    let Tag::Compound(nbt) = saved else {
        return;
    };
    nbt.insert("energy".into(), Tag::Int(state.energy));
    nbt.insert("lava".into(), Tag::Int(state.lava));
    nbt.insert("heat".into(), Tag::Double(state.heat.to_bits()));
    nbt.insert("produced".into(), Tag::Int(produced));
    nbt.insert("burning".into(), Tag::Int(i32::from(burning)));
}

/// A saved int, 0 when absent or of another type.
fn int(saved: &Tag, key: &str) -> i32 {
    match saved.get(key) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

/// A saved double (`Tag::Double` holds IEEE bits), `default` when absent.
fn double(saved: &Tag, key: &str, default: f64) -> f64 {
    match saved.get(key) {
        Some(Tag::Double(bits)) => f64::from_bits(*bits),
        _ => default,
    }
}

/// What `/mekanismgenerators info` prints about one block entity.
pub(crate) struct Sight {
    pub(crate) id: String,
    pub(crate) pos: [i32; 3],
    pub(crate) energy: i32,
    pub(crate) lava: i32,
    pub(crate) temperature: f64,
    pub(crate) produced: i32,
    pub(crate) burning: bool,
    /// The block state's `facing`: the side the generator emits from.
    pub(crate) facing: Option<String>,
    pub(crate) fuel: Option<(String, u32)>,
}

impl Sight {
    /// The report line, with the world time it was taken at: `t=` is what the
    /// in-game check uses to prove a reply is fresh.
    pub(crate) fn line(&self, t: i64) -> String {
        let fuel = match &self.fuel {
            Some((item, count)) => format!("{item} x{count}"),
            None => "empty".to_owned(),
        };
        let facing = self.facing.as_deref().unwrap_or("unknown");
        format!(
            "{} at {},{},{}: {}/{} FE, lava {}/{} mB, temp {:.1} K, producing {} FE/t, burning {}, \
             facing {facing}, fuel {fuel}, t={t}",
            self.id,
            self.pos[0],
            self.pos[1],
            self.pos[2],
            self.energy,
            CAPACITY,
            self.lava,
            LAVA_CAPACITY,
            self.temperature,
            self.produced,
            self.burning,
        )
    }
}

/// `/mekanismgenerators info <pos>`: the report line, or `None` when no heat
/// generator stands there. The block id is the test, as the retired bundle's
/// `native_with_mod_block_entity` only ever saw this mod's block entities: a
/// chest at that position is "No block entity there.", not a generator at 0 FE.
pub(crate) fn info_line(host: &mut dyn ServerHost, pos: [i32; 3]) -> Option<String> {
    let level = host.level(host.dimension())?;
    let id = level.block(&*host, pos)?.id(&*host);
    if id != GENERATOR {
        return None;
    }
    let entity = host.block_entity(level, pos)?;
    let saved = Tag::Compound(entity.data(&*host)?);
    let fuel = entity
        .item_slot(&*host, FUEL_SLOT as u32)
        .ok()
        .flatten()
        .map(|stack| (stack.item().id(&*host), stack.count()));
    let sight = Sight {
        id,
        pos,
        energy: int(&saved, "energy"),
        lava: int(&saved, "lava"),
        temperature: double(&saved, "heat", AMBIENT_TEMPERATURE * HEAT_CAPACITY) / HEAT_CAPACITY,
        produced: int(&saved, "produced"),
        burning: int(&saved, "burning") != 0,
        facing: level.block_property(&*host, pos, "facing"),
        fuel,
    };
    Some(sight.line(level.game_time(&*host).unwrap_or(0)))
}

/// `/mekanismgenerators energy <amount> <pos>`: set the buffer's amount, the
/// harness control the in-game check uses. Answers with the amount it stored,
/// or `None` when no heat generator stands there (a foreign block entity is left
/// completely alone: writing `energy` into a chest's saved data would be a
/// foreign key in someone else's compound).
pub(crate) fn set_energy(host: &mut dyn ServerHost, pos: [i32; 3], amount: i32) -> Option<i32> {
    let level = host.level(host.dimension())?;
    if level.block(&*host, pos)?.id(&*host) != GENERATOR {
        return None;
    }
    let entity = host.block_entity(level, pos)?;
    let energy = amount.clamp(0, CAPACITY);
    let mut patch = Compound::new();
    patch.insert("energy".into(), Tag::Int(energy));
    // `merge_data` is `/data merge block`: it loads the patch and marks the chunk
    // changed, which is the `setChanged` the retired bundle had to fake.
    entity.merge_data(host, patch).ok().map(|_| energy)
}

/// What the world offers one tick: the ambient temperature, how many of the six
/// neighbours are lava, and whether the dimension is ultrawarm. The
/// biome-dependent ambient temperature is a recorded gap.
fn environment(host: &dyn Host, level: Option<LevelRef>, pos: [i32; 3]) -> Environment {
    let lava = host.block_key(LAVA_BLOCK);
    let lava_sides = level.map_or(0, |level| {
        u8::try_from(
            Direction::ALL
                .iter()
                .filter(|face| {
                    let [dx, dy, dz] = face.normal();
                    let at = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
                    lava.is_some_and(|lava| level.block(host, at) == Some(lava))
                })
                .count(),
        )
        .unwrap_or(0)
    });
    Environment {
        ambient_temperature: AMBIENT_TEMPERATURE,
        lava_sides,
        ultrawarm: host.dimension() == NETHER,
    }
}

/// The engine's fuel value (`minecraft:cooking_fuel.burn_time`), 0 if none.
fn burn_time(host: &dyn Host, stack: &ItemStack) -> u32 {
    host.item_burn_time(stack.item()).unwrap_or(0).max(0) as u32
}

/// `ItemStack.shrink(1)` + `getCraftingRemainingItem`: one item is consumed and a
/// remainder (a lava bucket's bucket) takes its place when the stack runs out.
fn consume(host: &dyn Host, stack: &ItemStack) -> Option<ItemStack> {
    stack
        .clone()
        .shrink(1)
        .or_else(|| host.item_crafting_remainder(stack))
}

/// The face a block state's `facing` names: the carried barrel template's
/// property, which is the direction the generator faces and therefore the side
/// `TileEntityGenerator` emits from (`ENERGY_SIDES = {RelativeSide.FRONT}`).
fn facing_of(name: &str) -> Option<Direction> {
    match name {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}

/// The side this generator emits from: its block state's `facing`, or `None`
/// when the state names no direction this can read (nothing is pushed then).
fn emitting_face(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<Direction> {
    facing_of(level.block_property(host, pos, "facing")?.as_str())
}

/// Push the buffer into the block in front, as `TileEntityGenerator`'s
/// `CableUtils.emit` does on its one `ENERGY_SIDES` front side: at most
/// `MAX_OUTPUT` FE a tick, only what the front block's own handler accepts, and
/// only out of this generator's buffer.
///
/// The buffer lives in the compound this tick holds, so the move is measured by
/// the acceptor's reply instead of extracting through our own capability: that
/// would read the state the tick has already taken out of the block entity.
/// Answers the FE that left.
fn emit_front(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    state: &mut HeatGenerator,
) -> i32 {
    let Some(face) = emitting_face(&*host, level, pos) else {
        return 0;
    };
    let [dx, dy, dz] = face.normal();
    let at = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
    let Some(target) = level.capability(&mut *host, &ENERGY, at, Some(face.opposite())) else {
        return 0;
    };
    let stored = to_i32(target.amount(&mut *host));
    let room = to_i32(target.capacity(&mut *host)).saturating_sub(stored);
    let offer = state.output_offer(room, MAX_OUTPUT);
    if offer == 0 {
        return 0;
    }
    // One transaction, so an acceptor that over-answers moves nothing at all.
    let accepted = host
        .transaction(|host, tx| Ok::<u32, ()>(target.insert(host, tx, to_u32(offer))))
        .map_or(0, to_i32);
    state.emit(accepted)
}

/// A capability's amount as the runtime's `i32` (it never exceeds a buffer's
/// capacity, which is an `i32` here).
fn to_i32(amount: u32) -> i32 {
    i32::try_from(amount).unwrap_or(i32::MAX)
}

/// A wanted amount as the capability's `u32` (positive, by every caller).
fn to_u32(amount: i32) -> u32 {
    u32::try_from(amount.max(0)).unwrap_or(0)
}

/// The generator's behaviour: a block entity, ticked on the server.
pub(crate) struct HeatGeneratorPart;

#[mod_api::horizon_part]
impl BlockPart for HeatGeneratorPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for HeatGeneratorPart {
    /// `EntityBlock.newBlockEntity`: an empty saved compound; the tick fills it.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        Some(Compound::new())
    }

    /// `EntityBlock.getTicker`: it ticks. Never clear a value an earlier mod set.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: load one fuel item into the lava tank when it fits, then
    /// run the generator's tick. The payload is left `Changed` only when
    /// something moved, so an idle generator does not dirty its chunk every tick
    /// (the retired bundle's `mark_modified` condition).
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let pos = event.pos;
        let resized = event.entity.items.len() != SLOTS;
        if resized {
            event.entity.items.resize(SLOTS, None);
        }
        let mut saved = Tag::Compound(std::mem::take(&mut event.entity.nbt));
        let before = read(&saved);
        let mut state = before;
        let mut fuel_used = false;
        if let Some(stack) = event.entity.items[FUEL_SLOT].clone()
            && state.accept_fuel(burn_time(host, &stack))
        {
            event.entity.items[FUEL_SLOT] = consume(host, &stack);
            fuel_used = true;
        }
        let level = host.level(host.dimension());
        let tick = state.tick(environment(host, level, pos));
        // Upstream emits after the generator's own update, into the front side.
        if let Some(level) = level {
            emit_front(host, level, pos, &mut state);
        }
        write(&mut saved, &state, tick.produced, tick.burned_lava);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        if !resized && !fuel_used && state == before {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fuel_slot_is_the_only_slot_and_every_face_reaches_it() {
        let container = container();
        assert_eq!(container.slots, SLOTS, "one fuel slot");
        assert_eq!(container.inputs, vec![FUEL_SLOT], "fuel may be inserted");
        assert_eq!(container.outputs, vec![FUEL_SLOT], "and taken back out");
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| vec![FUEL_SLOT]),
            "automation reaches the slot from every face: there is no side configuration"
        );
        assert_eq!(container.menu, None, "no GUI in this slice");
    }

    #[test]
    fn the_generator_declares_its_output_only_buffer() {
        let buffer = battery();
        assert_eq!(buffer.capacity, CAPACITY, "the generator's own capacity");
        assert_eq!(buffer.max_insert, 0, "nothing may fill it: output only");
        assert_eq!(
            buffer.max_extract, MAX_OUTPUT,
            "it drains at its output rate"
        );
        assert_eq!(buffer.insert_faces, [false; 6]);
        assert_eq!(buffer.extract_faces, [true; 6]);
    }

    #[test]
    fn a_fresh_compound_is_an_empty_ambient_generator() {
        assert_eq!(read(&Tag::Compound(Compound::new())), HeatGenerator::new());
    }

    #[test]
    fn the_tick_s_state_reads_back_from_the_compound_it_wrote() {
        let mut saved = Tag::Compound(Compound::new());
        let state = HeatGenerator::from_saved(49_152, 10_730, 6_755.0);
        write(&mut saved, &state, 112, true);
        assert_eq!(read(&saved), state, "energy, lava and heat round-trip");
        assert_eq!(
            saved.get("energy"),
            Some(&Tag::Int(49_152)),
            "the buffer is the int `energy` the energy capability reads"
        );
        assert_eq!(saved.get("produced"), Some(&Tag::Int(112)));
        assert_eq!(saved.get("burning"), Some(&Tag::Int(1)));
        assert_eq!(
            read(&saved).temperature(),
            675.5,
            "the heat is a double of the capacitor's stored heat"
        );
    }

    #[test]
    fn the_info_line_is_the_one_the_e2e_asserts() {
        let sight = Sight {
            id: GENERATOR.to_owned(),
            pos: [0, 100, 0],
            energy: 49_152,
            lava: 10_730,
            temperature: 675.46,
            produced: 112,
            burning: true,
            facing: Some("north".to_owned()),
            fuel: None,
        };
        assert_eq!(
            sight.line(768),
            "mekanismgenerators:heat_generator at 0,100,0: 49152/160000 FE, lava 10730/24000 mB, \
             temp 675.5 K, producing 112 FE/t, burning true, facing north, fuel empty, t=768"
        );
        let with_fuel = Sight {
            fuel: Some(("minecraft:coal".to_owned(), 55)),
            ..sight
        };
        assert!(
            with_fuel
                .line(459)
                .ends_with(", burning true, facing north, fuel minecraft:coal x55, t=459"),
            "a loaded fuel slot prints its item and count"
        );
    }

    #[test]
    fn the_emitting_face_is_the_one_the_block_state_names() {
        assert_eq!(facing_of("north"), Some(Direction::North));
        assert_eq!(facing_of("south"), Some(Direction::South));
        assert_eq!(facing_of("west"), Some(Direction::West));
        assert_eq!(facing_of("east"), Some(Direction::East));
        assert_eq!(facing_of("up"), Some(Direction::Up));
        assert_eq!(facing_of("down"), Some(Direction::Down));
        assert_eq!(facing_of("sideways"), None, "an unknown name emits nowhere");
        assert_eq!(facing_of(""), None);
        assert_eq!(
            Direction::North.normal(),
            [0, 0, -1],
            "north is -z: the e2e's cube goes there"
        );
    }
}
