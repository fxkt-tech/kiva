use super::*;

fn actor(app: &mut App, team: Team, position: Vec2, damage: f32) -> Entity {
    actor_with_loadout(app, team, position, damage, [None; 4])
}

fn actor_with_loadout(
    app: &mut App,
    team: Team,
    position: Vec2,
    damage: f32,
    abilities: AbilityLoadout,
) -> Entity {
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Imported pose fixture".into(),
            team,
            position,
            stats: UnitStats {
                attack_delivery: BasicAttackDelivery::Direct,
                base_attributes: BaseAttributes {
                    attack_damage: damage,
                    attack_speed: MAX_ATTACK_SPEED,
                    attack_range: 4.0,
                    ..default()
                },
                ..default()
            },
            abilities,
            respawn_delay: Some(3.0),
        },
    )
    .unwrap()
}

fn instance(state: &UnitState, id: Entity) -> Instance {
    Instance {
        root: Entity::PLACEHOLDER,
        profile: 1,
        players: vec![],
        life: state.life_ref(id),
        phase: Phase::Idle,
        phase_total: 1.0,
        last_remaining: 0.0,
        clock: 0.0,
        last_time: 0.0,
        last_cast: None,
        bind_pose: vec![],
        root_joint: None,
    }
}

#[test]
fn real_death_completes_its_visual_fall_and_holds_without_shortening_respawn() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(20.0, 1.0, vec![]));
    let attacker = actor(&mut app, Team::Ally, Vec2::ZERO, 10000.0);
    let target = actor(&mut app, Team::Enemy, Vec2::X, 1.0);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(attacker, UnitCommand::AttackTarget(target));
    for _ in 0..16 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let state = app.world().get::<UnitState>(target).unwrap();
    assert!(!state.alive());
    let death = state.death_tick().unwrap() as f64 / TICK_RATE;
    let instance = instance(state, target);
    let source_duration = 3.3333333;
    let halfway = sample_time(
        Phase::Dead,
        source_duration,
        &instance,
        state,
        death + 0.325,
    );
    assert!((halfway - source_duration * 0.5).abs() < 1e-5);
    assert_eq!(
        sample_time(Phase::Dead, source_duration, &instance, state, death + 0.7),
        source_duration
    );
    assert_eq!(
        sample_time(Phase::Dead, source_duration, &instance, state, death + 2.0),
        source_duration
    );
    assert!(
        state.respawn_remaining().unwrap() > 2.0,
        "animation never owns the respawn countdown"
    );
}

#[test]
fn authoritative_attack_phase_seam_has_one_continuous_contact_pose() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(20.0, 1.0, vec![]));
    let target = actor(&mut app, Team::Ally, Vec2::ZERO, 1.0);
    let state = app.world().get::<UnitState>(target).unwrap();
    let mut instance = instance(state, target);
    instance.last_remaining = 0.0;
    let end_windup = sample_time(Phase::Windup, 2.0, &instance, state, 0.0);
    instance.last_remaining = instance.phase_total;
    let start_backswing = sample_time(Phase::Backswing, 2.0, &instance, state, 0.0);
    assert_eq!(end_windup, start_backswing);
    instance.last_remaining = 0.0;
    assert_eq!(
        sample_time(Phase::Backswing, 2.0, &instance, state, 0.0),
        2.0
    );
}

#[test]
fn completed_attack_between_display_frames_clears_the_older_cast_pose() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin))
        .add_systems(Update, record_events);
    crate::gameplay::prepare_world(app.world_mut());
    let source = actor_with_loadout(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        10000.0,
        [None, Some(AbilityId(12)), None, None],
    );
    let target = actor(&mut app, Team::Enemy, Vec2::X, 1.0);
    let mut model = instance(app.world().get::<UnitState>(source).unwrap(), source);
    model.profile = 0;
    app.world_mut().entity_mut(source).insert(model);
    app.world_mut().resource_mut::<CommandQueue>().submit(
        source,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::SelfCast,
        },
    );
    app.world_mut().run_schedule(FixedUpdate);
    app.world_mut().run_schedule(Update);
    let (_, cast_start) = app
        .world()
        .get::<Instance>(source)
        .unwrap()
        .last_cast
        .expect("the actual instant shield commit is remembered");

    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(source, UnitCommand::AttackTarget(target));
    // Do not display either the windup or the backswing. Killing the target
    // prevents a second attack from hiding the Ready boundary we need to test.
    for _ in 0..6 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(!app.world().get::<UnitState>(target).unwrap().alive());
    assert!(matches!(
        app.world().get::<UnitAction>(source),
        Some(UnitAction::Ready)
    ));
    let now = app.world().resource::<SimulationTick>().0 as f64 / TICK_RATE;
    assert!(
        now - cast_start < 0.45,
        "the old cast would still be eligible"
    );
    app.world_mut().run_schedule(Update);
    let model = app.world().get::<Instance>(source).unwrap();
    assert!(model.last_cast.is_none());
    let (phase, ..) = choose_phase(
        model,
        PhaseInput {
            state: app.world().get::<UnitState>(source).unwrap(),
            action: app.world().get::<UnitAction>(source).unwrap(),
            statuses: app.world().get::<UnitStatuses>(source).unwrap(),
            now,
            motion: 0.0,
            fraction: 0.0,
            abilities: app.world().resource::<AbilityCatalog>(),
        },
    );
    assert_eq!(
        phase,
        Phase::Idle,
        "a skipped attack must not revive a cast pose"
    );
}

#[test]
fn paused_animation_evaluation_keeps_the_committed_leap_root_correction() {
    use crate::client::{
        motion_view::{self, MotionView},
        presentation,
    };
    use bevy::{
        animation::{AnimatedBy, AnimationTargetId, animated_field},
        asset::AssetPlugin,
    };

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        AnimationPlugin,
        SimulationPlugin,
    ))
    .init_resource::<PresentationFrame>()
    .init_resource::<MotionView>()
    .add_systems(Update, (presentation::prepare, motion_view::sample).chain())
    .add_systems(
        PostUpdate,
        keep_committed_motion_root
            .after(AnimationSystems)
            .before(bevy::transform::TransformSystems::Propagate),
    );
    crate::gameplay::prepare_world(app.world_mut());
    app.update();
    let source = spawn_character(
        app.world_mut(),
        CharacterSpawnSpec {
            character: CharacterId(1),
            team: Team::Ally,
            position: Vec2::ZERO,
            loadout: LoadoutSelection::CharacterDefault,
            respawn_delay: None,
        },
    )
    .unwrap();
    let target = spawn_character(
        app.world_mut(),
        CharacterSpawnSpec {
            character: CharacterId(2),
            team: Team::Enemy,
            position: Vec2::X * 4.0,
            loadout: LoadoutSelection::CharacterDefault,
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut().resource_mut::<CommandQueue>().submit(
        source,
        UnitCommand::CastAbility {
            slot: 3,
            target: AbilityTarget::Unit(target),
        },
    );
    for _ in 0..5 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    app.world_mut().run_schedule(Update);
    let height = app.world().resource::<MotionView>().height(source);
    assert!(
        height > 0.0,
        "the real committed leap must own the display height"
    );

    let bind_height = 0.75;
    let root_name = Name::new("Root");
    let target_id = AnimationTargetId::from_name(&root_name);
    let mut clip = AnimationClip::default();
    clip.add_curve_to_target(
        target_id,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            UnevenSampleAutoCurve::new([
                (0.0, Vec3::new(1.0, 4.0, 2.0)),
                (1.0, Vec3::new(3.0, 8.0, 6.0)),
            ])
            .unwrap(),
        ),
    );
    let clip = app
        .world_mut()
        .resource_mut::<Assets<AnimationClip>>()
        .add(clip);
    let (graph, node) = AnimationGraph::from_clip(clip);
    let graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node).pause().set_seek_time(0.5);
    let root = app
        .world_mut()
        .spawn((
            root_name,
            target_id,
            AnimatedBy(source),
            Transform::from_xyz(0.0, bind_height, 0.0),
            ChildOf(source),
        ))
        .id();
    let mut model = instance(app.world().get::<UnitState>(source).unwrap(), source);
    model.root_joint = Some((root, bind_height));
    app.world_mut().entity_mut(source).insert((
        model,
        player,
        AnimationGraphHandle(graph),
        Transform::from_xyz(0.0, height, 0.0),
    ));
    // The first PostUpdate publishes the new graph's asset event; the second
    // threads the graph and evaluates the actual paused AnimationPlayer.
    app.world_mut().run_schedule(PostUpdate);
    app.world_mut().run_schedule(PostUpdate);
    let shown = *app.world().get::<Transform>(root).unwrap();
    assert_eq!(shown.translation, Vec3::new(2.0, bind_height, 4.0));
    let shown_global = *app.world().get::<GlobalTransform>(root).unwrap();
    let tick = app.world().resource::<SimulationTick>().0;

    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    for _ in 0..3 {
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut().run_schedule(Update);
        assert!(!app.world().resource::<PresentationFrame>().should_sync());
        app.world_mut().run_schedule(PostUpdate);
        assert_eq!(*app.world().get::<Transform>(root).unwrap(), shown);
        assert_eq!(
            *app.world().get::<GlobalTransform>(root).unwrap(),
            shown_global
        );
        assert_eq!(app.world().resource::<SimulationTick>().0, tick);
    }
}
