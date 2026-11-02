// The world's field of view, divided by Zoomify's zoom.
fn zoomify_fov(game: &crate::game::Game<'_>, value: f32) -> f32 {
    value / crate::game::zoomify::factor(game)
}
