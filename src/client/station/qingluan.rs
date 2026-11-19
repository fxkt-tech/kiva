//! The station companion stands in world space. The pool camera selects her meshes.
use crate::client::{animation::RigPose, skeleton::AnimationLibrary};
use crate::simulation::{Body, HitStop, SimulationActive};
use bevy::{camera::visibility::RenderLayers, light::NotShadowCaster, prelude::*};
use std::f32::consts::{PI, TAU};

#[derive(Resource, Clone, Copy)]
pub(in crate::client) struct Placement {
    feet: Vec3,
    wagon: Vec3,
    layer: usize,
}

pub(super) fn setup(world: &mut World, feet: Vec3, wagon: Vec3, layer: usize) {
    world.insert_resource(Placement { feet, wagon, layer });
}

#[derive(Component, Default)]
struct Gaze {
    seconds: f32,
    brother_yaw: Option<f32>,
}

pub(in crate::client) fn bind(world: &mut World, root: Entity, placement: Placement) {
    // RenderLayers and NotShadowCaster are per drawable, including the nested
    // hand weapons and quiver that the combat rig initially keeps hidden.
    let mut pending = vec![root];
    while let Some(entity) = pending.pop() {
        if let Some(children) = world.get::<Children>(entity) {
            pending.extend(children.iter());
        }
        if world.get::<Mesh3d>(entity).is_some() {
            world
                .entity_mut(entity)
                .insert((RenderLayers::layer(placement.layer), NotShadowCaster));
        }
    }
    world.entity_mut(root).insert((
        Gaze::default(),
        // Source-local flashes also read the emitting performer's layer.
        RenderLayers::layer(placement.layer),
    ));
}

fn smooth(clock: f32, start: f32, end: f32) -> f32 {
    let t = ((clock - start) / (end - start)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

fn yaw(direction: Vec3) -> f32 {
    (-direction.x).atan2(-direction.z)
}

pub(in crate::client) fn idle(world: &mut World, root: Entity, owner: Entity, stage: Placement) {
    let dt = if world.resource::<SimulationActive>().0 && !world.resource::<HitStop>().is_active() {
        world.resource::<Time>().delta_secs()
    } else {
        0.
    };
    let brother = world
        .get::<Body>(owner)
        .map_or(stage.wagon, Body::world_position);
    let delta = brother - stage.feet;
    let wagon_yaw = yaw(stage.wagon - stage.feet);
    let mut gaze = world.get_mut::<Gaze>(root).expect("bound station rig");
    // Track the brother during the look, then keep the departure direction.
    // Walking round her after she starts turning cannot reverse her pivot.
    if gaze.brother_yaw.is_none() || (gaze.seconds < 2.6 && dt > 0. && delta.xz().length() > 0.1) {
        gaze.brother_yaw = Some(if delta.xz().length() > 0.1 {
            yaw(delta)
        } else {
            wagon_yaw
        });
    }
    gaze.seconds += dt;
    let seconds = gaze.seconds;
    let brother_yaw = gaze.brother_yaw.unwrap();
    let body_start = brother_yaw + 0.28;
    let to_wagon = (wagon_yaw - body_start + PI).rem_euclid(TAU) - PI;
    let pelvis_turn = smooth(seconds, 3.25, 5.9);
    let gaze_turn = smooth(seconds, 2.6, 4.9);
    let neck_yaw =
        (-0.28 * (1. - gaze_turn) + (gaze_turn - pelvis_turn) * to_wagon).clamp(-0.91, 0.91);
    *world.get_mut::<Transform>(root).unwrap() = Transform::from_translation(stage.feet)
        .with_rotation(Quat::from_rotation_y(body_start + pelvis_turn * to_wagon));
    world.resource_scope(|world, library: Mut<AnimationLibrary>| {
        let mut pose = world.get_mut::<RigPose>(root).unwrap();
        pose.reference_pose = false;
        pose.source = None;
        pose.companion_hands = None;
        idle_pose(&library, seconds, neck_yaw, gaze_turn, &mut pose.model);
    });
}

/// Articulated look and breathing in the donor's model frame. FK propagates each
/// change before the shared avatar solver maps it to this body's own bone lengths.
fn idle_pose(lib: &AnimationLibrary, t: f32, turn: f32, reach: f32, model: &mut [Transform]) {
    let breath = (t * 1.65).sin();
    let basis = Transform::from_rotation(Quat::from_rotation_y(PI)).with_scale(Vec3::splat(1.1));
    let left_lift = (PI * smooth(t, 3.25, 4.55)).sin().max(0.);
    let right_lift = (PI * smooth(t, 4.55, 5.9)).sin().max(0.);
    for (index, bone) in lib.bones.iter().enumerate() {
        let parent = bone.parent.map_or(basis, |parent| model[parent]);
        let mut posed = parent.mul_transform(bone.rest);
        let rotation = match bone.name.as_str() {
            "thigh_l" => Quat::from_rotation_x(0.26 * left_lift),
            "calf_l" => Quat::from_rotation_x(-0.51 * left_lift),
            "foot_l" => Quat::from_rotation_x(0.25 * left_lift),
            "thigh_r" => Quat::from_rotation_x(0.26 * right_lift),
            "calf_r" => Quat::from_rotation_x(-0.51 * right_lift),
            "foot_r" => Quat::from_rotation_x(0.25 * right_lift),
            "spine_02" => Quat::from_rotation_y(turn * 0.16),
            "spine_03" => {
                Quat::from_rotation_x(0.018 * breath) * Quat::from_rotation_y(turn * 0.18)
            }
            "neck_01" => Quat::from_rotation_y(turn * 0.28),
            "Head" => {
                Quat::from_rotation_y(turn * 0.38) * Quat::from_rotation_x(0.10 - 0.07 * reach)
            }
            "upperarm_l" | "upperarm_r" | "lowerarm_l" | "lowerarm_r" => {
                let (child, goal) = match bone.name.as_str() {
                    "upperarm_l" => ("lowerarm_l", Vec3::new(-0.25, -0.96, 0.03)),
                    "upperarm_r" => ("lowerarm_r", Vec3::new(0.25, -0.96, 0.06)),
                    "lowerarm_l" => ("hand_l", Vec3::new(-0.04, -0.92, -0.37)),
                    _ => (
                        "hand_r",
                        Vec3::new(-0.17, -0.92 + 0.45 * reach, -0.43 - 0.47 * reach),
                    ),
                };
                let direction = posed.rotation * lib.bones[lib.bone(child)].rest.translation;
                Quat::from_rotation_arc(direction.normalize(), goal.normalize())
            }
            _ => Quat::IDENTITY,
        };
        posed.rotation = rotation * posed.rotation;
        model[index] = posed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{
        avatar::{self, Appearance, AvatarAppearance},
        companion::{self, CompanionView, SisterWeapon},
    };
    use crate::gameplay::station::LAYOUT;
    use crate::simulation::{ActionKind, Companion, CompanionState, WeaponPair};
    use bevy::mesh::skinning::SkinnedMesh;
    use std::time::Duration;

    #[test]
    fn station_gaze_deforms_its_skin_and_combat_uses_the_same_isolated_rig() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<AnimationLibrary>()
            .init_resource::<Time>()
            .init_resource::<Time<Fixed>>()
            .init_resource::<HitStop>()
            .insert_resource(SimulationActive(false))
            .add_systems(Update, (companion::sync, avatar::sync).chain());
        let world = app.world_mut();
        setup(world, LAYOUT.sister, LAYOUT.window, 2);
        let owner = world.spawn(Body::at(LAYOUT.spawn, 3., 0.3)).id();
        let performer = world
            .spawn((Companion { owner }, CompanionState::default()))
            .id();
        companion::setup(world);
        companion::setup(world);
        let root = world
            .query_filtered::<Entity, With<CompanionView>>()
            .single(world)
            .unwrap();
        assert_eq!(
            world.get::<AvatarAppearance>(root).unwrap().0,
            Appearance::Qingluan
        );
        world.run_schedule(Update);
        let n = world.entities().count_spawned();
        let feathers =
            |world: &mut World, prefix| avatar::sample_matching_surface(world, root, Some(prefix));
        let crest = feathers(world, "qingluan::temple_crest");
        let mantle = feathers(world, "qingluan::kingfisher_mantle");
        assert!(!crest.is_empty() && !mantle.is_empty());
        assert_eq!(
            world.get::<Transform>(root).unwrap().translation,
            LAYOUT.sister
        );
        world.resource_mut::<SimulationActive>().0 = true;
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(4.2));
        world.run_schedule(Update);
        for (prefix, before) in [
            ("qingluan::temple_crest", crest),
            ("qingluan::kingfisher_mantle", mantle),
        ] {
            let after = feathers(world, prefix);
            assert!(
                after.iter().zip(before).any(|(a, b)| a.distance(b) > 0.01),
                "skin {prefix}"
            );
        }
        world.resource_mut::<SimulationActive>().0 = false;
        let held = *world.get::<Transform>(root).unwrap();
        let pose = world.get::<RigPose>(root).unwrap().model.clone();
        world.run_schedule(Update);
        assert_eq!(world.get::<Transform>(root), Some(&held));
        assert_eq!(world.get::<RigPose>(root).unwrap().model, pose);
        world.resource_mut::<SimulationActive>().0 = true;
        world.run_schedule(Update);
        let root_pose = world.get::<Transform>(root).unwrap();
        let window = (LAYOUT.window - LAYOUT.sister).with_y(0.).normalize();
        assert!(root_pose.forward().as_vec3().dot(window) > 0.999);
        // Active skill and combo snapshots reposition the same performer on the
        // actual firing frame. The pool is the sole view allowed to draw her.
        for (kind, pair) in [
            (ActionKind::MusketShot, WeaponPair::HammerMusket),
            (ActionKind::ChainCombo, WeaponPair::ChainAxeBow),
        ] {
            let position = LAYOUT.sister + Vec3::X;
            *world.get_mut::<CompanionState>(performer).unwrap() = CompanionState {
                visible: true,
                position,
                previous_position: position,
                kind,
                pair,
                elapsed: 0.3,
                source_position: position + Vec3::new(0., 1.2, -0.55),
                aim_position: LAYOUT.window,
                ..default()
            };
            world.run_schedule(Update);
            assert_eq!(world.get::<Transform>(root).unwrap().translation, position);
            let weapons = world
                .query::<(&SisterWeapon, &Visibility)>()
                .iter(world)
                .filter(|(_, v)| **v == Visibility::Inherited)
                .map(|(w, _)| w.pair)
                .collect::<Vec<_>>();
            assert_eq!(weapons, vec![pair]);
            let mut skinned = 0;
            let mut rigid = 0;
            for entity in world.iter_entities().filter(|e| e.contains::<Mesh3d>()) {
                assert_eq!(entity.get::<RenderLayers>(), Some(&RenderLayers::layer(2)));
                assert!(entity.contains::<NotShadowCaster>());
                if entity.contains::<SkinnedMesh>() {
                    skinned += 1;
                } else {
                    rigid += 1;
                }
            }
            assert_eq!(skinned, 27);
            assert!(rigid > 0);
            world.get_mut::<CompanionState>(performer).unwrap().visible = false;
            world.run_schedule(Update);
            assert_eq!(
                world.get::<Transform>(root).unwrap().translation,
                LAYOUT.sister
            );
            assert_eq!(world.get::<Visibility>(root), Some(&Visibility::Visible));
            assert!(
                world
                    .query::<(&SisterWeapon, &Visibility)>()
                    .iter(world)
                    .all(|(_, v)| *v == Visibility::Hidden)
            );
            assert_eq!(world.entities().count_spawned(), n);
        }
    }
}
