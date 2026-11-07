//! Target selection, delivery and ordered effects are independent definitions.
use super::{DamageSpec, StatusSpec};
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
}
#[derive(Clone, Debug, PartialEq)]
pub struct ImpactDefinition {
    pub delivery: Delivery,
    pub effects: Vec<Effect>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum AbilityExecution {
    Impact(ImpactDefinition),
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
