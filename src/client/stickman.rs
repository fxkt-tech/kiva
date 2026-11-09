//! Rigid stickman parts follow the imported animation's bones in actor-local space.
use bevy::prelude::*;

use super::{animation::RigPose, skeleton::AnimationLibrary};
use crate::simulation::WeaponMode;

#[derive(Resource, Clone)]
struct StickmanMeshes {
    rod: Handle<Mesh>,
    sphere: Handle<Mesh>,
    head: Handle<Mesh>,
}

impl StickmanMeshes {
    fn shared(world: &mut World) -> Self {
        if !world.contains_resource::<Self>() {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            let geometry = Self {
                rod: meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(10)),
                sphere: meshes.add(Sphere::new(1.0).mesh().uv(16, 12)),
                head: meshes.add(Sphere::new(0.155).mesh().uv(24, 16)),
            };
            world.insert_resource(geometry);
        }
        world.resource::<Self>().clone()
    }
}

#[derive(Clone, Copy)]
pub(super) enum Attachment {
    Segment { from: usize, to: usize, radius: f32 },
    Joint { bone: usize, radius: f32 },
    Head { bone: usize },
    Weapon { bone: usize, mode: WeaponMode },
}

#[derive(Component)]
pub(super) struct BoneVisual {
    pub owner: Entity,
    pub attachment: Attachment,
}

pub(super) fn attachment_transform(attachment: &Attachment, pose: &[Transform]) -> Transform {
    match *attachment {
        Attachment::Segment { from, to, radius } => {
            segment_transform(pose[from].translation, pose[to].translation, radius)
        }
        Attachment::Joint { bone, radius } => {
            Transform::from_translation(pose[bone].translation).with_scale(Vec3::splat(radius))
        }
        Attachment::Head { bone } => {
            Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.105))
                .with_rotation(pose[bone].rotation)
        }
        Attachment::Weapon { bone, mode } => {
            // The authored hand holds a +Z sword. Current melee handles run +Y,
            // while the guns fire along -Z, so each keeps its own local correction.
            let correction = match mode {
                WeaponMode::Melee => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                WeaponMode::Ranged => Quat::from_rotation_y(std::f32::consts::PI),
            };
            Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.055))
                .with_rotation(pose[bone].rotation * correction)
        }
    }
}

/// Returns the body pieces so callers can associate existing hit-flash presentation.
pub(super) fn spawn(
    world: &mut World,
    owner: Entity,
    body_material: &Handle<StandardMaterial>,
    eyes_material: &Handle<StandardMaterial>,
) -> Vec<Entity> {
    let geometry = StickmanMeshes::shared(world);
    let library = world.resource::<AnimationLibrary>();
    let rig = RigPose::new(library);
    let mut attachments = Vec::new();
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
        attachments.push((
            geometry.rod.clone(),
            Attachment::Segment {
                from: library.bone(from),
                to: library.bone(to),
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
            geometry.sphere.clone(),
            Attachment::Joint {
                bone: library.bone(name),
                radius: if name == "pelvis" { 0.085 } else { 0.06 },
            },
        ));
    }
    let head_attachment = Attachment::Head {
        bone: library.bone("Head"),
    };
    let mut pieces = Vec::new();
    for (mesh, attachment) in attachments {
        pieces.push(bound_part(
            world,
            owner,
            &mesh,
            body_material,
            attachment,
            &rig.model,
        ));
    }
    let head = bound_part(
        world,
        owner,
        &geometry.head,
        body_material,
        head_attachment,
        &rig.model,
    );
    pieces.push(head);
    for x in [-0.055, 0.055] {
        let eye = world
            .spawn((
                Mesh3d(geometry.sphere.clone()),
                MeshMaterial3d(eyes_material.clone()),
                Transform::from_xyz(x, 0.025, 0.142).with_scale(Vec3::splat(0.022)),
            ))
            .id();
        world.entity_mut(head).add_child(eye);
    }
    world.entity_mut(owner).insert(rig);
    pieces
}

/// Binds an existing weapon child to the same animated wrist as the body geometry.
pub(super) fn attach_weapon(world: &mut World, owner: Entity, weapon: Entity, mode: WeaponMode) {
    let bone = world.resource::<AnimationLibrary>().bone("hand_r");
    let attachment = Attachment::Weapon { bone, mode };
    let rig = world
        .get::<RigPose>(owner)
        .expect("weapon owner has a stickman rig");
    let transform = attachment_transform(&attachment, &rig.model);
    world
        .entity_mut(weapon)
        .insert((BoneVisual { owner, attachment }, transform));
}

pub(super) fn sync_attachments(
    rigs: Query<&RigPose>,
    mut parts: Query<(&BoneVisual, &mut Transform)>,
) {
    for (visual, mut transform) in &mut parts {
        if let Ok(rig) = rigs.get(visual.owner) {
            *transform = attachment_transform(&visual.attachment, &rig.model);
        }
    }
}

fn segment_transform(from: Vec3, to: Vec3, radius: f32) -> Transform {
    let delta = to - from;
    Transform::from_translation((from + to) * 0.5)
        .with_rotation(Quat::from_rotation_arc(
            Vec3::Y,
            delta.normalize_or(Vec3::Y),
        ))
        .with_scale(Vec3::new(radius, delta.length().max(0.001), radius))
}

fn bound_part(
    world: &mut World,
    owner: Entity,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    attachment: Attachment,
    pose: &[Transform],
) -> Entity {
    let entity = world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            attachment_transform(&attachment, pose),
            BoneVisual { owner, attachment },
        ))
        .id();
    world.entity_mut(owner).add_child(entity);
    entity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Body;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<AnimationLibrary>()
            .add_systems(Update, sync_attachments);
        app
    }

    #[test]
    fn rods_connect_their_endpoints_even_when_tilted() {
        let from = Vec3::new(-0.3, 1.4, 0.2);
        let to = Vec3::new(0.1, 0.7, -0.3);
        let transform = segment_transform(from, to, 0.04);
        assert!(transform.transform_point(Vec3::NEG_Y * 0.5).distance(from) < 0.0001);
        assert!(transform.transform_point(Vec3::Y * 0.5).distance(to) < 0.0001);
    }

    #[test]
    fn characters_share_geometry_and_do_not_change_the_owners_authoritative_position() {
        let mut app = app();
        let world = app.world_mut();
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let position = Vec2::new(2.0, -3.0);
        let first = world
            .spawn((Body::new(position, 6.0, 0.35), Transform::default()))
            .id();
        let first_parts = spawn(world, first, &material, &material);
        let mesh_count = world.resource::<Assets<Mesh>>().len();
        let second = world.spawn(Transform::default()).id();
        let second_parts = spawn(world, second, &material, &material);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(world.get::<Body>(first).unwrap().position(), position);
        for (owner, parts) in [(first, first_parts), (second, second_parts)] {
            assert!(world.get::<RigPose>(owner).is_some());
            for entity in parts {
                assert_eq!(world.get::<ChildOf>(entity).unwrap().parent(), owner);
            }
        }
    }

    #[test]
    fn sampled_animation_moves_limbs_and_keeps_both_weapon_axes_bound_to_the_wrist() {
        let mut app = app();
        let world = app.world_mut();
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let owner = world.spawn(Transform::default()).id();
        let parts = spawn(world, owner, &material, &material);
        let mut weapons = Vec::new();
        for mode in [WeaponMode::Melee, WeaponMode::Ranged] {
            let weapon = world.spawn(Transform::default()).id();
            world.entity_mut(owner).add_child(weapon);
            attach_weapon(world, owner, weapon, mode);
            weapons.push(weapon);
        }
        let (first, second, hand) = {
            let library = world.resource::<AnimationLibrary>();
            (
                library.model_pose(&library.sample("Walk_Loop", 0.1, true)),
                library.model_pose(&library.sample("Walk_Loop", 0.55, true)),
                library.bone("hand_r"),
            )
        };
        world.get_mut::<RigPose>(owner).unwrap().model = first;
        world.run_schedule(Update);
        let previous: Vec<_> = parts
            .iter()
            .map(|&part| *world.get::<Transform>(part).unwrap())
            .collect();
        world.get_mut::<RigPose>(owner).unwrap().model = second;
        world.run_schedule(Update);
        assert!(parts.iter().zip(previous).any(|(&part, before)| {
            world
                .get::<Transform>(part)
                .unwrap()
                .translation
                .distance(before.translation)
                > 0.01
        }));
        let rig = world.get::<RigPose>(owner).unwrap();
        let wrist = rig.model[hand];
        for (weapon, mode) in weapons
            .into_iter()
            .zip([WeaponMode::Melee, WeaponMode::Ranged])
        {
            let transform = world.get::<Transform>(weapon).unwrap();
            assert!(
                transform
                    .translation
                    .distance(wrist.transform_point(Vec3::Y * 0.055))
                    < 0.0001
            );
            assert!(transform.rotation.is_finite());
            let authored_forward = wrist.rotation * Vec3::Z;
            let weapon_axis = match mode {
                WeaponMode::Melee => Vec3::Y,
                WeaponMode::Ranged => Vec3::NEG_Z,
            };
            assert!((transform.rotation * weapon_axis).distance(authored_forward) < 0.0001);
            assert_eq!(world.get::<ChildOf>(weapon).unwrap().parent(), owner);
        }
    }
}
