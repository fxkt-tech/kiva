use bevy::prelude::*;

use kiva::game;

use game::WerewolfGamePlugin;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        game::player_config::load_web_ai_config().await;
        run();
    });
}

fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    #[cfg(target_arch = "wasm32")]
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "AI 狼人杀观战台".to_string(),
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#kiva".into()),
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        #[cfg(target_arch = "wasm32")]
                        prevent_default_event_handling: true,
                        resolution: (1280, 800).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(WerewolfGamePlugin)
        .run();
}
