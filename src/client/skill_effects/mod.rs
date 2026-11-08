//! Cosmetic adapters for committed facts and authoritative live carriers.
mod capture;
mod drawing;
mod layers;
mod recipes;
mod render;
mod sampling;

use super::{session::ClientSession, skill_assets::SkillAssets};
use crate::{gameplay::MatchId, simulation::*};
use bevy::{ecs::message::MessageCursor, prelude::*};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, VecDeque};

pub(super) struct SkillEffectsPlugin;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SkillEffectsSet {
    Render,
}
impl Plugin for SkillEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Runtime>()
            .init_resource::<SkillEffectsView>()
            .add_systems(Startup, drawing::setup)
            .add_systems(
                FixedUpdate,
                capture::capture
                    .after(SimulationSet::Step)
                    .before(crate::gameplay::GameplaySet::Outcome),
            )
            .add_systems(
                Update,
                render::update
                    .in_set(SkillEffectsSet::Render)
                    .after(super::scene::sync_projectiles)
                    .after(super::battlefield::sync)
                    .after(super::skill_assets::SkillAssetsSet::Prepare),
            );
    }
}

/// Owned geometry only. It never enters the model picking subtree.
#[derive(Component)]
pub(super) struct ManagedEffect;

/// The fallback shield is hidden only after this actual visual group exists.
#[derive(Component)]
pub(super) struct OriginalShield(pub Entity);

#[derive(Clone, Default, Resource, Serialize)]
pub(super) struct SkillEffectsView {
    pub ready: bool,
    pub errors: Vec<String>,
    pub active: Vec<ActiveEffectView>,
    pub emitted: BTreeMap<u16, u64>,
    pub pending: usize,
}
#[derive(Clone, Serialize)]
pub(super) struct ActiveEffectView {
    pub ability_id: u16,
    pub phase: &'static str,
    pub layers: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SessionKey(MatchId, Entity);
fn session(world: &World) -> Option<SessionKey> {
    match world.get_resource::<ClientSession>()? {
        ClientSession::Active {
            player, match_id, ..
        } if world.get::<Unit>(*player).is_some() => Some(SessionKey(*match_id, *player)),
        _ => None,
    }
}
fn ability(origin: DamageOrigin) -> Option<AbilityId> {
    match origin {
        DamageOrigin::Ability(id) | DamageOrigin::EquipmentActive { ability: id, .. } => Some(id),
        _ => None,
    }
}

#[derive(Clone)]
enum FactKind {
    Cast { target: AbilityTarget, recast: bool },
    Impact(ImpactKind),
    ScoutArrived,
    TerrainRemoved { radius: f32 },
}
#[derive(Clone)]
struct Fact {
    tick: u64,
    ability: AbilityId,
    source: UnitLifeRef,
    team: Team,
    position: Vec2,
    direction: Vec2,
    kind: FactKind,
}

/// Last fixed-step observation, used only to timestamp real object removal.
#[derive(Clone, Copy)]
struct ObjectObservation {
    kind: ObjectKind,
    ability: Option<AbilityId>,
    source: UnitLifeRef,
    team: Team,
    position: Vec2,
    radius: f32,
}
impl From<&BattlefieldObject> for ObjectObservation {
    fn from(object: &BattlefieldObject) -> Self {
        Self {
            kind: object.kind(),
            ability: ability(object.origin()).filter(|id| recipes::known(*id)),
            source: object.owner(),
            team: object.team(),
            position: object.position(),
            radius: object.radius(),
        }
    }
}

#[derive(Resource, Default)]
struct Runtime {
    reader: MessageCursor<SimulationEvent>,
    session: Option<SessionKey>,
    needs_clear: bool,
    pending: VecDeque<Fact>,
    previous_objects: HashMap<Entity, ObjectObservation>,
    arrivals: HashMap<Entity, bool>,
    live: HashMap<render::LiveKey, render::LiveVisual>,
    projectile_art: HashMap<Entity, AbilityId>,
    bursts: VecDeque<render::Burst>,
    clock: f64,
    emitted: BTreeMap<u16, u64>,
}

fn observable(world: &World, team: Team, point: Vec2, contact: bool) -> bool {
    let Some(SessionKey(_, player)) = session(world) else {
        return false;
    };
    let Some(player) = world.get::<Unit>(player) else {
        return false;
    };
    // Contact can disclose an otherwise hidden victim even for our own AoE.
    (!contact && player.team() == team) || point_visible(world, player.team(), point)
}

fn asset_ready(world: &World) -> bool {
    world
        .get_resource::<SkillAssets>()
        .is_some_and(|assets| assets.status().ready)
}

#[cfg(test)]
mod tests;
