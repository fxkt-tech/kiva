//! Birth and respawn must stay valid when real temporary terrain covers home.
use bevy::prelude::*;
use kiva::{gameplay::prepare_world, simulation::*};

fn step(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
fn command(app: &mut App, unit: Entity, command: UnitCommand) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(unit, command);
}
fn caster_and_ring() -> (App, Entity, Entity) {
    caster_and_ring_with_obstacles(vec![])
}
fn caster_and_ring_with_obstacles(obstacles: Vec<Obstacle>) -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    prepare_world(app.world_mut());
    app.insert_resource(NavigationMap::new(20.0, 0.5, obstacles));
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            AbilityId(90),
            AbilityDefinition {
                name: "Public lethal hit".into(),
                targeting: Targeting::Enemy,
                range: 20.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Impact(ImpactDefinition {
                    delivery: Delivery::Direct,
                    effects: vec![Effect::Damage(DamageSpec {
                        kind: DamageType::True,
                        base: 10000.0,
                        ad_ratio: 0.0,
                        ap_ratio: 0.0,
                    })],
                }),
            },
        )
        .unwrap();
    let jarvan = spawn_character(
        app.world_mut(),
        CharacterSpawnSpec {
            character: CharacterId(1),
            team: Team::Ally,
            position: Vec2::ZERO,
            loadout: LoadoutSelection::CharacterDefault,
            respawn_delay: Some(3.0),
        },
    )
    .unwrap();
    let killer = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Visible enemy hero".into(),
            team: Team::Enemy,
            position: Vec2::new(3.5, 0.0),
            stats: UnitStats {
                kind: UnitKind::Hero,
                max_health: 10000.0,
                vision_radius: 20.0,
                ..default()
            },
            abilities: [Some(AbilityId(90)), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    command(
        &mut app,
        jarvan,
        UnitCommand::CastAbility {
            slot: 3,
            target: AbilityTarget::Unit(killer),
        },
    );
    step(&mut app, 25);
    assert!(
        !app.world()
            .resource::<NavigationMap>()
            .walkable(Vec2::ZERO, 0.45)
    );
    assert!(
        app.world_mut()
            .query::<&BattlefieldObject>()
            .iter(app.world())
            .any(|o| o.kind() == ObjectKind::Terrain)
    );
    (app, jarvan, killer)
}

#[test]
fn a_real_ring_covering_home_does_not_respawn_the_hero_inside_a_wall_or_change_home() {
    let (mut app, jarvan, killer) = caster_and_ring();
    let life = app
        .world()
        .get::<UnitState>(jarvan)
        .unwrap()
        .life_ref(jarvan);
    command(
        &mut app,
        killer,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(jarvan),
        },
    );
    step(&mut app, 1);
    assert!(!app.world().get::<UnitState>(jarvan).unwrap().alive());
    step(&mut app, 180);
    let state = app.world().get::<UnitState>(jarvan).unwrap();
    assert!(state.alive());
    assert_ne!(state.life_ref(jarvan), life);
    let position = app.world().get::<UnitPose>(jarvan).unwrap().position();
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .walkable(position, 0.45),
        "respawn must choose a legal nearby point while the wall is still alive: {position:?}"
    );
    assert!(position.distance(Vec2::ZERO) < 2.0);
    assert!(
        !app.world()
            .resource::<NavigationMap>()
            .walkable(Vec2::ZERO, 0.45)
    );
    command(
        &mut app,
        jarvan,
        UnitCommand::MoveTo(position + Vec2::X * 0.3),
    );
    step(&mut app, 5);
    assert!(
        app.world()
            .get::<UnitPose>(jarvan)
            .unwrap()
            .position()
            .distance(position)
            > 0.1
    );
    step(&mut app, 60);
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .walkable(Vec2::ZERO, 0.45)
    );
    command(
        &mut app,
        killer,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(jarvan),
        },
    );
    step(&mut app, 181);
    assert!(app.world().get::<UnitState>(jarvan).unwrap().alive());
    assert_eq!(
        app.world().get::<UnitPose>(jarvan).unwrap().position(),
        Vec2::ZERO,
        "the temporary projection must never overwrite home"
    );
}

#[test]
fn a_new_actor_uses_the_same_temporary_placement_as_its_allocation_free_preflight() {
    let (mut app, _, _) = caster_and_ring();
    let spec = UnitSpec {
        name: "New actor at a temporarily blocked home".into(),
        team: Team::Ally,
        position: Vec2::ZERO,
        stats: UnitStats::default(),
        abilities: [None; 4],
        respawn_delay: None,
    };
    let count = app.world().entities().len();
    let expected = app
        .world()
        .resource::<NavigationMap>()
        .resolve_placement(spec.position, spec.stats.radius)
        .unwrap();
    assert_ne!(expected, Vec2::ZERO);
    validate_unit_spec(app.world(), &spec).unwrap();
    assert_eq!(
        app.world().entities().len(),
        count,
        "placement preflight cannot allocate entities"
    );
    let actor = spawn_unit(app.world_mut(), spec).unwrap();
    let position = app.world().get::<UnitPose>(actor).unwrap().position();
    assert_eq!(position, expected);
    assert!(position.distance(Vec2::ZERO) < 2.0);
    let map = app.world().resource::<NavigationMap>();
    assert!(map.walkable(position, 0.45));
    assert!(
        map.path_static(Vec2::ZERO, Vec2::new(10.0, 0.0), 0.45)
            .is_ok()
    );
    assert!(map.path(Vec2::ZERO, Vec2::new(10.0, 0.0), 0.45).is_err());
}

#[test]
fn placement_and_static_reachability_preserve_real_obstacles_instead_of_repairing_bad_configuration()
 {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(
        10.0,
        0.5,
        vec![Obstacle {
            center: Vec2::ZERO,
            half_size: Vec2::new(0.2, 10.0),
        }],
    ));
    let map = app.world().resource::<NavigationMap>();
    assert_eq!(map.resolve_placement(Vec2::ZERO, 0.45), None);
    assert_eq!(map.resolve_placement(Vec2::new(11.0, 0.0), 0.45), None);
    assert_eq!(
        map.resolve_placement(Vec2::new(-4.0, 0.0), 0.45),
        Some(Vec2::new(-4.0, 0.0))
    );
    assert_eq!(
        map.path_static(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0), 0.45),
        Err(PathError::Unreachable)
    );
    let count = app.world().entities().len();
    assert!(
        spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Static blocked actor".into(),
                team: Team::Ally,
                position: Vec2::ZERO,
                stats: UnitStats::default(),
                abilities: [None; 4],
                respawn_delay: None,
            }
        )
        .is_err()
    );
    assert_eq!(app.world().entities().len(), count);
}

#[test]
fn an_enclosed_home_waits_for_the_temporary_wall_to_expire_without_teleporting_through_static_walls()
 {
    let obstacles = vec![
        Obstacle {
            center: Vec2::new(-0.65, 0.0),
            half_size: Vec2::new(0.1, 0.75),
        },
        Obstacle {
            center: Vec2::new(0.65, 0.0),
            half_size: Vec2::new(0.1, 0.75),
        },
        Obstacle {
            center: Vec2::new(0.0, -0.65),
            half_size: Vec2::new(0.75, 0.1),
        },
        Obstacle {
            center: Vec2::new(0.0, 0.65),
            half_size: Vec2::new(0.75, 0.1),
        },
    ];
    let (mut app, jarvan, killer) = caster_and_ring_with_obstacles(obstacles);
    command(
        &mut app,
        killer,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(jarvan),
        },
    );
    step(&mut app, 1);
    let dead = app
        .world()
        .get::<UnitState>(jarvan)
        .unwrap()
        .life_ref(jarvan);
    step(&mut app, 180);
    assert!(
        !app.world().get::<UnitState>(jarvan).unwrap().alive(),
        "no safe local position must defer the new life"
    );
    assert_eq!(
        app.world()
            .get::<UnitState>(jarvan)
            .unwrap()
            .life_ref(jarvan),
        dead
    );
    assert_eq!(
        app.world()
            .resource::<NavigationMap>()
            .resolve_placement(Vec2::ZERO, 0.45),
        None
    );
    step(&mut app, 35);
    assert!(app.world().get::<UnitState>(jarvan).unwrap().alive());
    assert_ne!(
        app.world()
            .get::<UnitState>(jarvan)
            .unwrap()
            .life_ref(jarvan),
        dead
    );
    assert_eq!(
        app.world().get::<UnitPose>(jarvan).unwrap().position(),
        Vec2::ZERO
    );
}
