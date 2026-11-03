use bevy::prelude::*;

mod game;

pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Static hosts may return index.html for nonexistent .meta files.
                    #[cfg(target_arch = "wasm32")]
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Kiva · Sword Training".into(),
                        resolution: (1280, 800).into(),
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#game".into()),
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        #[cfg(target_arch = "wasm32")]
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(game::GamePlugin)
        .run();
}
