//! Bounded, monotonically numbered combat facts for presentation to consume.
use crate::simulation::{ActionSnapshot, MeleeStage, WeaponPair};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatEffect {
    MeleeStart,
    Melee,
    Shot,
    SniperShot,
    SniperImpact,
    Hammer,
    ChainSweep,
    AxeThrow,
    Hit,
    Deflect,
    Block,
    Parry,
    Dodge,
    Defeat,
    Respawn,
    /// The player entered the enraged state.
    Enrage,
    EnemyWindup,
    Rejected,
}
#[derive(Clone, Copy, Debug)]
pub struct CombatEvent {
    pub sequence: u64,
    pub actor: Entity,
    pub source_owner: Entity,
    pub pair: Option<WeaponPair>,
    pub melee_stage: Option<MeleeStage>,
    pub action_id: u64,
    pub hit_index: u8,
    pub kind: CombatEffect,
    pub position: Vec3,
    pub end: Vec3,
    pub radius: f32,
}
#[derive(Resource, Default)]
pub struct CombatFeedback {
    next_sequence: u64,
    events: VecDeque<CombatEvent>,
}
impl CombatFeedback {
    pub fn events(&self) -> impl Iterator<Item = &CombatEvent> {
        self.events.iter()
    }
    pub(in crate::simulation) fn push(
        &mut self,
        actor: Entity,
        kind: CombatEffect,
        position: Vec3,
        end: Vec3,
        radius: f32,
    ) {
        self.emit(CombatEvent {
            sequence: 0,
            actor,
            source_owner: actor,
            pair: None,
            melee_stage: None,
            action_id: 0,
            hit_index: 0,
            kind,
            position,
            end,
            radius,
        });
    }
    pub(super) fn emit(&mut self, mut event: CombatEvent) {
        self.next_sequence += 1;
        event.sequence = self.next_sequence;
        self.events.push_back(event);
        if self.events.len() > 128 {
            self.events.pop_front();
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn action(
        &mut self,
        actor: Entity,
        owner: Entity,
        action: &ActionSnapshot,
        index: u8,
        kind: CombatEffect,
        position: Vec3,
        end: Vec3,
        radius: f32,
    ) {
        self.emit(CombatEvent {
            sequence: 0,
            actor,
            source_owner: owner,
            pair: Some(action.pair),
            melee_stage: action.melee_stage,
            action_id: action.action_id,
            hit_index: index,
            kind,
            position,
            end,
            radius,
        });
    }
}
#[derive(Resource, Default)]
pub(in crate::simulation) struct ActionIds(u64);
impl ActionIds {
    pub(in crate::simulation) fn next(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}
