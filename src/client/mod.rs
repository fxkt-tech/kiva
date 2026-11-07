//! Application adapters. Gameplay imports never point back into this module.
mod abilities;
mod battlefield;
mod camera;
mod character;
mod content;
mod controls;
mod equipment;
mod match_view;
mod motion_view;
mod pointer;
mod presentation;
mod scene;
mod session;
#[cfg(test)]
mod test_support;
mod ui;
mod unit_appearance;
mod visibility;
mod web;
mod world_scene;

use crate::simulation::{SimulationControl, SimulationPlugin};
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
                    title: "Kiva · Single lane".into(),
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
    .add_plugins((
        SimulationPlugin,
        crate::gameplay::GameplayPlugin,
        crate::gameplay::AiPlugin,
    ))
    .insert_resource(SimulationControl {
        paused: cfg!(target_arch = "wasm32"),
    })
    .insert_resource(ClearColor(Color::srgb_u8(12, 23, 25)))
    .init_resource::<presentation::PresentationFrame>()
    .init_resource::<visibility::VisibleWorld>()
    .init_resource::<motion_view::MotionView>()
    .init_resource::<character::AnimationLibrary>()
    .init_resource::<camera::CameraRig>()
    .init_resource::<controls::PointerState>()
    .init_resource::<pointer::GamePointer>()
    .init_resource::<controls::ClientFeedback>()
    .init_resource::<controls::PlayerController>()
    .init_resource::<equipment::EquipmentAdapter>()
    .add_systems(
        Startup,
        (
            content::setup,
            session::setup,
            scene::setup,
            character::setup,
            unit_appearance::setup,
            battlefield::setup,
            camera::setup,
        )
            .chain(),
    )
    // Input is sampled once before zero or more fixed steps, and enqueued.
    .add_systems(
        RunFixedMainLoop,
        (
            session::process_requests,
            equipment::process_requests,
            controls::collect_input,
        )
            .chain()
            .in_set(bevy::app::RunFixedMainLoopSystems::BeforeFixedMainLoop),
    )
    .add_systems(
        Update,
        (
            visibility::refresh,
            presentation::prepare,
            motion_view::sample,
            scene::attach_unit_visuals,
            scene::sync_units,
            scene::sync_ground_bases,
            character::record_events,
            character::sync_poses,
            character::sync_attachments,
            unit_appearance::sync_simple_models,
            battlefield::attach,
            battlefield::sync,
            (scene::attach_projectiles, scene::sync_effects).chain(),
            controls::receive_feedback,
            controls::refresh_aim,
        )
            .chain(),
    )
    .add_systems(
        PostUpdate,
        camera::update.before(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(
        PostUpdate,
        ui::publish
            .after(bevy::transform::TransformSystems::Propagate)
            .after(bevy::camera::CameraUpdateSystems),
    )
    .add_systems(PostStartup, web::ready);
    #[cfg(target_arch = "wasm32")]
    app.add_systems(PostUpdate, ui::publish_labels.after(ui::publish));
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(
        Startup,
        (
            ui::native_hud,
            session::start_native
                .after(session::setup)
                .before(scene::setup),
        ),
    )
    .add_systems(Update, ui::update_native_hud);
    app.run();
}
