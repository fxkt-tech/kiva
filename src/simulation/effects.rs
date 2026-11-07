//! Immutable payloads and ordered hit packets, shared by every delivery type.
use super::*;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub(super) enum EffectSnapshot {
    Damage(DamageSnapshot),
    ApplyStatus(StatusSpec),
    Displace(DisplacementSpec),
    ArmorReduction { fraction: f32, duration: f32 },
}
#[derive(Clone, Debug)]
pub struct ImpactSnapshot {
    pub(super) source: UnitLifeRef,
    pub(super) team: Team,
    pub(super) origin: DamageOrigin,
    pub(super) position: Vec2,
    pub(super) facing: Vec2,
    pub(super) sequence: u64,
    pub(super) delivery: Delivery,
    effects: Arc<[EffectSnapshot]>,
    attack_passives: Arc<[ImpactSnapshot]>,
    pub(super) unit_passives: Arc<[AttackPassive]>,
    pub(super) attack_arrows: u8,
}
impl ImpactSnapshot {
    pub fn delivery(&self) -> Delivery {
        self.delivery
    }
}
#[derive(Clone, Copy)]
pub(super) enum ImpactTarget {
    Unit(UnitLifeRef),
    Point(Vec2),
    Direction(Vec2),
    SelfCast,
}
#[derive(Clone)]
pub(super) struct HitPacket {
    stage: u8,
    pub payload: ImpactSnapshot,
    carrier: u64,
    target: UnitLifeRef,
    center: Vec2,
    direction: Vec2,
}
#[derive(Resource, Default)]
pub(super) struct PendingImpacts {
    next_sequence: u64,
    packets: Vec<HitPacket>,
}
impl PendingImpacts {
    /// Discard hits from the finished trial while preserving sequence identity.
    pub(super) fn clear_pending(&mut self) {
        self.packets.clear();
    }

    fn sequence(&mut self) -> u64 {
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .expect("impact sequence space exhausted");
        self.next_sequence
    }
    pub fn hit(
        &mut self,
        stage: u8,
        payload: ImpactSnapshot,
        carrier: u64,
        target: UnitLifeRef,
        center: Vec2,
        direction: Vec2,
    ) {
        self.packets.push(HitPacket {
            stage,
            payload,
            carrier,
            target,
            center,
            direction,
        });
    }
}
/// Normalize with wide intermediates so legal large finite directions do not
/// overflow length_squared and turn a delivery or displacement into NaN.
pub(super) fn direction(value: Vec2, fallback: Vec2) -> Vec2 {
    let length = f64::from(value.x).hypot(f64::from(value.y));
    if length > 1e-9 && length.is_finite() {
        Vec2::new(
            (f64::from(value.x) / length) as f32,
            (f64::from(value.y) / length) as f32,
        )
    } else {
        fallback
    }
}

pub(super) fn snapshot(
    world: &mut World,
    source: Entity,
    origin: DamageOrigin,
    definition: &ImpactDefinition,
) -> Option<ImpactSnapshot> {
    let unit = world.get::<Unit>(source)?;
    let life = world.get::<UnitState>(source)?.life_ref(source);
    let pose = world.get::<UnitPose>(source)?;
    let mut effects = Vec::with_capacity(definition.effects.len());
    for effect in &definition.effects {
        effects.push(match *effect {
            Effect::Damage(spec) => {
                EffectSnapshot::Damage(spec.snapshot(unit.attributes(), origin)?)
            }
            Effect::ApplyStatus(spec) => EffectSnapshot::ApplyStatus(spec),
            Effect::Displace(spec) => EffectSnapshot::Displace(spec),
            Effect::BonusAttackDamage { damage, ratio } => {
                let damage = damage.snapshot(unit.attributes(), origin)?;
                let bonus = (unit.attributes().attack_damage()
                    - unit.stats.base_attributes.attack_damage)
                    .max(0.0);
                let raw = f64::from(damage.raw()) + f64::from(ratio) * f64::from(bonus);
                if !raw.is_finite() || raw > f64::from(f32::MAX) {
                    return None;
                }
                EffectSnapshot::Damage(damage.with_raw(raw as f32))
            }
            Effect::ArmorReduction { fraction, duration } => {
                EffectSnapshot::ArmorReduction { fraction, duration }
            }
        });
    }
    if definition
        .effects
        .iter()
        .any(|effect| matches!(effect, Effect::Damage(_) | Effect::BonusAttackDamage { .. }))
    {
        for passive in &unit.stats.attack_passives {
            if let AttackPassive::Frost { slow, duration } = *passive {
                effects.push(EffectSnapshot::ApplyStatus(StatusSpec::Slow {
                    fraction: slow,
                    duration: super::mechanics::duration(duration),
                }));
            }
        }
    }
    let unit_passives = unit.stats.attack_passives.clone();
    let (team, position, facing) = (
        unit.team(),
        pose.position(),
        direction(pose.facing(), Vec2::NEG_Y),
    );
    let sequence = world.resource_mut::<PendingImpacts>().sequence();
    Some(ImpactSnapshot {
        source: life,
        team,
        origin,
        position,
        facing,
        sequence,
        delivery: definition.delivery,
        effects: effects.into(),
        attack_passives: Arc::from([]),
        unit_passives: unit_passives.into(),
        attack_arrows: 1,
    })
}

/// A successful attack commits all attached effects before its attack interval.
pub(super) fn snapshot_attack(
    world: &mut World,
    source: Entity,
    definition: &ImpactDefinition,
) -> Option<ImpactSnapshot> {
    let mut payload = snapshot(world, source, DamageOrigin::BasicAttack, definition)?;
    payload.attack_arrows = world
        .get::<UnitMechanics>(source)
        .map_or(1, UnitMechanics::arrows);
    payload.attack_passives = super::equipment::snapshot_passives(world, source)?.into();
    Some(payload)
}

pub(super) fn emit(
    world: &mut World,
    payload: ImpactSnapshot,
    target: ImpactTarget,
    anchor: Vec2,
    range: f32,
) {
    let carrier = world.resource_mut::<PendingImpacts>().sequence();
    let target_position = match target {
        ImpactTarget::Unit(life) => world
            .get::<UnitPose>(life.entity())
            .map_or(anchor, UnitPose::position),
        ImpactTarget::Point(point) => point,
        _ => anchor,
    };
    let travel = match target {
        ImpactTarget::Direction(value) => direction(value, payload.facing),
        _ => direction(target_position - anchor, payload.facing),
    };
    let attack_arrows = payload.attack_arrows;
    match payload.delivery {
        Delivery::Direct => {
            let ImpactTarget::Unit(life) = target else {
                unreachable!("validated direct target");
            };
            // Direct effects use the caster's committed origin as their center.
            let center = payload.position;
            world
                .resource_mut::<PendingImpacts>()
                .hit(0, payload, carrier, life, center, travel);
        }
        Delivery::Projectile { speed, radius } => {
            let homing = if let ImpactTarget::Unit(life) = target {
                Some(life)
            } else {
                None
            };
            world.spawn(Projectile {
                payload,
                carrier,
                position: anchor,
                previous: anchor,
                direction: travel,
                remaining: range,
                speed,
                radius,
                homing,
                behavior: default(),
                arrows: attack_arrows,
            });
        }
        Delivery::Area { radius, delay } => {
            world.spawn(AreaEffect {
                payload,
                carrier,
                position: target_position,
                radius,
                remaining: delay,
                duration: delay,
            });
        }
    }
}

pub(super) fn resolve(world: &mut World) {
    let mut packets = std::mem::take(&mut world.resource_mut::<PendingImpacts>().packets);
    packets.sort_by_key(|p| {
        (
            p.stage,
            p.payload.sequence,
            p.carrier,
            p.target.entity().to_bits(),
        )
    });
    for packet in packets {
        let passives = super::attack_traits::prepare_hit(world, packet.target, &packet.payload);
        let damaged = resolve_payload(world, &packet, &packet.payload);
        if damaged {
            super::attack_traits::finish_hit(world, packet.target, &packet.payload, passives);
        }
        if damaged && packet.payload.origin == DamageOrigin::BasicAttack {
            // Only the ordinary attack owns this discovery step. Attached
            // snapshots contain no discovery of their own, so procs cannot recur.
            for passive in packet.payload.attack_passives.iter() {
                if !packet.target.is_current(world) {
                    break;
                }
                resolve_payload(world, &packet, passive);
            }
        }
    }
}

fn resolve_payload(world: &mut World, packet: &HitPacket, payload: &ImpactSnapshot) -> bool {
    if world
        .get::<Unit>(packet.target.entity())
        .is_none_or(|unit| !unit.class().accepts(payload.origin))
    {
        return false;
    }
    let mut damaged = false;
    for (index, effect) in payload.effects.iter().copied().enumerate() {
        if !packet.target.is_current(world) {
            break;
        }
        match effect {
            EffectSnapshot::Damage(damage) => {
                damaged |=
                    super::combat::apply_damage(world, payload.source, packet.target, damage) > 0.0;
            }
            EffectSnapshot::ApplyStatus(spec) => {
                super::statuses::apply(world, packet.target, payload, index, spec);
            }
            EffectSnapshot::ArmorReduction { fraction, duration } => {
                super::mechanics::armor_reduction(world, packet.target, payload, fraction, duration)
            }
            EffectSnapshot::Displace(spec) => super::movement::start_forced(
                world,
                packet.target,
                spec,
                packet.center,
                packet.direction,
                payload.facing,
            ),
        }
    }
    damaged
}

pub(super) fn append_status(mut payload: ImpactSnapshot, status: StatusSpec) -> ImpactSnapshot {
    let mut effects = payload.effects.to_vec();
    effects.push(EffectSnapshot::ApplyStatus(status));
    payload.effects = effects.into();
    payload
}
