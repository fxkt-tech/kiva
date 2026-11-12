//! Cross-layer acceptance uses the real woodland geometry and physics schedule.
use bevy::{prelude::*, transform::TransformPlugin};
use kiva::{
    gameplay,
    simulation::{
        ActionIntent, Body, CombatEffect, CombatFeedback, CombatIntent, Combatant, Facing, Health,
        MotionState, MoveIntent, PhysicsPose, SimulationActive, SimulationPlugin, SpawnPoint,
        WeaponPair, spawn_box,
    },
};
use std::time::Duration;

fn tick(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().run_schedule(FixedUpdate);
    }
}

#[test]
fn the_shared_bamboo_mesh_really_blocks_the_motor_near_the_clearing() {
    let (mut app, actor) = courtyard_actor(Vec3::ZERO);
    let center = Vec2::new(0.0, -4.0);
    let stem = *app
        .world()
        .resource::<gameplay::Level>()
        .bamboo
        .iter()
        .min_by(|a, b| {
            a.base
                .xz()
                .distance_squared(center)
                .total_cmp(&b.base.xz().distance_squared(center))
        })
        .unwrap();
    let direction = (stem.base.xz() - center).normalize();
    let start = stem.base.xz() - direction * 1.15;
    kiva::simulation::teleport_actor(app.world_mut(), actor, Vec3::new(start.x, 0.0, start.y));
    tick(&mut app, 10);
    move_actor(&mut app, actor, direction);
    let mut minimum = f32::INFINITY;
    for _ in 0..35 {
        tick(&mut app, 1);
        minimum = minimum.min(feet(&app, actor).xz().distance(stem.base.xz()));
    }
    assert!(
        minimum < 0.75,
        "fixture did not actually reach the bamboo: {minimum}"
    );
    assert!(
        minimum >= stem.radius + 0.35 - 0.08,
        "motor crossed the rendered trunk: {minimum}"
    );
    assert!(feet(&app, actor).y.abs() < 0.10);
}

#[test]
fn scaled_capsules_keep_feet_through_crouch_headroom_teleport_and_revival() {
    use avian3d::prelude::Position;
    let (mut app, old_actor) = courtyard_actor(Vec3::ZERO);
    app.world_mut().despawn(old_actor);
    let actor = app
        .world_mut()
        .spawn((
            Body::at(Vec3::ZERO, 6.0, 0.35).scaled(1.45),
            MoveIntent::default(),
            ActionIntent::default(),
            MotionState::default(),
            Facing::default(),
        ))
        .id();
    tick(&mut app, 12);
    let standing = app.world().get::<Body>(actor).unwrap().standing_height();
    let crouched = app.world().get::<Body>(actor).unwrap().crouched_height();
    assert!((app.world().get::<Position>(actor).unwrap().y - standing * 0.5).abs() < 0.03);
    app.world_mut()
        .get_mut::<ActionIntent>(actor)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 3);
    assert!(app.world().get::<MotionState>(actor).unwrap().is_crouched());
    assert!(feet(&app, actor).y.abs() < 0.03);
    assert!((app.world().get::<Position>(actor).unwrap().y - crouched * 0.5).abs() < 0.03);
    let roof = spawn_box(
        app.world_mut(),
        Vec3::new(0.0, 2.35, 0.0),
        Vec3::new(3.0, 0.2, 3.0),
        false,
    );
    tick(&mut app, 2);
    app.world_mut()
        .get_mut::<ActionIntent>(actor)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 3);
    assert!(
        app.world().get::<MotionState>(actor).unwrap().is_crouched(),
        "scaled standing capsule ignored the roof"
    );
    app.world_mut().despawn(roof);
    tick(&mut app, 2);
    app.world_mut()
        .get_mut::<ActionIntent>(actor)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 3);
    assert!(!app.world().get::<MotionState>(actor).unwrap().is_crouched());
    assert!(feet(&app, actor).y.abs() < 0.03);
    let destination = Vec3::new(5.0, 3.0, -2.0);
    kiva::simulation::teleport_actor(app.world_mut(), actor, destination);
    assert_eq!(feet(&app, actor), destination);
    assert_eq!(
        app.world().get::<Position>(actor).unwrap().0,
        destination + Vec3::Y * standing * 0.5
    );
    assert_eq!(app.world().get::<Body>(actor).unwrap().scale(), 1.45);
    assert_eq!(
        app.world().get::<Body>(actor).unwrap().velocity(),
        Vec3::ZERO
    );
    app.world_mut().entity_mut(actor).insert((
        Health::new(100.0),
        Combatant::new(WeaponPair::HammerMusket),
        CombatIntent::default(),
        SpawnPoint(Vec2::ZERO),
    ));
    kiva::simulation::teleport_actor(app.world_mut(), actor, Vec3::new(30.0, -13.0, 0.0));
    tick(&mut app, 1);
    assert!(!app.world().get::<Health>(actor).unwrap().is_alive());
    tick(&mut app, 190);
    assert!(app.world().get::<Health>(actor).unwrap().is_alive());
    assert!(feet(&app, actor).length() < 0.03);
    assert!((app.world().get::<Position>(actor).unwrap().y - standing * 0.5).abs() < 0.03);
    assert_eq!(app.world().get::<Body>(actor).unwrap().scale(), 1.45);
}

fn courtyard_actor(feet: Vec3) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, SimulationPlugin));
    app.finish();
    gameplay::setup(app.world_mut());
    // Keep the actual terrain and boundaries; isolate locomotion from AI.
    let mut actors = app.world_mut().query_filtered::<Entity, With<Body>>();
    let actors: Vec<_> = actors.iter(app.world()).collect();
    for entity in actors {
        app.world_mut().despawn(entity);
    }
    let entity = app
        .world_mut()
        .spawn((
            Body::at(feet, 6.0, 0.35),
            MoveIntent::default(),
            ActionIntent::default(),
            MotionState::default(),
            Facing::default(),
        ))
        .id();
    tick(&mut app, 30);
    (app, entity)
}

fn move_actor(app: &mut App, actor: Entity, direction: Vec2) {
    app.world_mut()
        .get_mut::<MoveIntent>(actor)
        .unwrap()
        .set(direction);
    app.world_mut()
        .get_mut::<ActionIntent>(actor)
        .unwrap()
        .set_movement_held(direction != Vec2::ZERO);
}

fn feet(app: &App, actor: Entity) -> Vec3 {
    app.world().get::<Body>(actor).unwrap().world_position()
}

#[test]
fn actual_hill_has_a_walkable_approach_a_supported_summit_and_a_descent() {
    let (mut app, actor) = courtyard_actor(Vec3::new(10.2, 0.05, 8.5));
    move_actor(&mut app, actor, Vec2::X);
    for _ in 0..160 {
        tick(&mut app, 1);
        if feet(&app, actor).x >= 16.25 {
            break;
        }
    }
    move_actor(&mut app, actor, Vec2::ZERO);
    tick(&mut app, 45);
    let summit = feet(&app, actor);
    assert!(
        summit.x >= 16.0,
        "could not walk up the gentle hill: {summit}"
    );
    assert!(
        (2.3..2.85).contains(&summit.y),
        "not supported near the summit: {summit}"
    );
    assert!(app.world().get::<MotionState>(actor).unwrap().is_grounded());
    tick(&mut app, 90);
    assert!(
        feet(&app, actor).distance(summit) < 0.2,
        "standing on the hill drifted"
    );
    move_actor(&mut app, actor, Vec2::NEG_X);
    tick(&mut app, 110);
    move_actor(&mut app, actor, Vec2::ZERO);
    tick(&mut app, 30);
    let bottom = feet(&app, actor);
    assert!(bottom.x < 11.0, "did not descend the hill: {bottom}");
    assert!(bottom.y.abs() < 0.12, "did not reach flat ground: {bottom}");
}

#[test]
fn double_jumps_from_the_hill_refill_only_after_landing_back_on_the_slope() {
    assert_double_jump_on_slope(Vec3::new(14.0, 1.75, 8.5));
}

fn assert_double_jump_on_slope(origin: Vec3) {
    let (mut app, actor) = courtyard_actor(origin);
    let motion = app.world().get::<MotionState>(actor).unwrap();
    assert!(motion.is_grounded());
    assert!((0.7..0.99).contains(&motion.support_normal().y));
    for used in 1..=2 {
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.jumps_used(), used);
        assert!(!motion.is_grounded());
        assert!(app.world().get::<Body>(actor).unwrap().velocity().y > 5.5);
        if used == 1 {
            tick(&mut app, 25);
            assert!(app.world().get::<Body>(actor).unwrap().velocity().y < 0.0);
        }
    }
    let velocity = app.world().get::<Body>(actor).unwrap().velocity().y;
    app.world_mut()
        .get_mut::<ActionIntent>(actor)
        .unwrap()
        .request_jump();
    tick(&mut app, 1);
    assert_eq!(
        app.world().get::<MotionState>(actor).unwrap().jumps_used(),
        2
    );
    assert!(app.world().get::<Body>(actor).unwrap().velocity().y < velocity);
    for _ in 0..120 {
        tick(&mut app, 1);
        if app.world().get::<MotionState>(actor).unwrap().is_grounded() {
            break;
        }
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().jumps_used(),
            2
        );
    }
    let motion = app.world().get::<MotionState>(actor).unwrap();
    assert!(motion.is_grounded(), "did not land back on the slope");
    assert_eq!(motion.jumps_used(), 0);
    assert!((0.7..0.99).contains(&motion.support_normal().y));
    assert!(
        feet(&app, actor).y > 0.5,
        "landing was forced down to world zero"
    );
}

#[test]
fn walking_into_the_steep_flank_does_not_climb_to_the_hilltop() {
    let (mut app, actor) = courtyard_actor(Vec3::new(20.2, 0.05, 8.5));
    move_actor(&mut app, actor, Vec2::NEG_X);
    let mut maximum_height = 0.0_f32;
    let mut trajectory = Vec::new();
    for step in 0..240 {
        tick(&mut app, 1);
        maximum_height = maximum_height.max(feet(&app, actor).y);
        if step % 30 == 0 {
            trajectory.push(feet(&app, actor));
        }
    }
    assert!(
        maximum_height < 2.35,
        "walking climbed the over-limit flank: {maximum_height}"
    );
    // Sliding around the flank is valid; crossing its X coordinate does not
    // mean the character climbed the steep face or reached the summit.
    assert!(
        trajectory
            .iter()
            .any(|point| point.x > 18.0 && point.y > 0.5),
        "the fixture must actually reach the steep flank: {trajectory:?}"
    );
}

#[test]
fn an_actual_jump_can_clear_the_low_rim_and_falling_out_respawns_the_actor() {
    let (mut app, actor) = courtyard_actor(Vec3::new(24.0, 0.05, 0.0));
    app.world_mut().entity_mut(actor).insert((
        Health::new(100.0),
        Combatant::new(WeaponPair::HammerMusket),
        CombatIntent::default(),
        SpawnPoint(Vec2::ZERO),
    ));
    move_actor(&mut app, actor, Vec2::X);
    app.world_mut()
        .get_mut::<ActionIntent>(actor)
        .unwrap()
        .request_jump();
    tick(&mut app, 40);
    assert!(
        feet(&app, actor).x > 25.6,
        "an invisible boundary blocked the jump"
    );
    assert!(!app.world().get::<MotionState>(actor).unwrap().is_grounded());
    tick(&mut app, 360);
    assert!(app.world().get::<Health>(actor).unwrap().is_alive());
    assert!(
        feet(&app, actor).length() < 0.02,
        "did not return to the spawn point"
    );
    let feedback = app.world().resource::<CombatFeedback>();
    assert!(
        feedback
            .events()
            .any(|event| event.actor == actor && event.kind == CombatEffect::Defeat)
    );
    assert!(
        feedback
            .events()
            .any(|event| event.actor == actor && event.kind == CombatEffect::Respawn)
    );
}

#[test]
fn pausing_the_real_courtyard_freezes_falling_actor_and_all_dynamic_props() {
    let (mut app, actor) = courtyard_actor(Vec3::new(0.0, 6.0, 0.0));
    let falling_box = spawn_box(app.world_mut(), Vec3::new(3.0, 6.0, 0.0), Vec3::ONE, true);
    tick(&mut app, 5);
    let box_height = app
        .world()
        .get::<PhysicsPose>(falling_box)
        .unwrap()
        .position()
        .y;
    assert!(
        (5.0..6.0).contains(&box_height),
        "fixture box must still be falling"
    );
    let actor_position = feet(&app, actor);
    let actor_velocity = app.world().get::<Body>(actor).unwrap().velocity();
    assert!(actor_velocity.y < 0.0);
    let mut props = app.world_mut().query::<(Entity, &PhysicsPose)>();
    let props: Vec<_> = props
        .iter(app.world())
        .map(|(entity, pose)| (entity, pose.position(), pose.rotation()))
        .collect();
    assert!(!props.is_empty());
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    tick(&mut app, 120);
    assert_eq!(feet(&app, actor), actor_position);
    assert_eq!(
        app.world().get::<Body>(actor).unwrap().velocity(),
        actor_velocity
    );
    for (entity, position, rotation) in props {
        let pose = app.world().get::<PhysicsPose>(entity).unwrap();
        assert_eq!(pose.position(), position);
        assert_eq!(pose.rotation(), rotation);
    }
    app.world_mut().resource_mut::<SimulationActive>().0 = true;
    tick(&mut app, 1);
    assert!(feet(&app, actor).y < actor_position.y);
    assert!(
        app.world()
            .get::<PhysicsPose>(falling_box)
            .unwrap()
            .position()
            .y
            < box_height
    );
}
