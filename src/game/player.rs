//! Rigid stickman geometry bound to imported shoulder/elbow/wrist/knee bones.
use super::{
    animation::RigPose,
    combat::{Fighter, Side},
    skeleton::AnimationLibrary,
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Appearance {
    pub material: Handle<StandardMaterial>,
    pub color: Color,
}
#[derive(Component)]
pub(super) struct SwordTip {
    pub owner: Entity,
    pub previous: Option<Vec3>,
}
pub(super) enum Attachment {
    Segment { from: usize, to: usize, radius: f32 },
    Joint { bone: usize, radius: f32 },
    Head { bone: usize },
    Sword { bone: usize },
}
#[derive(Component)]
pub(super) struct BoneVisual {
    pub owner: Entity,
    pub attachment: Attachment,
}
pub(super) fn attachment_transform(attachment: &Attachment, pose: &[Transform]) -> Transform {
    match *attachment {
        Attachment::Segment { from, to, radius } => {
            let a = pose[from].translation;
            let b = pose[to].translation;
            let delta = b - a;
            Transform::from_translation((a + b) * 0.5)
                .with_rotation(Quat::from_rotation_arc(
                    Vec3::Y,
                    delta.normalize_or(Vec3::Y),
                ))
                .with_scale(Vec3::new(radius, delta.length().max(0.001), radius))
        }
        Attachment::Joint { bone, radius } => {
            Transform::from_translation(pose[bone].translation).with_scale(Vec3::splat(radius))
        }
        Attachment::Head { bone } => {
            Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.105))
                .with_rotation(pose[bone].rotation)
        }
        Attachment::Sword { bone } => {
            Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.055))
                .with_rotation(pose[bone].rotation)
        }
    }
}
pub(super) fn spawn_fighters(
    mut commands: Commands,
    library: Res<AnimationLibrary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rod = meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(10));
    let sphere = meshes.add(Sphere::new(1.0).mesh().uv(16, 12));
    let head = meshes.add(Sphere::new(0.155).mesh().uv(24, 16));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(213, 231, 235),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        ..default()
    });
    let gold = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(218, 170, 69),
        metallic: 0.6,
        ..default()
    });
    let dark = materials.add(Color::srgb_u8(25, 32, 39));
    let eyes = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(225, 248, 249),
        emissive: LinearRgba::new(0.2, 0.5, 0.5, 1.0),
        ..default()
    });
    for side in [Side::Player, Side::Dummy] {
        let color = if side == Side::Player {
            Color::srgb_u8(65, 191, 190)
        } else {
            Color::srgb_u8(223, 146, 83)
        };
        let body = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.62,
            ..default()
        });
        let rig = RigPose::new(&library);
        let root = commands
            .spawn((
                Name::new(if side == Side::Player {
                    "Stickman swordsman"
                } else {
                    "Stickman training dummy"
                }),
                Fighter::new(side),
                Appearance {
                    material: body.clone(),
                    color,
                },
                Transform::from_translation(side.home())
                    .with_rotation(Quat::from_rotation_y(side.yaw())),
                Visibility::default(),
            ))
            .id();
        let mut attachments = Vec::new();
        // Each rod spans two animated joints, preserving contact when bending.
        for (a, b, radius) in [
            ("pelvis", "spine_01", 0.075),
            ("spine_01", "spine_02", 0.075),
            ("spine_02", "spine_03", 0.075),
            ("spine_03", "neck_01", 0.065),
            ("neck_01", "Head", 0.055),
            ("upperarm_l", "upperarm_r", 0.06),
            ("thigh_l", "thigh_r", 0.065),
            ("upperarm_l", "lowerarm_l", 0.052),
            ("lowerarm_l", "hand_l", 0.047),
            ("upperarm_r", "lowerarm_r", 0.052),
            ("lowerarm_r", "hand_r", 0.047),
            ("thigh_l", "calf_l", 0.062),
            ("calf_l", "foot_l", 0.055),
            ("thigh_r", "calf_r", 0.062),
            ("calf_r", "foot_r", 0.055),
            ("foot_l", "ball_l", 0.055),
            ("foot_r", "ball_r", 0.055),
        ] {
            attachments.push((
                rod.clone(),
                Attachment::Segment {
                    from: library.bone(a),
                    to: library.bone(b),
                    radius,
                },
            ));
        }
        for name in [
            "pelvis",
            "spine_02",
            "upperarm_l",
            "lowerarm_l",
            "hand_l",
            "upperarm_r",
            "lowerarm_r",
            "hand_r",
            "thigh_l",
            "calf_l",
            "foot_l",
            "thigh_r",
            "calf_r",
            "foot_r",
        ] {
            attachments.push((
                sphere.clone(),
                Attachment::Joint {
                    bone: library.bone(name),
                    radius: if name == "pelvis" { 0.085 } else { 0.06 },
                },
            ));
        }
        for (mesh, attachment) in attachments {
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(body.clone()),
                attachment_transform(&attachment, &rig.model),
                BoneVisual {
                    owner: root,
                    attachment,
                },
                ChildOf(root),
            ));
        }
        let attachment = Attachment::Head {
            bone: library.bone("Head"),
        };
        commands
            .spawn((
                Mesh3d(head.clone()),
                MeshMaterial3d(body),
                attachment_transform(&attachment, &rig.model),
                BoneVisual {
                    owner: root,
                    attachment,
                },
                ChildOf(root),
            ))
            .with_children(|head| {
                for x in [-0.055, 0.055] {
                    head.spawn((
                        Mesh3d(sphere.clone()),
                        MeshMaterial3d(eyes.clone()),
                        Transform::from_xyz(x, 0.025, 0.142).with_scale(Vec3::splat(0.022)),
                    ));
                }
            });
        let attachment = Attachment::Sword {
            bone: library.bone("hand_r"),
        };
        commands
            .spawn((
                Name::new("Right hand sword socket"),
                attachment_transform(&attachment, &rig.model),
                Visibility::default(),
                BoneVisual {
                    owner: root,
                    attachment,
                },
                ChildOf(root),
            ))
            .with_children(|sword| {
                for (material, position, scale) in [
                    (
                        dark.clone(),
                        Vec3::new(0.0, 0.0, -0.035),
                        Vec3::new(0.065, 0.065, 0.22),
                    ),
                    (
                        gold.clone(),
                        Vec3::new(0.0, 0.0, 0.09),
                        Vec3::new(0.29, 0.05, 0.055),
                    ),
                    (
                        metal.clone(),
                        Vec3::new(0.0, 0.0, 0.56),
                        Vec3::new(0.07, 0.028, 0.91),
                    ),
                ] {
                    sword.spawn((
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(position).with_scale(scale),
                    ));
                }
                sword.spawn((
                    SwordTip {
                        owner: root,
                        previous: None,
                    },
                    Transform::from_xyz(0.0, 0.0, 1.02),
                    Visibility::default(),
                ));
            });
        commands.entity(root).insert(rig);
    }
}
