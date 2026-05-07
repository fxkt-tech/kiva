use bevy::prelude::*;

mod game;

use game::WerewolfGamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AI 狼人杀观战台".to_string(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WerewolfGamePlugin)
        .run();
}
