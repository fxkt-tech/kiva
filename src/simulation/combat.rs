//! Projectile collision, damage, death and respawn. Rendering never decides hits.
use super::{
    damage::DamageRequest,
    model::{Movement, PendingDamage},
    *,
};
use bevy::prelude::*;

pub(super) fn advance_projectiles(
    mut commands: Commands,
    map: Res<NavigationMap>,
    units: Query<(Entity, &Unit, &UnitState, &UnitPose)>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
    mut damage: ResMut<PendingDamage>,
) {
    for (entity, mut projectile) in &mut projectiles {
        if let Some(target) = projectile.homing {
            let Ok((_, _, state, pose)) = units.get(target) else {
                commands.entity(entity).despawn();
                continue;
            };
            if !state.alive() {
                commands.entity(entity).despawn();
                continue;
            }
            projectile.direction = (pose.position - projectile.position).normalize_or_zero();
        }
        projectile.previous = projectile.position;
        let distance = (projectile.speed * TICK_SECONDS).min(projectile.remaining);
        let requested = projectile.position + projectile.direction * distance;
        let end = map.sweep(projectile.position, requested, projectile.radius);
        let mut hit = None;
        for (id, unit, state, pose) in &units {
            if !state.alive()
                || unit.team == projectile.team
                || projectile.homing.is_some_and(|target| target != id)
            {
                continue;
            }
            if let Some(fraction) = segment_circle(
                projectile.position,
                end,
                pose.position,
                unit.stats.radius + projectile.radius,
            ) && hit.is_none_or(|(old_fraction, old_id): (f32, Entity)| {
                fraction < old_fraction
                    || (fraction == old_fraction && id.to_bits() < old_id.to_bits())
            }) {
                hit = Some((fraction, id));
            }
        }
        projectile.position = end;
        projectile.remaining -= distance;
        if let Some((_, target)) = hit {
            damage.0.push(DamageRequest {
                source: projectile.source,
                target,
                damage: projectile.damage,
            });
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
    mut damage: ResMut<PendingDamage>,
) {
    for (entity, mut area) in &mut areas {
        area.remaining -= TICK_SECONDS;
        if area.remaining > 1e-5 {
            continue;
        }
        for (target, unit, state, pose) in &units {
            if state.alive()
                && unit.team != area.team
                && pose.position.distance(area.position) <= area.radius + unit.stats.radius
            {
                damage.0.push(DamageRequest {
                    source: area.source,
                    target,
                    damage: area.damage,
                });
            }
        }
        commands.entity(entity).despawn();
    }
}

pub(super) fn resolve_damage(world: &mut World) {
    let damage = std::mem::take(&mut world.resource_mut::<PendingDamage>().0);
    for DamageRequest {
        source,
        target,
        damage,
    } in damage
    {
        let Some(unit) = world.get::<Unit>(target) else {
            continue;
        };
        let attributes = *unit.attributes();
        let Some(mut state) = world.get_mut::<UnitState>(target) else {
            continue;
        };
        if !state.alive() {
            continue;
        }
        let new_health = damage.health_after(&attributes, state.health);
        let amount = state.health - new_health;
        if amount <= 0.0 {
            continue;
        }
        state.health = new_health;
        let died = !state.alive();
        world.write_message(SimulationEvent::Damaged {
            source,
            target,
            amount,
            kind: damage.kind(),
            origin: damage.origin(),
        });
        if died {
            let delay = world.get::<Unit>(target).expect("unit").respawn_delay;
            super::attributes::clear_on_death(&mut world.get_mut::<Unit>(target).expect("unit"));
            world
                .get_mut::<UnitState>(target)
                .expect("unit state")
                .respawn_remaining = delay;
            *world.get_mut::<UnitAction>(target).expect("unit action") = UnitAction::Ready;
            *world.get_mut::<UnitOrder>(target).expect("unit order") = UnitOrder::Idle;
            *world.get_mut::<Movement>(target).expect("unit movement") = Movement::default();
            world.write_message(SimulationEvent::Died { unit: target });
        }
    }
}

pub(super) fn respawn_units(
    mut units: Query<(
        Entity,
        &Unit,
        &mut UnitState,
        &mut UnitPose,
        &mut AbilitySlots,
    )>,
    mut events: MessageWriter<SimulationEvent>,
) {
    for (entity, unit, mut state, mut pose, mut slots) in &mut units {
        if state.alive() {
            continue;
        }
        let Some(remaining) = state.respawn_remaining.as_mut() else {
            continue;
        };
        *remaining -= TICK_SECONDS;
        if *remaining > 1e-5 {
            continue;
        }
        state.health = unit.stats.max_health;
        state.mana = unit.stats.max_mana;
        state.respawn_remaining = None;
        state.attack_cooldown = 0;
        slots.cooldowns = [0.0; 4];
        super::movement::reset_pose(&mut pose, unit.home);
        events.write(SimulationEvent::Respawned { unit: entity });
    }
}
