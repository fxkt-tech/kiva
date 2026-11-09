//! Device input and presentation compose the headless demo.
mod animation;
mod camera;
mod combat;
pub(super) mod controls;
mod hud;
mod scene;
mod skeleton;
mod stickman;
mod web;

use bevy::prelude::*;

/// Rendering owns these roots; their owners are simulation entities.
#[derive(Component, Clone, Copy)]
pub(super) struct ActorView {
    pub owner: Entity,
}

#[derive(Component)]
pub(super) struct PlayerView;

#[derive(Component)]
pub(super) struct PropView {
    pub owner: Entity,
}

fn display_alpha(time: &Time<Fixed>, active: &crate::simulation::SimulationActive) -> f32 {
    if active.0 {
        time.overstep_fraction()
    } else {
        1.0
    }
}

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
    .init_resource::<controls::AimDistance>()
    .init_resource::<skeleton::AnimationLibrary>()
    .add_systems(
        Startup,
        (
            crate::gameplay::setup,
            scene::setup,
            combat::setup,
            camera::setup,
            web::ready,
        )
            .chain(),
    )
    .add_systems(
        RunFixedMainLoop,
        controls::sample.in_set(bevy::app::RunFixedMainLoopSystems::BeforeFixedMainLoop),
    )
    .add_systems(
        Update,
        (
            scene::sync_player,
            scene::sync_props,
            combat::sync_enemies,
            combat::sync_health_bars,
            combat::sync_weapons,
            animation::animate,
            stickman::sync_attachments,
            combat::sync_markers,
            combat::effects,
            hud::sync,
            camera::follow,
        )
            .chain(),
    );
    app.run();
}
