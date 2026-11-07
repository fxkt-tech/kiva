//! Terrain placement uses public casts and final collision geometry, including
//! existing walls. No simulation position or lifetime is rewritten by fixtures.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

fn arena(obstacles: Vec<Obstacle>) -> App {
    arena_with_ring(
        obstacles,
        RingTerrainSpec {
            radius: 2.0,
            thickness: 0.25,
            duration: 3.0,
            recast_delay: 0.1,
        },
    )
}
fn arena_with_ring(obstacles: Vec<Obstacle>, ring: RingTerrainSpec) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(20.0, 0.5, obstacles));
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            AbilityId(1),
            AbilityDefinition {
                name: "Test leap and terrain".into(),
                targeting: Targeting::Enemy,
                range: 15.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 1.0,
                mana_cost: 10.0,
                execution: AbilityExecution::Leap(LeapSpec {
                    duration: 0.1,
                    radius: 1.0,
                    effects: vec![Effect::Damage(DamageSpec::physical(1.0))],
                    terrain: Some(ring),
                    heroes_only: true,
                }),
            },
        )
        .unwrap();
    app
}
fn unit(app: &mut App, team: Team, point: Vec2, caster: bool) -> Entity {
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Terrain actor".into(),
            team,
            position: point,
            stats: UnitStats {
                kind: UnitKind::Hero,
                max_health: 100.0,
                max_mana: 100.0,
                mana_regen: 0.0,
                ..default()
            },
            abilities: [caster.then_some(AbilityId(1)), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap()
}
fn step(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
fn cast(app: &mut App, caster: Entity, target: Entity) {
    app.world_mut().resource_mut::<CommandQueue>().submit(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(target),
        },
    );
    step(app, 8);
}
fn position(app: &App, unit: Entity) -> Vec2 {
    app.world().get::<UnitPose>(unit).unwrap().position()
}
fn assert_all_walkable(app: &mut App, expected_walls: usize) {
    let map = app.world().resource::<NavigationMap>().clone();
    for (unit, pose) in app
        .world_mut()
        .query::<(&Unit, &UnitPose)>()
        .iter(app.world())
    {
        assert!(
            map.walkable(pose.position(), unit.definition().radius),
            "terrain stranded an actor at {:?}",
            pose.position()
        );
    }
    assert_eq!(
        app.world_mut()
            .query::<&BattlefieldObject>()
            .iter(app.world())
            .filter(|object| object.kind() == ObjectKind::Terrain)
            .count(),
        expected_walls
    );
}

#[test]
fn a_blocked_radial_push_finds_a_nearby_safe_position_instead_of_stranding_the_actor() {
    let mut app = arena(vec![Obstacle {
        center: Vec2::new(5.29, 0.0),
        half_size: Vec2::splat(0.1),
    }]);
    let caster = unit(&mut app, Team::Ally, Vec2::ZERO, true);
    let target = unit(&mut app, Team::Enemy, Vec2::new(4.0, 0.0), false);
    let original = Vec2::new(5.95, 0.0);
    let affected = unit(&mut app, Team::Ally, original, false);
    cast(&mut app, caster, target);
    assert_all_walkable(&mut app, 1);
    assert!(position(&app, affected).distance(original) < 2.0);
    let before = position(&app, affected);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(affected, UnitCommand::MoveTo(Vec2::new(8.0, 0.0)));
    step(&mut app, 20);
    assert!(
        position(&app, affected).distance(before) > 0.5,
        "the displaced actor must remain able to walk"
    );
}

#[test]
fn overlapping_rings_resolve_against_both_walls_and_recasting_one_preserves_the_other() {
    let mut app = arena(vec![]);
    let first = unit(&mut app, Team::Ally, Vec2::ZERO, true);
    let first_target = unit(&mut app, Team::Enemy, Vec2::new(4.0, 0.0), false);
    let second = unit(&mut app, Team::Ally, Vec2::new(8.5, 2.0), true);
    let second_target = unit(&mut app, Team::Enemy, Vec2::new(6.8, 0.0), false);
    let original = Vec2::new(5.2, 0.0);
    let affected = unit(&mut app, Team::Ally, original, false);
    cast(&mut app, first, first_target);
    assert_eq!(position(&app, affected), original);
    cast(&mut app, second, second_target);
    assert_all_walkable(&mut app, 2);
    assert!(position(&app, affected).distance(original) < 2.0);
    step(&mut app, 6);
    app.world_mut().resource_mut::<CommandQueue>().submit(
        second,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert_all_walkable(&mut app, 1);
    assert!(
        !app.world()
            .resource::<NavigationMap>()
            .walkable(Vec2::new(6.0, 0.0), 0.45),
        "the first caster's wall remains after the second caster's recast"
    );
}

#[test]
fn a_high_speed_dash_cannot_skip_a_legal_thin_ring_in_either_direction() {
    let ring = RingTerrainSpec {
        radius: 2.013,
        thickness: 0.0001,
        duration: 3.0,
        recast_delay: 0.1,
    };
    let mut app = arena_with_ring(vec![], ring);
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            AbilityId(2),
            AbilityDefinition {
                name: "Fast thin-ring dash".into(),
                targeting: Targeting::Point,
                range: 10.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Dash { speed: 600.0 },
            },
        )
        .unwrap();
    let center = Vec2::new(4.0, 0.0);
    let caster = unit(&mut app, Team::Ally, Vec2::ZERO, true);
    let target = unit(&mut app, Team::Enemy, center, false);
    let radius = 0.0001;
    let runner = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Small dash actor".into(),
            team: Team::Ally,
            position: center,
            stats: UnitStats {
                radius,
                ..default()
            },
            abilities: [Some(AbilityId(2)), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    cast(&mut app, caster, target);
    let map = app.world().resource::<NavigationMap>();
    let from_outside = map.sweep(center + Vec2::X * 10.0, center, radius);
    assert!(from_outside.x > center.x + ring.radius + ring.thickness + radius);
    assert!(map.walkable(from_outside, radius));
    app.world_mut().resource_mut::<CommandQueue>().submit(
        runner,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Point(center + Vec2::X * 10.0),
        },
    );
    step(&mut app, 1);
    let stopped = position(&app, runner);
    assert!(
        stopped.x > center.x + 2.0,
        "the dash must advance to the wall"
    );
    assert!(stopped.x < center.x + ring.radius - ring.thickness - radius);
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .walkable(stopped, radius)
    );
    assert!(!is_dashing(app.world(), runner));
}

#[test]
fn an_outer_grazing_intersection_shorter_than_a_sampling_step_still_blocks() {
    let mut app = arena(vec![]);
    let center = Vec2::new(4.0, 0.0);
    let caster = unit(&mut app, Team::Ally, Vec2::ZERO, true);
    let target = unit(&mut app, Team::Enemy, center, false);
    cast(&mut app, caster, target);
    let radius = 0.0001;
    let height = 2.0 + 0.25 + radius - 0.00001;
    let start = center + Vec2::new(-0.04, height);
    let end = center + Vec2::new(0.04, height);
    let map = app.world().resource::<NavigationMap>();
    assert!(map.walkable(start, radius));
    assert!(map.walkable(end, radius));
    assert!(!map.line_clear(start, end, radius));
    let stopped = map.sweep(start, end, radius);
    assert!(stopped.x > start.x && stopped.x < center.x);
    assert!(map.walkable(stopped, radius));
}

#[test]
fn no_local_exit_rejects_the_whole_wall_without_partial_push_or_refunding_committed_damage() {
    let mut app = arena(vec![
        Obstacle {
            center: Vec2::new(1.0, 0.0),
            half_size: Vec2::new(0.5, 2.0),
        },
        Obstacle {
            center: Vec2::new(3.0, 0.0),
            half_size: Vec2::new(0.5, 2.0),
        },
        Obstacle {
            center: Vec2::new(2.0, 1.0),
            half_size: Vec2::splat(0.5),
        },
        Obstacle {
            center: Vec2::new(2.0, -1.0),
            half_size: Vec2::splat(0.5),
        },
    ]);
    let caster = unit(&mut app, Team::Ally, Vec2::new(-4.0, 0.0), true);
    let target = unit(&mut app, Team::Enemy, Vec2::ZERO, false);
    let movable_point = Vec2::new(0.0, -2.0);
    let movable = unit(&mut app, Team::Ally, movable_point, false);
    let pocket = Vec2::new(2.0, 0.0);
    let enclosed = unit(&mut app, Team::Ally, pocket, false);
    cast(&mut app, caster, target);
    assert_all_walkable(&mut app, 0);
    assert_eq!(
        position(&app, movable),
        movable_point,
        "placement must not apply a successful prefix"
    );
    assert_eq!(
        position(&app, enclosed),
        pocket,
        "placement must not cross an old obstacle"
    );
    assert_eq!(app.world().get::<UnitState>(caster).unwrap().mana(), 90.0);
    assert_eq!(app.world().get::<UnitState>(target).unwrap().health(), 99.0);
    assert!(!app.world().get::<UnitMechanics>(caster).unwrap().recast(0));
    let mut cursor = MessageCursor::<SimulationEvent>::default();
    assert!(
        cursor
            .read(app.world().resource::<Messages<SimulationEvent>>())
            .any(
                |event| matches!(event, SimulationEvent::TerrainPlacementFailed { unit, position }
            if *unit == caster && *position == Vec2::ZERO)
            ),
        "failed post-commit placement must be an observable result"
    );
}
