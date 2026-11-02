// A click on the mod's settings page.
fn zoomify_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::zoomify as z;
    if mod_id == z::ID {
        z::click(game, button);
    }
}
