use super::{
    controls::{PlayerController, PointerState},
    web,
};
use crate::simulation::{NavigationMap, SimulationControl, UnitPose};
use bevy::{input::mouse::MouseWheel, prelude::*};

#[derive(Component)]
pub(super) struct GameCamera;
#[derive(Resource)]
pub(super) struct CameraRig {
    pub center: Vec2,
    pub locked: bool,
    pub zoom: f32,
}
impl Default for CameraRig {
    fn default() -> Self {
        Self {
            center: Vec2::new(0.0, 3.0),
            locked: false,
            zoom: 1.0,
        }
    }
}
const OFFSET: Vec3 = Vec3::new(10.0, 25.0, 20.0);
pub(super) fn setup(mut commands: Commands, rig: Res<CameraRig>) {
    let target = Vec3::new(rig.center.x, 0.0, rig.center.y);
    commands.spawn((
        GameCamera,
        Camera3d::default(),
        Transform::from_translation(target + OFFSET).looking_at(target, Vec3::Y),
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy system parameters declare independent ECS access"
)]
pub(super) fn update(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    control: Res<SimulationControl>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    pointer: Res<PointerState>,
    player: Res<PlayerController>,
    poses: Query<&UnitPose>,
    map: Res<NavigationMap>,
    mut rig: ResMut<CameraRig>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
) {
    let scroll: f32 = wheel.read().map(|event| event.y).sum();
    if !control.paused {
        if keys.just_pressed(KeyCode::KeyY) {
            rig.locked = !rig.locked;
        }
        if !pointer.over_ui {
            rig.zoom = (rig.zoom - scroll * 0.045).clamp(0.65, 1.5);
        }
        if (rig.locked || keys.pressed(KeyCode::Space))
            && let Ok(pose) = poses.get(player.player)
        {
            rig.center = pose.interpolated(fixed.overstep_fraction());
        } else {
            let mut direction = Vec2::new(
                f32::from(keys.pressed(KeyCode::ArrowRight))
                    - f32::from(keys.pressed(KeyCode::ArrowLeft)),
                f32::from(keys.pressed(KeyCode::ArrowDown))
                    - f32::from(keys.pressed(KeyCode::ArrowUp)),
            );
            if !pointer.over_ui
                && let Some(cursor) = window.cursor_position()
            {
                let margin = 14.0;
                if cursor.x >= 0.0 && cursor.x < margin {
                    direction.x -= 1.0;
                }
                if cursor.x <= window.width() && cursor.x > window.width() - margin {
                    direction.x += 1.0;
                }
                if cursor.y >= 0.0 && cursor.y < margin {
                    direction.y -= 1.0;
                }
                if cursor.y <= window.height() && cursor.y > window.height() - margin {
                    direction.y += 1.0;
                }
            }
            let right = Vec2::new(OFFSET.z, -OFFSET.x).normalize();
            let down = Vec2::new(OFFSET.x, OFFSET.z).normalize();
            let speed = 18.0 * rig.zoom * time.delta_secs();
            rig.center += (right * direction.x + down * direction.y).clamp_length_max(1.0) * speed;
        }
        let bound = map.half_size() - 5.0;
        rig.center = rig.center.clamp(Vec2::splat(-bound), Vec2::splat(bound));
    }
    let target = Vec3::new(rig.center.x, 0.0, rig.center.y);
    **camera = Transform::from_translation(target + OFFSET * rig.zoom).looking_at(target, Vec3::Y);
    web::set_camera_lock(rig.locked);
}
