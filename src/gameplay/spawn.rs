//! Transactional mode construction delegates all combat state to its factories.
use super::{
    definition::{MINIONS_PER_TEAM, MatchDefinition},
    map::MapDefinition,
    match_state::{
        Controller, MatchHandle, MatchId, MatchRecord, MatchSequence, MatchState, PendingWave,
        UnitRole, WaveMember,
    },
};
use crate::simulation::*;
use bevy::prelude::*;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchError {
    NotPrepared,
    AlreadyStarted,
    UnknownCharacter(CharacterId),
    InvalidDefinition(&'static str),
    SpawnFailed(&'static str),
    CharacterSpawn(CharacterError),
    IdentityExhausted,
}
impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPrepared => f.write_str("match content has not been prepared"),
            Self::AlreadyStarted => f.write_str("reset the current match before starting another"),
            Self::UnknownCharacter(id) => write!(f, "unknown match character {}", id.0),
            Self::InvalidDefinition(reason) | Self::SpawnFailed(reason) => f.write_str(reason),
            Self::CharacterSpawn(error) => error.fmt(f),
            Self::IdentityExhausted => f.write_str("match identity space exhausted"),
        }
    }
}
impl std::error::Error for MatchError {}

struct PlannedUnit {
    spec: UnitSpec,
    character: Option<CharacterId>,
    role: UnitRole,
    controller: Option<Controller>,
    wave: Option<WaveMember>,
}

fn base_spec(definition: &MatchDefinition, map: &MapDefinition, team: Team) -> UnitSpec {
    UnitSpec {
        name: match team {
            Team::Ally => "我方基地",
            Team::Enemy => "敌方基地",
        }
        .into(),
        team,
        position: map.base_position(team),
        stats: definition.base_stats().clone(),
        abilities: [None; 4],
        respawn_delay: None,
    }
}

fn hero_spec(
    world: &World,
    definition: &MatchDefinition,
    map: &MapDefinition,
    team: Team,
    character: CharacterId,
) -> Result<UnitSpec, MatchError> {
    let content = world
        .resource::<CharacterCatalog>()
        .get(character)
        .ok_or(MatchError::UnknownCharacter(character))?;
    if content.stats.class != UnitClass::Actor {
        return Err(MatchError::InvalidDefinition("match heroes must be actors"));
    }
    Ok(UnitSpec {
        name: content.name.clone(),
        team,
        position: map.hero_spawn(team),
        stats: content.stats.clone(),
        abilities: content.default_loadout,
        respawn_delay: Some(definition.hero_respawn_delay()),
    })
}

fn wave_plan(definition: &MatchDefinition, map: &MapDefinition, wave: u64) -> Vec<PlannedUnit> {
    (0..MINIONS_PER_TEAM)
        .flat_map(|slot| pair_plan(definition, map, wave, slot))
        .collect()
}

fn pair_plan(
    definition: &MatchDefinition,
    map: &MapDefinition,
    wave: u64,
    slot: u8,
) -> [PlannedUnit; 2] {
    [Team::Ally, Team::Enemy].map(|team| {
        let melee = slot < 3;
        PlannedUnit {
            spec: UnitSpec {
                name: if melee { "近战兵" } else { "远程兵" }.into(),
                team,
                position: map.creep_spawns(team)[usize::from(slot)],
                stats: if melee {
                    definition.melee_stats()
                } else {
                    definition.ranged_stats()
                }
                .clone(),
                abilities: [None; 4],
                respawn_delay: None,
            },
            character: None,
            role: if melee {
                UnitRole::MeleeMinion
            } else {
                UnitRole::RangedMinion
            },
            controller: Some(Controller::MinionAi),
            wave: Some(WaveMember { wave, slot }),
        }
    })
}

fn pending_wave(
    definition: &MatchDefinition,
    wave: u64,
    start_tick: u64,
) -> Result<PendingWave, MatchError> {
    let interval = definition.minion_spawn_interval_ticks();
    // Validate the complete emission schedule before allocating any roots.
    interval
        .checked_mul(u64::from(MINIONS_PER_TEAM - 1))
        .and_then(|duration| start_tick.checked_add(duration))
        .ok_or(MatchError::InvalidDefinition("wave emission tick overflow"))?;
    Ok(PendingWave {
        wave,
        next_slot: 1,
        next_tick: start_tick
            .checked_add(interval)
            .ok_or(MatchError::InvalidDefinition("wave emission tick overflow"))?,
    })
}

fn validate_plan(
    world: &World,
    plan: &[PlannedUnit],
    map: &MapDefinition,
) -> Result<(), MatchError> {
    for item in plan {
        validate_unit_spec(world, &item.spec).map_err(MatchError::SpawnFailed)?;
        if item.spec.stats.class == UnitClass::Actor {
            world
                .resource::<NavigationMap>()
                .path(
                    item.spec.position,
                    map.base_position(super::opponent(item.spec.team)),
                    item.spec.stats.radius,
                )
                .map_err(|_| {
                    MatchError::InvalidDefinition("spawn cannot reach the opposing base")
                })?;
        }
    }
    Ok(())
}

fn create_plan(world: &mut World, plan: &[PlannedUnit]) -> Result<Vec<Entity>, MatchError> {
    let mut roots = Vec::with_capacity(plan.len());
    for item in plan {
        let result = if let Some(character) = item.character {
            spawn_character(
                world,
                CharacterSpawnSpec {
                    character,
                    team: item.spec.team,
                    position: item.spec.position,
                    loadout: LoadoutSelection::CharacterDefault,
                    respawn_delay: item.spec.respawn_delay,
                },
            )
            .map_err(MatchError::CharacterSpawn)
        } else {
            spawn_unit(world, item.spec.clone()).map_err(MatchError::SpawnFailed)
        };
        let entity = match result {
            Ok(entity) => entity,
            Err(error) => {
                for root in roots {
                    world.despawn(root);
                }
                return Err(error);
            }
        };
        world.entity_mut(entity).insert(item.role);
        if let Some(controller) = item.controller {
            world.entity_mut(entity).insert(controller);
        }
        if let Some(wave) = item.wave {
            world.entity_mut(entity).insert(wave);
        }
        roots.push(entity);
    }
    Ok(roots)
}

/// Preflights the entire opening roster, then creates bases, heroes and the
/// first minion pair atomically. Later pairs follow the simulation-tick schedule.
pub fn start_match(
    world: &mut World,
    player_character: CharacterId,
) -> Result<MatchHandle, MatchError> {
    if !world.contains_resource::<super::Prepared>() {
        return Err(MatchError::NotPrepared);
    }
    if world.resource::<MatchState>().current.is_some() {
        return Err(MatchError::AlreadyStarted);
    }
    if world.resource::<SimulationTick>().0 != 0 {
        return Err(MatchError::InvalidDefinition(
            "new matches must start at simulation tick zero",
        ));
    }
    let definition = world.resource::<MatchDefinition>().clone();
    let map = world.resource::<MapDefinition>().clone();
    if !definition.valid() {
        return Err(MatchError::InvalidDefinition("invalid match definition"));
    }
    let id = world
        .resource::<MatchSequence>()
        .0
        .checked_add(1)
        .ok_or(MatchError::IdentityExhausted)?;
    let mut plan = Vec::with_capacity(16);
    for team in [Team::Ally, Team::Enemy] {
        plan.push(PlannedUnit {
            spec: base_spec(&definition, &map, team),
            character: None,
            role: UnitRole::Base,
            controller: None,
            wave: None,
        });
    }
    for (team, character, controller) in [
        (Team::Ally, player_character, Controller::Human),
        (
            Team::Enemy,
            definition.opponent_character(),
            Controller::HeroAi,
        ),
    ] {
        plan.push(PlannedUnit {
            spec: hero_spec(world, &definition, &map, team, character)?,
            character: Some(character),
            role: UnitRole::Hero,
            controller: Some(controller),
            wave: None,
        });
    }
    let static_roots = plan.len();
    plan.extend(wave_plan(&definition, &map, 1));
    validate_plan(world, &plan, &map)?;
    let pending = pending_wave(&definition, 1, 0)?;
    let roots = create_plan(world, &plan[..static_roots + 2])?;
    let player = roots[2];
    let id = MatchId(id);
    world.resource_mut::<MatchSequence>().0 = id.0;
    world.resource_mut::<MatchState>().current = Some(MatchRecord {
        id,
        player_character,
        bases: [roots[0], roots[1]],
        heroes: [roots[2], roots[3]],
        wave_number: 1,
        next_wave_tick: definition.wave_interval_ticks(),
        pending_wave: Some(pending),
        finished: None,
        wave_error: None,
        definition,
    });
    // Capture/focus remains the client's authority. Starting a prepared match
    // does not clear a user pause; native/headless callers explicitly resume.
    Ok(MatchHandle { id, player })
}

pub(super) fn start_wave(
    world: &mut World,
    wave: u64,
    start_tick: u64,
) -> Result<PendingWave, MatchError> {
    let definition = world
        .resource::<MatchState>()
        .current
        .as_ref()
        .expect("wave belongs to an active match")
        .definition
        .clone();
    let map = world.resource::<MapDefinition>().clone();
    let plan = wave_plan(&definition, &map, wave);
    validate_plan(world, &plan, &map)?;
    let pending = pending_wave(&definition, wave, start_tick)?;
    create_plan(world, &plan[..2])?;
    Ok(pending)
}

pub(super) fn spawn_wave_pair(world: &mut World, wave: u64, slot: u8) -> Result<(), MatchError> {
    let definition = &world
        .resource::<MatchState>()
        .current
        .as_ref()
        .expect("pair belongs to an active match")
        .definition;
    let map = world.resource::<MapDefinition>().clone();
    let plan = pair_plan(definition, &map, wave, slot);
    // Navigation/content may have changed since whole-wave preflight. Recheck
    // the current pair and roll back only its roots on a factory failure.
    validate_plan(world, &plan, &map)?;
    create_plan(world, &plan)?;
    Ok(())
}

/// Ends the current mode instance while preserving prepared content and all
/// identity allocators. New AI components live under the deleted unit roots.
pub fn reset_match(world: &mut World) {
    reset_simulation(world);
    world.resource_mut::<MatchState>().current = None;
    world
        .resource_mut::<Messages<super::MatchFinished>>()
        .clear();
}
