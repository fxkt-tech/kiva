//! Indicator dimensions are derived from the equipped execution, not its slot.
use crate::{client::ability_targeting::TargetIntent, simulation::*};
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub(super) struct Geometry {
    pub shape: &'static str,
    pub origin: Vec2,
    pub target: Option<Vec2>,
    pub direction: Vec2,
    pub range: f32,
    pub width: Option<f32>,
    pub radius: Option<f32>,
    pub rays: u8,
    pub angle: f32,
}

pub(super) fn derive(
    definition: &AbilityDefinition,
    intent: &TargetIntent,
    origin: Vec2,
    enemy_position: Option<Vec2>,
) -> Geometry {
    let target = intent.resolved.or(intent.original);
    let point = match target {
        Some(AbilityTarget::Point(point)) if point.is_finite() => Some(point),
        Some(AbilityTarget::Unit(_)) => enemy_position,
        Some(AbilityTarget::SelfCast) => Some(origin),
        _ => None,
    };
    let direction = match target {
        Some(AbilityTarget::Direction(value)) => value,
        _ => point.map_or(Vec2::NEG_Y, |point| point - origin),
    };
    let direction = direction
        .as_dvec2()
        .try_normalize()
        .map_or(Vec2::NEG_Y, |direction| direction.as_vec2());
    let mut result = Geometry {
        shape: "target",
        origin,
        target: point,
        direction,
        range: definition.range,
        width: None,
        radius: None,
        rays: 0,
        angle: 0.0,
    };
    match &definition.execution {
        AbilityExecution::Line(spec) => {
            result.shape = "line";
            result.width = Some(spec.width);
        }
        AbilityExecution::Volley(spec) => {
            result.shape = "volley";
            result.width = Some(spec.radius * 2.0);
            result.rays = spec.count;
            result.angle = spec.angle;
        }
        AbilityExecution::Arrow(spec) => {
            result.shape = "arrow";
            result.width = Some(spec.radius * 2.0);
        }
        AbilityExecution::Standard(spec) => {
            result.shape = "standard";
            if let Delivery::Area { radius, .. } = spec.impact.delivery {
                result.radius = Some(radius);
            }
        }
        AbilityExecution::Scout(spec) => {
            result.shape = "scout";
            result.radius = Some(spec.vision_radius);
        }
        AbilityExecution::Leap(spec) => {
            result.shape = "leap";
            result.radius = Some(spec.radius);
        }
        AbilityExecution::Shield(spec) => {
            result.shape = "shield";
            result.radius = Some(spec.radius);
        }
        AbilityExecution::Empower(_) => result.shape = "self",
        AbilityExecution::Dash { .. } => result.shape = "dash",
        AbilityExecution::Impact(impact) | AbilityExecution::Channel { pulse: impact, .. } => {
            match impact.delivery {
                Delivery::Area { radius, .. } => {
                    result.shape = "area";
                    result.radius = Some(radius);
                }
                Delivery::Projectile { radius, .. }
                    if definition.targeting == Targeting::Direction =>
                {
                    result.shape = "projectile";
                    result.width = Some(radius * 2.0);
                }
                _ => {}
            }
        }
    }
    if definition.targeting == Targeting::Direction && intent.original.is_some() {
        result.target = Some(origin + direction * definition.range);
    }
    result
}

impl Geometry {
    pub fn directions(&self) -> impl Iterator<Item = Vec2> + '_ {
        (0..self.rays).map(|index| {
            let angle = if self.rays <= 1 {
                0.0
            } else {
                -self.angle * 0.5 + self.angle * f32::from(index) / f32::from(self.rays - 1)
            };
            Vec2::from_angle(angle).rotate(self.direction)
        })
    }
}
