use super::{controls::PointerState, session::ClientSession, web};
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
    session: Res<ClientSession>,
    poses: Query<&UnitPose>,
    map: Res<NavigationMap>,
    mut rig: ResMut<CameraRig>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
) {
    if session.player().is_none() {
        *rig = CameraRig::default();
    }
    let size = web::canvas_size().unwrap_or(Vec2::new(window.width(), window.height()));
    let scroll: f32 = wheel.read().map(|event| event.y).sum();
    if !control.paused && session.player().is_some() {
        if keys.just_pressed(KeyCode::KeyY) {
            rig.locked = !rig.locked;
        }
        if !pointer.over_ui {
            rig.zoom = (rig.zoom - scroll * 0.045).clamp(0.65, 1.5);
        }
        if (rig.locked || keys.pressed(KeyCode::Space))
            && let Some(player) = session.player()
            && let Ok(pose) = poses.get(player)
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
                && let Some(cursor) = pointer.position
            {
                let margin = 14.0;
                if cursor.x >= 0.0 && cursor.x < margin {
                    direction.x -= 1.0;
                }
                if cursor.x <= size.x && cursor.x > size.x - margin {
                    direction.x += 1.0;
                }
                if cursor.y >= 0.0 && cursor.y < margin {
                    direction.y -= 1.0;
                }
                if cursor.y <= size.y && cursor.y > size.y - margin {
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

#[cfg(test)]
mod tests {
    use super::super::{content, session};
    use super::*;
    use crate::simulation::SimulationPlugin;

    #[test]
    fn selecting_restores_the_existing_camera_to_its_initial_rig() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<PointerState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<MouseWheel>()
            .insert_resource(CameraRig {
                center: Vec2::new(15.0, -9.0),
                locked: true,
                zoom: 1.4,
            })
            .add_systems(Update, update);
        content::setup(app.world_mut());
        session::setup(app.world_mut());
        let camera = app
            .world_mut()
            .spawn((GameCamera, Transform::default()))
            .id();
        app.world_mut().spawn(Window::default());
        app.world_mut().run_schedule(Update);
        let rig = app.world().resource::<CameraRig>();
        assert_eq!(rig.center, CameraRig::default().center);
        assert!(!rig.locked);
        assert_eq!(rig.zoom, 1.0);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<GameCamera>>()
                .iter(app.world())
                .collect::<Vec<_>>(),
            [camera]
        );
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            Vec3::new(0.0, 0.0, 3.0) + OFFSET
        );
    }
}
