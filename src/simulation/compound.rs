//! Compound executions coordinate existing action, impact and movement owners.
use super::{
    effects::{self, ImpactTarget},
    mechanics::{self, ProjectileBehavior},
    *,
};
use bevy::prelude::*;
use std::collections::BTreeSet;

#[derive(Component, Clone, Debug)]
pub(super) struct CompoundMotion {
    pub destination: Vec2,
    pub speed: f32,
    pub done: bool,
    pub previous: Vec2,
    pub source: UnitLifeRef,
    pub origin: Vec2,
    pub lifetime: Option<(u64, u64)>,
    pub payload: ImpactSnapshot,
    pub kind: MotionKind,
}
#[derive(Clone, Debug)]
pub(super) enum MotionKind {
    Link {
        traverse_terrain: bool,
        can_cast_during_motion: bool,
        width: f32,
        hit: BTreeSet<UnitLifeRef>,
    },
    Leap {
        radius: f32,
        terrain: Option<RingTerrainSpec>,
        slot: Option<usize>,
    },
}
pub(super) fn preflight(
    world: &World,
    entity: Entity,
    source: CastSource,
    definition: &AbilityDefinition,
) -> Result<(), Rejection> {
    let state = world
        .get::<UnitMechanics>(entity)
        .ok_or(Rejection::InvalidUnit)?;
    match definition.execution {
        AbilityExecution::Empower(spec)
            if state.focus() < spec.required_stacks || state.empowered() =>
        {
            Err(Rejection::InsufficientStacks)
        }
        AbilityExecution::Scout(_) => {
            if let CastSource::AbilitySlot(slot) = source {
                if state.charges(slot).is_some_and(|c| c > 0) {
                    Ok(())
                } else {
                    Err(Rejection::NoCharges)
                }
            } else {
                Err(Rejection::NoCharges)
            }
        }
        AbilityExecution::Shield(spec) => {
            if shield_amount(world, entity, &spec).is_some() {
                Ok(())
            } else {
                Err(Rejection::InvalidDamage)
            }
        }
        _ => Ok(()),
    }
}
pub(super) fn is_recast(world: &World, entity: Entity, source: CastSource) -> bool {
    if let CastSource::AbilitySlot(slot) = source {
        world
            .get::<UnitMechanics>(entity)
            .is_some_and(|state| state.recast(slot))
    } else {
        false
    }
}
pub(super) fn recast(
    world: &mut World,
    entity: Entity,
    source: CastSource,
) -> Result<(), Rejection> {
    let CastSource::AbilitySlot(slot) = source else {
        return Err(Rejection::InvalidTarget);
    };
    let tick = world.resource::<SimulationTick>().0;
    let state = world
        .get::<UnitMechanics>(entity)
        .ok_or(Rejection::InvalidUnit)?;
    if !state.recast_ready(slot, tick) {
        return Err(Rejection::OnCooldown);
    }
    let object = state.recasts[slot].expect("recast").object;
    world.resource_mut::<NavigationMap>().remove_ring(object);
    world.despawn(object);
    world
        .get_mut::<UnitMechanics>(entity)
        .expect("mechanics")
        .recasts[slot] = None;
    Ok(())
}
pub(super) fn impact_definition(execution: &AbilityExecution) -> Option<ImpactDefinition> {
    let effects = match execution {
        AbilityExecution::Line(spec) => spec.effects.clone(),
        AbilityExecution::Shield(spec) => spec.slow.into_iter().map(Effect::ApplyStatus).collect(),
        AbilityExecution::Standard(spec) => return Some(spec.impact.clone()),
        AbilityExecution::Leap(spec) => spec.effects.clone(),
        AbilityExecution::Volley(spec) => spec.effects.clone(),
        AbilityExecution::Arrow(spec) => spec.effects.clone(),
        _ => return None,
    };
    Some(ImpactDefinition {
        delivery: Delivery::Area {
            radius: 1.0,
            delay: 0.0,
        },
        effects,
    })
}

pub(super) fn execute(
    world: &mut World,
    entity: Entity,
    source: CastSource,
    definition: &AbilityDefinition,
    target: AbilityTarget,
    payload: Option<ImpactSnapshot>,
) {
    let position = world.get::<UnitPose>(entity).expect("pose").position;
    let life = world
        .get::<UnitState>(entity)
        .expect("state")
        .life_ref(entity);
    let team = world.get::<Unit>(entity).expect("unit").team;
    let tick = world.resource::<SimulationTick>().0;
    match &definition.execution {
        AbilityExecution::Line(spec) => {
            let AbilityTarget::Direction(value) = target else {
                unreachable!()
            };
            let direction = effects::direction(value, Vec2::X);
            let end = position + direction * definition.range;
            let payload = payload.expect("line snapshot");
            queue_line(
                world,
                &payload,
                position,
                end,
                spec.width,
                &mut BTreeSet::new(),
            );
            if let Some(link) = spec.link {
                let flag = world
                    .query::<&BattlefieldObject>()
                    .iter(world)
                    .filter(|object| {
                        object.kind == ObjectKind::Standard
                            && object.owner == life
                            && object.active_at <= tick
                    })
                    .filter_map(|object| {
                        let delta = object.position - position;
                        let along = delta.dot(direction);
                        let across = (delta - direction * along).length();
                        (along >= 0.0 && along <= link.reach && across <= link.width * 0.5)
                            .then_some((along, object.position))
                    })
                    .max_by(|a, b| a.0.total_cmp(&b.0));
                if let Some((_, destination)) = flag {
                    let knockup = ImpactDefinition {
                        delivery: Delivery::Area {
                            radius: 1.0,
                            delay: 0.0,
                        },
                        effects: vec![Effect::ApplyStatus(StatusSpec::Airborne {
                            duration: mechanics::duration(link.knockup),
                        })],
                    };
                    let origin = payload.origin;
                    if let Some(payload) = effects::snapshot(world, entity, origin, &knockup) {
                        super::movement::start_compound(
                            world,
                            entity,
                            CompoundMotion {
                                destination,
                                speed: link.speed,
                                done: false,
                                previous: position,
                                source: life,
                                origin: position,
                                lifetime: None,
                                payload,
                                kind: MotionKind::Link {
                                    traverse_terrain: link.traverse_terrain,
                                    can_cast_during_motion: link.can_cast_during_motion,
                                    width: link.width,
                                    hit: BTreeSet::new(),
                                },
                            },
                        );
                    }
                }
            }
        }
        AbilityExecution::Shield(spec) => {
            let amount = shield_amount(world, entity, spec).expect("preflight shield amount");
            world
                .get_mut::<UnitMechanics>(entity)
                .expect("mechanics")
                .shields
                .push(mechanics::ShieldInstance {
                    amount,
                    until: tick + mechanics::ticks(spec.duration),
                });
            if let Some(payload) = payload {
                queue_area(world, &payload, position, spec.radius, None);
            }
        }
        AbilityExecution::Standard(spec) => {
            let AbilityTarget::Point(point) = target else {
                unreachable!()
            };
            effects::emit(
                world,
                payload.expect("standard snapshot"),
                ImpactTarget::Point(point),
                position,
                definition.range,
            );
            super::battlefield::spawn_standard(world, life, team, point, spec);
        }
        AbilityExecution::Leap(spec) => {
            let AbilityTarget::Unit(target) = target else {
                unreachable!()
            };
            let destination = world.get::<UnitPose>(target).expect("target pose").position;
            let slot = if let CastSource::AbilitySlot(slot) = source {
                Some(slot)
            } else {
                None
            };
            super::movement::start_compound(
                world,
                entity,
                CompoundMotion {
                    destination,
                    speed: position.distance(destination) / spec.duration,
                    done: false,
                    previous: position,
                    source: life,
                    origin: position,
                    lifetime: Some((
                        mechanics::ticks(spec.duration),
                        mechanics::ticks(spec.duration),
                    )),
                    payload: payload.expect("leap snapshot"),
                    kind: MotionKind::Leap {
                        radius: spec.radius,
                        terrain: spec.terrain,
                        slot,
                    },
                },
            );
        }
        AbilityExecution::Empower(spec) => {
            let modifier_source = {
                let mut state = world.get_mut::<UnitMechanics>(entity).expect("mechanics");
                state.focus = 0;
                state.focus_until = 0;
                state.empowered_until = tick + mechanics::ticks(spec.duration);
                state.multiplier = spec.attack_damage_multiplier;
                state.arrows = spec.arrows;
                state.empower_source
            };
            let modifier = AttributeModifier {
                additive_percent: AttributeDelta {
                    attack_speed: spec.attack_speed,
                    ..default()
                },
                removal: ModifierRemoval::OnDeath,
                ..default()
            };
            let _ = attributes::set_immediate(world, entity, modifier_source, Some(modifier));
            if spec.reset_attack {
                world
                    .get_mut::<UnitState>(entity)
                    .expect("state")
                    .attack_cooldown = 0;
            }
        }
        AbilityExecution::Volley(spec) => {
            let AbilityTarget::Direction(value) = target else {
                unreachable!()
            };
            let direction = effects::direction(value, Vec2::X);
            let payload = payload.expect("volley snapshot");
            let group = payload.sequence;
            for index in 0..spec.count {
                let angle = if spec.count == 1 {
                    0.0
                } else {
                    -spec.angle * 0.5 + spec.angle * index as f32 / (spec.count - 1) as f32
                };
                let travel = Vec2::from_angle(angle).rotate(direction);
                let mut payload = payload.clone();
                payload.delivery = Delivery::Projectile {
                    speed: spec.speed,
                    radius: spec.radius,
                };
                spawn_projectile(
                    world,
                    payload,
                    position,
                    travel,
                    definition.range,
                    ProjectileBehavior::Volley {
                        group,
                        expires: tick + mechanics::ticks(definition.range / spec.speed) + 2,
                        hero_slow: spec.hero_slow,
                    },
                    1,
                );
            }
        }
        AbilityExecution::Scout(spec) => {
            let AbilityTarget::Point(destination) = target else {
                unreachable!()
            };
            if let CastSource::AbilitySlot(slot) = source {
                let mut state = world.get_mut::<UnitMechanics>(entity).expect("mechanics");
                let charge = state.charges[slot].as_mut().expect("scout charges");
                charge.count -= 1;
                if charge.next.is_none() {
                    charge.next = Some(tick + charge.recharge);
                }
            }
            super::battlefield::spawn_scout(world, life, team, position, destination, *spec);
        }
        AbilityExecution::Arrow(spec) => {
            let AbilityTarget::Direction(value) = target else {
                unreachable!()
            };
            let mut payload = payload.expect("arrow snapshot");
            payload.delivery = Delivery::Projectile {
                speed: spec.speed,
                radius: spec.radius,
            };
            spawn_projectile(
                world,
                payload,
                position,
                effects::direction(value, Vec2::X),
                definition.range,
                ProjectileBehavior::Arrow {
                    spec: spec.clone(),
                    traveled: 0.0,
                },
                1,
            );
        }
        _ => unreachable!("compound execution"),
    }
}
fn spawn_projectile(
    world: &mut World,
    payload: ImpactSnapshot,
    position: Vec2,
    direction: Vec2,
    range: f32,
    behavior: ProjectileBehavior,
    arrows: u8,
) {
    let Delivery::Projectile { speed, radius } = payload.delivery else {
        unreachable!()
    };
    world.spawn(Projectile {
        carrier: payload.sequence,
        payload,
        position,
        previous: position,
        direction,
        remaining: range,
        speed,
        radius,
        homing: None,
        behavior,
        arrows,
    });
}
pub(super) fn queue_area(
    world: &mut World,
    payload: &ImpactSnapshot,
    center: Vec2,
    radius: f32,
    exclude: Option<Entity>,
) {
    let targets: Vec<_> = world
        .query::<(Entity, &Unit, &UnitState, &UnitPose)>()
        .iter(world)
        .filter(|(entity, unit, state, pose)| {
            Some(*entity) != exclude
                && state.alive()
                && unit.team != payload.team
                && unit.class().accepts(payload.origin)
                && pose.position.distance(center) <= radius + unit.stats.radius
        })
        .map(|(entity, _, state, _)| state.life_ref(entity))
        .collect();
    for target in targets {
        world.resource_mut::<effects::PendingImpacts>().hit(
            1,
            payload.clone(),
            payload.sequence,
            target,
            center,
            payload.facing,
        );
    }
}
fn queue_line(
    world: &mut World,
    payload: &ImpactSnapshot,
    start: Vec2,
    end: Vec2,
    width: f32,
    hit: &mut BTreeSet<UnitLifeRef>,
) {
    let segment = end - start;
    let length = segment.length_squared();
    let targets: Vec<_> = world
        .query::<(Entity, &Unit, &UnitState, &UnitPose)>()
        .iter(world)
        .filter(|(_, unit, state, pose)| {
            let t = if length > 1e-8 {
                (pose.position - start).dot(segment) / length
            } else {
                0.0
            };
            state.alive()
                && unit.team != payload.team
                && unit.class().accepts(payload.origin)
                && pose.position.distance(start + segment * t.clamp(0.0, 1.0))
                    <= width * 0.5 + unit.stats.radius
        })
        .map(|(entity, _, state, _)| state.life_ref(entity))
        .collect();
    for target in targets {
        if hit.insert(target) {
            world.resource_mut::<effects::PendingImpacts>().hit(
                1,
                payload.clone(),
                payload.sequence,
                target,
                start,
                effects::direction(segment, payload.facing),
            );
        }
    }
}
pub(super) fn advance_motion_impacts(world: &mut World) {
    let motions: Vec<_> = world
        .query::<(Entity, &CompoundMotion, &UnitPose)>()
        .iter(world)
        .map(|(id, motion, pose)| (id, motion.clone(), pose.position))
        .collect();
    for (entity, mut motion, position) in motions {
        if !motion.source.is_current(world) {
            world.entity_mut(entity).remove::<CompoundMotion>();
            continue;
        }
        match &mut motion.kind {
            MotionKind::Link { width, hit, .. } => queue_line(
                world,
                &motion.payload,
                motion.previous,
                position,
                *width,
                hit,
            ),
            MotionKind::Leap {
                radius,
                terrain,
                slot,
            } if motion.done => {
                queue_area(world, &motion.payload, position, *radius, None);
                if let Some(spec) = *terrain {
                    super::battlefield::spawn_ring(
                        world,
                        motion.source,
                        motion.payload.team,
                        position,
                        spec,
                        *slot,
                    );
                }
            }
            _ => {}
        }
        if motion.done {
            world.entity_mut(entity).remove::<CompoundMotion>();
        } else {
            motion.previous = position;
            world.entity_mut(entity).insert(motion);
        }
    }
}
pub(super) fn advance_objects(world: &mut World) {
    super::battlefield::advance(world);
}

fn shield_amount(world: &World, entity: Entity, spec: &ShieldSpec) -> Option<f32> {
    let unit = world.get::<Unit>(entity)?;
    let origin = world.get::<UnitPose>(entity)?.position;
    let bonus =
        (unit.attributes().attack_damage() - unit.stats.base_attributes.attack_damage).max(0.0);
    let enemies = world
        .iter_entities()
        .filter(|other| {
            other.get::<Unit>().is_some_and(|enemy| {
                enemy.team != unit.team
                    && enemy.stats.kind == UnitKind::Hero
                    && other.get::<UnitState>().is_some_and(UnitState::alive)
                    && other.get::<UnitPose>().is_some_and(|pose| {
                        pose.position.distance(origin) <= spec.radius + enemy.stats.radius
                    })
            })
        })
        .count();
    let amount = f64::from(spec.base)
        + f64::from(spec.bonus_ad_ratio) * f64::from(bonus)
        + f64::from(spec.max_health_per_enemy_hero)
            * f64::from(unit.stats.max_health)
            * enemies as f64;
    (amount.is_finite() && amount <= f64::from(f32::MAX)).then_some(amount as f32)
}
