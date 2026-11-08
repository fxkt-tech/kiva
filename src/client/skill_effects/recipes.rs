//! Ability identity chooses art; all dimensions come from authoritative facts.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Shape {
    Thrust { length: f32, width: f32 },
    Nova { radius: f32 },
    StandardDrop,
    GroundImpact { radius: f32 },
    CrystalImpact { radius: f32 },
    Spark { gold: bool, size: f32 },
    WallBreak { radius: f32 },
}
pub(super) struct Recipe {
    pub shape: Shape,
    pub seconds: f64,
    pub position: Vec2,
    pub height: f32,
    pub direction: Vec2,
}

pub(super) fn known(id: AbilityId) -> bool {
    matches!(id.0, 11..=14 | 21..=24)
}

pub(super) fn recipe(world: &World, fact: &Fact) -> Option<Recipe> {
    let definition = world.get_resource::<AbilityCatalog>()?.get(fact.ability)?;
    let mut result = Recipe {
        shape: Shape::Spark {
            gold: false,
            size: 0.6,
        },
        seconds: 0.22,
        position: fact.position,
        height: 0.9,
        direction: fact.direction,
    };
    match (&fact.kind, fact.ability.0, &definition.execution) {
        (FactKind::Cast { recast: true, .. }, _, _) => return None,
        (FactKind::Cast { .. }, 11, AbilityExecution::Line(spec)) => {
            result.shape = Shape::Thrust {
                length: definition.range,
                width: spec.width,
            };
            result.height = 0.8;
            result.seconds = 0.32;
        }
        (FactKind::Cast { .. }, 12, AbilityExecution::Shield(spec)) => {
            result.shape = Shape::Nova {
                radius: spec.radius,
            };
            result.height = 0.07;
            result.seconds = 0.48;
        }
        (
            FactKind::Cast {
                target: AbilityTarget::Point(point),
                ..
            },
            13,
            AbilityExecution::Standard(spec),
        ) => {
            result.shape = Shape::StandardDrop;
            result.position = *point;
            result.height = 0.0;
            result.seconds = match spec.impact.delivery {
                Delivery::Area { delay, .. } => f64::from(delay.max(TICK_SECONDS)),
                _ => 0.25,
            };
        }
        (FactKind::Cast { .. }, 14, _) => {
            result.shape = Shape::Spark {
                gold: true,
                size: 1.0,
            };
        }
        (FactKind::Cast { .. }, 21 | 22 | 24, _) => {
            result.shape = Shape::Spark {
                gold: false,
                size: 0.85,
            };
            result.seconds = 0.18;
        }
        (FactKind::Cast { .. }, 23, _) | (FactKind::ScoutArrived, _, _) => return None,
        (FactKind::Impact(ImpactKind::Contact { .. }), 11..=14, _) => {
            // Landing owns the large ground effect; contacts are small and
            // remain correct even when shields absorb the entire payload.
            result.shape = Shape::Spark {
                gold: true,
                size: 0.7,
            };
        }
        (FactKind::Impact(ImpactKind::Contact { .. }), 21 | 22, _) => {
            result.shape = Shape::Spark {
                gold: false,
                size: 0.7,
            };
        }
        (FactKind::Impact(ImpactKind::AreaActivated { radius, .. }), 14, _) => {
            result.shape = Shape::GroundImpact { radius: *radius };
            result.height = 0.04;
            result.seconds = 0.85;
        }
        (FactKind::Impact(ImpactKind::AreaActivated { .. }), 13, _) => {
            result.shape = Shape::Nova { radius: 0.7 };
            result.height = 0.07;
            result.seconds = 0.35;
        }
        (
            FactKind::Impact(ImpactKind::ProjectileEnded { reason, .. }),
            24,
            AbilityExecution::Arrow(spec),
        ) => {
            if matches!(reason, ProjectileEndReason::Hit(_)) {
                result.shape = Shape::CrystalImpact {
                    radius: spec.impact_radius,
                };
                result.height = 0.08;
                result.seconds = 0.65;
            } else {
                // A spent/blocked global arrow dissipates. It does not create
                // a fake damage circle at the cursor or the map boundary.
                result.shape = Shape::Spark {
                    gold: false,
                    size: 0.7,
                };
            }
        }
        (FactKind::TerrainRemoved { radius }, 14, _) => {
            result.shape = Shape::WallBreak { radius: *radius };
            result.height = 0.0;
            result.seconds = 0.35;
        }
        _ => return None,
    }
    Some(result)
}

pub(super) fn projectile_ability(world: &World, projectile: &Projectile) -> Option<AbilityId> {
    ability(projectile.origin())
        .or_else(|| projectile.empower_origin().and_then(ability))
        .filter(|id| matches!(id.0, 21 | 22 | 24))
        .or_else(|| {
            (projectile.origin() == DamageOrigin::BasicAttack
                && world
                    .get::<CharacterIdentity>(projectile.source_life().entity())
                    .is_some_and(|identity| identity.id() == CharacterId(2)))
            .then_some(AbilityId(0))
        })
}
