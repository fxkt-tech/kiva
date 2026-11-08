//! Device input and presentation compose the headless demo.
mod camera;
pub(super) mod controls;
mod scene;
mod web;

use bevy::prelude::*;

pub(super) fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Kiva · Top-down / WASD + Mouse".into(),
            resolution: (1440, 900).into(),
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#game".into()),
            #[cfg(target_arch = "wasm32")]
            fit_canvas_to_parent: true,
            #[cfg(target_arch = "wasm32")]
            prevent_default_event_handling: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(crate::simulation::SimulationPlugin)
    .insert_resource(ClearColor(Color::srgb_u8(15, 22, 29)))
    .init_resource::<controls::ViewYaw>()
    .init_resource::<controls::CaptureState>()
    .add_systems(
        Startup,
        (
            crate::gameplay::setup,
            scene::setup,
            camera::setup,
            web::ready,
        )
            .chain(),
    )
    .add_systems(
        RunFixedMainLoop,
        controls::sample.in_set(bevy::app::RunFixedMainLoopSystems::BeforeFixedMainLoop),
    )
    .add_systems(Update, (scene::sync_player, camera::follow).chain());
    app.run();
}
