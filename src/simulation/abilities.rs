//! Read-only action and companion snapshots; combat owns the runtime, the
//! skill numbers and marker execution.
use super::WeaponPair;
use bevy::prelude::*;

/// A committed ordinary attack's stage, independent of skill combo windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeleeStage {
    #[default]
    One,
    Two,
    Three,
}
impl MeleeStage {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::One => Some(Self::Two),
            Self::Two => Some(Self::Three),
            Self::Three => None,
        }
    }
}
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
}
/// Which attack timeline an [`AttackClock`] runs on. A key into the
/// presentation clip table and the baked hit data; it carries no display facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackId {
    PlayerMelee { pair: WeaponPair, stage: MeleeStage },
    PlayerSkill(ActionKind),
    Boss { stage: MeleeStage },
    Enemy,
}

/// One attack's authoritative clock, the same shape for the player, ordinary
/// enemies and the Boss. Presentation samples its attack pose from `elapsed`
/// instead of keeping a render-time clock of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackClock {
    /// Identity of this attack: an action id, an enemy attack id or a Boss sequence.
    pub action_id: u64,
    pub track: TrackId,
    pub elapsed: f32,
    pub duration: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionSnapshot {
    pub action_id: u64,
    pub kind: ActionKind,
    pub pair: WeaponPair,
    pub melee_stage: Option<MeleeStage>,
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
impl ActionSnapshot {
    pub fn clock(&self) -> AttackClock {
        AttackClock {
            action_id: self.action_id,
            track: match (self.kind, self.melee_stage) {
                (ActionKind::Melee, Some(stage)) => TrackId::PlayerMelee {
                    pair: self.pair,
                    stage,
                },
                (kind, _) => TrackId::PlayerSkill(kind),
            },
            elapsed: self.elapsed,
            duration: self.duration,
        }
    }
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
