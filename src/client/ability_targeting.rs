//! Shared input intent and read-only cast eligibility for controls and drawing.
use super::abilities;
use crate::simulation::*;
use bevy::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum TargetState {
    Ready,
    ApproachingEnemy,
    Invalid(Rejection),
    NoTarget,
}

#[derive(Clone, Debug)]
pub(super) struct TargetIntent {
    pub ability: AbilityId,
    /// Preserve the click for submission. A preview's clamped point must never
    /// replace it: the simulation resolves again after windup, at commit.
    pub original: Option<AbilityTarget>,
    pub resolved: Option<AbilityTarget>,
    pub state: TargetState,
}

impl TargetIntent {
    pub fn submission(&self) -> Option<AbilityTarget> {
        matches!(
            self.state,
            TargetState::Ready | TargetState::ApproachingEnemy
        )
        .then_some(self.original)
        .flatten()
    }
}

pub(super) fn resolve_intent(
    world: &World,
    player: Entity,
    source: CastSource,
    ground: Option<Vec2>,
    candidates: &[Entity],
) -> Option<TargetIntent> {
    let info = abilities::resolve(
        source,
        world.get::<AbilitySlots>(player)?,
        world.get::<EquipmentSlots>(player)?,
        world.get_resource::<AbilityCatalog>()?,
        world.get_resource::<EquipmentCatalog>()?,
    )?;
    let recast = matches!(source, CastSource::AbilitySlot(slot)
        if world.get::<UnitMechanics>(player).is_some_and(|state| state.recast(slot)));
    let make = |target| {
        let (resolved, state) = match preview_cast(world, player, source, target) {
            Ok(target) => (Some(target), TargetState::Ready),
            Err(Rejection::OutOfRange) if matches!(target, AbilityTarget::Unit(_)) => {
                (Some(target), TargetState::ApproachingEnemy)
            }
            Err(reason) => (None, TargetState::Invalid(reason)),
        };
        TargetIntent {
            ability: info.ability,
            original: Some(target),
            resolved,
            state,
        }
    };
    if info.definition.targeting == Targeting::Enemy && !recast {
        let team = world.get::<Unit>(player)?.team();
        let mut rejected = None;
        for &target in candidates {
            if !world.get::<UnitState>(target).is_some_and(UnitState::alive)
                || !world
                    .get::<Unit>(target)
                    .is_some_and(|unit| unit.team() != team)
                || !team_can_see(world, team, target)
            {
                continue;
            }
            let intent = make(AbilityTarget::Unit(target));
            if intent.submission().is_some() {
                return Some(intent);
            }
            rejected.get_or_insert(intent);
        }
        return Some(rejected.unwrap_or(TargetIntent {
            ability: info.ability,
            original: None,
            resolved: None,
            state: TargetState::NoTarget,
        }));
    }
    let target = if recast {
        Some(AbilityTarget::SelfCast)
    } else {
        match info.definition.targeting {
            Targeting::Direction => ground.and_then(|point| {
                let delta = point - world.get::<UnitPose>(player)?.position();
                (delta.is_finite() && delta.length_squared() > 1e-6)
                    .then_some(AbilityTarget::Direction(delta))
            }),
            Targeting::Point => ground.map(AbilityTarget::Point),
            Targeting::SelfCast => Some(AbilityTarget::SelfCast),
            Targeting::Enemy => unreachable!("enemy intent handled above"),
        }
    };
    Some(target.map_or(
        TargetIntent {
            ability: info.ability,
            original: None,
            resolved: None,
            state: TargetState::NoTarget,
        },
        make,
    ))
}
