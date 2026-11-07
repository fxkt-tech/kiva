//! Rigid geometry follows the imported joints, with a right-hand sword socket.
use super::{
    rig::AnimationLibrary,
    weapon::{self, WeaponStyle},
};
use bevy::prelude::*;

#[derive(Resource)]
pub(in crate::client) struct ModelAssets {
    rod: Handle<Mesh>,
    joint: Handle<Mesh>,
    head: Handle<Mesh>,
    pub(super) cube: Handle<Mesh>,
    pub(super) metal: Handle<StandardMaterial>,
    pub(super) gold: Handle<StandardMaterial>,
    pub(super) dark: Handle<StandardMaterial>,
    eyes: Handle<StandardMaterial>,
}

pub(in crate::client) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ModelAssets {
        rod: meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(10)),
        joint: meshes.add(Sphere::new(1.0).mesh().uv(16, 12)),
        head: meshes.add(Sphere::new(0.155).mesh().uv(24, 16)),
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        metal: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(213, 231, 235),
            metallic: 0.7,
            perceptual_roughness: 0.3,
            ..default()
        }),
        gold: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(218, 170, 69),
            metallic: 0.6,
            ..default()
        }),
        dark: materials.add(Color::srgb_u8(25, 32, 39)),
        eyes: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(225, 248, 249),
            emissive: LinearRgba::new(0.2, 0.5, 0.5, 1.0),
            ..default()
        }),
    });
}

#[derive(Component)]
pub(super) struct CharacterModel;

#[derive(Clone, Copy)]
pub(super) enum Attachment {
    Segment { from: usize, to: usize, radius: f32 },
    Joint { bone: usize, radius: f32 },
    Head { bone: usize },
    Sword { bone: usize },
    Hand { bone: usize },
}

impl Attachment {
    pub fn transform(self, pose: &[Transform]) -> Transform {
        match self {
            Self::Segment { from, to, radius } => {
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
            Self::Joint { bone, radius } => {
                Transform::from_translation(pose[bone].translation).with_scale(Vec3::splat(radius))
            }
            Self::Head { bone } => {
                Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.105))
                    .with_rotation(pose[bone].rotation)
            }
            Self::Sword { bone } | Self::Hand { bone } => {
                Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.055))
                    .with_rotation(pose[bone].rotation)
            }
        }
    }
}

#[derive(Component)]
pub(in crate::client) struct BoneVisual {
    pub(super) owner: Entity,
    pub(super) attachment: Attachment,
}

pub(in crate::client) fn spawn_model(
    commands: &mut Commands,
    owner: Entity,
    assets: &ModelAssets,
    library: &AnimationLibrary,
    pose: &[Transform],
    body: Handle<StandardMaterial>,
    weapon: WeaponStyle,
) {
    commands.entity(owner).with_children(|root| {
        root.spawn((
            CharacterModel,
            Name::new("Skeletal character"),
            Transform::default(),
            Visibility::Inherited,
        ))
        .with_children(|model| {
            for (from, to, radius) in [
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
                let attachment = Attachment::Segment {
                    from: library.bone(from),
                    to: library.bone(to),
                    radius,
                };
                model.spawn((
                    Mesh3d(assets.rod.clone()),
                    MeshMaterial3d(body.clone()),
                    attachment.transform(pose),
                    BoneVisual { owner, attachment },
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
                let attachment = Attachment::Joint {
                    bone: library.bone(name),
                    radius: if name == "pelvis" { 0.085 } else { 0.06 },
                };
                model.spawn((
                    Mesh3d(assets.joint.clone()),
                    MeshMaterial3d(body.clone()),
                    attachment.transform(pose),
                    BoneVisual { owner, attachment },
                ));
            }
            let attachment = Attachment::Head {
                bone: library.bone("Head"),
            };
            model
                .spawn((
                    Mesh3d(assets.head.clone()),
                    MeshMaterial3d(body),
                    attachment.transform(pose),
                    BoneVisual { owner, attachment },
                ))
                .with_children(|head| {
                    for x in [-0.055, 0.055] {
                        head.spawn((
                            Mesh3d(assets.joint.clone()),
                            MeshMaterial3d(assets.eyes.clone()),
                            Transform::from_xyz(x, 0.025, 0.142).with_scale(Vec3::splat(0.022)),
                        ));
                    }
                });
            let hand = library.bone("hand_r");
            let attachment = if weapon == WeaponStyle::Sword {
                Attachment::Sword { bone: hand }
            } else {
                Attachment::Hand { bone: hand }
            };
            model
                .spawn((
                    Name::new(format!("{} hand socket", weapon.label())),
                    attachment.transform(pose),
                    Visibility::Inherited,
                    BoneVisual { owner, attachment },
                ))
                .with_children(|socket| weapon::spawn(socket, assets, weapon));
        });
    });
}
