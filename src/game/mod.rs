mod animation;
mod arena;
mod camera;
mod combat;
mod effects;
mod hud;
mod input;
mod player;
mod skeleton;
mod sound;

use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Look,
    Combat,
    Effects,
    Animation,
    Camera,
    Hud,
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_u8(24, 32, 39)))
            .init_resource::<input::Controls>()
            .init_resource::<camera::OrbitCamera>()
            .init_resource::<combat::Impacts>()
            .init_resource::<combat::SwordWaves>()
            .init_resource::<combat::CombatStatus>()
            .init_resource::<sound::Sounds>()
            .init_resource::<skeleton::AnimationLibrary>()
            .configure_sets(
                Update,
                (
                    GameSet::Input,
                    GameSet::Look,
                    GameSet::Combat,
                    GameSet::Effects,
                    GameSet::Animation,
                    GameSet::Camera,
                    GameSet::Hud,
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                (
                    arena::spawn_arena,
                    camera::spawn_camera,
                    player::spawn_fighters,
                    effects::setup_effects,
                    hud::spawn_hud,
                ),
            )
            .add_systems(Update, input::collect_input.in_set(GameSet::Input))
            .add_systems(Update, camera::rotate_camera.in_set(GameSet::Look))
            .add_systems(Update, combat::simulate.in_set(GameSet::Combat))
            .add_systems(
                Update,
                (
                    effects::impact_effects,
                    effects::update_effects,
                    effects::sword_trails,
                    effects::sword_waves,
                    sound::play_sounds,
                )
                    .chain()
                    .in_set(GameSet::Effects),
            )
            .add_systems(
                Update,
                animation::animate_fighters.in_set(GameSet::Animation),
            )
            .add_systems(Update, camera::follow_player.in_set(GameSet::Camera))
            .add_systems(
                Update,
                (hud::update_hud, effects::damage_numbers).in_set(GameSet::Hud),
            );
    }
}
