// Declare `/mekanismgenerators info|energy`.
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::mekanismgenerators_state::on_register_commands(commands);
}
