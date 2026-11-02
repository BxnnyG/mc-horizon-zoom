// Mouse turning while zoomed: slower, as Zoomify's relative sensitivity.
fn zoomify_turn(game: &crate::game::Game<'_>, value: f64) -> f64 {
    value * crate::game::zoomify::turn(game)
}
