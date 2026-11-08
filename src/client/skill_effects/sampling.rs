//! Sample only authoritative live carriers and current attached unit state.
//! Source life identifies an attachment, never an independent projectile's lifetime.
use super::*;
use crate::client::{scene::UnitVisual, visibility::VisibleWorld};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum LiveKey {
    Carrier(Entity),
    Shield(Entity),
    Focus(Entity),
}
impl LiveKey {
    pub fn owner(self) -> Entity {
        match self {
            Self::Carrier(id) | Self::Shield(id) | Self::Focus(id) => id,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LiveKind {
    Projectile,
    Area,
    Standard,
    Terrain,
    Scout,
    Reveal(bool),
    Shield,
    Focus(bool),
}
#[derive(Clone, Copy)]
pub(super) struct Request {
    pub key: LiveKey,
    pub kind: LiveKind,
    pub ability: u16,
    pub source: UnitLifeRef,
    pub position: Vec2,
    pub radius: f32,
    pub arrows: u8,
    pub progress: f32,
}
pub(super) fn requests(
    world: &mut World,
    arrivals: &HashMap<Entity, bool>,
    projectile_art: &HashMap<Entity, AbilityId>,
) -> Vec<Request> {
    let visible = |world: &World, id| {
        world
            .get_resource::<VisibleWorld>()
            .is_none_or(|v| v.contains(id))
    };
    let mut result = Vec::new();
    for (id, projectile) in world.query::<(Entity, &Projectile)>().iter(world) {
        if !visible(world, id) {
            continue;
        }
        if let Some(ability) = recipes::projectile_ability(world, projectile)
            .or_else(|| projectile_art.get(&id).copied())
        {
            result.push(Request {
                key: LiveKey::Carrier(id),
                kind: LiveKind::Projectile,
                ability: ability.0,
                source: projectile.source_life(),
                position: projectile.position(),
                radius: projectile.radius(),
                arrows: projectile.arrows(),
                progress: 0.0,
            });
        }
    }
    for (id, area) in world.query::<(Entity, &AreaEffect)>().iter(world) {
        if let Some(ability) = ability(area.origin()).filter(|id| recipes::known(*id))
            && visible(world, id)
        {
            result.push(Request {
                key: LiveKey::Carrier(id),
                kind: LiveKind::Area,
                ability: ability.0,
                source: area.source_life(),
                position: area.position(),
                radius: area.radius(),
                arrows: 1,
                progress: area.progress(),
            });
        }
    }
    for (id, object) in world.query::<(Entity, &BattlefieldObject)>().iter(world) {
        let Some(ability) = ability(object.origin()).filter(|id| recipes::known(*id)) else {
            continue;
        };
        if !visible(world, id) {
            continue;
        }
        let kind = match (ability.0, object.kind()) {
            (13, ObjectKind::Standard) => LiveKind::Standard,
            (14, ObjectKind::Terrain) => LiveKind::Terrain,
            (23, ObjectKind::Scout) => LiveKind::Scout,
            (23, ObjectKind::Reveal) => LiveKind::Reveal(arrivals.get(&id) == Some(&true)),
            // Arrow impact/reveal is already represented by its resolved end.
            (24, ObjectKind::Reveal) => LiveKind::Reveal(false),
            _ => continue,
        };
        result.push(Request {
            key: LiveKey::Carrier(id),
            kind,
            ability: ability.0,
            source: object.owner(),
            position: object.position(),
            radius: if matches!(kind, LiveKind::Reveal(_)) {
                object.vision_radius()
            } else {
                object.radius()
            },
            arrows: 1,
            progress: 0.0,
        });
    }
    for (id, state, pose, mechanics, slots) in world
        .query_filtered::<(
            Entity,
            &UnitState,
            &UnitPose,
            &UnitMechanics,
            &AbilitySlots,
        ), With<UnitVisual>>()
        .iter(world)
    {
        if !state.alive() || !visible(world, id) {
            continue;
        }
        if mechanics
            .shields()
            .any(|shield| shield.amount > 0.0 && ability(shield.origin) == Some(AbilityId(12)))
        {
            result.push(Request {
                key: LiveKey::Shield(id),
                kind: LiveKind::Shield,
                ability: 12,
                source: state.life_ref(id),
                position: pose.position(),
                radius: 1.1,
                arrows: 1,
                progress: 0.0,
            });
        }
        let empowered = mechanics.empower_origin().and_then(ability) == Some(AbilityId(21));
        let ready = (0..4).any(|slot| slots.ability(slot) == Some(AbilityId(21)))
            && world
                .get_resource::<AbilityCatalog>()
                .and_then(|catalog| catalog.get(AbilityId(21)))
                .is_some_and(|definition| {
                    matches!(definition.execution, AbilityExecution::Empower(spec)
                    if mechanics.focus() >= spec.required_stacks)
                });
        if empowered || ready {
            result.push(Request {
                key: LiveKey::Focus(id),
                kind: LiveKind::Focus(empowered),
                ability: 21,
                source: state.life_ref(id),
                position: pose.position(),
                radius: 0.8,
                arrows: 1,
                progress: 0.0,
            });
        }
    }
    result
}
