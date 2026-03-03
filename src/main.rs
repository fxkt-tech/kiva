mod game;
mod net;
mod plugins;
mod states;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use plugins::KivaPlugins;
use states::WorldState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "四方诛杀 v0.2.0".into(),
                resolution: WindowResolution::new(1280, 720),
                resizable: false,
                decorations: true,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .init_state::<WorldState>()
        .add_systems(Startup, setup)
        .add_plugins(KivaPlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
