// Game-side glue for Mekanism: Generators slice 1 (the Heat Generator).
//
// Injected into `crate::game` as `mekanismgenerators_state`; it needs the game,
// so it is not part of the standalone library (only `runtime.rs` is).
//
// The FE buffer is the block entity's own (`native_block_entity_energy`, API
// 142): its amount is the int `energy` in the saved compound, so it survives a
// save and the block's tick reads it there. The lava tank and the heat
// capacitor are this port's own saved fields (`lava`, `heat`); upstream keeps
// them in a fluid tank and a heat capacitor. The fuel slot is a container
// (`native_block_entity_container`, API 16) that only accepts items the engine
// gives a burn time (the `minecraft:cooking_fuel` data map).
//
// Slice 1 stores the FE it makes and does not push it into neighbours: the
// brief stops at "stores energy". API 142's `native_move_energy` could feed a
// cable later (the mekanism and powah ports use it at this pin).

use horizon_client::containers::{ModBlockEntity, ModContainer, ModEnergyBuffer};
use horizon_client::items::ItemStack;
use horizon_core::geometry::Direction;
use horizon_world::nbt::{Compound, JavaString, Tag};

use crate::game::mekanismgenerators::{
    AMBIENT_TEMPERATURE, CAPACITY, Environment, HEAT_CAPACITY, HeatGenerator, LAVA_CAPACITY,
    MAX_OUTPUT,
};
use crate::game::native_server::{NativeCommand, NativeServerPlayer};

/// The slice's one block.
const GENERATOR: &str = "mekanismgenerators:heat_generator";

/// The fuel slot (upstream's `fuelSlot`; the energy slot is not ported).
const FUEL_SLOT: usize = 0;

/// The lava block a neighbour's fluid state names.
const LAVA_BLOCK: &str = "minecraft:lava";

/// The Nether, the pack's only ultrawarm dimension.
const NETHER: &str = "minecraft:the_nether";

/// A saved int of the block entity's compound, 0 when absent.
fn be_int(entity: &ModBlockEntity, key: &str) -> i32 {
    match entity.nbt.get(&JavaString::from(key)) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

fn set_be_int(entity: &mut ModBlockEntity, key: &str, value: i32) {
    entity
        .nbt
        .insert(JavaString::from(key), Tag::Int(value));
}

/// A saved double of the block entity's compound, `default` when absent.
fn be_double(entity: &ModBlockEntity, key: &str, default: f64) -> f64 {
    match entity.nbt.get(&JavaString::from(key)) {
        Some(Tag::Double(bits)) => f64::from_bits(*bits),
        _ => default,
    }
}

fn set_be_double(entity: &mut ModBlockEntity, key: &str, value: f64) {
    entity
        .nbt
        .insert(JavaString::from(key), Tag::Double(value.to_bits()));
}

/// The saved generator; a fresh entity starts empty and ambient.
fn generator_state(entity: &ModBlockEntity) -> HeatGenerator {
    HeatGenerator::from_saved(
        be_int(entity, "energy"),
        be_int(entity, "lava"),
        be_double(entity, "heat", AMBIENT_TEMPERATURE * HEAT_CAPACITY),
    )
}

fn store(entity: &mut ModBlockEntity, state: &HeatGenerator, produced: i32, burning: bool) {
    set_be_int(entity, "energy", state.energy);
    set_be_int(entity, "lava", state.lava);
    set_be_double(entity, "heat", state.heat);
    set_be_int(entity, "produced", produced);
    set_be_int(entity, "burning", i32::from(burning));
}

/// `BlockEntity.setChanged`: the chunk is written at the next save.
fn mark_modified(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
}

/// The engine's fuel value (`minecraft:cooking_fuel.burn_time`), 0 if none.
fn fuel_burn_time(game: &crate::game::Game<'_>, stack: &ItemStack) -> u32 {
    let providers = game.res.providers();
    let items = game.res.items();
    providers.burn_time(items, stack, false).max(0) as u32
}

/// `ItemStack.shrink` + `getCraftingRemainingItem`: one item is consumed and a
/// remainder (a lava bucket's bucket) takes its place when the stack runs out.
fn consume_fuel(game: &crate::game::Game<'_>, entity: &mut ModBlockEntity, stack: &ItemStack) {
    let remainder = game
        .res
        .items()
        .get(&stack.item)
        .and_then(|info| info.remainder.clone());
    let left = stack.count - 1;
    entity.items[FUEL_SLOT] = if left > 0 {
        let mut kept = stack.clone();
        kept.count = left;
        Some(kept)
    } else {
        remainder.map(|item| ItemStack::new(item.as_str(), 1))
    };
}

/// What the world offers this tick: how many neighbours are lava and whether
/// the dimension is ultrawarm. The ambient temperature is the pin's default;
/// the biome-dependent temperature is a recorded gap.
fn environment(game: &crate::game::Game<'_>, pos: [i32; 3]) -> Environment {
    let dimension = game.native_dimension();
    let lava_sides = match game.native_level(dimension) {
        Some(level) => u8::try_from(
            Direction::ALL
                .iter()
                .filter(|face| {
                    let [dx, dy, dz] = face.normal();
                    game.native_level_block(level, [pos[0] + dx, pos[1] + dy, pos[2] + dz])
                        .as_deref()
                        == Some(LAVA_BLOCK)
                })
                .count(),
        )
        .unwrap_or(0),
        None => 0,
    };
    Environment {
        ambient_temperature: AMBIENT_TEMPERATURE,
        lava_sides,
        ultrawarm: dimension == NETHER,
    }
}

/// The generator gets a block entity (`native_block_entity`, API 15).
#[allow(private_interfaces)]
pub fn on_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if block == GENERATOR {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

/// The generator ticks on the server (`native_block_entity_ticks`, API 15).
#[allow(private_interfaces)]
pub fn on_block_entity_ticks(block: &str, ticks: &mut bool) {
    if block == GENERATOR {
        *ticks = true;
    }
}

/// The fuel slot (`native_block_entity_container`, API 16). Hoppers may insert
/// fuel and take it back out; there is no menu in this slice.
#[allow(private_interfaces)]
pub fn on_block_entity_container(block: &str, container: &mut Option<ModContainer>) {
    if block != GENERATOR {
        return;
    }
    *container = Some(ModContainer {
        slots: 1,
        faces: std::array::from_fn(|_| vec![FUEL_SLOT]),
        inputs: vec![FUEL_SLOT],
        outputs: vec![FUEL_SLOT],
        menu: None,
    });
}

/// The FE buffer (`native_block_entity_energy`, API 142): 160,000 FE that a
/// side may drain at up to 1,020 FE/t, but nothing may insert into (upstream's
/// generator buffer is output-only).
#[allow(private_interfaces)]
pub fn on_block_entity_energy(block: &str, energy: &mut Option<ModEnergyBuffer>) {
    *energy = (block == GENERATOR).then_some(ModEnergyBuffer {
        capacity: CAPACITY,
        max_insert: 0,
        max_extract: MAX_OUTPUT,
        insert_faces: [false; 6],
        extract_faces: [true; 6],
    });
}

/// One server tick (`native_block_entity_tick`, API 17): load one fuel item
/// into the lava tank when it fits, then run the generator's tick.
#[allow(private_interfaces)]
pub fn on_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
) {
    if entity.id != GENERATOR {
        return;
    }
    if entity.items.len() <= FUEL_SLOT {
        entity.items.resize(FUEL_SLOT + 1, None);
    }
    let before = generator_state(entity);
    let mut state = before;
    let mut fuel_used = false;
    if let Some(stack) = entity.items[FUEL_SLOT].clone() {
        let burn = fuel_burn_time(game, &stack);
        if state.accept_fuel(burn) {
            consume_fuel(game, entity, &stack);
            fuel_used = true;
        }
    }
    let tick = state.tick(environment(game, pos));
    store(entity, &state, tick.produced, tick.burned_lava);
    if fuel_used || state != before {
        mark_modified(game, pos);
    }
}

/// What `/mekanismgenerators info` prints.
struct Report {
    id: String,
    energy: i32,
    lava: i32,
    temperature: f64,
    produced: i32,
    burning: bool,
    fuel: Option<ItemStack>,
}

/// `/mekanismgenerators info|energy` (`register_commands`, API 8).
#[allow(private_interfaces)]
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("mekanismgenerators info <pos:block_pos>"));
    commands.push(NativeCommand::op(
        "mekanismgenerators energy <amount:integer> <pos:block_pos>",
        2,
    ));
}

fn parse_pos(words: &[&str]) -> Option<[i32; 3]> {
    Some([
        words.first()?.parse().ok()?,
        words.get(1)?.parse().ok()?,
        words.get(2)?.parse().ok()?,
    ])
}

/// The `/mekanismgenerators` commands (`perform_command`, API 8). `info` prints
/// the generator and `energy` sets its stored FE; they are the harness controls
/// the in-game check uses, not upstream gameplay.
#[allow(private_interfaces)]
pub fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    match words.as_slice() {
        ["mekanismgenerators", "info", x, y, z] => {
            *ran = true;
            let Some(pos) = parse_pos(&[x, y, z]) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "Usage: /mekanismgenerators info <x> <y> <z>",
                ));
                return;
            };
            let Some(report) = game.native_with_mod_block_entity(pos, |entity| Report {
                id: entity.id.clone(),
                energy: be_int(entity, "energy"),
                lava: be_int(entity, "lava"),
                temperature: be_double(entity, "heat", AMBIENT_TEMPERATURE * HEAT_CAPACITY)
                    / HEAT_CAPACITY,
                produced: be_int(entity, "produced"),
                burning: be_int(entity, "burning") != 0,
                fuel: entity
                    .items
                    .get(FUEL_SLOT)
                    .and_then(Option::clone),
            }) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "No block entity there.",
                ));
                return;
            };
            let now = game.world.game_time;
            let Report {
                id,
                energy,
                lava,
                temperature,
                produced,
                burning,
                fuel,
            } = report;
            let fuel = match fuel {
                Some(stack) => format!("{} x{}", stack.item, stack.count),
                None => "empty".to_owned(),
            };
            game.native_feedback(horizon_client::text::Text::literal(format!(
                "{id} at {},{},{}: {energy}/{CAPACITY} FE, lava {lava}/{LAVA_CAPACITY} mB, temp {temperature:.1} K, producing {produced} FE/t, burning {burning}, fuel {fuel}, t={now}",
                pos[0], pos[1], pos[2],
            )));
        }
        ["mekanismgenerators", "energy", amount, x, y, z] => {
            *ran = true;
            let Some(pos) = parse_pos(&[x, y, z]) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "Usage: /mekanismgenerators energy <amount> <x> <y> <z>",
                ));
                return;
            };
            let Ok(amount) = amount.parse::<i32>() else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "The amount must be a whole number of FE.",
                ));
                return;
            };
            let Some(after) = game.native_with_mod_block_entity(pos, |entity| {
                let energy = amount.clamp(0, CAPACITY);
                set_be_int(entity, "energy", energy);
                energy
            }) else {
                game.native_feedback(horizon_client::text::Text::literal(
                    "No block entity there.",
                ));
                return;
            };
            game.native_feedback(horizon_client::text::Text::literal(format!(
                "Energy set to {after} FE."
            )));
        }
        _ => {}
    }
}
