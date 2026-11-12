//! Ranged attacks in flight. A shot is a small sphere which leaves the
//! companion's weapon, flies at its row's speed, optionally steers toward its
//! locked target, and hits the first thing its path touches: an enemy takes
//! the shot, anything else stops it. Nothing is decided when it is fired.
use super::hit::{Blow, CombatWorld, Enemies, Landed, hit_enemy, target_valid};
use super::skills::{Control, Guidance, ProjectileSpec};
use super::{CombatEffect, CombatFeedback, Combatant, Energy, Health};
use crate::simulation::{ActionSnapshot, Body, ComboWindow, Enemy, HitStop, WeaponPair};
use avian3d::prelude::{Collider, ShapeCastConfig, SpatialQueryFilter};
use bevy::prelude::*;

/// What a shot does to the enemy it reaches.
#[derive(Clone, Copy)]
pub(super) struct Payload {
    pub damage: f32,
    pub control: Control,
    /// Opened when the shot controls the action's original target.
    pub window: Option<f32>,
    /// Dedicated impact presentation.
    pub sniper: bool,
    /// The landing shot requests the sniper hit stop.
    pub hit_stop: bool,
}

#[derive(Component)]
pub struct Projectile {
    owner: Entity,
    /// The logical companion who fired it.
    performer: Entity,
    /// The action which fired it, frozen at launch: its identity outlives it.
    action: ActionSnapshot,
    index: u8,
    target: Option<Entity>,
    origin: Vec3,
    previous: Vec3,
    position: Vec3,
    direction: Vec3,
    travelled: f32,
    /// Rule time in flight, out and back.
    age: f32,
    spec: ProjectileSpec,
    payload: Payload,
    /// The melee weapon itself: it flies back to its owner afterwards.
    thrown: bool,
    returning: bool,
}

impl Projectile {
    #[expect(clippy::too_many_arguments, reason = "one explicit launch record")]
    pub(super) fn launch(
        owner: Entity,
        performer: Entity,
        action: ActionSnapshot,
        index: u8,
        (origin, aim): (Vec3, Vec3),
        spec: ProjectileSpec,
        payload: Payload,
        thrown: bool,
    ) -> Self {
        Self {
            owner,
            performer,
            action,
            index,
            // Only a locked shot remembers whom it was fired at.
            target: action
                .target
                .filter(|_| matches!(spec.guidance, Guidance::Locked { .. })),
            origin,
            previous: origin,
            position: origin,
            direction: (aim - origin).normalize_or(Vec3::NEG_Z),
            travelled: 0.0,
            age: 0.0,
            spec,
            payload,
            thrown,
            returning: false,
        }
    }

    pub fn owner(&self) -> Entity {
        self.owner
    }
    pub fn pair(&self) -> WeaponPair {
        self.action.pair
    }
    pub fn action_id(&self) -> u64 {
        self.action.action_id
    }
    pub fn index(&self) -> u8 {
        self.index
    }
    pub fn position(&self) -> Vec3 {
        self.position
    }
    /// Between the previous and the current fixed step, for display.
    pub fn interpolated(&self, alpha: f32) -> Vec3 {
        self.previous.lerp(self.position, alpha.clamp(0.0, 1.0))
    }
    pub fn direction(&self) -> Vec3 {
        self.direction
    }
    /// Seconds of rule time since it was fired; frozen with the simulation.
    pub fn age(&self) -> f32 {
        self.age
    }
    /// The thrown melee weapon rather than a bullet or an arrow.
    pub fn is_thrown(&self) -> bool {
        self.thrown
    }
    pub fn is_returning(&self) -> bool {
        self.returning
    }
    pub fn radius(&self) -> f32 {
        self.spec.radius
    }
    /// Whether it still steers toward a target.
    pub fn is_locked(&self) -> bool {
        self.target.is_some()
    }

    /// Turn toward `aim` by at most the guidance's rate; within one step of
    /// it the shot goes straight there, so a locked shot cannot orbit.
    fn steer(&mut self, aim: Vec3, turn_rate: f32, dt: f32) {
        let offset = aim - self.position;
        let desired = offset.normalize_or(self.direction);
        let angle = self.direction.angle_between(desired);
        let share = if offset.length() <= self.spec.speed * dt || angle <= turn_rate * dt {
            1.0
        } else {
            turn_rate * dt / angle
        };
        let turn = Quat::from_rotation_arc(self.direction, desired);
        self.direction = (Quat::IDENTITY.slerp(turn, share) * self.direction).normalize();
    }

    /// The shot is spent: a thrown weapon turns back, anything else is gone.
    fn finish(&mut self) -> bool {
        self.returning = self.thrown;
        self.target = None;
        !self.thrown
    }
}

type Owners<'w, 's> = Query<
    'w,
    's,
    (
        &'static Body,
        &'static Health,
        &'static mut Combatant,
        &'static mut Energy,
    ),
    Without<Enemy>,
>;

/// Remove every shot in flight, e.g. when the enemy roster is replaced.
pub fn clear_projectiles(world: &mut World) {
    let flying: Vec<_> = world
        .query_filtered::<Entity, With<Projectile>>()
        .iter(world)
        .collect();
    for projectile in flying {
        world.despawn(projectile);
    }
    for mut combat in world.query::<&mut Combatant>().iter_mut(world) {
        combat.thrown = None;
    }
}

pub(in crate::simulation) fn advance_projectiles(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut commands: Commands,
    effects: (ResMut<CombatFeedback>, ResMut<HitStop>),
    mut projectiles: Query<(Entity, &mut Projectile)>,
    mut owners: Owners,
    mut enemies: Enemies,
) {
    let (mut feedback, mut hit_stop) = effects;
    let dt = time.delta_secs();
    for (entity, mut projectile) in &mut projectiles {
        let Ok((body, health, mut combat, mut energy)) = owners.get_mut(projectile.owner) else {
            commands.entity(entity).despawn();
            continue;
        };
        if !health.is_alive() {
            if projectile.thrown {
                combat.thrown = None;
            }
            commands.entity(entity).despawn();
            continue;
        }
        let step = projectile.spec.speed * dt;
        projectile.previous = projectile.position;
        projectile.age += dt;
        if projectile.returning {
            // Back to the hand, through anything; a chain pull which has
            // started meanwhile takes the weapon over.
            let home = body.aim_position() - projectile.position;
            if home.length() <= step || combat.pulls() {
                combat.thrown = None;
                commands.entity(entity).despawn();
            } else {
                projectile.direction = home.normalize();
                let direction = projectile.direction;
                projectile.position += direction * step;
            }
            continue;
        }
        if let Guidance::Locked { turn_rate } = projectile.spec.guidance
            && let Some(target) = projectile.target
        {
            match enemies.get(target) {
                Ok((_, target_body, _, _, _, _, target_health, _, _, boss, _))
                    if target_valid(target_health, boss) =>
                {
                    projectile.steer(target_body.aim_position(), turn_rate, dt);
                }
                // The target is gone: the shot flies on where it was heading.
                _ => projectile.target = None,
            }
        }
        let step = step.min(projectile.spec.max_range - projectile.travelled);
        let direction = projectile.direction;
        let hit = Dir3::new(direction).ok().and_then(|heading| {
            world.spatial.cast_shape_predicate(
                &Collider::sphere(projectile.spec.radius),
                projectile.position,
                Quat::IDENTITY,
                heading,
                &ShapeCastConfig::from_max_distance(step),
                &SpatialQueryFilter::from_excluded_entities([projectile.owner]),
                // Sensors and bodies which can no longer be hit do not stop it.
                &|other| {
                    !world.sensors.contains(other)
                        && enemies
                            .get(other)
                            .map_or(true, |(.., health, _, _, boss, _)| {
                                target_valid(health, boss)
                            })
                },
            )
        });
        let Some(hit) = hit else {
            projectile.position += direction * step;
            projectile.travelled += step;
            if projectile.travelled + 1e-4 >= projectile.spec.max_range && projectile.finish() {
                commands.entity(entity).despawn();
            }
            continue;
        };
        projectile.position += direction * hit.distance;
        let at = hit.point1;
        if enemies.contains(hit.entity) {
            let payload = projectile.payload;
            let landed = hit_enemy(
                Blow {
                    target: hit.entity,
                    source: projectile.performer,
                    index: projectile.index,
                    damage: payload.damage,
                    control: payload.control,
                    origin: projectile.origin,
                    contact: Some(at),
                },
                projectile.owner,
                &projectile.action,
                &mut enemies,
                &mut feedback,
            );
            if landed != Landed::Missed {
                energy.gain_sun(1);
                if payload.sniper {
                    feedback.action(
                        hit.entity,
                        projectile.owner,
                        &projectile.action,
                        projectile.index,
                        CombatEffect::SniperImpact,
                        at,
                        projectile.origin,
                        0.0,
                    );
                }
                if payload.hit_stop {
                    hit_stop.request_sniper();
                }
            }
            if landed == Landed::Survived
                && projectile.action.target == Some(hit.entity)
                && let Some(remaining) = payload.window
            {
                combat.combo = Some(ComboWindow {
                    source_action_id: projectile.action.action_id,
                    pair: projectile.action.pair,
                    original_target: hit.entity,
                    remaining,
                });
            }
        } else {
            feedback.action(
                projectile.performer,
                projectile.owner,
                &projectile.action,
                projectile.index,
                CombatEffect::ShotImpact,
                at,
                projectile.origin,
                projectile.spec.radius,
            );
        }
        if projectile.finish() {
            commands.entity(entity).despawn();
        }
    }
}
