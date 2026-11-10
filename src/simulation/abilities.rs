//! Read-only action and companion snapshots; combat owns marker execution.
use super::WeaponPair;
use bevy::prelude::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    Melee,
    HammerSlam,
    MusketShot,
    ChainSweep,
    AxeShot,
    HammerCombo,
    ChainCombo,
}
impl ActionKind {
    pub fn is_combo(self) -> bool {
        matches!(self, Self::HammerCombo | Self::ChainCombo)
    }
    pub(super) fn duration(self, pair: WeaponPair) -> f32 {
        match self {
            Self::Melee => pair.melee().3,
            Self::HammerSlam | Self::ChainSweep => 0.50,
            Self::MusketShot => 0.40,
            Self::AxeShot => 0.75,
            Self::HammerCombo => 0.96,
            Self::ChainCombo => 0.93,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ActionSnapshot {
    pub action_id: u64,
    pub kind: ActionKind,
    pub pair: WeaponPair,
    pub target: Option<Entity>,
    pub elapsed: f32,
    pub duration: f32,
    pub shot_count: u8,
    pub aim_position: Vec3,
    pub source_position: Vec3,
    pub companion_position: Vec3,
}
#[derive(Clone, Copy, Debug)]
pub struct SequenceSnapshot {
    pub action_id: u64,
    pub kind: ActionKind,
    pub pair: WeaponPair,
    pub target: Entity,
    pub elapsed: f32,
    pub duration: f32,
    pub shot_count: u8,
    pub aim_position: Vec3,
    pub source_position: Vec3,
    pub companion_position: Vec3,
    pub cancelled: bool,
}
impl SequenceSnapshot {
    pub const AIM_DURATION: f32 = 0.30;
    pub const HOLD_DURATION: f32 = 0.38;
    pub const RETURN_DURATION: f32 = 0.28;
    pub const TOTAL_DURATION: f32 = 0.96;
    pub fn target(self) -> Entity {
        self.target
    }
    pub fn elapsed(self) -> f32 {
        self.elapsed
    }
    pub fn aim_position(self) -> Vec3 {
        self.aim_position
    }
    pub fn has_fired(self) -> bool {
        self.shot_count > 0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ComboWindow {
    pub source_action_id: u64,
    pub pair: WeaponPair,
    pub original_target: Entity,
    pub remaining: f32,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionAvailability {
    pub attack: bool,
    pub melee_skill: bool,
    pub ranged_skill: bool,
    pub combo: bool,
    pub defense: bool,
    pub equip: bool,
    pub movement_actions: bool,
}
#[derive(Component, Clone, Copy, Debug)]
pub struct Companion {
    pub owner: Entity,
}
#[derive(Component, Clone, Copy, Debug)]
pub struct CompanionState {
    pub visible: bool,
    pub position: Vec3,
    pub previous_position: Vec3,
    pub facing: Vec2,
    pub pair: WeaponPair,
    pub action_id: u64,
    pub kind: ActionKind,
    pub elapsed: f32,
    pub shot_count: u8,
    pub source_position: Vec3,
    pub aim_position: Vec3,
}
impl Default for CompanionState {
    fn default() -> Self {
        Self {
            visible: false,
            position: Vec3::ZERO,
            previous_position: Vec3::ZERO,
            facing: Vec2::NEG_Y,
            pair: WeaponPair::default(),
            action_id: 0,
            kind: ActionKind::MusketShot,
            elapsed: 0.0,
            shot_count: 0,
            source_position: Vec3::ZERO,
            aim_position: Vec3::ZERO,
        }
    }
}
pub(super) struct ActionRuntime {
    pub snapshot: ActionSnapshot,
    pub previous: f32,
    pub markers: u8,
    pub cancelled: bool,
    pub return_remaining: f32,
    pub pull_stalled: f32,
    pub pull_stopped: bool,
}
impl ActionRuntime {
    pub fn sequence(&self) -> Option<SequenceSnapshot> {
        let s = self.snapshot;
        s.kind.is_combo().then(|| SequenceSnapshot {
            action_id: s.action_id,
            kind: s.kind,
            pair: s.pair,
            target: s.target.expect("combo has target"),
            elapsed: s.elapsed,
            duration: s.duration,
            shot_count: s.shot_count,
            aim_position: s.aim_position,
            source_position: s.source_position,
            companion_position: s.companion_position,
            cancelled: self.cancelled,
        })
    }
    pub fn cancel(&mut self) {
        if !self.cancelled {
            self.cancelled = true;
            self.return_remaining = 0.28;
            let start = if self.snapshot.kind == ActionKind::HammerCombo {
                0.68
            } else {
                0.65
            };
            self.snapshot.elapsed = self.snapshot.elapsed.max(start);
        }
    }
    pub fn marker(&mut self, index: u8, time: f32) -> bool {
        let bit = 1 << index;
        if self.markers & bit == 0 && self.previous < time && self.snapshot.elapsed + 1e-6 >= time {
            self.markers |= bit;
            true
        } else {
            false
        }
    }
}
