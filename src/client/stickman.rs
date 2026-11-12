//! Wrist attachments which follow the shared skinned avatar.
use bevy::prelude::*;

use super::{
    animation::RigPose,
    avatar::{Appearance, AvatarAppearance, AvatarAssetCache, AvatarAssets},
    skeleton::AnimationLibrary,
};
use crate::simulation::WeaponMode;

#[derive(Clone, Copy)]
pub(super) enum Attachment {
    Weapon { bone: usize, mode: WeaponMode },
}

#[derive(Component)]
pub(super) struct BoneVisual {
    pub owner: Entity,
    pub attachment: Attachment,
}

pub(super) fn attachment_transform(attachment: &Attachment, pose: &[Transform]) -> Transform {
    let Attachment::Weapon { bone, mode } = *attachment;
    // Authored hands hold a +Z sword. The imported blade is centered on its
    // grip and extends along +Y; ranged guns retain their established -Z axis.
    let correction = match mode {
        WeaponMode::Melee => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        WeaponMode::Ranged => Quat::from_rotation_y(std::f32::consts::PI),
    };
    Transform::from_translation(pose[bone].transform_point(Vec3::Y * 0.055))
        .with_rotation(pose[bone].rotation * correction)
}

pub(super) fn attach_weapon(world: &mut World, owner: Entity, weapon: Entity, mode: WeaponMode) {
    let bone = world.resource::<AnimationLibrary>().bone("hand_r");
    let attachment = Attachment::Weapon { bone, mode };
    let rig = world
        .get::<RigPose>(owner)
        .expect("weapon owner has an avatar rig");
    let appearance = world
        .get::<AvatarAppearance>(owner)
        .map_or(Appearance::default(), |current| current.0);
    let assets = AvatarAssets::cached(
        world.get_resource::<AvatarAssets>(),
        world.get_resource::<AvatarAssetCache>(),
        appearance,
    )
    .expect("weapon owner's appearance has been loaded");
    let pose = assets.attachment_pose(rig);
    let transform = attachment_transform(&attachment, &pose);
    world
        .entity_mut(weapon)
        .insert((BoneVisual { owner, attachment }, transform));
}

pub(super) fn sync_attachments(
    assets: Option<Res<AvatarAssets>>,
    cache: Option<Res<AvatarAssetCache>>,
    rigs: Query<(&RigPose, Option<&AvatarAppearance>)>,
    mut parts: Query<(&BoneVisual, &mut Transform)>,
) {
    for (visual, mut transform) in &mut parts {
        if let Ok((rig, appearance)) = rigs.get(visual.owner) {
            let appearance = appearance.map_or(Appearance::default(), |current| current.0);
            let pose = AvatarAssets::cached(assets.as_deref(), cache.as_deref(), appearance)
                .map_or_else(|| rig.model.clone(), |assets| assets.attachment_pose(rig));
            *transform = attachment_transform(&visual.attachment, &pose);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Body;

    #[test]
    fn characters_share_geometry_and_preserve_authoritative_position() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<AnimationLibrary>();
        let position = Vec2::new(2.0, -3.0);
        let first = world
            .spawn((Body::new(position, 6.0, 0.35), Transform::default()))
            .id();
        let first_parts = crate::client::avatar::spawn(&mut world, first);
        let mesh_count = world.resource::<Assets<Mesh>>().len();
        let material_count = world.resource::<Assets<StandardMaterial>>().len();
        let image_count = world.resource::<Assets<Image>>().len();
        let second = world.spawn(Transform::default()).id();
        let second_parts = crate::client::avatar::spawn(&mut world, second);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(
            world.resource::<Assets<StandardMaterial>>().len(),
            material_count
        );
        assert_eq!(world.resource::<Assets<Image>>().len(), image_count);
        assert_eq!(world.get::<Body>(first).unwrap().position(), position);
        for (owner, parts) in [(first, first_parts), (second, second_parts)] {
            assert!(world.get::<RigPose>(owner).is_some());
            assert!(parts.len() >= 13);
            for entity in parts {
                assert_eq!(world.get::<ChildOf>(entity).unwrap().parent(), owner);
                assert!(
                    world
                        .get::<bevy::mesh::skinning::SkinnedMesh>(entity)
                        .is_some()
                );
            }
        }
    }

    #[test]
    fn both_weapon_axes_follow_the_authored_wrist_orientation() {
        let library = AnimationLibrary::default();
        let pose = library.model_pose(&library.sample("Walk_Loop", 0.55, true));
        let hand = library.bone("hand_r");
        for mode in [WeaponMode::Melee, WeaponMode::Ranged] {
            let transform = attachment_transform(&Attachment::Weapon { bone: hand, mode }, &pose);
            let axis = if mode == WeaponMode::Melee {
                Vec3::Y
            } else {
                Vec3::NEG_Z
            };
            assert!((transform.rotation * axis).distance(pose[hand].rotation * Vec3::Z) < 0.0001);
            assert!(
                transform
                    .translation
                    .distance(pose[hand].transform_point(Vec3::Y * 0.055))
                    < 0.0001
            );
        }
    }
}
