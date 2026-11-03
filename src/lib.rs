use bevy::prelude::*;

mod game;

pub fn run() {
    let mut app = App::new();
    app.add_plugins(
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
    .add_plugins(game::GamePlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_systems(PostStartup, notify_web_ready);

    app.run();
}

// Report readiness from the engine, independently of Trunk's generated loader.
#[cfg(target_arch = "wasm32")]
fn notify_web_ready() {
    if let Some(window) = web_sys::window() {
        let event = web_sys::CustomEvent::new("kiva-ready")
            .expect("creating the game readiness event should succeed");
        let _ = window.dispatch_event(&event);
    }
}
