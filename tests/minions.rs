use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::{gameplay::*, simulation::*};

fn match_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin, GameplayPlugin, AiPlugin));
    prepare_world(app.world_mut());
    start_match(app.world_mut(), CharacterId(1)).unwrap();
    let heroes: Vec<_> = [Team::Ally, Team::Enemy]
        .into_iter()
        .map(|team| app.world().resource::<MatchState>().hero(team).unwrap())
        .collect();
    for hero in heroes {
        app.world_mut().entity_mut(hero).insert(Controller::Human);
    }
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app
}
fn ticks(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
fn command(app: &mut App, unit: Entity, command: UnitCommand) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(unit, command);
}
fn focus(app: &mut App) {
    // Finish the real emission schedule before clearing units for a focused
    // command scenario; no private pending state is changed or clock rewound.
    ticks(app, 150);
    let creeps: Vec<_> = app
        .world_mut()
        .query::<(Entity, &UnitRole)>()
        .iter(app.world())
        .filter(|(_, role)| matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion))
        .map(|(entity, _)| entity)
        .collect();
    for entity in creeps {
        app.world_mut().despawn(entity);
    }
}
fn spawn(app: &mut App, team: Team, point: Vec2, role: UnitRole, controller: Controller) -> Entity {
    let unit = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Lane fixture".into(),
            team,
            position: point,
            stats: UnitStats {
                kind: if role == UnitRole::Hero {
                    UnitKind::Hero
                } else {
                    UnitKind::Minion
                },
                vision_radius: 12.0,
                max_health: 2000.0,
                attack_delivery: BasicAttackDelivery::Direct,
                base_attributes: BaseAttributes {
                    attack_damage: 12.0,
                    attack_speed: 0.5,
                    attack_range: 0.7,
                    move_speed: 2.5,
                    ..default()
                },
                attack_windup_fraction: 0.8,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut().entity_mut(unit).insert((role, controller));
    unit
}

#[test]
fn two_real_waves_meet_attack_and_clean_dead_minion_roots() {
    let mut app = match_app();
    ticks(&mut app, 150);
    let initial: Vec<_> = app
        .world_mut()
        .query::<(Entity, &UnitRole)>()
        .iter(app.world())
        .filter(|(_, role)| matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion))
        .map(|(entity, _)| entity)
        .collect();
    assert_eq!(initial.len(), 12);
    let mut events = MessageCursor::<SimulationEvent>::default();
    ticks(&mut app, 1200);
    let hits = events.read(app.world().resource::<Messages<SimulationEvent>>())
        .filter(|event| matches!(event, SimulationEvent::Damaged { source, target, .. } if initial.contains(source) && initial.contains(target)))
        .count();
    assert!(
        hits > 10,
        "real melee and ranged minions must engage each other"
    );
    let alive = app
        .world_mut()
        .query::<(&UnitRole, &UnitState)>()
        .iter(app.world())
        .filter(|(role, _)| matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion))
        .map(|(_, state)| state.alive())
        .collect::<Vec<_>>();
    assert!(alive.len() < 12 && alive.iter().all(|alive| *alive));
    assert!(
        initial
            .iter()
            .any(|entity| app.world().get_entity(*entity).is_err())
    );
}

#[test]
fn minion_prefers_creeps_and_finishes_long_preparation_without_command_spam() {
    let mut app = match_app();
    focus(&mut app);
    let axis = app
        .world()
        .resource::<MapDefinition>()
        .lane_direction(Team::Ally);
    let actor = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitRole::MeleeMinion,
        Controller::MinionAi,
    );
    let creep = spawn(
        &mut app,
        Team::Enemy,
        axis * 3.0,
        UnitRole::MeleeMinion,
        Controller::Human,
    );
    let _closer_hero = spawn(
        &mut app,
        Team::Enemy,
        axis * 1.5,
        UnitRole::Hero,
        Controller::Human,
    );
    ticks(&mut app, 14);
    assert!(
        matches!(app.world().get::<UnitOrder>(actor), Some(UnitOrder::AttackTarget(life)) if life.entity()==creep)
    );
    let mut events = MessageCursor::<SimulationEvent>::default();
    ticks(&mut app, 300);
    assert!(app.world().get::<UnitState>(creep).unwrap().health() < 2000.0);
    assert!(events.read(app.world().resource::<Messages<SimulationEvent>>())
        .any(|event| matches!(event, SimulationEvent::AttackReleased { source, target } if *source==actor && *target==creep)));
    assert!(
        matches!(app.world().get::<UnitOrder>(actor), Some(UnitOrder::AttackTarget(life)) if life.entity()==creep)
    );
}

#[test]
fn fleeing_hero_does_not_pull_minion_outside_the_lane_forever() {
    let mut app = match_app();
    focus(&mut app);
    let map = app.world().resource::<MapDefinition>().clone();
    let actor = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitRole::MeleeMinion,
        Controller::MinionAi,
    );
    let hero = spawn(
        &mut app,
        Team::Enemy,
        map.lane_direction(Team::Ally) * 2.0,
        UnitRole::Hero,
        Controller::Human,
    );
    ticks(&mut app, 14);
    assert!(
        matches!(app.world().get::<UnitOrder>(actor), Some(UnitOrder::AttackTarget(life)) if life.entity()==hero)
    );
    command(&mut app, hero, UnitCommand::MoveTo(Vec2::X * 12.0));
    ticks(&mut app, 650);
    assert!(
        !matches!(app.world().get::<UnitOrder>(actor), Some(UnitOrder::AttackTarget(life)) if life.entity()==hero)
    );
    let point = app.world().get::<UnitPose>(actor).unwrap().position();
    assert!(
        (point - map.base_position(Team::Ally))
            .dot(map.lane_lateral(Team::Ally))
            .abs()
            < map.lane_width() * 0.5
    );
    assert!(point.dot(map.lane_direction(Team::Ally)) > 3.0);
}

fn fragile_enemy(app: &mut App, point: Vec2, respawn: Option<f32>) -> Entity {
    let entity = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Fragile screen".into(),
            team: Team::Enemy,
            position: point,
            stats: UnitStats {
                kind: if respawn.is_some() {
                    UnitKind::Hero
                } else {
                    UnitKind::Minion
                },
                max_health: 12.0,
                base_attributes: BaseAttributes {
                    attack_speed: 0.0,
                    ..default()
                },
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: respawn,
        },
    )
    .unwrap();
    app.world_mut().entity_mut(entity).insert((
        if respawn.is_some() {
            UnitRole::Hero
        } else {
            UnitRole::MeleeMinion
        },
        Controller::Human,
    ));
    entity
}

#[test]
fn minion_clears_a_screen_then_advances_to_damage_the_actual_base() {
    let mut app = match_app();
    focus(&mut app);
    let enemy_hero = app
        .world()
        .resource::<MatchState>()
        .hero(Team::Enemy)
        .unwrap();
    app.world_mut().despawn(enemy_hero);
    let base = app
        .world()
        .resource::<MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let map = app.world().resource::<MapDefinition>().clone();
    let axis = map.lane_direction(Team::Ally);
    let actor = spawn(
        &mut app,
        Team::Ally,
        map.base_position(Team::Enemy) - axis * 4.0,
        UnitRole::MeleeMinion,
        Controller::MinionAi,
    );
    let creep = fragile_enemy(&mut app, map.base_position(Team::Enemy) - axis * 3.0, None);
    ticks(&mut app, 450);
    assert!(
        app.world().get_entity(creep).is_err(),
        "dead screen is cleaned by the mode"
    );
    assert!(app.world().get::<UnitState>(base).unwrap().health() < 3000.0);
    assert!(
        matches!(app.world().get::<UnitOrder>(actor), Some(UnitOrder::AttackTarget(life)) if life.entity() == base)
    );
}

#[test]
fn minion_reacquires_a_respawned_hero_as_a_new_life() {
    let mut app = match_app();
    focus(&mut app);
    let actor = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitRole::MeleeMinion,
        Controller::MinionAi,
    );
    let victim = fragile_enemy(&mut app, Vec2::X * 0.7, Some(0.2));
    let previous = app
        .world()
        .get::<UnitState>(victim)
        .unwrap()
        .life_ref(victim);
    ticks(&mut app, 150);
    let current = app
        .world()
        .get::<UnitState>(victim)
        .unwrap()
        .life_ref(victim);
    assert_ne!(current, previous);
    assert!(
        matches!(app.world().get::<UnitOrder>(actor), Some(UnitOrder::AttackTarget(life)) if *life == current)
    );
    ticks(&mut app, 120);
    assert_ne!(
        app.world()
            .get::<UnitState>(victim)
            .unwrap()
            .life_ref(victim),
        current,
        "fresh life can be attacked normally after reacquisition"
    );
}

#[test]
fn normal_soft_avoidance_separates_walkers_without_moving_idle_attackers() {
    let mut app = match_app();
    focus(&mut app);
    let a = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitRole::MeleeMinion,
        Controller::Human,
    );
    let b = spawn(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitRole::MeleeMinion,
        Controller::Human,
    );
    for unit in [a, b] {
        app.world_mut().entity_mut(unit).insert(SoftAvoidance);
        command(&mut app, unit, UnitCommand::MoveTo(Vec2::X * 6.0));
    }
    ticks(&mut app, 60);
    let pa = app.world().get::<UnitPose>(a).unwrap().position();
    let pb = app.world().get::<UnitPose>(b).unwrap().position();
    assert!(pa.is_finite() && pb.is_finite());
    assert!(pa.x > 2.0 && pb.x > 2.0);
    assert!(
        pa.distance(pb) > 0.3,
        "overlapping ordinary walkers must visibly separate"
    );
    for unit in [a, b] {
        command(&mut app, unit, UnitCommand::Stop);
    }
    ticks(&mut app, 1);
    let stopped = [
        app.world().get::<UnitPose>(a).unwrap().position(),
        app.world().get::<UnitPose>(b).unwrap().position(),
    ];
    ticks(&mut app, 60);
    assert_eq!(
        app.world().get::<UnitPose>(a).unwrap().position(),
        stopped[0]
    );
    assert_eq!(
        app.world().get::<UnitPose>(b).unwrap().position(),
        stopped[1]
    );
}

#[test]
fn side_by_side_minions_keep_forward_progress_in_a_narrow_static_corridor() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin))
        .insert_resource(NavigationMap::new(
            20.0,
            1.0,
            vec![
                Obstacle {
                    center: Vec2::new(0.0, 1.45),
                    half_size: Vec2::new(9.0, 1.0),
                },
                Obstacle {
                    center: Vec2::new(0.0, -1.45),
                    half_size: Vec2::new(9.0, 1.0),
                },
            ],
        ));
    let mut units = Vec::new();
    for side in [-0.22, 0.22] {
        let entity = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Corridor walker".into(),
                team: Team::Ally,
                position: Vec2::new(-6.0, side),
                stats: UnitStats {
                    radius: 0.2,
                    base_attributes: BaseAttributes {
                        move_speed: 2.5,
                        ..default()
                    },
                    ..default()
                },
                abilities: [None; 4],
                respawn_delay: None,
            },
        )
        .unwrap();
        app.world_mut().entity_mut(entity).insert(SoftAvoidance);
        command(&mut app, entity, UnitCommand::MoveTo(Vec2::new(6.0, side)));
        units.push(entity);
    }
    ticks(&mut app, 310);
    for entity in units {
        let point = app.world().get::<UnitPose>(entity).unwrap().position();
        assert!(
            point.x > 5.8,
            "optional separation must not stall a valid narrow route: {point:?}"
        );
        assert!(point.y.abs() <= 0.251);
    }
}

#[test]
fn soft_avoidance_does_not_change_dash_or_external_displacement_endpoints() {
    for forced in [false, true] {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(NavigationMap::new(20.0, 1.0, vec![]));
        let ability = AbilityId(100);
        let definition = AbilityDefinition {
            name: "Crowd motion".into(),
            targeting: if forced {
                Targeting::Enemy
            } else {
                Targeting::Point
            },
            range: 10.0,
            windup: 0.0,
            recovery: 0.0,
            cooldown: 0.0,
            mana_cost: 0.0,
            execution: if forced {
                AbilityExecution::Impact(ImpactDefinition {
                    delivery: Delivery::Direct,
                    effects: vec![Effect::Displace(DisplacementSpec {
                        distance: 5.0,
                        speed: 10.0,
                        direction: DisplacementDirection::AlongImpact,
                    })],
                })
            } else {
                AbilityExecution::Dash { speed: 10.0 }
            },
        };
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(ability, definition)
            .unwrap();
        let unit = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Moving".into(),
                team: Team::Ally,
                position: Vec2::ZERO,
                stats: default(),
                abilities: [Some(ability), None, None, None],
                respawn_delay: None,
            },
        )
        .unwrap();
        let neighbor = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Crowd".into(),
                team: Team::Ally,
                position: Vec2::X,
                stats: default(),
                abilities: [None; 4],
                respawn_delay: None,
            },
        )
        .unwrap();
        app.world_mut().entity_mut(unit).insert(SoftAvoidance);
        app.world_mut().entity_mut(neighbor).insert(SoftAvoidance);
        if forced {
            let caster = spawn_unit(
                app.world_mut(),
                UnitSpec {
                    name: "Caster".into(),
                    team: Team::Enemy,
                    position: Vec2::NEG_X,
                    stats: default(),
                    abilities: [Some(ability), None, None, None],
                    respawn_delay: None,
                },
            )
            .unwrap();
            command(
                &mut app,
                caster,
                UnitCommand::CastAbility {
                    slot: 0,
                    target: AbilityTarget::Unit(unit),
                },
            );
        } else {
            command(
                &mut app,
                unit,
                UnitCommand::CastAbility {
                    slot: 0,
                    target: AbilityTarget::Point(Vec2::X * 5.0),
                },
            );
        }
        ticks(&mut app, 40);
        assert!(
            app.world()
                .get::<UnitPose>(unit)
                .unwrap()
                .position()
                .distance(Vec2::X * 5.0)
                < 0.001
        );
    }
}

#[test]
#[ignore = "explicit 30 minute simulation soak"]
fn half_hour_real_waves_do_not_accumulate_dead_roots_or_silently_skip_spawns() {
    half_hour_match(CharacterId(1));
}

#[test]
#[ignore = "explicit 30 minute Jarvan object lifecycle soak"]
fn half_hour_jarvan_ai_waves_preserve_objects_and_reset() {
    half_hour_match(CharacterId(2));
}

fn half_hour_match(player: CharacterId) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin, GameplayPlugin, AiPlugin));
    prepare_world(app.world_mut());
    let definition = app.world().resource::<MatchDefinition>().clone();
    let base = UnitStats {
        max_health: 100_000_000.0,
        ..definition.base_stats().clone()
    };
    app.world_mut()
        .insert_resource(definition.with_base_stats(base));
    let static_entities = app.world().entities().count_spawned();
    start_match(app.world_mut(), player).unwrap();
    let opponent = app
        .world()
        .resource::<MatchState>()
        .hero(Team::Enemy)
        .unwrap();
    let opponent = app.world().get::<CharacterIdentity>(opponent).unwrap().id();
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    let mut emitted = std::collections::HashSet::new();
    let mut per_wave = [0; 31];
    record_emitted_pair(&mut app, 1, 0, &mut emitted);
    per_wave[0] = 2;
    let mut peak = 0;
    let mut peak_entities = 0;
    let mut peak_effects = 0;
    let mut seen_standard = false;
    let mut seen_terrain = false;
    // Complete wave31 as well: a wave starts at108000 and finishes at108150.
    for tick in 1_u64..=108150 {
        ticks(&mut app, 1);
        if tick.is_multiple_of(60) {
            peak_entities = peak_entities.max(app.world().entities().count_spawned());
            let effects = app
                .world_mut()
                .query_filtered::<Entity, Or<(With<Projectile>, With<AreaEffect>, With<BattlefieldObject>)>>()
                .iter(app.world())
                .count();
            peak_effects = peak_effects.max(effects);
            for object in app
                .world_mut()
                .query::<&BattlefieldObject>()
                .iter(app.world())
            {
                seen_standard |= object.kind() == ObjectKind::Standard && object.active(tick);
                seen_terrain |= object.kind() == ObjectKind::Terrain && object.active(tick);
            }
            let states = app
                .world_mut()
                .query::<(&UnitRole, &UnitState)>()
                .iter(app.world())
                .filter(|(role, _)| matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion))
                .map(|(_, state)| state.alive())
                .collect::<Vec<_>>();
            assert!(states.iter().all(|alive| *alive));
            peak = peak.max(states.len());
            app.world_mut()
                .resource_mut::<Messages<SimulationEvent>>()
                .update();
        }
        let offset = tick % 3600;
        if offset <= 150 && offset.is_multiple_of(30) {
            let wave = tick / 3600 + 1;
            record_emitted_pair(&mut app, wave, (offset / 30) as u8, &mut emitted);
            per_wave[(wave - 1) as usize] += 2;
            if offset == 150 {
                assert_eq!(per_wave[(wave - 1) as usize], 12);
            }
        }
    }
    assert_eq!(app.world().resource::<SimulationTick>().0, 108150);
    assert_eq!(app.world().resource::<MatchState>().wave_number(), 31);
    assert_eq!(emitted.len(), 31 * 12);
    assert!(per_wave.into_iter().all(|count| count == 12));
    if opponent == CharacterId(1) {
        assert!(
            seen_standard && seen_terrain,
            "the real Jarvan AI must exercise both flag and wall lifetimes"
        );
    }
    assert!(
        peak < 31 * 12,
        "real combat should have removed casualties; all waves still spawned"
    );
    reset_match(app.world_mut());
    assert_eq!(
        app.world_mut().query::<&Unit>().iter(app.world()).count(),
        0
    );
    assert_eq!(
        app.world().entities().count_spawned(),
        static_entities,
        "reset must clean units and independently committed effects"
    );
    println!(
        "player{} vs ai{},30 minutes + final2.5s,31 complete waves/372 unique emissions,peak {peak} living minions/{peak_entities} total entities/{peak_effects} effects (including battlefield objects); flags={seen_standard},terrain={seen_terrain}; no dead roots retained; reset returned to {static_entities} static entities",
        player.0, opponent.0
    );
}

fn record_emitted_pair(
    app: &mut App,
    wave: u64,
    slot: u8,
    emitted: &mut std::collections::HashSet<Entity>,
) {
    let pair = app
        .world_mut()
        .query::<(Entity, &Unit, &WaveMember)>()
        .iter(app.world())
        .filter(|(_, _, member)| member.wave() == wave && member.slot() == slot)
        .map(|(entity, unit, _)| (entity, unit.team()))
        .collect::<Vec<_>>();
    assert_eq!(pair.len(), 2, "wave{wave} slot{slot} must emit both sides");
    assert!(pair.iter().any(|(_, team)| *team == Team::Ally));
    assert!(pair.iter().any(|(_, team)| *team == Team::Enemy));
    for (entity, _) in pair {
        assert!(
            emitted.insert(entity),
            "a minion emission cannot repeat its identity"
        );
    }
}
