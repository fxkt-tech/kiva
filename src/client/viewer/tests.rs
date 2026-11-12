use super::*;
use crate::simulation::{
    ActionIntent, Body, CollisionWorld, CombatIntent, Combatant, Facing, Health, MotionState,
    MoveIntent, SimulationActive, SimulationPlugin, SpawnPoint, WeaponMode, WeaponPair,
};
use std::time::Duration;

fn session() -> Session {
    let library = AnimationLibrary::default();
    let source = SourceAnimationLibrary::bundled();
    Session::new(animation::preview_catalog(&library, &source))
}
fn app() -> App {
    app_with(false)
}
/// `server` adds the real asset server, so later selections load asynchronously.
fn app_with(server: bool) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::camera::visibility::VisibilityPlugin,
        loading::StartupPlugin,
    ));
    if server {
        app.add_plugins((
            bevy::asset::AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            },
            bundle::BundlePlugin,
        ));
    }
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<AnimationLibrary>()
        .init_resource::<SourceAnimationLibrary>()
        .init_resource::<Orbit>();
    setup(app.world_mut());
    app.add_systems(
        Update,
        (
            update,
            avatar::sync.run_if(skin_dirty),
            stickman::sync_attachments.run_if(skin_dirty),
            attachments.run_if(skin_dirty),
        )
            .chain(),
    );
    app.finish();
    app
}
fn apply(app: &mut App, command: &str) {
    app.world_mut().resource_mut::<Session>().apply(command);
    app.world_mut().run_schedule(Update);
}
fn asset_counts(world: &World) -> (usize, usize, usize, usize) {
    (
        world.resource::<Assets<Mesh>>().len(),
        world.resource::<Assets<StandardMaterial>>().len(),
        world.resource::<Assets<Image>>().len(),
        world
            .resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>()
            .len(),
    )
}
fn entity_count(world: &World) -> usize {
    world.iter_entities().count()
}

#[test]
fn commands_validate_and_seek_pause_without_resetting_configuration_time() {
    let mut s = session();
    s.apply("seek\t0.75");
    let frame = s.frame();
    s.apply("appearance\tbailingmiao");
    s.apply("weapon\tyinbingluo");
    assert_eq!(s.frame(), frame);
    assert!(!s.playing);
    assert_eq!(s.revision, 3);
    for command in [
        "speed\tNaN",
        "seek\tinf",
        "appearance\t../../secret",
        "weapon\t999",
        "action\tsource:nonexistent",
        "step\t2",
        "play\ttrue",
    ] {
        let before = (s.time, s.speed, s.action.clone(), s.appearance, s.weapon);
        s.apply(command);
        assert_eq!(
            (s.time, s.speed, s.action.clone(), s.appearance, s.weapon),
            before
        );
        assert!(!s.status.is_empty());
    }
    s.apply("seek\t-10");
    assert_eq!(s.time, 0.);
    s.apply("step\t1");
    assert_eq!(s.frame(), 1);
    s.apply("step\t-1");
    assert_eq!(s.frame(), 0);
    s.apply("seek\t3");
    s.apply("loop\t0");
    s.apply("play\t1");
    assert_eq!(s.time, 0.);
    s.advance(4.);
    assert!(!s.playing);
    assert_eq!(s.time, 3.);
    s.apply("mode\tsource");
    assert_eq!(s.mode, Mode::Source);
    s.apply("mode\treference");
    assert_eq!(s.duration, 0.);
    s.apply("mode\tgame");
    assert_eq!(s.action, "game:idle");
}
#[test]
fn wall_clock_partition_and_speed_do_not_change_canonical_frame() {
    let mut a = session();
    let mut b = session();
    a.apply("play\t1");
    b.apply("play\t1");
    b.apply("speed\t2");
    for _ in 0..60 {
        a.advance(1. / 60.);
    }
    for _ in 0..15 {
        b.advance(1. / 30.);
    }
    assert_eq!(a.frame(), b.frame());
    assert_eq!(a.frame(), 60);
}
#[test]
fn actual_assets_and_entity_counts_stabilize_across_repeated_body_and_weapon_switches() {
    let mut app = app();
    apply(&mut app, "seek\t0.75");
    for appearance in ["lihuowang", "bailingmiao", "choulingke"] {
        apply(&mut app, &format!("appearance\t{appearance}"));
    }
    for weapon in [
        "fulangmao",
        "zhexingcuishan",
        "yinbingluo",
        "yizhiying",
        "choulingke_blade",
        "none",
    ] {
        apply(&mut app, &format!("weapon\t{weapon}"));
    }
    apply(&mut app, "preset\tbrother_hammer");
    let counts = asset_counts(app.world());
    let entities = entity_count(app.world());
    for _ in 0..2 {
        for preset in [
            "sister_musket",
            "sister_bow",
            "enemy_blade",
            "brother_axe",
            "brother_hammer",
        ] {
            apply(&mut app, &format!("preset\t{preset}"));
            assert_eq!(app.world().resource::<Session>().frame(), 45);
        }
    }
    assert_eq!(asset_counts(app.world()), counts);
    assert_eq!(entity_count(app.world()), entities);
    assert!(app.world().get_resource::<SimulationActive>().is_none());
    assert_eq!(
        app.world_mut().query::<&Body>().iter(app.world()).count(),
        0
    );
    assert_eq!(
        app.world_mut()
            .query::<&super::super::ActorView>()
            .iter(app.world())
            .count(),
        0
    );
    let root = app.world().resource::<Scene>().root;
    assert!(app.world().get::<RigPose>(root).is_some());
}
#[test]
fn source_comparison_hides_hand_and_restores_actual_attachment_without_rebuilding_assets() {
    let mut app = app();
    apply(&mut app, "preset\tsister_bow");
    apply(&mut app, "seek\t0.7");
    let counts = asset_counts(app.world());
    let hand = app.world().resource::<Scene>().hand;
    let before = *app.world().get::<Transform>(hand).unwrap();
    apply(&mut app, "mode\tsource");
    assert_eq!(
        *app.world().get::<Visibility>(hand).unwrap(),
        Visibility::Hidden
    );
    apply(&mut app, "mode\tgame");
    apply(&mut app, "seek\t0.7");
    assert_eq!(
        *app.world().get::<Visibility>(hand).unwrap(),
        Visibility::Visible
    );
    assert_eq!(*app.world().get::<Transform>(hand).unwrap(), before);
    assert_eq!(asset_counts(app.world()), counts);
}

#[test]
fn source_mode_hides_real_weapon_meshes_after_selection_and_part_reshow() {
    let mut app = app();
    apply(&mut app, "preset\tsister_bow");
    apply(&mut app, "mode\tsource");
    for weapon in ["yinbingluo", "yizhiying"] {
        apply(&mut app, &format!("weapon\t{weapon}"));
        let mut scene = app.world_mut().remove_resource::<Scene>().unwrap();
        let mut session = app.world_mut().remove_resource::<Session>().unwrap();
        let hand_part = scene
            .parts
            .iter()
            .find(|part| part.label.starts_with("手持"))
            .unwrap()
            .id
            .clone();
        apply_parts(
            app.world_mut(),
            &mut scene,
            &mut session,
            vec![(hand_part.clone(), false), (hand_part, true)],
        );
        let hand = scene.hand;
        let body = body_meshes(app.world(), scene.root);
        app.world_mut().insert_resource(scene);
        app.world_mut().insert_resource(session);
        // Exercise Bevy's actual hierarchy propagation: checking only the
        // hidden parent misses descendants marked explicitly Visible.
        app.world_mut().run_schedule(PostUpdate);
        let mut meshes = Vec::new();
        descendants(app.world(), hand, &mut meshes);
        assert!(!meshes.is_empty());
        assert!(meshes.iter().all(|&entity| {
            !app.world()
                .get::<InheritedVisibility>(entity)
                .unwrap()
                .get()
        }));
        assert!(body.iter().any(|&entity| {
            app.world()
                .get::<InheritedVisibility>(entity)
                .unwrap()
                .get()
        }));
    }
}
#[test]
fn bounds_union_contains_current_rigid_attachment_and_excludes_stage_and_hidden_parts() {
    let mut world = World::new();
    let root = world.spawn(Transform::IDENTITY).id();
    let bounds = Aabb {
        center: Vec3::ZERO.into(),
        half_extents: Vec3::ONE.into(),
    };
    let body = world
        .spawn((
            Mesh3d::default(),
            bounds,
            GlobalTransform::from_xyz(0., 1., 0.),
            InheritedVisibility::VISIBLE,
        ))
        .id();
    let weapon = world
        .spawn((
            Mesh3d::default(),
            bounds,
            GlobalTransform::from_xyz(0., 3., 2.),
            InheritedVisibility::VISIBLE,
        ))
        .id();
    world.entity_mut(root).add_children(&[body, weapon]);
    world.spawn((
        Mesh3d::default(),
        bounds,
        GlobalTransform::from_scale(Vec3::splat(100.)),
        InheritedVisibility::VISIBLE,
    ));
    assert_eq!(
        current_bounds(&world, root),
        Some((Vec3::new(-1., 0., -1.), Vec3::new(1., 4., 3.)))
    );
    world.entity_mut(weapon).insert(InheritedVisibility::HIDDEN);
    assert_eq!(
        current_bounds(&world, root),
        Some((Vec3::new(-1., 0., -1.), Vec3::new(1., 2., 1.)))
    );
}

#[test]
fn actual_game_adapter_and_preview_drive_identical_real_palettes_and_weapon_mounts() {
    for appearance in [
        Appearance::Choulingke,
        Appearance::Lihuowang,
        Appearance::Bailingmiao,
    ] {
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, SimulationPlugin))
                .insert_resource(CollisionWorld::new(100., vec![]))
                .init_resource::<Assets<Mesh>>()
                .init_resource::<Assets<StandardMaterial>>()
                .init_resource::<AnimationLibrary>()
                .init_resource::<SourceAnimationLibrary>()
                .add_systems(
                    Update,
                    (animation::animate, avatar::sync, stickman::sync_attachments).chain(),
                );
            app.finish();
            let actor = app
                .world_mut()
                .spawn((
                    Body::new(Vec2::ZERO, 6., 0.35),
                    MoveIntent::default(),
                    ActionIntent::default(),
                    MotionState::default(),
                    Facing::default(),
                    Health::new(100.),
                    Combatant::new(pair),
                    CombatIntent::default(),
                    SpawnPoint(Vec2::ZERO),
                ))
                .id();
            let live = app
                .world_mut()
                .spawn((
                    super::super::ActorView { owner: actor },
                    Transform::IDENTITY,
                ))
                .id();
            avatar::spawn_with_appearance(app.world_mut(), live, appearance);
            let inspected = app.world_mut().spawn(Transform::IDENTITY).id();
            avatar::spawn_with_appearance(app.world_mut(), inspected, appearance);
            let live_weapon = app.world_mut().spawn(Transform::IDENTITY).id();
            let inspected_weapon = app.world_mut().spawn(Transform::IDENTITY).id();
            stickman::attach_weapon(app.world_mut(), live, live_weapon, WeaponMode::Melee);
            stickman::attach_weapon(
                app.world_mut(),
                inspected,
                inspected_weapon,
                WeaponMode::Melee,
            );
            app.world_mut()
                .get_mut::<CombatIntent>(actor)
                .unwrap()
                .request_attack();
            let dt = Duration::from_secs_f64(1. / 60.);
            let mut preview = PreviewRig::new(app.world().resource::<AnimationLibrary>());
            for frame in 1..=60 {
                app.world_mut().resource_mut::<Time<Fixed>>().advance_by(dt);
                app.world_mut().run_schedule(FixedUpdate);
                app.world_mut().resource_mut::<Time>().advance_by(dt);
                assert!(preview.seek(
                    app.world().resource::<AnimationLibrary>(),
                    app.world().resource::<SourceAnimationLibrary>(),
                    "game:attack",
                    pair,
                    false,
                    frame as f32 / 60.,
                    false
                ));
                app.world_mut()
                    .entity_mut(inspected)
                    .insert(preview.pose().clone());
                app.world_mut().run_schedule(Update);
                let joints = |world: &World, root: Entity| -> Vec<(String, Transform)> {
                    world
                        .get::<Children>(root)
                        .unwrap()
                        .iter()
                        .filter_map(|e| {
                            world.get::<avatar::PaletteJoint>(e).map(|_| {
                                (
                                    world.get::<Name>(e).unwrap().as_str().to_owned(),
                                    *world.get::<Transform>(e).unwrap(),
                                )
                            })
                        })
                        .collect()
                };
                let a = joints(app.world(), live);
                let b = joints(app.world(), inspected);
                assert_eq!(a.len(), b.len());
                for ((an, at), (bn, bt)) in a.iter().zip(b) {
                    assert_eq!(*an, bn);
                    assert!(
                        at.translation.distance(bt.translation) < 0.00001,
                        "{appearance:?} {pair:?} {frame} {an}"
                    );
                    assert!(at.rotation.dot(bt.rotation).abs() > 0.99999);
                }
                let live_mount = app.world().get::<Transform>(live_weapon).unwrap();
                let inspected_mount = app.world().get::<Transform>(inspected_weapon).unwrap();
                assert!(live_mount.translation.distance(inspected_mount.translation) < 0.00001);
                assert!(live_mount.rotation.dot(inspected_mount.rotation).abs() > 0.99999);
            }
        }
    }
}

#[test]
fn configuration_switch_restores_hidden_body_and_rejects_stale_part_identity() {
    let mut app = app();
    let mut scene = app.world_mut().remove_resource::<Scene>().unwrap();
    let mut session = app.world_mut().remove_resource::<Session>().unwrap();
    let old_id = scene.parts[0].id.clone();
    let hide = scene
        .parts
        .iter()
        .map(|part| (part.id.clone(), false))
        .collect();
    apply_parts(app.world_mut(), &mut scene, &mut session, hide);
    assert!(scene.parts.iter().all(|part| !part.visible));
    app.world_mut().insert_resource(scene);
    app.world_mut().insert_resource(session);
    apply(&mut app, "weapon\tzhexingcuishan");
    let mut scene = app.world_mut().remove_resource::<Scene>().unwrap();
    let mut session = app.world_mut().remove_resource::<Session>().unwrap();
    assert!(scene.parts.iter().all(|part| part.visible));
    assert!(
        scene
            .parts
            .iter()
            .flat_map(|part| &part.entities)
            .all(|&e| app.world().get::<Visibility>(e) == Some(&Visibility::Inherited))
    );
    assert!(!scene.parts.iter().any(|part| part.id == old_id));
    apply_parts(
        app.world_mut(),
        &mut scene,
        &mut session,
        vec![(old_id, false)],
    );
    assert!(scene.parts.iter().all(|part| part.visible));
    assert!(!session.status.is_empty());
    app.world_mut().insert_resource(scene);
    app.world_mut().insert_resource(session);
    // Reselecting an identical preset still begins a fresh selectable instance.
    apply(&mut app, "preset\tbrother_hammer");
    let old_revision = app.world().resource::<Session>().revision;
    apply(&mut app, "preset\tbrother_hammer");
    assert!(app.world().resource::<Session>().revision > old_revision);
    assert!(
        app.world()
            .resource::<Scene>()
            .parts
            .iter()
            .all(|part| part.visible)
    );
}

#[test]
fn paused_camera_and_noop_frames_keep_pose_and_bounds_cache_stable() {
    let mut app = app();
    apply(&mut app, "action\tgame:attack");
    apply(&mut app, "seek\t0.7");
    let root = app.world().resource::<Scene>().root;
    let pose = app.world().get::<RigPose>(root).unwrap().model.clone();
    app.world_mut().resource_mut::<Scene>().bounds_dirty = false;
    app.world_mut().resource_mut::<Orbit>().fit_requested = false;
    let counts = asset_counts(app.world());
    let evaluations = app.world().resource::<Scene>().bounds_evaluations;
    let started = std::time::Instant::now();
    for _ in 0..60 {
        app.world_mut()
            .resource_mut::<Orbit>()
            .rotate(Vec2::new(2., 1.));
        app.world_mut().run_schedule(Update);
        finish_view(app.world_mut());
    }
    eprintln!("60 paused camera frames: {:?}", started.elapsed());
    assert_eq!(app.world().get::<RigPose>(root).unwrap().model, pose);
    assert!(!app.world().resource::<Scene>().skin_dirty);
    assert!(!app.world().resource::<Scene>().bounds_dirty);
    assert_eq!(
        app.world().resource::<Scene>().bounds_evaluations,
        evaluations
    );
    assert_eq!(asset_counts(app.world()), counts);
}

#[test]
fn source_seek_to_loop_endpoint_retains_real_tail_sample() {
    let mut app = app();
    let id = app
        .world()
        .resource::<Session>()
        .catalog
        .iter()
        .find(|clip| clip.id.starts_with("cc0:") && clip.duration > 1.)
        .unwrap()
        .id
        .clone();
    apply(&mut app, &format!("action\t{id}"));
    let duration = app.world().resource::<Session>().duration;
    apply(&mut app, &format!("seek\t{duration}"));
    let root = app.world().resource::<Scene>().root;
    let trace = app.world().get::<RigPose>(root).unwrap().trace();
    assert!((trace.cc0_seconds - duration).abs() < 0.00001);
    assert!(app.world().resource::<Session>().looping);
    assert!(!app.world().resource::<Session>().playing);
}

#[test]
fn an_unloaded_selection_keeps_the_previous_model_and_revision_until_its_files_arrive() {
    let mut app = app_with(true);
    let shown = asset_counts(app.world());
    let registered = app.world().resource::<Scene>().registered_revision;
    apply(&mut app, "appearance\tbailingmiao");
    apply(&mut app, "weapon\tyizhiying");
    {
        let scene = app.world().resource::<Scene>();
        let session = app.world().resource::<Session>();
        assert_eq!(session.appearance, Appearance::Bailingmiao);
        assert!(scene.loading && !scene.load_failed);
        assert_eq!(scene.shown_appearance, Appearance::Lihuowang);
        assert_eq!(scene.shown_weapon, Weapon::Hammer);
        // The page keeps its "preparing" panel: this revision is unconfirmed.
        assert_ne!(session.revision, registered);
        assert_eq!(scene.registered_revision, registered);
        assert_eq!(session.status, "正在加载所选模型…");
    }
    assert_eq!(asset_counts(app.world()), shown);
    // The mounts and debug overlay keep using the model which is shown.
    app.world_mut().resource_mut::<Scene>().skin_dirty = true;
    app.world_mut().resource_mut::<Session>().debug.grips = true;
    app.world_mut().run_schedule(Update);
    finish_view(app.world_mut());

    for _ in 0..5_000 {
        app.update();
        if !app.world().resource::<Scene>().loading {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let scene = app.world().resource::<Scene>();
    let session = app.world().resource::<Session>();
    assert!(!scene.loading && !scene.load_failed);
    assert_eq!(scene.shown_appearance, Appearance::Bailingmiao);
    assert_eq!(scene.shown_weapon, Weapon::Bow);
    assert_eq!(scene.registered_revision, session.revision);
    assert_eq!(session.status, "");
}
