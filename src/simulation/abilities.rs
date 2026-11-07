//! Definitions contain rules, never key codes, meshes, or UI state.
use super::DamageSpec;
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbilityId(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Targeting {
    Direction,
    Point,
    Enemy,
    SelfCast,
}
#[derive(Clone, Debug)]
pub enum AbilityEffect {
    Projectile {
        speed: f32,
        radius: f32,
        damage: DamageSpec,
    },
    Area {
        radius: f32,
        delay: f32,
        damage: DamageSpec,
    },
    Dash {
        speed: f32,
    },
    Channel {
        duration: f32,
        interval: f32,
        radius: f32,
        damage: DamageSpec,
    },
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
    pub effect: AbilityEffect,
}
#[derive(Resource, Default)]
pub struct AbilityCatalog(BTreeMap<AbilityId, AbilityDefinition>);
impl AbilityCatalog {
    pub fn register(
        &mut self,
        id: AbilityId,
        definition: AbilityDefinition,
    ) -> Result<(), &'static str> {
        let nonnegative = |v: f32| v.is_finite() && v >= 0.0;
        let positive = |v: f32| v.is_finite() && v > 0.0;
        if definition.name.is_empty() || self.0.contains_key(&id) {
            return Err("empty name or duplicate ability id");
        }
        if ![
            definition.range,
            definition.windup,
            definition.recovery,
            definition.cooldown,
            definition.mana_cost,
        ]
        .into_iter()
        .all(nonnegative)
        {
            return Err("invalid ability timing, range or cost");
        }
        let valid = match definition.effect {
            AbilityEffect::Projectile {
                speed,
                radius,
                damage,
            } => {
                definition.targeting == Targeting::Direction
                    && positive(speed)
                    && positive(radius)
                    && damage.valid()
                    && positive(definition.range)
            }
            AbilityEffect::Area {
                radius,
                delay,
                damage,
            } => {
                matches!(
                    definition.targeting,
                    Targeting::Point | Targeting::Enemy | Targeting::SelfCast
                ) && positive(radius)
                    && nonnegative(delay)
                    && damage.valid()
            }
            AbilityEffect::Dash { speed } => {
                definition.targeting == Targeting::Point
                    && positive(speed)
                    && positive(definition.range)
            }
            AbilityEffect::Channel {
                duration,
                interval,
                radius,
                damage,
            } => {
                definition.targeting == Targeting::SelfCast
                    && positive(duration)
                    && positive(interval)
                    && interval >= super::TICK_SECONDS
                    && interval <= duration
                    && positive(radius)
                    && damage.valid()
            }
        };
        if !valid {
            return Err("invalid ability effect or targeting combination");
        }
        self.0.insert(id, definition);
        Ok(())
    }
    pub fn get(&self, id: AbilityId) -> Option<&AbilityDefinition> {
        self.0.get(&id)
    }
}
