use super::controls::ViewYaw;
use crate::gameplay::Player;
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct GameCamera;

const FOV: f32 = 50.0_f32.to_radians();
const BODY_CENTER: Vec3 = Vec3::new(0.0, 0.9, 0.0);

fn camera_pose(body_center: Vec3, yaw: f32) -> Transform {
    let forward = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
    Transform::from_translation(body_center - forward * 12.0 + Vec3::Y * 18.0)
        .looking_at(body_center + forward * 4.0, Vec3::Y)
}

pub(super) fn setup(
    mut commands: Commands,
    yaw: Res<ViewYaw>,
    player: Single<&Transform, With<Player>>,
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
    player: Single<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera: Single<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    **camera = camera_pose(player.translation + BODY_CENTER, yaw.yaw());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;

    #[test]
    fn actual_perspective_places_the_body_center_below_screen_center_at_every_heading() {
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
                assert!((0.64..0.67).contains(&screen_fraction), "{screen_fraction}");
            }
        }
    }
}
