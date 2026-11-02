// The zoom key went down or up.
fn zoomify_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::zoomify::key(game, key, pressed);
}
