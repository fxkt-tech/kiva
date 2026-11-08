//! Read-only facts captured at the existing authoritative commit/impact points.
use super::*;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImpactKind {
    Contact {
        target: UnitLifeRef,
    },
    AreaActivated {
        carrier_entity: Option<Entity>,
        radius: f32,
    },
    ProjectileEnded {
        carrier_entity: Entity,
        reason: ProjectileEndReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileEndReason {
    Hit(UnitLifeRef),
    Blocked,
    Expired,
    InvalidTarget,
}

pub(super) fn ability_committed(
    world: &mut World,
    unit: Entity,
    ability: AbilityId,
    cast_source: CastSource,
    target: AbilityTarget,
    target_life: Option<UnitLifeRef>,
    recast: bool,
) {
    let origin = world
        .get::<UnitPose>(unit)
        .expect("committing pose")
        .position;
    let source_life = world
        .get::<UnitState>(unit)
        .expect("committing life")
        .life_ref(unit);
    let position = match target {
        AbilityTarget::Point(point) => point,
        AbilityTarget::Unit(target) => {
            world
                .get::<UnitPose>(target)
                .expect("committed target pose")
                .position
        }
        _ => origin,
    };
    let target = match target {
        AbilityTarget::Direction(value) => {
            AbilityTarget::Direction(effects::direction(value, Vec2::X))
        }
        other => other,
    };
    world.write_message(SimulationEvent::AbilityCommitted {
        unit,
        ability,
        position,
        source_life,
        cast_source,
        origin,
        target,
        target_life,
        recast,
    });
}

pub(super) fn impact(
    payload: &ImpactSnapshot,
    carrier: u64,
    position: Vec2,
    direction: Vec2,
    kind: ImpactKind,
) -> SimulationEvent {
    SimulationEvent::ImpactResolved {
        source_life: payload.source,
        team: payload.team,
        origin: payload.origin,
        empower_origin: payload.empower_origin,
        carrier,
        position,
        direction,
        kind,
    }
}
