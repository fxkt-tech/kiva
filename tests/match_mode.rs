use bevy::prelude::*;
use kiva::{gameplay::*, simulation::*};

fn arena(definition: MatchDefinition) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin, GameplayPlugin))
        .insert_resource(definition);
    prepare_world(app.world_mut());
    app
}

fn begin(app: &mut App) -> MatchHandle {
    let handle = start_match(app.world_mut(), CharacterId(1)).unwrap();
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    handle
}

fn step(app: &mut App, count: usize) {
    // No Update is run between these battle ticks. This also exercises terminal
    // gating when one display frame contains several fixed steps.
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}

fn submit(app: &mut App, unit: Entity, command: UnitCommand) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(unit, command);
}

fn roots(app: &mut App) -> usize {
    app.world_mut().query::<&Unit>().iter(app.world()).count()
}

fn minions_in_wave(app: &mut App, wave: u64) -> [usize; 4] {
    let mut counts = [0; 4];
    for (unit, role, member) in app
        .world_mut()
        .query::<(&Unit, &UnitRole, &WaveMember)>()
        .iter(app.world())
    {
        if member.wave() != wave {
            continue;
        }
        let team = if unit.team() == Team::Ally { 0 } else { 2 };
        let kind = match role {
            UnitRole::MeleeMinion => 0,
            UnitRole::RangedMinion => 1,
            _ => panic!("non-minion in a wave"),
        };
        counts[team + kind] += 1;
        assert!((member.slot() < 3) == (*role == UnitRole::MeleeMinion));
    }
    counts
}

fn actor(
    app: &mut App,
    team: Team,
    position: Vec2,
    stats: UnitStats,
    abilities: AbilityLoadout,
) -> Entity {
    let stats = UnitStats {
        vision_radius: 13.5,
        ..stats
    };
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Mode test actor".into(),
            team,
            position,
            stats,
            abilities,
            respawn_delay: None,
        },
    )
    .unwrap()
}

fn base_striker(app: &mut App, team: Team, target: Entity) -> Entity {
    let point = app.world().get::<UnitPose>(target).unwrap().position() + Vec2::X * 3.0;
    actor(
        app,
        team,
        point,
        UnitStats {
            attack_delivery: BasicAttackDelivery::Direct,
            base_attributes: BaseAttributes {
                attack_damage: 10000.0,
                attack_speed: MAX_ATTACK_SPEED,
                attack_range: 4.0,
                ..default()
            },
            ..default()
        },
        [None; 4],
    )
}

fn damage_ability(targeting: Targeting, delivery: Delivery, damage: f32) -> AbilityDefinition {
    AbilityDefinition {
        name: "Mode fixture impact".into(),
        targeting,
        range: 12.0,
        windup: 0.0,
        recovery: 0.0,
        cooldown: 0.0,
        mana_cost: 0.0,
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery,
            effects: vec![Effect::Damage(DamageSpec {
                kind: DamageType::True,
                base: damage,
                ad_ratio: 0.0,
                ap_ratio: 0.0,
            })],
        }),
    }
}

#[test]
fn start_builds_two_bases_two_heroes_and_first_minion_pair_at_tick_zero() {
    let mut app = arena(MatchDefinition::default());
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Waiting
    );
    step(&mut app, 8);
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    let handle = begin(&mut app);
    assert_eq!(handle.id.to_string(), "1");
    assert_eq!(roots(&mut app), 6);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    let state = app.world().resource::<MatchState>();
    assert_eq!(state.id(), Some(handle.id));
    assert_eq!(state.hero(Team::Ally), Some(handle.player));
    assert_eq!(state.wave_number(), 1);
    assert_eq!(state.next_wave_tick(), Some(3600));
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    let map = app.world().resource::<MapDefinition>();
    assert_eq!(map.half_size(), 32.0);
    assert_eq!(map.lane_width(), 7.0);
    assert_eq!(map.base_position(Team::Ally), Vec2::new(-8.0, 24.0));
    assert_eq!(map.base_position(Team::Enemy), Vec2::new(8.0, -24.0));
    let definition = app.world().resource::<MatchDefinition>();
    assert_eq!(definition.melee_stats().max_health, 465.0);
    assert_eq!(definition.ranged_stats().max_health, 284.0);
    assert_eq!(definition.minion_spawn_interval_ticks(), 30);
    for team in [Team::Ally, Team::Enemy] {
        let hero = state.hero(team).unwrap();
        let base = state.base(team).unwrap();
        assert_eq!(
            app.world().get::<Unit>(base).unwrap().class(),
            UnitClass::Structure
        );
        assert_eq!(app.world().get::<UnitState>(base).unwrap().health(), 3000.0);
        assert_eq!(app.world().get::<Unit>(hero).unwrap().team(), team);
        assert!(app.world().get::<CharacterIdentity>(hero).is_some());
        assert_eq!(app.world().get::<UnitRole>(base), Some(&UnitRole::Base));
    }
    assert_eq!(
        app.world().get::<Controller>(handle.player),
        Some(&Controller::Human)
    );
}

#[test]
fn hero_spawns_fit_existing_camera_bounds_recovery_range_and_creep_spacing() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 150);
    let camera_bound = app.world().resource::<MapDefinition>().half_size() - 5.0;
    let recovery_radius = app.world().resource::<MatchDefinition>().recovery().radius;
    for team in [Team::Ally, Team::Enemy] {
        let hero = app.world().resource::<MatchState>().hero(team).unwrap();
        let base = app.world().resource::<MatchState>().base(team).unwrap();
        let point = app.world().get::<UnitPose>(hero).unwrap().position();
        let origin = app.world().get::<UnitPose>(base).unwrap().position();
        let radius = app.world().get::<Unit>(hero).unwrap().definition().radius;
        assert!(point.abs().cmplt(Vec2::splat(camera_bound)).all());
        assert!(point.distance(origin) < recovery_radius);
        for (unit, role, pose) in app
            .world_mut()
            .query::<(&Unit, &UnitRole, &UnitPose)>()
            .iter(app.world())
        {
            if unit.team() == team && matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion)
            {
                assert!(point.distance(pose.position()) > radius + unit.definition().radius);
            }
        }
    }
}

#[test]
fn ordered_symmetric_pairs_emit_only_at_their_exact_simulation_ticks() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    let mut previous_tick = 0;
    for tick in [0, 29, 30, 59, 60, 89, 90, 119, 120, 149, 150, 179] {
        step(&mut app, tick - previous_tick);
        previous_tick = tick;
        let emitted = (tick / 30 + 1).min(6);
        assert_eq!(roots(&mut app), 4 + emitted * 2, "at tick{tick}");
        let map = app.world().resource::<MapDefinition>().clone();
        let mut seen = [[false; 6]; 2];
        for (unit, state, pose, member) in app
            .world_mut()
            .query::<(&Unit, &UnitState, &UnitPose, &WaveMember)>()
            .iter(app.world())
        {
            let team = usize::from(unit.team() == Team::Enemy);
            let slot = usize::from(member.slot());
            assert!(!seen[team][slot], "duplicate slot at tick{tick}");
            seen[team][slot] = true;
            assert_eq!(member.wave(), 1);
            assert!(slot < emitted);
            assert_eq!(pose.position(), map.creep_spawns(unit.team())[slot]);
            assert_eq!(state.health(), if slot < 3 { 465.0 } else { 284.0 });
        }
        for slots in seen {
            assert!(slots[..emitted].iter().all(|seen| *seen));
            assert!(slots[emitted..].iter().all(|seen| !*seen));
        }
    }
    assert_eq!(
        app.world().resource::<MatchState>().next_wave_tick(),
        Some(3600)
    );
}

#[test]
fn paused_partial_wave_preserves_its_next_pair_without_wall_clock_catchup() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 29);
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    step(&mut app, 5000);
    assert_eq!(app.world().resource::<SimulationTick>().0, 29);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 1);
    assert_eq!(minions_in_wave(&mut app, 1), [2, 0, 2, 0]);
    step(&mut app, 1);
    assert_eq!(roots(&mut app), 8);
}

#[test]
fn invalid_or_overlapping_emission_schedules_are_rejected_without_allocating_roots() {
    for definition in [
        MatchDefinition::default().with_minion_spawn_interval_ticks(0),
        MatchDefinition::default().with_wave_interval_ticks(150),
        MatchDefinition::default().with_minion_spawn_interval_ticks(u64::MAX),
        MatchDefinition::default().with_minion_spawn_interval_ticks(u64::MAX / 5 + 1),
    ] {
        let mut app = arena(definition);
        assert!(matches!(
            start_match(app.world_mut(), CharacterId(1)),
            Err(MatchError::InvalidDefinition(_))
        ));
        assert_eq!(roots(&mut app), 0);
        assert_eq!(app.world().resource::<MatchState>().id(), None);
        app.world_mut().insert_resource(MatchDefinition::default());
        assert_eq!(begin(&mut app).id.to_string(), "1");
    }
}

#[test]
fn opening_preflight_checks_later_slots_before_creating_the_first_pair() {
    let mut app = arena(MatchDefinition::default());
    let map = app.world().resource::<MapDefinition>().clone();
    app.world_mut().insert_resource(NavigationMap::new(
        map.half_size(),
        1.0,
        vec![Obstacle {
            center: map.creep_spawns(Team::Enemy)[5],
            half_size: Vec2::splat(0.1),
        }],
    ));
    assert!(matches!(
        start_match(app.world_mut(), CharacterId(1)),
        Err(MatchError::SpawnFailed(_))
    ));
    assert_eq!(roots(&mut app), 0);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Waiting
    );
}

#[test]
fn a_later_pair_failure_preserves_earlier_minions_and_never_creates_half_a_pair() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 29);
    let earlier = app
        .world_mut()
        .query_filtered::<Entity, With<WaveMember>>()
        .iter(app.world())
        .collect::<Vec<_>>();
    let map = app.world().resource::<MapDefinition>().clone();
    app.world_mut().insert_resource(NavigationMap::new(
        map.half_size(),
        1.0,
        vec![Obstacle {
            center: map.creep_spawns(Team::Enemy)[1],
            half_size: Vec2::splat(0.1),
        }],
    ));
    step(&mut app, 1);
    assert_eq!(app.world().resource::<SimulationTick>().0, 30);
    assert!(app.world().resource::<MatchState>().wave_error().is_some());
    assert!(!app.world().resource::<MatchState>().can_advance());
    assert!(app.world().resource::<SimulationControl>().paused);
    assert_eq!(roots(&mut app), 6);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    assert!(
        earlier
            .iter()
            .all(|entity| app.world().get_entity(*entity).is_ok())
    );
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 200);
    assert_eq!(app.world().resource::<SimulationTick>().0, 30);
    assert_eq!(roots(&mut app), 6);
    reset_match(app.world_mut());
    app.world_mut().insert_resource(map.navigation());
    begin(&mut app);
    step(&mut app, 150);
    assert_eq!(minions_in_wave(&mut app, 1), [3, 3, 3, 3]);
}

#[test]
fn factory_failure_after_the_first_root_rolls_back_only_the_current_pair() {
    #[derive(Resource, Default)]
    struct AllocatedPairRoots(Vec<Entity>);

    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 29);
    let map = app.world().resource::<MapDefinition>().clone();
    let blocked = NavigationMap::new(
        map.half_size(),
        1.0,
        vec![Obstacle {
            center: map.creep_spawns(Team::Enemy)[1],
            half_size: Vec2::splat(0.1),
        }],
    );
    app.init_resource::<AllocatedPairRoots>();
    // A real ECS observer changes public navigation after the first factory
    // allocates, so whole-pair preflight succeeds but the second factory fails.
    // This exercises rollback without private state writes or a production hook.
    app.world_mut().add_observer(
        move |added: On<Add<UnitRole>>,
              units: Query<&Unit>,
              mut navigation: ResMut<NavigationMap>,
              mut allocated: ResMut<AllocatedPairRoots>| {
            if units.get(added.entity).unwrap().team() == Team::Ally {
                allocated.0.push(added.entity);
                *navigation = blocked.clone();
            }
        },
    );
    step(&mut app, 1);
    let allocated = &app.world().resource::<AllocatedPairRoots>().0;
    assert_eq!(allocated.len(), 1);
    assert!(app.world().get_entity(allocated[0]).is_err());
    assert_eq!(roots(&mut app), 6);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    assert!(app.world().resource::<MatchState>().wave_error().is_some());
    assert!(!app.world().resource::<MatchState>().can_advance());
    step(&mut app, 150);
    assert_eq!(app.world().resource::<SimulationTick>().0, 30);
    assert_eq!(roots(&mut app), 6);
}

#[test]
fn victory_on_a_pending_pair_tick_cancels_the_pair_and_all_later_emissions() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 28);
    let enemy = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let striker = base_striker(&mut app, Team::Ally, enemy);
    submit(&mut app, striker, UnitCommand::AttackTarget(enemy));
    step(&mut app, 200);
    assert_eq!(app.world().resource::<SimulationTick>().0, 30);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Finished(MatchOutcome::Winner(Team::Ally))
    );
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 200);
    assert_eq!(app.world().resource::<SimulationTick>().0, 30);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
}

#[test]
fn waves_use_simulation_time_without_repeats_or_pause_catchup() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 3599);
    assert_eq!(app.world().resource::<MatchState>().wave_number(), 1);
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    step(&mut app, 5000);
    assert_eq!(app.world().resource::<SimulationTick>().0, 3599);
    assert_eq!(app.world().resource::<MatchState>().wave_number(), 1);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 1);
    assert_eq!(app.world().resource::<SimulationTick>().0, 3600);
    assert_eq!(minions_in_wave(&mut app, 2), [1, 0, 1, 0]);
    assert_eq!(
        app.world().resource::<MatchState>().next_wave_tick(),
        Some(7200)
    );
    step(&mut app, 1);
    assert_eq!(minions_in_wave(&mut app, 2), [1, 0, 1, 0]);
    step(&mut app, 149);
    assert_eq!(minions_in_wave(&mut app, 2), [3, 3, 3, 3]);
    step(&mut app, 3450);
    assert_eq!(app.world().resource::<MatchState>().wave_number(), 3);
    assert_eq!(minions_in_wave(&mut app, 3), [1, 0, 1, 0]);
}

#[test]
fn prepare_is_idempotent_and_start_cannot_replace_a_live_match() {
    let mut app = arena(MatchDefinition::default());
    let first = begin(&mut app);
    let item = equip_equipment(app.world_mut(), first.player, 0, EquipmentId(1)).unwrap();
    step(&mut app, 4);
    prepare_world(app.world_mut());
    assert_eq!(app.world().resource::<SimulationTick>().0, 4);
    assert_eq!(app.world().resource::<MatchState>().id(), Some(first.id));
    assert_eq!(
        app.world()
            .get::<EquipmentSlots>(first.player)
            .unwrap()
            .item(0)
            .unwrap()
            .instance(),
        item
    );
    assert_eq!(
        start_match(app.world_mut(), CharacterId(2)),
        Err(MatchError::AlreadyStarted)
    );
    assert_eq!(roots(&mut app), 6);
}

#[test]
fn invalid_start_never_publishes_a_partial_roster_or_consumes_match_identity() {
    let mut app = arena(MatchDefinition::default().with_opponent(CharacterId(600)));
    assert_eq!(
        start_match(app.world_mut(), CharacterId(1)),
        Err(MatchError::UnknownCharacter(CharacterId(600)))
    );
    assert_eq!(roots(&mut app), 0);
    assert_eq!(app.world().resource::<MatchState>().id(), None);
    app.world_mut()
        .insert_resource(MatchDefinition::default().with_wave_interval_ticks(0));
    assert!(matches!(
        start_match(app.world_mut(), CharacterId(1)),
        Err(MatchError::InvalidDefinition(_))
    ));
    assert_eq!(roots(&mut app), 0);
    app.world_mut().insert_resource(MatchDefinition::default());
    assert_eq!(begin(&mut app).id.to_string(), "1");
}

#[test]
fn invalid_spawn_geometry_rejects_every_unit_before_construction() {
    let mut app = App::new();
    let map = MapDefinition::new(
        16.0,
        7.0,
        [Vec2::new(-15.5, 12.0), Vec2::new(4.0, -12.0)],
        vec![],
    )
    .unwrap();
    app.add_plugins((MinimalPlugins, SimulationPlugin, GameplayPlugin))
        .insert_resource(map);
    prepare_world(app.world_mut());
    assert!(matches!(
        start_match(app.world_mut(), CharacterId(1)),
        Err(MatchError::SpawnFailed(_))
    ));
    assert_eq!(roots(&mut app), 0);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Waiting
    );
}

#[test]
fn a_real_hero_can_approach_and_damage_the_walkable_base_target() {
    let mut app = arena(MatchDefinition::default());
    let hero = begin(&mut app).player;
    let base = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    assert!(!team_can_see(app.world(), Team::Ally, base));
    submit(&mut app, hero, UnitCommand::AttackTarget(base));
    step(&mut app, 1);
    assert!(matches!(
        app.world().get::<UnitOrder>(hero),
        Some(UnitOrder::Idle)
    ));
    // Static map geometry is known even in fog. Move there through the real
    // navigation command, then acquire the base only after it is visible.
    let map = app.world().resource::<MapDefinition>();
    let approach = map.base_position(Team::Enemy) - map.lane_direction(Team::Ally) * 3.0;
    submit(&mut app, hero, UnitCommand::MoveTo(approach));
    step(&mut app, 1200);
    assert!(
        app.world()
            .get::<UnitPose>(hero)
            .unwrap()
            .position()
            .distance(approach)
            < 0.1
    );
    assert!(team_can_see(app.world(), Team::Ally, base));
    submit(&mut app, hero, UnitCommand::AttackTarget(base));
    step(&mut app, 120);
    let health = app.world().get::<UnitState>(base).unwrap().health();
    assert!(
        health > 0.0 && health < 3000.0,
        "normal hero attacks must reach and damage the base: {health}"
    );
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Running
    );
}

#[test]
fn a_temporary_ring_across_the_next_wave_spawn_cannot_freeze_or_skip_the_pair() {
    let mut app = arena(MatchDefinition::default());
    let hero = begin(&mut app).player;
    let map = app.world().resource::<MapDefinition>().clone();
    let target = actor(
        &mut app,
        Team::Enemy,
        map.base_position(Team::Ally) + map.lane_direction(Team::Ally) * 0.5,
        UnitStats {
            kind: UnitKind::Hero,
            ..default()
        },
        [None; 4],
    );
    step(&mut app, 3578);
    submit(
        &mut app,
        hero,
        UnitCommand::CastAbility {
            slot: 3,
            target: AbilityTarget::Unit(target),
        },
    );
    step(&mut app, 21);
    let spawn = map.creep_spawns(Team::Ally)[0];
    let radius = app
        .world()
        .resource::<MatchDefinition>()
        .melee_stats()
        .radius;
    assert!(
        app.world_mut()
            .query::<&BattlefieldObject>()
            .iter(app.world())
            .any(|object| object.kind() == ObjectKind::Terrain && object.active(3599))
    );
    assert!(
        !app.world()
            .resource::<NavigationMap>()
            .walkable(spawn, radius)
    );
    step(&mut app, 1);
    let mode = app.world().resource::<MatchState>();
    assert!(
        mode.wave_error().is_none(),
        "a temporary wall is normal gameplay, not a configuration error: {:?}",
        mode.wave_error()
    );
    assert_eq!(mode.wave_number(), 2);
    assert!(!app.world().resource::<SimulationControl>().paused);
    assert_eq!(app.world().resource::<SimulationTick>().0, 3600);
    assert_eq!(minions_in_wave(&mut app, 2), [1, 0, 1, 0]);
    let newborn = app
        .world_mut()
        .query::<(Entity, &Unit, &WaveMember)>()
        .iter(app.world())
        .find(|(_, unit, member)| {
            unit.team() == Team::Ally && member.wave() == 2 && member.slot() == 0
        })
        .map(|(entity, _, _)| entity)
        .unwrap();
    let relocated = app.world().get::<UnitPose>(newborn).unwrap().position();
    assert!(
        relocated.distance(spawn) > 0.1 && relocated.distance(spawn) < 2.0,
        "a temporary collision should cause a local factory placement, not a missing or teleported minion"
    );
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .walkable(relocated, radius)
    );

    step(&mut app, 150);
    assert_eq!(app.world().resource::<SimulationTick>().0, 3750);
    assert_eq!(minions_in_wave(&mut app, 2), [3, 3, 3, 3]);
    assert!(app.world().resource::<MatchState>().can_advance());
    let inner_spawn = map.creep_spawns(Team::Ally)[3];
    let inner_radius = app
        .world()
        .resource::<MatchDefinition>()
        .ranged_stats()
        .radius;
    let navigation = app.world().resource::<NavigationMap>();
    assert!(navigation.walkable(inner_spawn, inner_radius));
    assert!(
        navigation
            .path(inner_spawn, map.base_position(Team::Enemy), inner_radius)
            .is_err(),
        "an actor inside the still-active ring is allowed to wait for its temporary route to reopen"
    );
    for (unit, pose, member) in app
        .world_mut()
        .query::<(&Unit, &UnitPose, &WaveMember)>()
        .iter(app.world())
    {
        if member.wave() == 2 {
            assert!(
                app.world()
                    .resource::<NavigationMap>()
                    .walkable(pose.position(), unit.definition().radius)
            );
            if unit.team() == Team::Enemy {
                assert_eq!(
                    pose.position(),
                    map.creep_spawns(Team::Enemy)[usize::from(member.slot())],
                    "temporary terrain on one side cannot move the unaffected opposing formation"
                );
            }
        }
    }
    step(&mut app, 61);
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .path(inner_spawn, map.base_position(Team::Enemy), inner_radius)
            .is_ok()
    );
    assert!(app.world().resource::<MatchState>().can_advance());
    assert_eq!(minions_in_wave(&mut app, 2), [3, 3, 3, 3]);
}

#[test]
fn a_terminal_tick_stops_future_fixed_steps_even_if_pause_is_cleared() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    let base = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let striker = base_striker(&mut app, Team::Ally, base);
    submit(&mut app, striker, UnitCommand::AttackTarget(base));
    step(&mut app, 20);
    assert_eq!(app.world().resource::<SimulationTick>().0, 2);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Finished(MatchOutcome::Winner(Team::Ally))
    );
    assert_eq!(
        app.world().resource::<MatchState>().finished_tick(),
        Some(2)
    );
    assert!(app.world().resource::<SimulationControl>().paused);
    assert_eq!(app.world().resource::<Messages<MatchFinished>>().len(), 1);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    submit(&mut app, striker, UnitCommand::MoveTo(Vec2::ZERO));
    step(&mut app, 20);
    assert_eq!(app.world().resource::<SimulationTick>().0, 2);
    assert_eq!(app.world().resource::<Messages<MatchFinished>>().len(), 1);
}

#[test]
fn simultaneous_base_deaths_are_a_single_draw_after_complete_damage_resolution() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    let ally = app
        .world()
        .resource::<MatchState>()
        .base(Team::Ally)
        .unwrap();
    let enemy = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let a = base_striker(&mut app, Team::Ally, enemy);
    let b = base_striker(&mut app, Team::Enemy, ally);
    submit(&mut app, a, UnitCommand::AttackTarget(enemy));
    submit(&mut app, b, UnitCommand::AttackTarget(ally));
    step(&mut app, 10);
    assert_eq!(app.world().get::<UnitState>(ally).unwrap().health(), 0.0);
    assert_eq!(app.world().get::<UnitState>(enemy).unwrap().health(), 0.0);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Finished(MatchOutcome::Draw)
    );
    assert_eq!(app.world().resource::<SimulationTick>().0, 2);
    assert_eq!(app.world().resource::<Messages<MatchFinished>>().len(), 1);
}

#[test]
fn victory_on_a_wave_boundary_does_not_spawn_the_due_wave() {
    let mut app = arena(MatchDefinition::default());
    begin(&mut app);
    step(&mut app, 3598);
    let enemy = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let striker = base_striker(&mut app, Team::Ally, enemy);
    submit(&mut app, striker, UnitCommand::AttackTarget(enemy));
    step(&mut app, 2);
    assert_eq!(app.world().resource::<SimulationTick>().0, 3600);
    assert!(matches!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Finished(_)
    ));
    assert_eq!(app.world().resource::<MatchState>().wave_number(), 1);
    assert_eq!(minions_in_wave(&mut app, 2), [0; 4]);
}

#[test]
fn wave_construction_failure_pauses_without_partial_or_repeated_spawns() {
    let mut app = arena(
        MatchDefinition::default()
            .with_wave_interval_ticks(6)
            .with_minion_spawn_interval_ticks(1),
    );
    begin(&mut app);
    step(&mut app, 5);
    let spawn = app
        .world()
        .resource::<MapDefinition>()
        .creep_spawns(Team::Enemy)[5];
    app.world_mut().insert_resource(NavigationMap::new(
        32.0,
        1.0,
        vec![Obstacle {
            center: spawn,
            half_size: Vec2::splat(0.2),
        }],
    ));
    step(&mut app, 8);
    let state = app.world().resource::<MatchState>();
    assert_eq!(state.wave_number(), 1);
    assert!(state.wave_error().is_some());
    assert!(!state.can_advance());
    assert_eq!(app.world().resource::<SimulationTick>().0, 6);
    assert_eq!(roots(&mut app), 16);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 8);
    assert_eq!(roots(&mut app), 16);
    assert_eq!(app.world().resource::<SimulationTick>().0, 6);
}

#[test]
fn dead_creep_cleanup_keeps_committed_projectile_and_area_effects_alive() {
    for (targeting, delivery, target) in [
        (
            Targeting::Direction,
            Delivery::Projectile {
                speed: 4.0,
                radius: 0.1,
            },
            AbilityTarget::Direction(Vec2::X),
        ),
        (
            Targeting::Point,
            Delivery::Area {
                radius: 1.0,
                delay: 0.3,
            },
            AbilityTarget::Point(Vec2::X * 3.0),
        ),
    ] {
        let mut app = arena(MatchDefinition::default());
        begin(&mut app);
        let effect = AbilityId(600);
        let kill = AbilityId(601);
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(effect, damage_ability(targeting, delivery, 100.0))
            .unwrap();
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                kill,
                damage_ability(Targeting::Enemy, Delivery::Direct, 1000.0),
            )
            .unwrap();
        let creep = actor(
            &mut app,
            Team::Ally,
            Vec2::ZERO,
            UnitStats::default(),
            [Some(effect), None, None, None],
        );
        app.world_mut()
            .entity_mut(creep)
            .insert(UnitRole::RangedMinion);
        let child = app.world_mut().spawn(ChildOf(creep)).id();
        let enemy = actor(
            &mut app,
            Team::Enemy,
            Vec2::X * 3.0,
            UnitStats {
                max_health: 1000.0,
                ..default()
            },
            [Some(kill), None, None, None],
        );
        submit(
            &mut app,
            creep,
            UnitCommand::CastAbility { slot: 0, target },
        );
        submit(
            &mut app,
            enemy,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Unit(creep),
            },
        );
        step(&mut app, 1);
        assert!(app.world().get_entity(creep).is_err());
        assert!(app.world().get_entity(child).is_err());
        assert_eq!(
            app.world_mut()
                .query::<&Projectile>()
                .iter(app.world())
                .count()
                + app
                    .world_mut()
                    .query::<&AreaEffect>()
                    .iter(app.world())
                    .count(),
            1
        );
        step(&mut app, 60);
        assert_eq!(app.world().get::<UnitState>(enemy).unwrap().health(), 900.0);
    }
}

#[test]
fn base_recovery_heals_immediately_under_fire_with_nearby_enemies_for_both_heroes() {
    for team in [Team::Ally, Team::Enemy] {
        let mut app = arena(MatchDefinition::default());
        begin(&mut app);
        let hero = app.world().resource::<MatchState>().hero(team).unwrap();
        let maximum = app
            .world()
            .get::<Unit>(hero)
            .unwrap()
            .definition()
            .max_health;
        let regen = app
            .world()
            .get::<Unit>(hero)
            .unwrap()
            .definition()
            .health_regen;
        let point = app.world().get::<UnitPose>(hero).unwrap().position();
        let hit = AbilityId(602);
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                hit,
                damage_ability(Targeting::Enemy, Delivery::Direct, 60.0),
            )
            .unwrap();
        let source = actor(
            &mut app,
            opponent(team),
            point + Vec2::X * 3.0,
            UnitStats::default(),
            [Some(hit), None, None, None],
        );
        submit(
            &mut app,
            source,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Unit(hero),
            },
        );
        step(&mut app, 1);
        let health = app.world().get::<UnitState>(hero).unwrap().health();
        assert!((health - (maximum - 60.0 + maximum * 0.20 / 60.0)).abs() < 0.001);
        step(&mut app, 10);
        let before_hit = app.world().get::<UnitState>(hero).unwrap().health();
        assert!((before_hit - (health + 10.0 * (regen + maximum * 0.20) / 60.0)).abs() < 0.005);
        submit(
            &mut app,
            source,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Unit(hero),
            },
        );
        step(&mut app, 1);
        let health = app.world().get::<UnitState>(hero).unwrap().health();
        assert!((health - (before_hit - 60.0 + (regen + maximum * 0.20) / 60.0)).abs() < 0.001);
        step(&mut app, 200);
        assert_eq!(
            app.world().get::<UnitState>(hero).unwrap().health(),
            maximum
        );
    }
}

#[test]
fn reset_removes_the_entire_trial_and_preserves_content_and_monotonic_identities() {
    let mut app = arena(MatchDefinition::default());
    let first = begin(&mut app);
    let first_item = equip_equipment(app.world_mut(), first.player, 0, EquipmentId(1)).unwrap();
    let child = app.world_mut().spawn(ChildOf(first.player)).id();
    let character_count = app.world().resource::<CharacterCatalog>().iter().count();
    step(&mut app, 45);
    assert_eq!(minions_in_wave(&mut app, 1), [2, 0, 2, 0]);
    reset_match(app.world_mut());
    assert_eq!(roots(&mut app), 0);
    assert!(app.world().get_entity(child).is_err());
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Waiting
    );
    assert_eq!(
        app.world().resource::<CharacterCatalog>().iter().count(),
        character_count
    );
    assert_eq!(app.world().resource::<NavigationMap>().half_size(), 32.0);
    assert!(app.world().resource::<SimulationControl>().paused);
    let second = start_match(app.world_mut(), CharacterId(2)).unwrap();
    assert_ne!(second.id, first.id);
    assert_ne!(second.player, first.player);
    assert_eq!(second.id.to_string(), "2");
    let second_item = equip_equipment(app.world_mut(), second.player, 0, EquipmentId(1)).unwrap();
    assert_ne!(first_item, second_item);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 29);
    assert_eq!(minions_in_wave(&mut app, 1), [1, 0, 1, 0]);
    step(&mut app, 1);
    assert_eq!(minions_in_wave(&mut app, 1), [2, 0, 2, 0]);
}
