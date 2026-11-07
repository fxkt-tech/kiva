//! Delivery geometry, health and life transitions. Effects resolve elsewhere.
use super::{effects::PendingImpacts, *};
use bevy::prelude::*;

pub(super) fn advance_projectiles(
    mut commands: Commands,
    map: Res<NavigationMap>,
    units: Query<(Entity, &Unit, &UnitState, &UnitPose)>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
    mut impacts: ResMut<PendingImpacts>,
) {
    for (entity, mut projectile) in &mut projectiles {
        if let Some(target) = projectile.homing {
            let Ok((_, _, state, pose)) = units.get(target.entity()) else {
                commands.entity(entity).despawn();
                continue;
            };
            if !state.alive() || state.life_ref(target.entity()) != target {
                commands.entity(entity).despawn();
                continue;
            }
            projectile.direction = super::effects::direction(
                pose.position - projectile.position,
                projectile.direction,
            );
        }
        projectile.previous = projectile.position;
        let distance = (projectile.speed * TICK_SECONDS).min(projectile.remaining);
        let requested = projectile.position + projectile.direction * distance;
        let end = map.sweep(projectile.position, requested, projectile.radius);
        let mut hit = None;
        for (id, unit, state, pose) in &units {
            if !state.alive()
                || unit.team == projectile.payload.team
                || projectile
                    .homing
                    .is_some_and(|target| target != state.life_ref(id))
            {
                continue;
            }
            if let Some(fraction) = segment_circle(
                projectile.position,
                end,
                pose.position,
                unit.stats.radius + projectile.radius,
            ) && hit.is_none_or(|(old, old_id): (f32, Entity)| {
                fraction < old || (fraction == old && id.to_bits() < old_id.to_bits())
            }) {
                hit = Some((fraction, id));
            }
        }
        projectile.position = end;
        projectile.remaining -= distance;
        if let Some((_, target)) = hit {
            let state = units.get(target).expect("hit unit").2;
            // Projectile centers retain the caster's committed origin;
            // AlongImpact separately follows the actual collision direction.
            impacts.hit(
                1,
                projectile.payload.clone(),
                projectile.carrier,
                state.life_ref(target),
                projectile.payload.position,
                projectile.direction,
            );
        }
        if hit.is_some() || projectile.remaining <= 0.001 || end.distance_squared(requested) > 1e-6
        {
            commands.entity(entity).despawn();
        }
    }
}
fn segment_circle(start: Vec2, end: Vec2, center: Vec2, radius: f32) -> Option<f32> {
    let delta = end - start;
    let offset = start - center;
    let c = offset.length_squared() - radius * radius;
    if c <= 0.0 {
        return Some(0.0);
    }
    let a = delta.length_squared();
    if a < 1e-10 {
        return None;
    }
    let b = offset.dot(delta);
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b - discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&t).then_some(t)
}
pub(super) fn advance_areas(
    mut commands: Commands,
    units: Query<(Entity, &Unit, &UnitState, &UnitPose)>,
    mut areas: Query<(Entity, &mut AreaEffect)>,
    mut impacts: ResMut<PendingImpacts>,
) {
    for (entity, mut area) in &mut areas {
        area.remaining -= TICK_SECONDS;
        if area.remaining > 1e-5 {
            continue;
        }
        // Packet keys sort recipients by entity, independent of this iteration.
        for (target, unit, state, pose) in &units {
            if state.alive()
                && unit.team != area.payload.team
                && pose.position.distance(area.position) <= area.radius + unit.stats.radius
            {
                // Areas use their frozen impact center, never the live caster.
                impacts.hit(
                    2,
                    area.payload.clone(),
                    area.carrier,
                    state.life_ref(target),
                    area.position,
                    area.payload.facing,
                );
            }
        }
        commands.entity(entity).despawn();
    }
}
pub(super) fn apply_damage(
    world: &mut World,
    source: UnitLifeRef,
    target: UnitLifeRef,
    damage: DamageSnapshot,
) -> f32 {
    if !target.is_current(world) {
        return 0.0;
    }
    let attributes = *world
        .get::<Unit>(target.entity())
        .expect("unit")
        .attributes();
    let mut state = world
        .get_mut::<UnitState>(target.entity())
        .expect("unit state");
    let new_health = damage.health_after(&attributes, state.health);
    let amount = state.health - new_health;
    if amount <= 0.0 {
        return 0.0;
    }
    state.health = new_health;
    let died = !state.alive();
    world.write_message(SimulationEvent::Damaged {
        source: source.entity(),
        target: target.entity(),
        amount,
        kind: damage.kind(),
        origin: damage.origin(),
    });
    if died {
        die(world, target.entity());
    }
    amount
}
fn die(world: &mut World, target: Entity) {
    let delay = world.get::<Unit>(target).expect("unit").respawn_delay;
    super::movement::clear_all(world, target);
    super::statuses::clear(world, target);
    super::attributes::clear_on_death(&mut world.get_mut::<Unit>(target).expect("unit"));
    world
        .get_mut::<UnitState>(target)
        .expect("unit state")
        .respawn_remaining = delay;
    *world.get_mut::<UnitAction>(target).expect("unit action") = UnitAction::Ready;
    *world.get_mut::<UnitOrder>(target).expect("unit order") = UnitOrder::Idle;
    world.write_message(SimulationEvent::Died { unit: target });
}
pub(super) fn respawn_units(world: &mut World) {
    let units: Vec<_> = world
        .query_filtered::<Entity, With<Unit>>()
        .iter(world)
        .collect();
    for entity in units {
        let due = {
            let mut state = world.get_mut::<UnitState>(entity).expect("unit state");
            if state.alive() {
                continue;
            }
            let Some(remaining) = state.respawn_remaining.as_mut() else {
                continue;
            };
            *remaining -= TICK_SECONDS;
            *remaining <= 1e-5
        };
        if !due {
            continue;
        }
        let unit = world.get::<Unit>(entity).expect("unit");
        let (health, mana, home) = (unit.stats.max_health, unit.stats.max_mana, unit.home);
        {
            let mut state = world.get_mut::<UnitState>(entity).expect("unit state");
            state.life_epoch = state
                .life_epoch
                .checked_add(1)
                .expect("life epoch space exhausted");
            state.health = health;
            state.mana = mana;
            state.respawn_remaining = None;
            state.attack_cooldown = 0;
        }
        world
            .get_mut::<AbilitySlots>(entity)
            .expect("slots")
            .cooldowns = [0.0; 4];
        super::movement::clear_all(world, entity);
        super::movement::reset_pose(&mut world.get_mut::<UnitPose>(entity).expect("pose"), home);
        super::statuses::clear(world, entity);
        world.write_message(SimulationEvent::Respawned { unit: entity });
    }
}
