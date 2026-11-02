// The wheel while zooming: scroll zoom, kept from the hotbar.
fn zoomify_wheel(game: &mut crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    if !*claimed && crate::game::zoomify::wheel(game, steps) {
        *claimed = true;
    }
}
