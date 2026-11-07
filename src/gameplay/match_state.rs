//! One match identity and terminal result; combat runtime remains in simulation.
use super::definition::MatchDefinition;
use super::map::team_index;
use crate::simulation::{CharacterId, Team};
use bevy::prelude::*;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatchId(pub(super) u64);
impl fmt::Display for MatchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchOutcome {
    Winner(Team),
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchPhase {
    Waiting,
    Running,
    Finished(MatchOutcome),
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitRole {
    Hero,
    MeleeMinion,
    RangedMinion,
    Base,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    Human,
    HeroAi,
    MinionAi,
}

/// Identity of a formation slot, not another unit position or movement state.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveMember {
    pub(super) wave: u64,
    pub(super) slot: u8,
}
impl WaveMember {
    pub fn wave(&self) -> u64 {
        self.wave
    }
    pub fn slot(&self) -> u8 {
        self.slot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchHandle {
    pub id: MatchId,
    pub player: Entity,
}

/// Only mode emission timing lives here; actual minion state stays on units.
#[derive(Clone, Copy)]
pub(super) struct PendingWave {
    pub wave: u64,
    pub next_slot: u8,
    pub next_tick: u64,
}

pub(super) struct MatchRecord {
    pub id: MatchId,
    pub player_character: CharacterId,
    pub heroes: [Entity; 2],
    pub bases: [Entity; 2],
    pub wave_number: u64,
    pub next_wave_tick: u64,
    pub pending_wave: Option<PendingWave>,
    pub finished: Option<(MatchOutcome, u64)>,
    pub wave_error: Option<String>,
    pub definition: MatchDefinition,
}

#[derive(Resource, Default)]
pub struct MatchState {
    pub(super) current: Option<MatchRecord>,
}
impl MatchState {
    pub fn id(&self) -> Option<MatchId> {
        self.current.as_ref().map(|m| m.id)
    }
    pub fn phase(&self) -> MatchPhase {
        self.current.as_ref().map_or(MatchPhase::Waiting, |m| {
            m.finished.map_or(MatchPhase::Running, |(result, _)| {
                MatchPhase::Finished(result)
            })
        })
    }
    pub fn is_running(&self) -> bool {
        self.phase() == MatchPhase::Running
    }
    /// A construction failure pauses the mode instead of silently skipping a
    /// configured wave. It can only be cleared by starting a new match.
    pub fn can_advance(&self) -> bool {
        self.is_running() && self.wave_error().is_none()
    }
    pub fn player_character(&self) -> Option<CharacterId> {
        self.current.as_ref().map(|m| m.player_character)
    }
    pub fn hero(&self, team: Team) -> Option<Entity> {
        self.current.as_ref().map(|m| m.heroes[team_index(team)])
    }
    pub fn base(&self, team: Team) -> Option<Entity> {
        self.current.as_ref().map(|m| m.bases[team_index(team)])
    }
    pub fn wave_number(&self) -> u64 {
        self.current.as_ref().map_or(0, |m| m.wave_number)
    }
    pub fn next_wave_tick(&self) -> Option<u64> {
        self.current
            .as_ref()
            .filter(|m| m.finished.is_none())
            .map(|m| m.next_wave_tick)
    }
    pub fn finished_tick(&self) -> Option<u64> {
        self.current
            .as_ref()
            .and_then(|m| m.finished.map(|(_, tick)| tick))
    }
    pub fn wave_error(&self) -> Option<&str> {
        self.current.as_ref().and_then(|m| m.wave_error.as_deref())
    }
}

#[derive(Resource, Default)]
pub(super) struct MatchSequence(pub u64);

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchFinished {
    pub id: MatchId,
    pub outcome: MatchOutcome,
    pub tick: u64,
}
