//! Application adapters. Gameplay imports never point back into this module.
mod camera;
mod content;
mod controls;
mod scene;
mod ui;
mod web;

use crate::simulation::SimulationPlugin;
use bevy::prelude::*;

pub(super) fn run() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                #[cfg(target_arch = "wasm32")]
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kiva · The proving grounds".into(),
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
            }),
    )
    .add_plugins(SimulationPlugin)
    .insert_resource(ClearColor(Color::srgb_u8(12, 23, 25)))
    .init_resource::<camera::CameraRig>()
    .init_resource::<controls::PointerState>()
    .init_resource::<controls::ClientFeedback>()
    .add_systems(
        Startup,
        (content::setup, scene::setup, camera::setup).chain(),
    )
    // Input is sampled once before zero or more fixed steps, and enqueued.
    .add_systems(
        RunFixedMainLoop,
        (controls::collect_input, controls::patrol_units)
            .chain()
            .in_set(bevy::app::RunFixedMainLoopSystems::BeforeFixedMainLoop),
    )
    .add_systems(
        Update,
        (
            scene::attach_unit_visuals,
            scene::sync_units,
            scene::sync_effects,
            controls::receive_feedback,
        )
            .chain(),
    )
    .add_systems(
        PostUpdate,
        camera::update.before(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(
        PostUpdate,
        ui::publish.after(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(PostStartup, web::ready);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Startup, ui::native_hud)
        .add_systems(Update, ui::update_native_hud);
    app.run();
}
