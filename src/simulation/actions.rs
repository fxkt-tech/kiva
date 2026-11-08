//! Action clocks, interruptions and the single atomic skill commit point.
use super::{
    effects::{self, ImpactTarget},
    orders::{resolve_target, valid_enemy_life},
    *,
};
use bevy::prelude::*;

pub(super) fn interrupt_control(world: &mut World, entity: Entity, kind: StatusKind) {
    let cancel = match kind {
        StatusKind::Stun | StatusKind::Airborne => true,
        StatusKind::Root => match world.get::<UnitAction>(entity) {
            Some(UnitAction::Casting { ability, .. }) => world
                .resource::<AbilityCatalog>()
                .get(*ability)
                .is_some_and(|d| d.execution.requires_mobility()),
            _ => false,
        },
        StatusKind::Slow => false,
    };
    if cancel && let Some(mut action) = world.get_mut::<UnitAction>(entity) {
        *action = UnitAction::Ready;
    }
}

pub(super) fn advance_actions(world: &mut World) {
    super::equipment::advance_cooldowns(world);
    let mut actors: Vec<Entity> = world
        .query_filtered::<Entity, With<Unit>>()
        .iter(world)
        .collect();
    // Snapshot sequence allocation must not depend on archetype/query order.
    actors.sort_by_key(|id| id.to_bits());
    for entity in actors {
        if !world.get::<UnitState>(entity).is_some_and(UnitState::alive) {
            continue;
        }
        {
            let unit = world.get::<Unit>(entity).expect("unit");
            let (max_mana, regen) = (unit.stats.max_mana, unit.stats.mana_regen);
            let mut state = world.get_mut::<UnitState>(entity).expect("unit state");
            state.attack_cooldown = state.attack_cooldown.saturating_sub(1);
            state.mana = (state.mana + regen * TICK_SECONDS).min(max_mana);
        }
        super::recovery::regenerate_health(world, entity);
        for cooldown in &mut world
            .get_mut::<AbilitySlots>(entity)
            .expect("unit slots")
            .cooldowns
        {
            *cooldown = (*cooldown - TICK_SECONDS).max(0.0);
        }
        let action = world
            .get::<UnitAction>(entity)
            .expect("unit action")
            .clone();
        let next = match action {
            UnitAction::Ready => continue,
            UnitAction::Recovery { remaining } if remaining > TICK_SECONDS + 1e-6 => {
                UnitAction::Recovery {
                    remaining: remaining - TICK_SECONDS,
                }
            }
            UnitAction::Recovery { .. } => UnitAction::Ready,
            UnitAction::AttackBackswing {
                remaining,
                duration,
            } => {
                if remaining > 1 {
                    UnitAction::AttackBackswing {
                        remaining: remaining - 1,
                        duration,
                    }
                } else {
                    UnitAction::Ready
                }
            }
            UnitAction::AttackWindup {
                target,
                remaining,
                interval,
            } => {
                if !world
                    .get::<ControlState>(entity)
                    .is_some_and(ControlState::can_attack)
                    || !valid_enemy_life(world, entity, target)
                {
                    UnitAction::Ready
                } else if remaining > 1 {
                    UnitAction::AttackWindup {
                        target,
                        remaining: remaining - 1,
                        interval,
                    }
                } else if release_attack(world, entity, target, interval) {
                    UnitAction::AttackBackswing {
                        remaining: interval,
                        duration: interval,
                    }
                } else {
                    UnitAction::Ready
                }
            }
            UnitAction::Casting {
                source,
                ability,
                target,
                target_life,
                remaining,
            } => {
                if remaining > TICK_SECONDS + 1e-6 {
                    UnitAction::Casting {
                        source,
                        ability,
                        target,
                        target_life,
                        remaining: remaining - TICK_SECONDS,
                    }
                } else {
                    commit_cast(world, entity, source, ability, target, target_life)
                }
            }
            UnitAction::Channeling {
                remaining,
                next_pulse,
                interval,
                pulse,
                recovery,
            } => {
                let remaining = remaining - TICK_SECONDS;
                let active_time = TICK_SECONDS.min(remaining + TICK_SECONDS);
                let mut next_pulse = next_pulse - active_time;
                while next_pulse <= 1e-5 {
                    let position = world.get::<UnitPose>(entity).expect("unit pose").position;
                    effects::emit(world, pulse.clone(), ImpactTarget::SelfCast, position, 0.0);
                    next_pulse += interval;
                }
                if remaining <= 1e-5 {
                    UnitAction::Recovery {
                        remaining: recovery,
                    }
                } else {
                    UnitAction::Channeling {
                        remaining,
                        next_pulse,
                        interval,
                        pulse,
                        recovery,
                    }
                }
            }
        };
        *world.get_mut::<UnitAction>(entity).expect("unit action") = next;
    }
}

fn release_attack(world: &mut World, entity: Entity, target: UnitLifeRef, interval: u32) -> bool {
    let unit = world.get::<Unit>(entity).expect("unit");
    let origin = world.get::<UnitPose>(entity).expect("unit pose").position;
    let target_position = world
        .get::<UnitPose>(target.entity())
        .expect("target pose")
        .position;
    let target_radius = world
        .get::<Unit>(target.entity())
        .expect("target")
        .stats
        .radius;
    if origin.distance(target_position)
        > unit.attributes().attack_range() + unit.stats.radius + target_radius
        || !world
            .resource::<NavigationMap>()
            .projectile_line_clear(origin, target_position, 0.05)
    {
        return false;
    }
    let definition = ImpactDefinition {
        delivery: unit.stats.attack_delivery.delivery(),
        effects: vec![Effect::Damage(DamageSpec {
            kind: DamageType::Physical,
            base: 0.0,
            ad_ratio: world
                .get::<UnitMechanics>(entity)
                .map_or(1.0, |state| state.multiplier),
            ap_ratio: 0.0,
        })],
    };
    let Some(payload) = effects::snapshot_attack(world, entity, &definition) else {
        world.write_message(SimulationEvent::CommandRejected {
            unit: entity,
            reason: Rejection::InvalidDamage,
        });
        return false;
    };
    world
        .get_mut::<UnitState>(entity)
        .expect("unit state")
        .attack_cooldown = interval;
    super::attack_traits::released(world, entity, target);
    effects::emit(world, payload, ImpactTarget::Unit(target), origin, 100.0);
    world.write_message(SimulationEvent::AttackReleased {
        source: entity,
        target: target.entity(),
    });
    true
}

/// Read-only cast preflight shared by autonomous decisions and actual starts.
/// The returned target is resolved for the current pose only; submission and
/// commit still validate again, and this function never spends or queues work.
pub fn preview_cast(
    world: &World,
    entity: Entity,
    source: CastSource,
    target: AbilityTarget,
) -> Result<AbilityTarget, Rejection> {
    let state = world
        .get::<UnitState>(entity)
        .ok_or(Rejection::InvalidUnit)?;
    if !state.alive() {
        return Err(Rejection::Dead);
    }
    if !super::movement::motion_allows_cast(world, entity) {
        return Err(Rejection::Busy);
    }
    let (ability, cooldown, _) = cast_binding(world, entity, source)?;
    if super::compound::is_recast(world, entity, source) {
        if target != AbilityTarget::SelfCast {
            return Err(Rejection::InvalidTarget);
        }
        if !world
            .get::<ControlState>(entity)
            .is_some_and(ControlState::can_cast)
        {
            return Err(Rejection::Controlled);
        }
        if let CastSource::AbilitySlot(slot) = source
            && world
                .get::<UnitMechanics>(entity)
                .is_some_and(|state| state.recast_ready(slot, world.resource::<SimulationTick>().0))
        {
            return Ok(AbilityTarget::SelfCast);
        }
        return Err(Rejection::OnCooldown);
    }
    if cooldown > 1e-5 {
        return Err(Rejection::OnCooldown);
    }
    let definition = world
        .resource::<AbilityCatalog>()
        .get(ability)
        .ok_or(Rejection::NoAbility)?;
    if !world
        .get::<ControlState>(entity)
        .is_some_and(|control| control.permits(definition))
    {
        return Err(Rejection::Controlled);
    }
    if world
        .get::<UnitState>(entity)
        .ok_or(Rejection::InvalidUnit)?
        .mana
        < definition.mana_cost
    {
        return Err(Rejection::InsufficientMana);
    }
    if !matches!(
        world.get::<UnitAction>(entity),
        Some(
            UnitAction::Ready
                | UnitAction::Recovery { .. }
                | UnitAction::AttackWindup { .. }
                | UnitAction::AttackBackswing { .. }
        )
    ) {
        return Err(Rejection::Busy);
    }
    super::compound::preflight(world, entity, source, definition)?;
    resolve_target(world, entity, definition, target)
}

/// Both command sources use this same validation before replacing an action.
pub(super) fn begin_cast(
    world: &mut World,
    entity: Entity,
    source: CastSource,
    target: AbilityTarget,
) -> Result<(), Rejection> {
    // Retain the original click; resolve from the current pose at commit,
    // after windup and before spending any resources.
    preview_cast(world, entity, source, target)?;
    if super::compound::is_recast(world, entity, source) {
        super::compound::recast(world, entity, source)?;
        let (ability, _, _) = cast_binding(world, entity, source)?;
        super::facts::ability_committed(
            world,
            entity,
            ability,
            source,
            AbilityTarget::SelfCast,
            None,
            true,
        );
        return Ok(());
    }
    let (ability, _, _) = cast_binding(world, entity, source).expect("validated binding");
    let remaining = world
        .resource::<AbilityCatalog>()
        .get(ability)
        .expect("validated ability")
        .windup;
    let target_life = if let AbilityTarget::Unit(target) = target {
        Some(
            world
                .get::<UnitState>(target)
                .expect("validated target")
                .life_ref(target),
        )
    } else {
        None
    };
    *world
        .get_mut::<UnitAction>(entity)
        .expect("validated action") = UnitAction::Casting {
        source,
        ability,
        target,
        target_life,
        remaining,
    };
    super::movement::clear_route(
        &mut world
            .get_mut::<super::model::Movement>(entity)
            .expect("unit movement"),
    );
    Ok(())
}

pub(super) fn cast_binding(
    world: &World,
    unit: Entity,
    source: CastSource,
) -> Result<(AbilityId, f32, DamageOrigin), Rejection> {
    match source {
        CastSource::AbilitySlot(slot) => {
            let slots = world
                .get::<AbilitySlots>(unit)
                .ok_or(Rejection::NoAbility)?;
            let ability = slots.ability(slot).ok_or(Rejection::NoAbility)?;
            Ok((
                ability,
                slots.cooldown(slot).expect("known slot"),
                DamageOrigin::Ability(ability),
            ))
        }
        CastSource::EquipmentSlot { slot, instance } => {
            let (equipment, ability, cooldown) =
                super::equipment::binding(world, unit, slot, instance)
                    .ok_or(Rejection::NoAbility)?;
            Ok((
                ability,
                cooldown,
                DamageOrigin::EquipmentActive {
                    equipment,
                    instance,
                    ability,
                },
            ))
        }
    }
}

fn commit_cooldown(world: &mut World, unit: Entity, source: CastSource, cooldown: f32) {
    match source {
        CastSource::AbilitySlot(slot) => {
            world
                .get_mut::<AbilitySlots>(unit)
                .expect("validated slots")
                .cooldowns[slot] = cooldown
        }
        CastSource::EquipmentSlot { slot, instance } => {
            super::equipment::commit_cooldown(world, unit, slot, instance, cooldown)
        }
    }
}

pub(super) fn cancel_equipment_cast(
    world: &mut World,
    unit: Entity,
    instance: EquipmentInstanceId,
) {
    super::cast_orders::cancel_equipment(world, unit, instance);
    let cancel = match world.get::<UnitAction>(unit) {
        Some(UnitAction::Casting {
            source: CastSource::EquipmentSlot {
                instance: current, ..
            },
            ..
        }) => *current == instance,
        Some(UnitAction::Channeling { pulse, .. }) => {
            matches!(pulse.origin, DamageOrigin::EquipmentActive { instance: current, .. } if current == instance)
        }
        _ => false,
    };
    if cancel {
        *world.get_mut::<UnitAction>(unit).expect("matched action") = UnitAction::Ready;
    }
}

fn reject_cast(world: &mut World, unit: Entity, reason: Rejection) -> UnitAction {
    world.write_message(SimulationEvent::CommandRejected { unit, reason });
    UnitAction::Ready
}

fn commit_cast(
    world: &mut World,
    entity: Entity,
    source: CastSource,
    id: AbilityId,
    target: AbilityTarget,
    target_life: Option<UnitLifeRef>,
) -> UnitAction {
    let binding = cast_binding(world, entity, source);
    let (current, cooldown, damage_origin) = match binding {
        Ok(binding) if binding.0 == id => binding,
        _ => return reject_cast(world, entity, Rejection::NoAbility),
    };
    let definition = world
        .resource::<AbilityCatalog>()
        .get(current)
        .expect("resolved ability")
        .clone();
    let validation = resolve_target(world, entity, &definition, target).and_then(|resolved| {
        if cooldown > 1e-5 {
            return Err(Rejection::OnCooldown);
        }
        if target_life.is_some_and(|life| !life.is_current(world)) {
            return Err(Rejection::InvalidTarget);
        }
        if !world
            .get::<ControlState>(entity)
            .is_some_and(|c| c.permits(&definition))
        {
            return Err(Rejection::Controlled);
        }
        super::compound::preflight(world, entity, source, &definition)?;
        if world.get::<UnitState>(entity).expect("unit state").mana < definition.mana_cost {
            Err(Rejection::InsufficientMana)
        } else {
            Ok(resolved)
        }
    });
    let target = match validation {
        Ok(target) => target,
        Err(reason) => return reject_cast(world, entity, reason),
    };
    let impact = match &definition.execution {
        AbilityExecution::Impact(impact) | AbilityExecution::Channel { pulse: impact, .. } => {
            let Some(snapshot) = effects::snapshot(world, entity, damage_origin, impact) else {
                world.write_message(SimulationEvent::CommandRejected {
                    unit: entity,
                    reason: Rejection::InvalidDamage,
                });
                return UnitAction::Ready;
            };
            Some(snapshot)
        }
        _ => {
            if let Some(impact) = super::compound::impact_definition(&definition.execution) {
                let Some(snapshot) = effects::snapshot(world, entity, damage_origin, &impact)
                else {
                    return reject_cast(world, entity, Rejection::InvalidDamage);
                };
                Some(snapshot)
            } else {
                None
            }
        }
    };
    // All targets, permissions and every effect snapshot are valid before costs.
    // Only a committed cast replaces a ground destination; a rejected windup
    // must retain the old purpose. Ordinary spells preserve explicit attacks.
    // Autonomous movement separately replaces those when it actually starts.
    if let Some(mut order) = world.get_mut::<UnitOrder>(entity)
        && matches!(*order, UnitOrder::MoveTo(_) | UnitOrder::AttackMove { .. })
    {
        *order = UnitOrder::Idle;
    }
    world.get_mut::<UnitState>(entity).expect("unit state").mana -= definition.mana_cost;
    commit_cooldown(world, entity, source, definition.cooldown);
    let origin = world.get::<UnitPose>(entity).expect("unit pose").position;
    let point = match target {
        AbilityTarget::Point(p) => p,
        AbilityTarget::Unit(t) => world.get::<UnitPose>(t).expect("validated pose").position,
        _ => origin,
    };
    super::facts::ability_committed(world, entity, id, source, target, target_life, false);
    let recovery = UnitAction::Recovery {
        remaining: definition.recovery,
    };
    match definition.execution.clone() {
        AbilityExecution::Impact(_) => {
            let target = match target {
                AbilityTarget::Unit(_) => {
                    ImpactTarget::Unit(target_life.expect("locked enemy life"))
                }
                AbilityTarget::Point(point) => ImpactTarget::Point(point),
                AbilityTarget::Direction(direction) => ImpactTarget::Direction(direction),
                AbilityTarget::SelfCast => ImpactTarget::SelfCast,
            };
            effects::emit(
                world,
                impact.expect("prepared impact"),
                target,
                origin,
                definition.range,
            );
            recovery
        }
        AbilityExecution::Dash { speed } => {
            super::movement::start_dash(world, entity, point, speed);
            recovery
        }
        AbilityExecution::Channel {
            duration, interval, ..
        } => UnitAction::Channeling {
            remaining: duration,
            next_pulse: interval,
            interval,
            pulse: impact.expect("prepared pulse"),
            recovery: definition.recovery,
        },
        _ => {
            super::compound::execute(
                world,
                entity,
                source,
                damage_origin,
                &definition,
                target,
                impact,
            );
            recovery
        }
    }
}
