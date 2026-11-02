// The mod's settings page.
fn zoomify_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::zoomify as z;
    if mod_id == z::ID {
        buttons.extend(z::labels(game));
    }
}
