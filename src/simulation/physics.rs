//! The sole 3D movement authority. Rendering reads snapshots, never rigid-body transforms.

use avian3d::collision::collider::contact_query::contact_manifolds;
use avian3d::prelude::*;
use bevy::prelude::*;

use super::{ActionIntent, Body, Combatant, Facing, Health, MotionState, MoveIntent};

pub const STANDING_HEIGHT: f32 = 1.73;
pub const CROUCHED_HEIGHT: f32 = 1.40;
const WALKABLE_NORMAL: f32 = std::f32::consts::FRAC_1_SQRT_2;
const SUPPORT_SKIN: f32 = 0.07;

#[derive(Clone, Copy, Debug)]
pub struct SurfaceHit {
    pub entity: Entity,
    pub position: Vec3,
    pub normal: Vec3,
}

pub fn surface_below(
    query: &SpatialQuery,
    origin: Vec3,
    max_distance: f32,
    excluded: impl IntoIterator<Item = Entity>,
) -> Option<SurfaceHit> {
    let filter = SpatialQueryFilter::from_excluded_entities(excluded);
    query
        .cast_ray(origin, Dir3::NEG_Y, max_distance, true, &filter)
        .map(|hit| SurfaceHit {
            entity: hit.entity,
            position: origin + Vec3::NEG_Y * hit.distance,
            normal: hit.normal,
        })
}

#[derive(Clone, Copy)]
pub(super) struct Support {
    pub hit: SurfaceHit,
    pub velocity: Vec3,
}

/// Read-only center pose of a box or terrain body. Only the physics bridge
/// publishes it.
#[derive(Component)]
pub struct PhysicsPose {
    position: Vec3,
    rotation: Quat,
}

impl PhysicsPose {
    fn new(position: Vec3, rotation: Quat) -> Self {
        Self { position, rotation }
    }
    pub fn position(&self) -> Vec3 {
        self.position
    }
    pub fn rotation(&self) -> Quat {
        self.rotation
    }
}

pub fn character_collider(radius: f32, height: f32) -> Collider {
    Collider::capsule(radius, (height - 2.0 * radius).max(0.0))
}

pub fn spawn_box(world: &mut World, center: Vec3, size: Vec3, dynamic: bool) -> Entity {
    assert!(center.is_finite() && size.is_finite() && size.cmpgt(Vec3::ZERO).all());
    let mut entity = world.spawn((
        if dynamic {
            RigidBody::Dynamic
        } else {
            RigidBody::Static
        },
        Position(center),
        Rotation::IDENTITY,
        Collider::cuboid(size.x, size.y, size.z),
        PhysicsPose::new(center, Quat::IDENTITY),
        Friction::new(0.7),
        Restitution::new(0.0),
    ));
    if dynamic {
        entity.insert((
            ColliderDensity(25.0),
            LinearDamping(0.05),
            AngularDamping(0.08),
            SweptCcd::new().with_filter(CcdFilter::ALL),
        ));
    }
    entity.id()
}

pub fn spawn_terrain(world: &mut World, vertices: Vec<Vec3>, triangles: Vec<[u32; 3]>) -> Entity {
    let collider =
        Collider::trimesh_with_config(vertices, triangles, TrimeshFlags::FIX_INTERNAL_EDGES);
    world
        .spawn((
            RigidBody::Static,
            Position(Vec3::ZERO),
            Rotation::IDENTITY,
            collider,
            Friction::new(0.8),
            PhysicsPose::new(Vec3::ZERO, Quat::IDENTITY),
        ))
        .id()
}

/// Immediate simulation teleport for fixtures and explicit reset flows.
pub fn teleport_actor(world: &mut World, entity: Entity, feet: Vec3) {
    assert!(feet.is_finite());
    let Some((radius, height)) = world
        .get::<Body>(entity)
        .map(|body| (body.radius(), body.standing_height()))
    else {
        return;
    };
    let mut target = world.entity_mut(entity);
    if let Some(mut body) = target.get_mut::<Body>() {
        body.previous = feet;
        body.position = feet;
        body.velocity = Vec3::ZERO;
        body.teleport_request = None;
    }
    if let Some(mut motion) = target.get_mut::<MotionState>() {
        motion.reset_at(feet.y);
        motion.capsule_height = height;
    }
    if let Some(mut history) = target.get_mut::<super::motion_feedback::MotionFeedbackState>() {
        history.clear();
    }
    target.insert((
        Position(feet + Vec3::Y * height * 0.5),
        Rotation::IDENTITY,
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
        character_collider(radius, height),
    ));
    target.remove::<Sleeping>();
}

pub(super) fn flush_teleports(world: &mut World) {
    let requests: Vec<_> = world
        .query::<(Entity, &Body)>()
        .iter(world)
        .filter_map(|(entity, body)| body.teleport_request.map(|feet| (entity, feet)))
        .collect();
    for (entity, feet) in requests {
        teleport_actor(world, entity, feet);
    }
}

/// Give new actors their rigid body, collider and first genuine support.
pub(super) fn install(world: &mut World) {
    #[cfg(test)]
    super::geometry::install_fixture(world);
    let actors: Vec<_> = world
        .query_filtered::<(Entity, &Body), Without<RigidBody>>()
        .iter(world)
        .map(|(entity, body)| (entity, body.position, body.radius(), body.standing_height()))
        .collect();
    for (entity, feet, radius, height) in actors {
        let shape = character_collider(radius, height);
        let center = feet + Vec3::Y * height * 0.5;
        // New actors need a genuine support query before their first input is
        // consumed. This queries the same collider shapes as the solver; it does
        // not infer grounding from world height or a terrain formula.
        let support = world
            .query::<(
                Entity,
                &Position,
                &Rotation,
                &Collider,
                Option<&LinearVelocity>,
                Option<&AngularVelocity>,
                Has<Sensor>,
            )>()
            .iter(world)
            .filter(|(other, _, _, _, _, _, sensor)| *other != entity && !sensor)
            .filter_map(
                |(other, position, rotation, collider, linear, angular, _)| {
                    // An exact zero-depth GJK touch can have an arbitrary normal.
                    // Probe a skin below the real pose to obtain a stable support
                    // manifold, without moving the actual rigid body.
                    let mut manifolds = Vec::new();
                    contact_manifolds(
                        &shape,
                        center - Vec3::Y * SUPPORT_SKIN,
                        Quat::IDENTITY,
                        collider,
                        position.0,
                        rotation.0,
                        0.0,
                        &mut manifolds,
                    );
                    manifolds
                        .into_iter()
                        .filter_map(|manifold| {
                            let normal = -manifold.normal;
                            if normal.y < WALKABLE_NORMAL {
                                return None;
                            }
                            manifold
                                .points
                                .into_iter()
                                .filter_map(|contact| {
                                    // Manifold.point is the WORLD-space midpoint. Recover
                                    // the support-side surface point, including translation
                                    // and rotation, before computing its angular velocity.
                                    let point = contact.point + normal * contact.penetration * 0.5;
                                    (point.y <= feet.y + radius + SUPPORT_SKIN).then_some(Support {
                                        hit: SurfaceHit {
                                            entity: other,
                                            position: point,
                                            normal,
                                        },
                                        velocity: linear.map_or(Vec3::ZERO, |linear| linear.0)
                                            + angular
                                                .map_or(Vec3::ZERO, |angular| angular.0)
                                                .cross(point - position.0),
                                    })
                                })
                                .next()
                        })
                        .max_by(|a, b| a.hit.normal.y.total_cmp(&b.hit.normal.y))
                },
            )
            .max_by(|a, b| a.hit.normal.y.total_cmp(&b.hit.normal.y));
        world.entity_mut(entity).insert((
            RigidBody::Dynamic,
            Position(center),
            Rotation::IDENTITY,
            character_collider(radius, height),
            LockedAxes::ROTATION_LOCKED,
            ColliderDensity(100.0),
            // The motor supplies controlled tangential traction. Solver friction
            // would otherwise reduce every requested walking speed a second time
            // and stick the capsule to walls during jumps.
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::new(0.0),
            SweptCcd::new()
                .with_filter(CcdFilter::ALL)
                .with_threshold(0.05),
        ));
        if let Some(mut motion) = world.get_mut::<MotionState>(entity) {
            motion.capsule_height = height;
            motion.publish(feet.y, 0.0, support);
        }
    }
    flush_teleports(world);
}

pub(super) fn fixed_clock(mut clock: ResMut<Time>, fixed: Res<Time<Fixed>>) {
    // Bevy does this in its real loop; explicitly retaining it also makes manual
    // headless FixedUpdate ticks execute exactly the same physics delta.
    *clock = fixed.as_generic();
}

type Motors = (
    Entity,
    &'static mut Body,
    &'static mut LinearVelocity,
    &'static MoveIntent,
    Option<&'static mut MotionState>,
    Option<&'static mut ActionIntent>,
    Option<&'static Facing>,
    Option<&'static Health>,
    Option<&'static mut Combatant>,
    Option<&'static super::DefenseState>,
    Option<&'static super::SlowEffect>,
);

pub(super) fn motor(
    mut feedback: super::motion_feedback::MotionRecorder,
    mut actors: Query<Motors>,
) {
    let dt = feedback.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (
        entity,
        mut body,
        mut velocity,
        intent,
        motion,
        actions,
        facing,
        health,
        mut combat,
        defense,
        slow,
    ) in &mut actors
    {
        body.previous = body.position;
        if health.is_some_and(|health| !health.is_alive()) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            if let Some(mut motion) = motion {
                let actual_height = motion.capsule_height;
                motion.reset_at(body.position.y);
                // Let the normal headroom transition restore the standing shape
                // without changing the feet or falsifying collider dimensions.
                motion.capsule_height = actual_height;
            }
            if let Some(mut actions) = actions {
                actions.clear();
            }
            continue;
        }
        let direction = if combat.as_deref().is_some_and(Combatant::movement_locked) {
            Vec2::ZERO
        } else {
            intent.0
        };
        let Some(mut motion) = motion else {
            let change = (direction * body.speed - velocity.xz()).clamp_length_max(120.0 * dt);
            velocity.x += change.x;
            velocity.z += change.y;
            continue;
        };
        let mut idle = ActionIntent::default();
        let mut actions = actions;
        let previous_jump = motion.jumps_used();
        let was_dodging = motion.is_dodging();
        let was_sliding = motion.is_sliding();
        let mut delta = motion.advance(
            actions.as_deref_mut().unwrap_or(&mut idle),
            direction,
            facing.map_or(Vec2::NEG_Y, Facing::direction),
            body.speed,
            dt,
            defense.is_some_and(super::DefenseState::is_active),
        );
        if let Some(combat) = combat.as_deref_mut() {
            let jumped = motion.jumps_used() > previous_jump;
            let dodged = !was_dodging && motion.is_dodging();
            if jumped || dodged || (!was_sliding && motion.is_sliding()) {
                combat.interrupt_melee_chain();
            }
            if jumped {
                feedback.push(entity, super::MotionKind::Jump, &body, &motion);
            }
            if dodged {
                feedback.push(entity, super::MotionKind::Dodge, &body, &motion);
            }
        }
        if let Some(combat) = combat.as_deref()
            && let Some(pull) = combat.pull_velocity()
        {
            delta = pull * dt;
        }
        if defense.is_some_and(super::DefenseState::is_active) {
            delta = direction
                * (body.speed * 0.35).min(if motion.is_crouched() { 1.6 } else { f32::MAX })
                * dt;
        }
        if let Some(slow) = slow {
            delta *= slow.factor();
        }
        if let Some(vertical) = motion.pending_vertical_velocity.take() {
            velocity.y = vertical;
        }
        let support = motion.support;
        let mut target = Vec3::new(delta.x / dt, 0.0, delta.y / dt);
        let normal = support.map_or(Vec3::Y, |support| support.hit.normal);
        if support.is_some() {
            let magnitude = target.length();
            target = (target - normal * target.dot(normal)).normalize_or_zero() * magnitude;
        } else if let Some(steep) = motion.steep_normal {
            let wall = Vec3::new(steep.x, 0.0, steep.z).normalize_or_zero();
            target -= wall * target.dot(wall).min(0.0);
        }
        let support_velocity = support.map_or(Vec3::ZERO, |support| support.velocity);
        let relative = velocity.0 - support_velocity;
        let tangent = relative - normal * relative.dot(normal);
        let acceleration = if motion.is_dodging() {
            720.0
        } else if motion.is_sliding() {
            360.0
        } else if support.is_some() {
            120.0
        } else {
            36.0
        };
        velocity.0 += (target - tangent).clamp_length_max(acceleration * dt);
        if support.is_some() {
            let gravity = Vec3::NEG_Y * super::actions::GRAVITY;
            velocity.0 -= (gravity - normal * gravity.dot(normal)) * dt;
        }
    }
}

/// Query headroom before mutating physical poses. Keeping this as a separate
/// exclusive transition avoids aliasing SpatialQuery's read-only collider data.
pub(super) fn resize_characters(world: &mut World) {
    let candidates: Vec<_> = world
        .query::<(Entity, &Body, &Position, &MotionState)>()
        .iter(world)
        .filter_map(|(entity, body, position, motion)| {
            let wanted = if motion.is_crouched() {
                body.crouched_height()
            } else {
                body.standing_height()
            };
            (wanted != motion.capsule_height).then_some((
                entity,
                body.radius(),
                position.0 - Vec3::Y * motion.capsule_height * 0.5,
                motion.capsule_height,
                wanted,
            ))
        })
        .collect();
    if candidates.is_empty() {
        return;
    }
    let mut state = bevy::ecs::system::SystemState::<SpatialQuery>::new(world);
    let decisions: Vec<_> = {
        let spatial = state.get_mut(world).unwrap();
        candidates
            .into_iter()
            .map(|(entity, radius, feet, previous, wanted)| {
                let center = feet + Vec3::Y * wanted * 0.5;
                let shape = character_collider(radius * 0.98, wanted - 0.02);
                let filter = SpatialQueryFilter::from_excluded_entities([entity]);
                let blocked = wanted > previous
                    && !spatial
                        .shape_intersections(
                            &shape,
                            center + Vec3::Y * 0.02,
                            Quat::IDENTITY,
                            &filter,
                        )
                        .is_empty();
                (entity, radius, center, wanted, blocked)
            })
            .collect()
    };
    for (entity, radius, center, wanted, blocked) in decisions {
        let mut actor = world.entity_mut(entity);
        if blocked {
            actor.get_mut::<MotionState>().unwrap().keep_crouched();
        } else {
            actor.insert((Position(center), character_collider(radius, wanted)));
            actor.get_mut::<MotionState>().unwrap().capsule_height = wanted;
        }
    }
}

type SupportBodies = (
    &'static Position,
    &'static Rotation,
    &'static LinearVelocity,
    &'static AngularVelocity,
    Option<&'static ComputedCenterOfMass>,
);

type PublishedActors = (
    Entity,
    &'static Position,
    &'static LinearVelocity,
    &'static mut Body,
    Option<&'static mut MotionState>,
    Option<&'static MoveIntent>,
    Option<&'static Health>,
    Option<&'static mut super::motion_feedback::MotionFeedbackState>,
);

pub(super) fn publish(
    mut feedback: super::motion_feedback::MotionRecorder,
    collisions: Collisions,
    spatial: SpatialQuery,
    bodies: Query<SupportBodies>,
    sensors: Query<(), With<Sensor>>,
    mut actors: Query<PublishedActors>,
    mut poses: Query<(&Position, &Rotation, &mut PhysicsPose)>,
) {
    for (entity, position, velocity, mut body, motion, movement, health, history) in &mut actors {
        let height = motion
            .as_ref()
            .map_or(body.standing_height(), |motion| motion.capsule_height);
        let feet = position.0 - Vec3::Y * height * 0.5;
        body.position = feet;
        body.velocity = velocity.0;
        let Some(mut motion) = motion else {
            continue;
        };
        let mut support = None;
        let mut steep = None;
        let mut candidates = Vec::new();
        for pair in collisions.iter().filter(|pair| pair.is_touching()) {
            if sensors.contains(pair.collider1) || sensors.contains(pair.collider2) {
                continue;
            }
            let (other, sign) = if pair.collider1 == entity {
                (pair.body2.unwrap_or(pair.collider2), -1.0)
            } else if pair.collider2 == entity {
                (pair.body1.unwrap_or(pair.collider1), 1.0)
            } else {
                continue;
            };
            for manifold in &pair.manifolds {
                let normal = manifold.normal * sign;
                for contact in &manifold.points {
                    if contact.point.y <= feet.y + body.radius() + SUPPORT_SKIN
                        && contact.penetration > -SUPPORT_SKIN
                    {
                        candidates.push(SurfaceHit {
                            entity: other,
                            position: contact.point,
                            normal,
                        });
                    }
                }
            }
        }
        let shape = character_collider(body.radius() * 0.96, height - 0.025);
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        for hit in spatial.shape_hits(
            &shape,
            position.0,
            Quat::IDENTITY,
            Dir3::NEG_Y,
            8,
            &ShapeCastConfig::from_max_distance(SUPPORT_SKIN),
            &filter,
        ) {
            if sensors.contains(hit.entity) {
                continue;
            }
            candidates.push(SurfaceHit {
                entity: hit.entity,
                position: hit.point1,
                normal: hit.normal1,
            });
        }
        for hit in candidates {
            if hit.normal.y < WALKABLE_NORMAL {
                if hit.normal.y > 0.01 {
                    steep = Some(hit.normal);
                }
                continue;
            }
            let support_velocity = bodies.get(hit.entity).map_or(
                Vec3::ZERO,
                |(position, rotation, linear, angular, center)| {
                    let center =
                        position.0 + rotation.0 * center.map_or(Vec3::ZERO, |center| center.0);
                    linear.0 + angular.0.cross(hit.position - center)
                },
            );
            if (velocity.0 - support_velocity).dot(hit.normal) > 0.6 {
                continue;
            }
            if support
                .as_ref()
                .is_none_or(|old: &Support| hit.normal.y > old.hit.normal.y)
            {
                support = Some(Support {
                    hit,
                    velocity: support_velocity,
                });
            }
        }
        motion.steep_normal = steep;
        motion.publish(feet.y, velocity.y, support);
        if let Some(mut history) = history {
            if body.teleport_request.is_some() || health.is_some_and(|health| !health.is_alive()) {
                history.clear();
                continue;
            }
            let (landed, stepped) = history.publish(
                feedback.delta_secs(),
                &body,
                &motion,
                movement.is_some_and(|intent| intent.0 != Vec2::ZERO),
            );
            if landed {
                feedback.push(entity, super::MotionKind::Land, &body, &motion);
            }
            if stepped {
                feedback.push(entity, super::MotionKind::Footstep, &body, &motion);
            }
        }
    }
    for (position, rotation, mut pose) in &mut poses {
        pose.position = position.0;
        pose.rotation = rotation.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{SimulationActive, SimulationPlugin};
    use bevy::ecs::system::SystemState;
    use std::time::Duration;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, -0.25, 0.0),
            Vec3::new(30.0, 0.5, 30.0),
            false,
        );
        app
    }

    fn actor(app: &mut App, feet: Vec3) -> Entity {
        app.world_mut()
            .spawn((
                Body::at(feet, 6.0, 0.35),
                MoveIntent::default(),
                ActionIntent::default(),
                MotionState::default(),
                Facing::default(),
            ))
            .id()
    }

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn jump(app: &mut App, entity: Entity) {
        app.world_mut()
            .get_mut::<ActionIntent>(entity)
            .unwrap()
            .request_jump();
        tick(app, 1);
    }

    #[test]
    fn first_fixed_tick_accepts_ground_actions_from_actual_spawn_contact() {
        for crouch in [false, true] {
            let mut app = app();
            let entity = actor(&mut app, Vec3::ZERO);
            let mut intent = app.world_mut().get_mut::<ActionIntent>(entity).unwrap();
            if crouch {
                intent.request_crouch_toggle();
            } else {
                intent.request_dodge();
            }
            tick(&mut app, 1);
            let motion = app.world().get::<MotionState>(entity).unwrap();
            if crouch {
                assert!(motion.is_crouched());
            } else {
                assert!(motion.is_dodging());
            }
        }
    }

    #[test]
    fn first_install_support_point_and_rotation_velocity_are_in_world_space() {
        let mut app = app();
        let center = Vec3::new(5.0, 1.5, 4.0);
        let support = spawn_box(app.world_mut(), center, Vec3::new(3.0, 1.0, 3.0), true);
        app.world_mut().entity_mut(support).insert((
            Rotation(Quat::from_rotation_y(0.4)),
            LinearVelocity(Vec3::X),
            AngularVelocity(Vec3::Y * 2.0),
        ));
        let entity = actor(&mut app, Vec3::new(5.5, 2.02, 4.25));
        install(app.world_mut());
        let support = app
            .world()
            .get::<MotionState>(entity)
            .unwrap()
            .support
            .unwrap();
        assert!(
            (support.hit.position - Vec3::new(5.5, 2.0, 4.25)).length() < 0.01,
            "{:?}",
            support.hit
        );
        assert!(
            (support.velocity - Vec3::new(1.5, 0.0, -1.0)).length() < 0.03,
            "{:?}",
            support.velocity
        );
    }

    #[test]
    fn elevated_support_refills_two_jumps_and_leaving_its_edge_falls() {
        let mut app = app();
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(3.0, 1.0, 3.0),
            false,
        );
        let entity = actor(&mut app, Vec3::Y * 1.02);
        tick(&mut app, 12);
        assert!(
            app.world()
                .get::<MotionState>(entity)
                .unwrap()
                .is_grounded()
        );
        assert!((app.world().get::<Body>(entity).unwrap().world_position().y - 1.0).abs() < 0.03);
        jump(&mut app, entity);
        tick(&mut app, 24);
        jump(&mut app, entity);
        assert_eq!(
            app.world().get::<MotionState>(entity).unwrap().jumps_used(),
            2
        );
        let velocity = app.world().get::<Body>(entity).unwrap().velocity().y;
        jump(&mut app, entity);
        assert_eq!(
            app.world().get::<MotionState>(entity).unwrap().jumps_used(),
            2
        );
        assert!(app.world().get::<Body>(entity).unwrap().velocity().y < velocity);
        tick(&mut app, 100);
        let motion = app.world().get::<MotionState>(entity).unwrap();
        assert!(motion.is_grounded());
        assert_eq!(motion.jumps_used(), 0);
        assert!((app.world().get::<Body>(entity).unwrap().world_position().y - 1.0).abs() < 0.03);
        app.world_mut()
            .get_mut::<MoveIntent>(entity)
            .unwrap()
            .set(Vec2::X);
        tick(&mut app, 65);
        assert!(app.world().get::<Body>(entity).unwrap().world_position().y < 0.1);
    }

    #[test]
    fn a_dynamic_box_tips_from_off_center_impulse_and_two_boxes_settle() {
        let mut app = app();
        let lower = spawn_box(app.world_mut(), Vec3::new(0.0, 0.5, 0.0), Vec3::ONE, true);
        let upper = spawn_box(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Vec3::ONE, true);
        tick(&mut app, 180);
        assert!((app.world().get::<PhysicsPose>(lower).unwrap().position().y - 0.5).abs() < 0.03);
        assert!((app.world().get::<PhysicsPose>(upper).unwrap().position().y - 1.5).abs() < 0.04);
        let mut state = SystemState::<Query<Forces>>::new(app.world_mut());
        {
            let mut forces = state.get_mut(app.world_mut()).unwrap();
            forces
                .get_mut(upper)
                .unwrap()
                .apply_linear_impulse_at_point(Vec3::X * 50.0, Vec3::new(0.0, 1.9, 0.0));
        }
        tick(&mut app, 12);
        let pose = app.world().get::<PhysicsPose>(upper).unwrap();
        assert!(pose.position().x > 0.1);
        assert!(pose.rotation().angle_between(Quat::IDENTITY) > 0.08);
    }

    #[test]
    fn moving_support_carries_an_idle_actor_and_loss_of_support_falls() {
        let mut app = app();
        let platform = spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(3.0, 1.0, 3.0),
            true,
        );
        let entity = actor(&mut app, Vec3::Y * 1.02);
        tick(&mut app, 45);
        app.world_mut().entity_mut(platform).insert((
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            LinearVelocity(Vec3::X * 2.0),
        ));
        app.world_mut().entity_mut(platform).remove::<Sleeping>();
        let start = app.world().get::<Body>(entity).unwrap().world_position();
        tick(&mut app, 30);
        let end = app.world().get::<Body>(entity).unwrap().world_position();
        assert!(end.x - start.x > 0.45, "{start:?} -> {end:?}");
        assert!(
            app.world()
                .get::<MotionState>(entity)
                .unwrap()
                .is_grounded()
        );
        app.world_mut().despawn(platform);
        tick(&mut app, 12);
        assert!(
            !app.world()
                .get::<MotionState>(entity)
                .unwrap()
                .is_grounded()
        );
        assert!(app.world().get::<Body>(entity).unwrap().world_position().y < end.y - 0.2);
    }

    #[test]
    fn pause_freezes_falling_actors_and_rotating_props_on_the_first_tick() {
        let mut app = app();
        let entity = actor(&mut app, Vec3::Y * 4.0);
        let prop = spawn_box(app.world_mut(), Vec3::new(3.0, 4.0, 0.0), Vec3::ONE, true);
        app.world_mut()
            .entity_mut(prop)
            .insert(AngularVelocity(Vec3::Z * 2.0));
        tick(&mut app, 5);
        let position = app.world().get::<Position>(entity).unwrap().0;
        let velocity = app.world().get::<LinearVelocity>(entity).unwrap().0;
        let prop_pose = app.world().get::<PhysicsPose>(prop).unwrap();
        let prop_position = prop_pose.position();
        let prop_rotation = prop_pose.rotation();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        tick(&mut app, 120);
        assert_eq!(app.world().get::<Position>(entity).unwrap().0, position);
        assert_eq!(
            app.world().get::<LinearVelocity>(entity).unwrap().0,
            velocity
        );
        assert_eq!(
            app.world().get::<PhysicsPose>(prop).unwrap().position(),
            prop_position
        );
        assert_eq!(
            app.world().get::<PhysicsPose>(prop).unwrap().rotation(),
            prop_rotation
        );
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 1);
        assert!(app.world().get::<Position>(entity).unwrap().0.y < position.y);
    }

    #[test]
    fn a_ceiling_prevents_standing_until_the_actor_moves_clear() {
        let mut app = app();
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 1.7, 0.0),
            Vec3::new(3.0, 0.4, 3.0),
            false,
        );
        let entity = actor(&mut app, Vec3::new(-3.0, 0.0, 0.0));
        tick(&mut app, 3);
        app.world_mut()
            .get_mut::<ActionIntent>(entity)
            .unwrap()
            .request_crouch_toggle();
        app.world_mut()
            .get_mut::<MoveIntent>(entity)
            .unwrap()
            .set(Vec2::X);
        tick(&mut app, 120);
        app.world_mut()
            .get_mut::<MoveIntent>(entity)
            .unwrap()
            .stop();
        app.world_mut()
            .get_mut::<ActionIntent>(entity)
            .unwrap()
            .request_crouch_toggle();
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<MotionState>(entity)
                .unwrap()
                .is_crouched()
        );
        app.world_mut()
            .get_mut::<MoveIntent>(entity)
            .unwrap()
            .set(Vec2::X);
        tick(&mut app, 170);
        app.world_mut()
            .get_mut::<ActionIntent>(entity)
            .unwrap()
            .request_crouch_toggle();
        tick(&mut app, 1);
        assert!(
            !app.world()
                .get::<MotionState>(entity)
                .unwrap()
                .is_crouched()
        );
    }

    #[test]
    fn fast_capsule_and_dynamic_boxes_do_not_cross_thin_obstacles_or_each_other() {
        let mut app = app();
        spawn_box(
            app.world_mut(),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(0.08, 2.0, 3.0),
            false,
        );
        let entity = actor(&mut app, Vec3::ZERO);
        tick(&mut app, 2);
        app.world_mut()
            .entity_mut(entity)
            .insert(LinearVelocity(Vec3::X * 120.0));
        tick(&mut app, 1);
        let x = app.world().get::<Body>(entity).unwrap().world_position().x;
        // Avian's CCD uses a 5mm linear slop; no whole-body tunneling is allowed.
        assert!(x > 1.0 && x + 0.35 <= 1.966, "capsule x={x}");

        let left = spawn_box(app.world_mut(), Vec3::new(-3.0, 0.5, 5.0), Vec3::ONE, true);
        let right = spawn_box(app.world_mut(), Vec3::new(3.0, 0.5, 5.0), Vec3::ONE, true);
        tick(&mut app, 2);
        app.world_mut()
            .entity_mut(left)
            .insert(LinearVelocity(Vec3::X * 300.0));
        app.world_mut()
            .entity_mut(right)
            .insert(LinearVelocity(Vec3::NEG_X * 300.0));
        tick(&mut app, 1);
        let a = app.world().get::<PhysicsPose>(left).unwrap().position();
        let b = app.world().get::<PhysicsPose>(right).unwrap().position();
        assert!(a.x < b.x, "dynamic CCD must prevent swapping: {a:?}, {b:?}");
        assert!(b.x - a.x > 0.98);
    }

    #[test]
    fn a_ceiling_stops_jump_height_and_side_contacts_do_not_refill_jumps() {
        let mut app = app();
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 2.2, 0.0),
            Vec3::new(3.0, 0.4, 3.0),
            false,
        );
        let entity = actor(&mut app, Vec3::ZERO);
        jump(&mut app, entity);
        let mut highest = 0.0_f32;
        for _ in 0..25 {
            tick(&mut app, 1);
            highest = highest.max(app.world().get::<Body>(entity).unwrap().world_position().y);
        }
        assert!(
            highest > 0.1 && highest < 0.285,
            "head clearance limits foot height: {highest}"
        );

        spawn_box(
            app.world_mut(),
            Vec3::new(5.0, 3.0, 0.0),
            Vec3::new(0.2, 6.0, 3.0),
            false,
        );
        teleport_actor(app.world_mut(), entity, Vec3::new(4.3, 1.0, 0.0));
        app.world_mut()
            .get_mut::<MoveIntent>(entity)
            .unwrap()
            .set(Vec2::X);
        jump(&mut app, entity);
        jump(&mut app, entity);
        for _ in 0..25 {
            tick(&mut app, 1);
            let body = app.world().get::<Body>(entity).unwrap();
            let motion = app.world().get::<MotionState>(entity).unwrap();
            assert!(body.world_position().x + body.radius() < 4.905);
            assert!(!motion.is_grounded());
            assert_eq!(motion.jumps_used(), 2);
        }
    }
}
