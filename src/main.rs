mod plugins;

use bevy::prelude::*;
use plugins::KivaPlugins;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum WorldState {
    #[default]
    Menu, // 主菜单
    Game, // 21点游戏
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "四方诛杀 v0.1.0".into(),
                resolution: (1280., 720.).into(),
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

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        println!("despawn: {:?}", entity);
        commands.entity(entity).despawn();
    }
}
