//! Delivery geometry, health and life transitions. Effects resolve elsewhere.
use super::{effects::PendingImpacts, *};
use bevy::prelude::*;

pub(super) fn advance_projectiles(
    mut commands: Commands,
    map: Res<NavigationMap>,
    tick: Res<SimulationTick>,
    units: Query<(Entity, &Unit, &UnitState, &UnitPose)>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
    mut impacts: ResMut<PendingImpacts>,
    mut memory: ResMut<super::mechanics::MechanicMemory>,
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
        let behavior = projectile.behavior.clone();
        if let super::mechanics::ProjectileBehavior::Arrow { spec, .. } = &behavior {
            projectile.speed =
                (projectile.speed + spec.acceleration * TICK_SECONDS).min(spec.max_speed);
        }
        let distance = (projectile.speed * TICK_SECONDS).min(projectile.remaining);
        let requested = projectile.position + projectile.direction * distance;
        let end = map.projectile_sweep(projectile.position, requested, projectile.radius);
        let mut hit = None;
        for (id, unit, state, pose) in &units {
            if !state.alive()
                || unit.team == projectile.payload.team
                || !unit.class().accepts(projectile.payload.origin)
                || projectile
                    .homing
                    .is_some_and(|target| target != state.life_ref(id))
                || (matches!(behavior, super::mechanics::ProjectileBehavior::Arrow { .. })
                    && unit.stats.kind != UnitKind::Hero)
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
        if let Some((fraction, target)) = hit {
            let (_, target_unit, state, pose) = units.get(target).expect("hit target");
            let life = state.life_ref(target);
            let mut payload = projectile.payload.clone();
            let mut apply = true;
            match &behavior {
                super::mechanics::ProjectileBehavior::Volley {
                    group,
                    expires,
                    hero_slow,
                } => {
                    apply = memory.groups.insert((*group, life), *expires).is_none();
                    if target_unit.stats.kind == UnitKind::Hero
                        && let Some(slow) = hero_slow
                    {
                        payload = super::effects::append_status(payload, *slow);
                    }
                }
                super::mechanics::ProjectileBehavior::Arrow { spec, traveled } => {
                    let seconds = spec.stun_min
                        + (spec.stun_max - spec.stun_min)
                            * ((traveled + distance * fraction) / spec.stun_distance)
                                .clamp(0.0, 1.0);
                    payload = super::effects::append_status(
                        payload,
                        StatusSpec::Stun {
                            duration: super::mechanics::duration(seconds),
                        },
                    );
                    for (id, unit, other, other_pose) in &units {
                        if id != target
                            && other.alive()
                            && unit.team != projectile.payload.team
                            && unit.class().accepts(projectile.payload.origin)
                            && other_pose.position.distance(pose.position)
                                <= spec.impact_radius + unit.stats.radius
                        {
                            impacts.hit(
                                1,
                                projectile.payload.clone(),
                                projectile.carrier,
                                other.life_ref(id),
                                pose.position,
                                projectile.direction,
                            );
                        }
                    }
                }
                _ => {}
            }
            if apply {
                impacts.hit(
                    1,
                    payload,
                    projectile.carrier,
                    life,
                    projectile.payload.position,
                    projectile.direction,
                );
            }
        }
        projectile.position = end;
        projectile.remaining -= distance;
        if let super::mechanics::ProjectileBehavior::Arrow { traveled, .. } =
            &mut projectile.behavior
        {
            *traveled += distance;
        }
        if hit.is_some() || projectile.remaining <= 0.001 || end.distance_squared(requested) > 1e-6
        {
            if let super::mechanics::ProjectileBehavior::Arrow { spec, .. } = &behavior {
                let center = hit.map_or(end, |(_, target)| {
                    units.get(target).expect("hit").3.position
                });
                commands.spawn(super::battlefield::object(
                    projectile.payload.source,
                    projectile.payload.team,
                    center,
                    ObjectKind::Reveal,
                    spec.vision_radius,
                    spec.vision_radius,
                    tick.0 + super::mechanics::ticks(1.0),
                ));
            }
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
    if !world
        .get::<Unit>(target.entity())
        .expect("unit")
        .class()
        .accepts(damage.origin())
    {
        return 0.0;
    }
    let attributes = *world
        .get::<Unit>(target.entity())
        .expect("unit")
        .attributes();
    let tick = world.resource::<SimulationTick>().0;
    let hit = damage.mitigated_damage(&attributes);
    if hit <= 0.0 {
        return 0.0;
    }
    let remaining = super::mechanics::absorb(world, target.entity(), hit);
    let mut state = world.get_mut::<UnitState>(target.entity()).expect("state");
    let previous_health = state.health;
    state.health = (f64::from(previous_health) - remaining).max(0.0) as f32;
    let amount = previous_health - state.health;
    state.last_damaged_tick = Some(tick);
    let died = !state.alive();
    if amount > 0.0 {
        world.write_message(SimulationEvent::Damaged {
            source: source.entity(),
            target: target.entity(),
            amount,
            kind: damage.kind(),
            origin: damage.origin(),
        });
    }
    if died {
        die(world, target.entity());
    }
    hit.min(f64::from(f32::MAX)) as f32
}
fn die(world: &mut World, target: Entity) {
    let delay = world.get::<Unit>(target).expect("unit").respawn_delay;
    super::movement::clear_all(world, target);
    super::mechanics::clear_life(world, target);
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
        let (health, mana, home, radius) = (
            unit.stats.max_health,
            unit.stats.max_mana,
            unit.home,
            unit.stats.radius,
        );
        let Some(position) = world
            .resource::<NavigationMap>()
            .resolve_placement(home, radius)
        else {
            // Never publish a new life inside collision. If no safe local
            // point exists yet, retain the due respawn and retry next tick.
            continue;
        };
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
            state.last_damaged_tick = None;
            state.last_recovery_tick = None;
        }
        world
            .get_mut::<AbilitySlots>(entity)
            .expect("slots")
            .cooldowns = [0.0; 4];
        super::movement::clear_all(world, entity);
        super::movement::reset_pose(
            &mut world.get_mut::<UnitPose>(entity).expect("pose"),
            position,
        );
        super::statuses::clear(world, entity);
        world.write_message(SimulationEvent::Respawned { unit: entity });
    }
}
