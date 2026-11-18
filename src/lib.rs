//! Mekanism: Generators for Horizon — slice 1, the Heat Generator.
//!
//! One flat SDK package (mod API 0.2): the carried block, its item, models,
//! textures, lang, loot table, recipe and creative tab are the `assets/` and
//! `data/` here, and the behaviour is the one `impl Hooks` below. The rules
//! themselves live in [`runtime`], byte-identical to the retired `.hnmod`
//! bundle's `runtime.rs`; [`heat_generator`] is the glue.
//!
//! What the retired bundle expressed as seven injected helpers is here three
//! shapes: the block's block entity is a **Part** (`heat_generator::
//! HeatGeneratorPart`, attached to the type declared for the block), its
//! container and its energy buffer are **declarations** on the block key, and
//! the command is the typed `register_commands`/`run_mod_command` pair. Its
//! embedded-content helper has no successor: one `.hmod` carries the content and
//! the code, which is what that helper existed to work around.
use mod_api::{
    self, Command, CommandArgumentValue, Flow, Hooks, RegisterCommandsEvent, Registrar,
    RegistrationContext, RunModCommandEvent, ServerHost, Text,
};

mod heat_generator;

pub mod runtime;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the generator: one block with a block entity, one fuel slot and an
    /// output-only FE buffer.
    ///
    /// The retired bundle wrote the buffer into the `block_entity_energy` hook's
    /// shared output and had to be careful to leave a foreign block's entry
    /// alone (the PR #674 review's [P1]). A declaration is keyed by the block, so
    /// that hazard cannot recur: another mod's block is another key.
    fn register(&self, registrar: &mut dyn Registrar, context: &RegistrationContext) {
        let block = registrar.block_key(heat_generator::GENERATOR);
        registrar.block_container(&block, heat_generator::container());
        registrar.block_energy(&block, heat_generator::battery());
        let kind = heat_generator::declare(registrar);
        if let Err(error) = registrar.block_entity_part(&kind, heat_generator::HeatGeneratorPart) {
            context.refuse(&error.to_string());
        }
    }

    /// `/mekanismgenerators info|energy`: the harness controls the in-game check
    /// uses, since the slice has no GUI.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event
            .commands
            .push(Command::new("mekanismgenerators info <pos:block_pos>"));
        event.commands.push(Command::op(
            "mekanismgenerators energy <amount:integer> <pos:block_pos>",
            2,
        ));
    }

    /// The command bodies. `info` prints the generator, `energy` sets its stored
    /// FE; the arguments are parsed and resolved by the server's dispatcher, so
    /// this only reads the block entity and answers.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let literals: Vec<&str> = event.command.literals.iter().map(String::as_str).collect();
        let Some(CommandArgumentValue::BlockPos(pos)) = event.command.argument("pos") else {
            return Flow::Continue;
        };
        let line = match literals.as_slice() {
            ["mekanismgenerators", "info"] => heat_generator::info_line(host, *pos)
                .unwrap_or_else(|| "No block entity there.".to_owned()),
            ["mekanismgenerators", "energy"] => {
                let Some(CommandArgumentValue::Integer(amount)) = event.command.argument("amount")
                else {
                    return Flow::Continue;
                };
                match heat_generator::set_energy(host, *pos, *amount) {
                    Some(after) => format!("Energy set to {after} FE."),
                    None => "No block entity there.".to_owned(),
                }
            }
            _ => return Flow::Continue,
        };
        let _ = host.commands().feedback(Text::literal(line));
        Flow::Cancel
    }
}
