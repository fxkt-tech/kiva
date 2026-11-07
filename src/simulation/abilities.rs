//! Target selection, delivery and ordered effects are independent definitions.
use super::{DamageSpec, StatusSpec, mechanics_defs::*};
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbilityId(pub u16);
/// Complete four-slot construction loadout; `None` is an empty slot.
pub type AbilityLoadout = [Option<AbilityId>; 4];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Targeting {
    Direction,
    Point,
    Enemy,
    SelfCast,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delivery {
    Direct,
    Projectile { speed: f32, radius: f32 },
    Area { radius: f32, delay: f32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplacementDirection {
    AlongImpact,
    AwayFromCenter,
    TowardCenter,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplacementSpec {
    pub distance: f32,
    pub speed: f32,
    pub direction: DisplacementDirection,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Damage(DamageSpec),
    ApplyStatus(StatusSpec),
    Displace(DisplacementSpec),
    BonusAttackDamage { damage: DamageSpec, ratio: f32 },
    ArmorReduction { fraction: f32, duration: f32 },
}
#[derive(Clone, Debug, PartialEq)]
pub struct ImpactDefinition {
    pub delivery: Delivery,
    pub effects: Vec<Effect>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum AbilityExecution {
    Impact(ImpactDefinition),
    Line(LineSpec),
    Shield(ShieldSpec),
    Standard(StandardSpec),
    Leap(LeapSpec),
    Empower(EmpowerSpec),
    Volley(VolleySpec),
    Scout(ScoutSpec),
    Arrow(ArrowSpec),
    Dash {
        speed: f32,
    },
    Channel {
        duration: f32,
        interval: f32,
        pulse: ImpactDefinition,
    },
}
impl AbilityExecution {
    pub fn requires_mobility(&self) -> bool {
        matches!(self, Self::Dash { .. } | Self::Leap(_))
    }
    pub fn is_dash(&self) -> bool {
        matches!(self, Self::Dash { .. })
    }
}
#[derive(Clone, Debug)]
pub struct AbilityDefinition {
    pub name: String,
    pub targeting: Targeting,
    pub range: f32,
    pub windup: f32,
    pub recovery: f32,
    pub cooldown: f32,
    pub mana_cost: f32,
    pub execution: AbilityExecution,
}
fn positive(v: f32) -> bool {
    v.is_finite() && v > 0.0
}
fn nonnegative(v: f32) -> bool {
    v.is_finite() && v >= 0.0
}
pub(super) fn positive_time(v: f32) -> bool {
    positive(v) && representable_time(v)
}
fn representable_time(v: f32) -> bool {
    nonnegative(v)
        && f64::from(v) * super::TICK_RATE <= f64::from(u32::MAX)
        // Float clocks must be able to advance by a fixed step. A finite
        // seconds value can fit the tick bound yet round every decrement away.
        && (v == 0.0 || v - super::TICK_SECONDS < v)
}
impl ImpactDefinition {
    pub(super) fn valid(&self, targeting: Targeting, range: f32) -> bool {
        let delivery_valid = match self.delivery {
            Delivery::Direct => targeting == Targeting::Enemy,
            Delivery::Projectile { speed, radius } => {
                matches!(targeting, Targeting::Direction | Targeting::Enemy)
                    && positive(speed)
                    && positive(radius)
                    && positive(range)
            }
            Delivery::Area { radius, delay } => {
                matches!(
                    targeting,
                    Targeting::Point | Targeting::Enemy | Targeting::SelfCast
                ) && positive(radius)
                    && representable_time(delay)
            }
        };
        delivery_valid
            && !self.effects.is_empty()
            && self.effects.iter().all(|effect| match effect {
                Effect::Damage(spec) => spec.valid(),
                Effect::ApplyStatus(spec) => spec.valid(),
                Effect::Displace(spec) => positive(spec.distance) && positive(spec.speed),
                Effect::BonusAttackDamage { damage, ratio } => {
                    damage.valid() && nonnegative(*ratio)
                }
                Effect::ArmorReduction { fraction, duration } => {
                    positive(*fraction)
                        && *fraction <= 1.0
                        && positive(*duration)
                        && representable_time(*duration)
                }
            })
    }
}
#[derive(Resource, Default)]
pub struct AbilityCatalog(BTreeMap<AbilityId, AbilityDefinition>);
impl AbilityCatalog {
    pub fn register(
        &mut self,
        id: AbilityId,
        definition: AbilityDefinition,
    ) -> Result<(), &'static str> {
        if definition.name.is_empty() || self.0.contains_key(&id) {
            return Err("empty name or duplicate ability id");
        }
        if !nonnegative(definition.range)
            || !nonnegative(definition.mana_cost)
            || ![definition.windup, definition.recovery, definition.cooldown]
                .into_iter()
                .all(representable_time)
        {
            return Err("invalid ability timing, range or cost");
        }
        let valid = match &definition.execution {
            AbilityExecution::Line(spec) => {
                definition.targeting == Targeting::Direction
                    && positive(spec.width)
                    && valid_effects(&spec.effects)
                    && spec.link.is_none_or(|link| {
                        positive(link.reach)
                            && positive(link.width)
                            && positive(link.speed)
                            && positive_time(link.knockup)
                    })
            }
            AbilityExecution::Shield(spec) => {
                definition.targeting == Targeting::SelfCast
                    && nonnegative(spec.base)
                    && nonnegative(spec.bonus_ad_ratio)
                    && nonnegative(spec.max_health_per_enemy_hero)
                    && positive(spec.radius)
                    && positive_time(spec.duration)
                    && spec.slow.is_none_or(StatusSpec::valid)
            }
            AbilityExecution::Standard(spec) => {
                definition.targeting == Targeting::Point
                    && spec.impact.valid(Targeting::Point, definition.range)
                    && positive_time(spec.duration)
                    && positive(spec.aura_radius)
                    && nonnegative(spec.attack_speed)
                    && nonnegative(spec.passive_attack_speed)
                    && positive(spec.vision_radius)
            }
            AbilityExecution::Leap(spec) => {
                definition.targeting == Targeting::Enemy
                    && positive_time(spec.duration)
                    && positive(spec.radius)
                    && valid_effects(&spec.effects)
                    && spec.terrain.is_none_or(|ring| {
                        positive(ring.radius)
                            && positive(ring.thickness)
                            && ring.thickness < ring.radius
                            && positive_time(ring.duration)
                            && representable_time(ring.recast_delay)
                    })
            }
            AbilityExecution::Empower(spec) => {
                definition.targeting == Targeting::SelfCast
                    && spec.required_stacks > 0
                    && positive_time(spec.stack_duration)
                    && positive_time(spec.stack_decay)
                    && positive_time(spec.duration)
                    && nonnegative(spec.attack_speed)
                    && positive(spec.attack_damage_multiplier)
                    && spec.arrows > 0
            }
            AbilityExecution::Volley(spec) => {
                definition.targeting == Targeting::Direction
                    && spec.count > 0
                    && nonnegative(spec.angle)
                    && spec.angle <= std::f32::consts::TAU
                    && positive(spec.speed)
                    && representable_time(definition.range / spec.speed)
                    && positive(spec.radius)
                    && valid_effects(&spec.effects)
                    && spec.hero_slow.is_none_or(StatusSpec::valid)
            }
            AbilityExecution::Scout(spec) => {
                definition.targeting == Targeting::Point
                    && positive(spec.speed)
                    && representable_time(definition.range / spec.speed + spec.destination_duration)
                    && positive(spec.vision_radius)
                    && positive_time(spec.path_duration)
                    && positive_time(spec.destination_duration)
                    && spec.charges > 0
                    && spec.initial_charges <= spec.charges
                    && positive_time(spec.recharge)
            }
            AbilityExecution::Arrow(spec) => {
                definition.targeting == Targeting::Direction
                    && positive(spec.speed)
                    && positive(spec.max_speed)
                    && spec.max_speed >= spec.speed
                    && nonnegative(spec.acceleration)
                    && positive(spec.radius)
                    && positive(spec.impact_radius)
                    && valid_effects(&spec.effects)
                    && positive_time(spec.stun_min)
                    && positive_time(spec.stun_max)
                    && spec.stun_max >= spec.stun_min
                    && positive(spec.stun_distance)
                    && positive(spec.vision_radius)
            }
            AbilityExecution::Impact(impact) => {
                impact.valid(definition.targeting, definition.range)
            }
            AbilityExecution::Dash { speed } => {
                definition.targeting == Targeting::Point
                    && positive(*speed)
                    && positive(definition.range)
            }
            AbilityExecution::Channel {
                duration,
                interval,
                pulse,
            } => {
                definition.targeting == Targeting::SelfCast
                    && positive(*duration)
                    && positive(*interval)
                    && representable_time(*duration)
                    && representable_time(*interval)
                    && *interval >= super::TICK_SECONDS
                    && *interval <= *duration
                    && matches!(pulse.delivery, Delivery::Area { .. })
                    && pulse.valid(Targeting::SelfCast, definition.range)
            }
        };
        if !valid {
            return Err("invalid execution, delivery or effect combination");
        }
        self.0.insert(id, definition);
        Ok(())
    }
    pub fn get(&self, id: AbilityId) -> Option<&AbilityDefinition> {
        self.0.get(&id)
    }
}

pub(super) fn validate_loadout(
    loadout: &AbilityLoadout,
    catalog: &AbilityCatalog,
) -> Result<(), (usize, AbilityId)> {
    for (slot, ability) in loadout.iter().enumerate() {
        if let Some(ability) = ability
            && catalog.get(*ability).is_none()
        {
            return Err((slot, *ability));
        }
    }
    Ok(())
}

fn valid_effects(effects: &[Effect]) -> bool {
    ImpactDefinition {
        delivery: Delivery::Area {
            radius: 1.0,
            delay: 0.0,
        },
        effects: effects.to_vec(),
    }
    .valid(Targeting::SelfCast, 0.0)
}
