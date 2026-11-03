use super::{
    combat::{Fighter, Side},
    input::Controls,
};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub shake: f32,
}
impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.28,
            distance: 5.2,
            shake: 0.0,
        }
    }
}

#[derive(Component)]
pub(super) struct GameCamera;

pub(super) fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Third-person camera"),
        GameCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 6.5).looking_at(Vec3::Y, Vec3::Y),
    ));
}

pub(super) fn rotate_camera(mut orbit: ResMut<OrbitCamera>, controls: Res<Controls>) {
    if controls.reset {
        *orbit = OrbitCamera::default();
    }
    if !controls.active {
        return;
    }
    orbit.yaw -= controls.look_delta.x * 0.003;
    orbit.pitch = (orbit.pitch + controls.look_delta.y * 0.003).clamp(-0.08, 1.05);
    orbit.distance = (orbit.distance - controls.zoom * 0.35).clamp(2.8, 8.5);
}

pub(super) fn follow_player(
    time: Res<Time>,
    controls: Res<Controls>,
    mut orbit: ResMut<OrbitCamera>,
    fighters: Query<(&Fighter, &Transform), Without<GameCamera>>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
) {
    let Some((_, player)) = fighters
        .iter()
        .find(|(fighter, _)| fighter.side == Side::Player)
    else {
        return;
    };
    if controls.active {
        orbit.shake *= (-18.0 * time.delta_secs()).exp();
    } else {
        orbit.shake = 0.0;
    }
    let t = time.elapsed_secs();
    let shake = Vec3::new((t * 81.0).sin(), (t * 67.0).cos(), 0.0) * orbit.shake;
    let target =
        player.translation + Vec3::Y * 1.25 + Quat::from_rotation_y(orbit.yaw) * Vec3::NEG_Z * 0.8;
    let offset = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.cos() * orbit.pitch.cos(),
    ) * orbit.distance;
    let position = target + offset + shake;
    **camera = Transform::from_translation(position).looking_at(target + shake * 0.2, Vec3::Y);
}
