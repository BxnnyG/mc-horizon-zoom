// The mod's row in Options > Mods, with its key.
fn zoomify_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::zoomify as z;
    mods.push(crate::game::native_hooks::NativeMod {
        id: z::ID.into(),
        title: format!("{} {}", z::NAME, z::VERSION),
        description: z::DESCRIPTION.into(),
        keys: vec![["zoom".into(), "Zoom".into(), "key.keyboard.c".into()]],
    });
}
