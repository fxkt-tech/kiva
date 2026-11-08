use super::{
    animation::{self, RigPose},
    model::{self, Attachment, BoneVisual, CharacterModel},
    poses::{self, Gesture},
    rig::AnimationLibrary,
};
use crate::{
    client::{
        presentation::{self, PresentationFrame},
        scene, unit_appearance,
    },
    gameplay::MapDefinition,
    simulation::*,
};
use bevy::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, SimulationPlugin))
        .init_resource::<AnimationLibrary>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(NavigationMap::new(20.0, 1.0, vec![]))
        .insert_resource(MapDefinition::default())
        .init_resource::<PresentationFrame>()
        .add_systems(
            Startup,
            (scene::setup, model::setup, unit_appearance::setup).chain(),
        )
        .add_systems(
            Update,
            (
                presentation::prepare,
                scene::attach_unit_visuals,
                scene::sync_units,
                scene::sync_ground_bases,
                animation::record_events,
                animation::sync_poses,
                animation::sync_attachments,
            )
                .chain(),
        );
    app.update();
    app
}

fn spawn(
    app: &mut App,
    team: Team,
    position: Vec2,
    abilities: AbilityLoadout,
    stats: UnitStats,
) -> Entity {
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Rig test".into(),
            team,
            position,
            stats,
            abilities,
            respawn_delay: Some(0.1),
        },
    )
    .unwrap()
}

fn step(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
    app.world_mut().run_schedule(Update);
}

fn command(app: &mut App, entity: Entity, command: UnitCommand) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(entity, command);
}

fn gesture(app: &App, entity: Entity) -> Gesture {
    app.world().get::<RigPose>(entity).unwrap().gesture
}

fn weapon_socket(app: &mut App, owner: Entity) -> (Entity, Transform) {
    app.world_mut()
        .query::<(Entity, &BoneVisual, &Transform)>()
        .iter(app.world())
        .find(|(_, visual, _)| {
            visual.owner == owner
                && matches!(
                    visual.attachment,
                    Attachment::Sword { .. } | Attachment::Hand { .. }
                )
        })
        .map(|(entity, _, transform)| (entity, *transform))
        .unwrap()
}

#[test]
fn actual_champion_meshes_and_weapon_descendants_use_the_shared_model_picker() {
    use crate::client::{
        camera::GameCamera,
        picking::{self, PickGeometry, Pickable},
    };
    use bevy::camera::{CameraProjection, RenderTargetInfo, primitives::MeshAabb};
    let mut outside_root_profile = false;
    for character in [CharacterId(1), CharacterId(2)] {
        let mut app = app();
        crate::gameplay::prepare_world(app.world_mut());
        let hero = spawn_character(
            app.world_mut(),
            CharacterSpawnSpec {
                character,
                team: Team::Enemy,
                position: Vec2::ZERO,
                loadout: LoadoutSelection::CharacterDefault,
                respawn_delay: None,
            },
        )
        .unwrap();
        app.world_mut().run_schedule(Update);
        app.world_mut().run_schedule(PostUpdate);
        // The fixture has no renderer. Use the actual generated Mesh assets
        // for the same CPU bounds Bevy calculates, with visible model trees.
        let bounds: Vec<_> = app
            .world_mut()
            .query::<(Entity, &Mesh3d)>()
            .iter(app.world())
            .filter_map(|(id, mesh)| {
                app.world()
                    .resource::<Assets<Mesh>>()
                    .get(mesh)
                    .and_then(MeshAabb::get_aabb)
                    .map(|bounds| (id, bounds))
            })
            .collect();
        for (id, bounds) in bounds {
            app.world_mut().entity_mut(id).insert(bounds);
        }
        let visible: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<InheritedVisibility>>()
            .iter(app.world())
            .collect();
        for id in visible {
            *app.world_mut().get_mut::<InheritedVisibility>(id).unwrap() =
                InheritedVisibility::VISIBLE;
        }
        let root_model = app
            .world_mut()
            .query_filtered::<Entity, With<CharacterModel>>()
            .single(app.world())
            .unwrap();
        assert!(app.world().get::<PickGeometry>(root_model).is_some());
        let camera_transform =
            GlobalTransform::from(Transform::from_xyz(12.0, 1.0, 0.0).looking_at(Vec3::Y, Vec3::Y));
        let mut camera = Camera::default();
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: UVec2::new(1280, 800),
            scale_factor: 1.0,
        });
        camera.computed.clip_from_view = PerspectiveProjection {
            aspect_ratio: 1.6,
            ..default()
        }
        .get_clip_from_view();
        app.world_mut()
            .spawn((GameCamera, camera.clone(), camera_transform));
        let head = app
            .world_mut()
            .query::<(&BoneVisual, &GlobalTransform)>()
            .iter(app.world())
            .find(|(visual, _)| {
                visual.owner == hero && matches!(visual.attachment, Attachment::Head { .. })
            })
            .unwrap()
            .1
            .translation();
        let point = camera.world_to_viewport(&camera_transform, head).unwrap();
        assert_eq!(picking::candidates(app.world_mut(), point), vec![hero]);

        let profile = app.world().get::<Pickable>(hero).unwrap();
        let root = app.world().get::<GlobalTransform>(hero).unwrap();
        let mut profile_min = Vec2::splat(f32::INFINITY);
        let mut profile_max = Vec2::splat(f32::NEG_INFINITY);
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let world = root
                        .transform_point(profile.offset + profile.half_size * Vec3::new(x, y, z));
                    let point = camera.world_to_viewport(&camera_transform, world).unwrap();
                    profile_min = profile_min.min(point);
                    profile_max = profile_max.max(point);
                }
            }
        }
        let (socket, _) = weapon_socket(&mut app, hero);
        let mut descendants = vec![socket];
        let mut weapon_points = Vec::new();
        while let Some(id) = descendants.pop() {
            if let Some(children) = app.world().get::<Children>(id) {
                descendants.extend(children.iter());
            }
            if let Some(bounds) = app.world().get::<bevy::camera::primitives::Aabb>(id) {
                let transform = app.world().get::<GlobalTransform>(id).unwrap();
                for x in [-1.0, 1.0] {
                    for y in [-1.0, 1.0] {
                        for z in [-1.0, 1.0] {
                            let world = transform.transform_point(
                                Vec3::from(bounds.center)
                                    + Vec3::from(bounds.half_extents) * Vec3::new(x, y, z),
                            );
                            weapon_points
                                .push(camera.world_to_viewport(&camera_transform, world).unwrap());
                        }
                    }
                }
            }
        }
        let point = weapon_points
            .into_iter()
            .max_by(|a, b| {
                let outside = |p: Vec2| p.distance(p.clamp(profile_min, profile_max));
                outside(*a).total_cmp(&outside(*b))
            })
            .expect("actual spear/bow must have mesh descendants below its hand socket");
        outside_root_profile |= point.distance(point.clamp(profile_min, profile_max)) > 8.0;
        assert_eq!(
            picking::candidates(app.world_mut(), point),
            vec![hero],
            "actual {character:?} weapon endpoint must be selectable"
        );
    }
    assert!(
        outside_root_profile,
        "the real integration must include an endpoint outside the old root profile and padding"
    );
}

fn direct(effect: Effect) -> AbilityDefinition {
    AbilityDefinition {
        name: "Rig status".into(),
        targeting: Targeting::Enemy,
        range: 15.0,
        windup: 0.0,
        recovery: 0.0,
        cooldown: 0.0,
        mana_cost: 0.0,
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects: vec![effect],
        }),
    }
}

#[test]
fn gestures_preserve_bone_lengths_and_sword_socket() {
    let library = AnimationLibrary::default();
    let rest: Vec<_> = library.bones.iter().map(|bone| bone.rest).collect();
    let rest = library.model_pose(&rest);
    let hand = library.bone("hand_r");
    for gesture in [
        Gesture::Idle,
        Gesture::Run,
        Gesture::Windup,
        Gesture::Backswing,
        Gesture::Cast,
        Gesture::Channel,
        Gesture::Recovery,
        Gesture::Dash,
        Gesture::Airborne,
        Gesture::Stunned,
        Gesture::Forced,
    ] {
        for index in 0..=20 {
            let local = poses::sample(&library, gesture, index as f32 / 20.0, index as f32 * 0.03);
            assert!(local[0].translation.length() < 0.0001);
            let pose = library.model_pose(&local);
            for bone in &pose {
                assert!(bone.translation.is_finite() && bone.scale.is_finite());
                assert!((bone.rotation.length() - 1.0).abs() < 0.001);
            }
            for (a, b) in [("upperarm_r", "lowerarm_r"), ("lowerarm_r", "hand_r")] {
                let (a, b) = (library.bone(a), library.bone(b));
                assert!(
                    (pose[a].translation.distance(pose[b].translation)
                        - rest[a].translation.distance(rest[b].translation))
                    .abs()
                        < 0.001
                );
            }
            let socket = (Attachment::Sword { bone: hand }).transform(&pose);
            assert!(
                socket
                    .translation
                    .distance(pose[hand].transform_point(Vec3::Y * 0.055))
                    < 0.0001
            );
            assert!(socket.rotation.dot(pose[hand].rotation).abs() > 0.9999);
        }
    }
}

#[test]
fn attack_release_and_source_recovery_seams_preserve_bones_and_sword_socket() {
    let library = AnimationLibrary::default();
    let hand = library.bone("hand_r");
    for (before, after) in [
        (
            poses::sample(&library, Gesture::Windup, 1.0, 0.0),
            poses::sample(&library, Gesture::Backswing, 0.0, 0.0),
        ),
        (
            poses::sample(&library, Gesture::Backswing, 0.4 - 0.00001, 0.0),
            poses::sample(&library, Gesture::Backswing, 0.4 + 0.00001, 0.0),
        ),
    ] {
        let before = library.model_pose(&before);
        let after = library.model_pose(&after);
        for (index, (before, after)) in before.iter().zip(&after).enumerate() {
            assert!(
                before.translation.distance(after.translation) < 0.002,
                "bone {index} jumped"
            );
            assert!(
                before.rotation.dot(after.rotation).abs() > 0.99999,
                "bone {index} snapped"
            );
        }
        let before = (Attachment::Sword { bone: hand }).transform(&before);
        let after = (Attachment::Sword { bone: hand }).transform(&after);
        assert!(before.translation.distance(after.translation) < 0.002);
        assert!(before.rotation.dot(after.rotation).abs() > 0.99999);
    }
}

#[test]
fn all_units_get_one_rig_and_the_rendered_sword_follows_the_hand() {
    let mut app = app();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [None; 4],
        UnitStats::default(),
    );
    let enemy = spawn(
        &mut app,
        Team::Enemy,
        Vec2::X * 5.0,
        [None; 4],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    let count = app
        .world_mut()
        .query::<&CharacterModel>()
        .iter(app.world())
        .count();
    assert_eq!(count, 2);
    assert_eq!(
        app.world_mut()
            .query::<&BoneVisual>()
            .iter(app.world())
            .count(),
        66
    );
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world_mut()
            .query::<&CharacterModel>()
            .iter(app.world())
            .count(),
        count
    );
    for owner in [hero, enemy] {
        let (_, rendered) = weapon_socket(&mut app, owner);
        let library = app.world().resource::<AnimationLibrary>();
        let rig = app.world().get::<RigPose>(owner).unwrap();
        let expected = (Attachment::Sword {
            bone: library.bone("hand_r"),
        })
        .transform(&rig.model);
        assert_eq!(rendered, expected);
    }
    command(&mut app, hero, UnitCommand::AttackTarget(enemy));
    step(&mut app, 1);
    let (_, before) = weapon_socket(&mut app, hero);
    step(&mut app, 10);
    let (_, after) = weapon_socket(&mut app, hero);
    assert_ne!(before, after, "the actual visible hand socket must animate");
}

#[test]
fn locomotion_follows_travel_and_rooted_order_does_not_walk_in_place() {
    let mut app = app();
    let root = AbilityId(200);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            root,
            direct(Effect::ApplyStatus(StatusSpec::Root {
                duration: StatusDuration::from_millis(1000).unwrap(),
            })),
        )
        .unwrap();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [None; 4],
        UnitStats::default(),
    );
    let enemy = spawn(
        &mut app,
        Team::Enemy,
        Vec2::Y * 2.0,
        [Some(root), None, None, None],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    command(&mut app, hero, UnitCommand::MoveTo(Vec2::X * 10.0));
    step(&mut app, 1);
    assert_eq!(gesture(&app, hero), Gesture::Run);
    command(
        &mut app,
        enemy,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(hero),
        },
    );
    step(&mut app, 2);
    assert!(matches!(
        app.world().get::<UnitOrder>(hero),
        Some(UnitOrder::MoveTo(_))
    ));
    assert!(!app.world().get::<ControlState>(hero).unwrap().can_move());
    assert_eq!(gesture(&app, hero), Gesture::Idle);
}

#[test]
fn pause_retains_rendered_one_tick_backswing_and_stop_returns_to_idle() {
    let mut app = app();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [None; 4],
        UnitStats {
            base_attributes: BaseAttributes {
                attack_speed: MAX_ATTACK_SPEED,
                ..default()
            },
            ..default()
        },
    );
    let target = spawn(
        &mut app,
        Team::Enemy,
        Vec2::X * 2.0,
        [None; 4],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    command(&mut app, hero, UnitCommand::AttackTarget(target));
    step(&mut app, 1);
    assert_eq!(gesture(&app, hero), Gesture::Windup);
    step(&mut app, 1);
    assert!(matches!(
        app.world().get::<UnitAction>(hero),
        Some(UnitAction::AttackBackswing {
            remaining: 1,
            duration: 1
        })
    ));
    assert_eq!(gesture(&app, hero), Gesture::Backswing);
    let mut fixed = app.world_mut().resource_mut::<Time<Fixed>>();
    let overstep = fixed.timestep().mul_f32(0.9);
    fixed.accumulate_overstep(overstep);
    app.world_mut().run_schedule(Update);
    let (socket, rendered) = weapon_socket(&mut app, hero);
    let frozen = app.world().get::<RigPose>(hero).unwrap().local.clone();
    let clock = app.world().get::<RigPose>(hero).unwrap().last_time;
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    app.world_mut().run_schedule(Update);
    assert_eq!(*app.world().get::<Transform>(socket).unwrap(), rendered);
    assert_eq!(app.world().get::<RigPose>(hero).unwrap().local, frozen);
    assert_eq!(app.world().get::<RigPose>(hero).unwrap().last_time, clock);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .discard_overstep(overstep);
    app.world_mut().run_schedule(Update);
    assert_ne!(*app.world().get::<Transform>(socket).unwrap(), rendered);
    command(&mut app, hero, UnitCommand::Stop);
    step(&mut app, 1);
    assert_eq!(gesture(&app, hero), Gesture::Idle);
    step(&mut app, 12);
    let (_, neutral) = weapon_socket(&mut app, hero);
    assert_ne!(neutral, rendered);
}

#[test]
fn instant_dash_is_observed_without_casting_and_outlasts_recovery() {
    let mut app = app();
    let dash = AbilityId(201);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            dash,
            AbilityDefinition {
                name: "Instant dash".into(),
                targeting: Targeting::Point,
                range: 4.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Dash { speed: 12.0 },
            },
        )
        .unwrap();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [Some(dash), None, None, None],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    command(
        &mut app,
        hero,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Point(Vec2::X * 3.0),
        },
    );
    step(&mut app, 4);
    assert!(matches!(
        app.world().get::<UnitAction>(hero),
        Some(UnitAction::Ready)
    ));
    assert_eq!(gesture(&app, hero), Gesture::Dash);
    step(&mut app, 3);
    assert_eq!(gesture(&app, hero), Gesture::Dash);
    step(&mut app, 20);
    command(&mut app, hero, UnitCommand::MoveTo(Vec2::X * 6.0));
    step(&mut app, 1);
    assert_eq!(
        gesture(&app, hero),
        Gesture::Run,
        "a subsequent route is no longer a dash"
    );
}

#[test]
fn pause_before_first_dash_display_consumes_notice_without_reposing() {
    let mut app = app();
    let dash = AbilityId(205);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            dash,
            AbilityDefinition {
                name: "Paused dash".into(),
                targeting: Targeting::Point,
                range: 4.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Dash { speed: 12.0 },
            },
        )
        .unwrap();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [Some(dash), None, None, None],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    let (socket, rendered) = weapon_socket(&mut app, hero);
    let local = app.world().get::<RigPose>(hero).unwrap().local.clone();
    let time = app.world().get::<RigPose>(hero).unwrap().last_time;
    command(
        &mut app,
        hero,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Point(Vec2::X * 3.0),
        },
    );
    app.world_mut().run_schedule(FixedUpdate);
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    app.world_mut().run_schedule(Update);
    assert_eq!(*app.world().get::<Transform>(socket).unwrap(), rendered);
    assert_eq!(app.world().get::<RigPose>(hero).unwrap().local, local);
    assert_eq!(app.world().get::<RigPose>(hero).unwrap().last_time, time);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app.world_mut().run_schedule(Update);
    assert_eq!(
        gesture(&app, hero),
        Gesture::Dash,
        "the consumed commit notice survives pause"
    );
}

#[test]
fn stun_airborne_and_forced_motion_drive_bones_while_height_stays_on_the_root() {
    let mut app = app();
    let stun = AbilityId(206);
    let launch = AbilityId(207);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            stun,
            direct(Effect::ApplyStatus(StatusSpec::Stun {
                duration: StatusDuration::from_millis(1000).unwrap(),
            })),
        )
        .unwrap();
    let mut definition = direct(Effect::ApplyStatus(StatusSpec::Airborne {
        duration: StatusDuration::from_millis(800).unwrap(),
    }));
    if let AbilityExecution::Impact(impact) = &mut definition.execution {
        impact.effects.push(Effect::Displace(DisplacementSpec {
            distance: 1.0,
            speed: 10.0,
            direction: DisplacementDirection::AwayFromCenter,
        }));
    }
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(launch, definition)
        .unwrap();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [None; 4],
        UnitStats::default(),
    );
    let caster = spawn(
        &mut app,
        Team::Enemy,
        Vec2::Y * 2.0,
        [Some(stun), Some(launch), None, None],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    command(
        &mut app,
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(hero),
        },
    );
    step(&mut app, 1);
    assert_eq!(gesture(&app, hero), Gesture::Stunned);
    command(
        &mut app,
        caster,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::Unit(hero),
        },
    );
    step(&mut app, 4);
    assert_eq!(gesture(&app, hero), Gesture::Airborne);
    assert!(
        app.world()
            .get::<ControlState>(hero)
            .unwrap()
            .forced_motion()
    );
    assert!(app.world().get::<Transform>(hero).unwrap().translation.y > 0.0);
    let root = app.world().get::<Transform>(hero).unwrap().translation;
    let (_, socket) = weapon_socket(&mut app, hero);
    let rig = app.world().get::<RigPose>(hero).unwrap();
    let library = app.world().resource::<AnimationLibrary>();
    let expected = (Attachment::Sword {
        bone: library.bone("hand_r"),
    })
    .transform(&rig.model);
    assert_eq!(
        socket, expected,
        "socket is model-local; root adds airborne height once"
    );
    assert_eq!(
        app.world().get::<Transform>(hero).unwrap().translation,
        root
    );
}

#[test]
fn casting_channel_and_recovery_follow_authoritative_action_stages() {
    let mut app = app();
    let channel = AbilityId(202);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            channel,
            AbilityDefinition {
                name: "Visual channel".into(),
                targeting: Targeting::SelfCast,
                range: 0.0,
                windup: 0.15,
                recovery: 0.2,
                cooldown: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Channel {
                    duration: 0.3,
                    interval: 0.1,
                    pulse: ImpactDefinition {
                        delivery: Delivery::Area {
                            radius: 1.0,
                            delay: 0.0,
                        },
                        effects: vec![Effect::Damage(DamageSpec {
                            kind: DamageType::True,
                            base: 1.0,
                            ad_ratio: 0.0,
                            ap_ratio: 0.0,
                        })],
                    },
                },
            },
        )
        .unwrap();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [Some(channel), None, None, None],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    command(
        &mut app,
        hero,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert_eq!(gesture(&app, hero), Gesture::Cast);
    step(&mut app, 10);
    assert_eq!(gesture(&app, hero), Gesture::Channel);
    step(&mut app, 18);
    assert_eq!(gesture(&app, hero), Gesture::Recovery);
    step(&mut app, 20);
    assert_eq!(gesture(&app, hero), Gesture::Idle);
}

#[test]
fn commit_death_respawn_in_one_display_frame_drops_old_dash_and_reset_cleans_hierarchy() {
    let mut app = app();
    let dash = AbilityId(203);
    let kill = AbilityId(204);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            dash,
            AbilityDefinition {
                name: "Old life dash".into(),
                targeting: Targeting::Point,
                range: 4.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Dash { speed: 12.0 },
            },
        )
        .unwrap();
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            kill,
            direct(Effect::Damage(DamageSpec {
                kind: DamageType::True,
                base: 1000.0,
                ad_ratio: 0.0,
                ap_ratio: 0.0,
            })),
        )
        .unwrap();
    let hero = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [Some(dash), None, None, None],
        UnitStats::default(),
    );
    let enemy = spawn(
        &mut app,
        Team::Enemy,
        Vec2::Y * 2.0,
        [Some(kill), None, None, None],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    let old_life = app.world().get::<UnitState>(hero).unwrap().life_ref(hero);
    let (old_socket, _) = weapon_socket(&mut app, hero);
    command(
        &mut app,
        hero,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Point(Vec2::X * 3.0),
        },
    );
    command(
        &mut app,
        enemy,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(hero),
        },
    );
    step(&mut app, 12);
    let state = app.world().get::<UnitState>(hero).unwrap();
    assert!(state.alive());
    assert_ne!(state.life_ref(hero), old_life);
    assert_eq!(gesture(&app, hero), Gesture::Idle);
    assert_eq!(
        weapon_socket(&mut app, hero).0,
        old_socket,
        "respawn reuses one complete model"
    );
    assert_eq!(
        app.world().get::<UnitPose>(hero).unwrap().position(),
        Vec2::ZERO
    );
    reset_simulation(app.world_mut());
    assert_eq!(
        app.world_mut()
            .query::<&CharacterModel>()
            .iter(app.world())
            .count(),
        0
    );
    assert_eq!(
        app.world_mut()
            .query::<&BoneVisual>()
            .iter(app.world())
            .count(),
        0
    );
    assert!(app.world().get_entity(old_socket).is_err());
    spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        [None; 4],
        UnitStats::default(),
    );
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world_mut()
            .query::<&CharacterModel>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query::<&BoneVisual>()
            .iter(app.world())
            .count(),
        33
    );
}

#[test]
fn match_roles_use_separate_models_and_terminal_roots_and_bones_sync_once_then_freeze() {
    use crate::{
        client::{
            test_support,
            unit_appearance::{BaseCore, MinionModel},
        },
        gameplay::*,
    };
    let mut app = app();
    app.add_plugins(GameplayPlugin);
    prepare_world(app.world_mut());
    let handle = start_match(app.world_mut(), CharacterId(1)).unwrap();
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app.add_systems(
        Update,
        crate::client::unit_appearance::sync_simple_models.after(animation::sync_attachments),
    );
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world_mut()
            .query::<&CharacterModel>()
            .iter(app.world())
            .count(),
        2
    );
    assert_eq!(
        app.world_mut()
            .query::<&BoneVisual>()
            .iter(app.world())
            .count(),
        66
    );
    assert_eq!(
        app.world_mut()
            .query::<&MinionModel>()
            .iter(app.world())
            .count(),
        2
    );
    for _ in 0..150 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world_mut()
            .query::<&MinionModel>()
            .iter(app.world())
            .count(),
        12
    );
    assert_eq!(
        app.world_mut()
            .query::<&BaseCore>()
            .iter(app.world())
            .count(),
        2
    );
    for (role, appearance, pickable, rig) in app
        .world_mut()
        .query::<(
            &UnitRole,
            &crate::client::unit_appearance::UnitAppearance,
            &crate::client::picking::Pickable,
            Option<&RigPose>,
        )>()
        .iter(app.world())
    {
        assert_eq!(pickable.offset, appearance.pick_center);
        assert_eq!(pickable.half_size, appearance.pick_half_size);
        assert!(appearance.label_offset.y > appearance.pick_center.y + appearance.pick_half_size.y);
        assert_eq!(rig.is_some(), *role == UnitRole::Hero);
    }
    let before = app
        .world()
        .get::<Transform>(handle.player)
        .unwrap()
        .translation;
    let origin = app
        .world()
        .get::<UnitPose>(handle.player)
        .unwrap()
        .position();
    command(
        &mut app,
        handle.player,
        UnitCommand::MoveTo(origin + Vec2::X * 2.0),
    );
    test_support::finish_match(&mut app, Team::Ally);
    let point = app
        .world()
        .get::<UnitPose>(handle.player)
        .unwrap()
        .position();
    assert_ne!(point, origin);
    app.world_mut().run_schedule(Update);
    let final_root = *app.world().get::<Transform>(handle.player).unwrap();
    assert_ne!(final_root.translation, before);
    assert_eq!(final_root.translation, Vec3::new(point.x, 0.0, point.y));
    let enemy_base = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    assert_eq!(
        *app.world().get::<Visibility>(enemy_base).unwrap(),
        Visibility::Inherited
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<&Visibility, With<BaseCore>>()
            .iter(app.world())
            .filter(|visible| **visible == Visibility::Hidden)
            .count(),
        1
    );
    let local = app
        .world()
        .get::<RigPose>(handle.player)
        .unwrap()
        .local
        .clone();
    let time = app.world().get::<RigPose>(handle.player).unwrap().last_time;
    let (socket, sword) = weapon_socket(&mut app, handle.player);
    let overstep = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .mul_f32(0.9);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(overstep);
    for _ in 0..3 {
        app.world_mut().run_schedule(Update);
    }
    assert_eq!(
        *app.world().get::<Transform>(handle.player).unwrap(),
        final_root
    );
    assert_eq!(
        app.world().get::<RigPose>(handle.player).unwrap().local,
        local
    );
    assert_eq!(
        app.world().get::<RigPose>(handle.player).unwrap().last_time,
        time
    );
    assert_eq!(*app.world().get::<Transform>(socket).unwrap(), sword);
}

#[test]
fn spear_and_bow_keep_finite_animated_bones_and_hand_attachment_lengths() {
    use super::weapon::WeaponStyle;
    let library = AnimationLibrary::default();
    let rest = library.model_pose(&library.sample(super::rig::Clip::Idle, 0.0, false));
    for weapon in [WeaponStyle::Spear, WeaponStyle::Bow] {
        let before = poses::sample_weapon(&library, Gesture::Windup, 0.0, 0.0, weapon);
        let after = poses::sample_weapon(&library, Gesture::Windup, 1.0, 0.0, weapon);
        assert_ne!(
            before, after,
            "weapon users retain their actual attack gestures"
        );
        for frame in [
            before,
            after,
            poses::sample_weapon(&library, Gesture::Run, 0.5, 0.3, weapon),
        ] {
            let model = library.model_pose(&frame);
            for bone in &model {
                assert!(bone.translation.is_finite());
                assert!((bone.rotation.length() - 1.0).abs() < 0.001);
            }
            for (from, to) in [
                ("upperarm_r", "lowerarm_r"),
                ("lowerarm_r", "hand_r"),
                ("upperarm_l", "lowerarm_l"),
                ("lowerarm_l", "hand_l"),
            ] {
                let (a, b) = (library.bone(from), library.bone(to));
                assert!(
                    (model[a].translation.distance(model[b].translation)
                        - rest[a].translation.distance(rest[b].translation))
                    .abs()
                        < 0.001
                );
            }
        }
    }
}
