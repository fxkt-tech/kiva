use super::PlayerView;
use super::controls::ViewYaw;
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct GameCamera;

const FOV: f32 = 50.0_f32.to_radians();
const BODY_CENTER: Vec3 = Vec3::new(0.0, 0.9, 0.0);
const CAMERA_HEIGHT: f32 = 5.0;
const CAMERA_BACK: f32 = CAMERA_HEIGHT * 0.7;
const BODY_SCREEN_Y: f32 = 2.0 / 3.0;

fn camera_pose(body_center: Vec3, yaw: f32) -> Transform {
    let heading = Quat::from_rotation_y(yaw);
    let forward = heading * Vec3::NEG_Z;
    // Convert the desired screen position into an angle below the optical axis.
    let body_view_angle = ((2.0 * BODY_SCREEN_Y - 1.0) * (FOV * 0.5).tan()).atan();
    let pitch = CAMERA_HEIGHT.atan2(CAMERA_BACK) - body_view_angle;
    Transform::from_translation(body_center - forward * CAMERA_BACK + Vec3::Y * CAMERA_HEIGHT)
        .with_rotation(heading * Quat::from_rotation_x(-pitch))
}

pub(super) fn setup(
    mut commands: Commands,
    yaw: Res<ViewYaw>,
    player: Single<&Transform, With<PlayerView>>,
) {
    commands.spawn((
        GameCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV,
            ..default()
        }),
        camera_pose(player.translation + BODY_CENTER, yaw.yaw()),
    ));
}

pub(super) fn follow(
    yaw: Res<ViewYaw>,
    player: Single<&Transform, (With<PlayerView>, Without<GameCamera>)>,
    mut camera: Single<&mut Transform, (With<GameCamera>, Without<PlayerView>)>,
) {
    **camera = camera_pose(player.translation + BODY_CENTER, yaw.yaw());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;

    #[test]
    fn actual_perspective_places_the_body_center_at_the_lower_third_at_every_heading() {
        for aspect in [4.0 / 3.0, 16.0 / 9.0, 1.0] {
            let projection = PerspectiveProjection {
                fov: FOV,
                aspect_ratio: aspect,
                ..default()
            }
            .get_clip_from_view();
            for yaw in [0.0, -std::f32::consts::FRAC_PI_2, 2.4] {
                let center = Vec3::new(17.0, 0.9, -9.0);
                let camera = camera_pose(center, yaw);
                let view = Mat4::from_scale_rotation_translation(
                    camera.scale,
                    camera.rotation,
                    camera.translation,
                )
                .inverse();
                let clip = projection * view * center.extend(1.0);
                let ndc = clip.truncate() / clip.w;
                assert!(ndc.x.abs() < 0.0001);
                let screen_fraction = (1.0 - ndc.y) * 0.5;
                assert!(
                    (screen_fraction - 2.0 / 3.0).abs() < 0.0001,
                    "{screen_fraction}"
                );
            }
        }
    }
}
