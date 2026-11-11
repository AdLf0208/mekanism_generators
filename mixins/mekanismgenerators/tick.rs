// One tick of the heat generator.
fn observe_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::mekanismgenerators_state::on_block_entity_tick(game, pos, entity);
}
