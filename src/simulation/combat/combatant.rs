//! The player's combat commands and state. Only the rule systems write it.
use super::runtime::ActionRuntime;
use super::skills::CHAIN_GRACE;
use super::{Arsenal, CombatEffect, CombatFeedback, Energy};
use crate::simulation::defense::decay;
use crate::simulation::{
    ActionAvailability, ActionKind, ActionSnapshot, ComboWindow, DefenseState, MeleeStage,
    SequenceSnapshot, Stamina, WeaponPair,
};
use bevy::prelude::*;

pub(super) const ATTACK: u8 = 1;
pub(super) const MELEE_SKILL: u8 = 2;
pub(super) const RANGED_SKILL: u8 = 4;
pub(super) const COMBO: u8 = 8;
pub(super) const TOGGLE: u8 = 16;
pub(super) const NEXT_PAIR: u8 = 32;
pub(super) const RAGE: u8 = 64;

#[derive(Component, Default)]
pub struct CombatIntent {
    pub(super) commands: u8,
    pub(super) pair: Option<WeaponPair>,
    pub(super) cancel_melee_chain: bool,
}
impl CombatIntent {
    pub fn request_attack(&mut self) {
        self.commands |= ATTACK;
    }
    pub fn request_melee_skill(&mut self) {
        self.commands |= MELEE_SKILL;
    }
    pub fn request_ranged_skill(&mut self) {
        self.commands |= RANGED_SKILL;
    }
    pub fn request_combo(&mut self) {
        self.commands |= COMBO;
    }
    pub fn request_toggle_target(&mut self) {
        self.commands ^= TOGGLE;
    }
    pub fn request_pair(&mut self, pair: WeaponPair) {
        self.pair = Some(pair);
    }
    /// Cycle to the next weapon pair.
    pub fn request_next_pair(&mut self) {
        self.commands |= NEXT_PAIR;
    }
    /// Release the rage skill, if the meter is full.
    pub fn request_rage(&mut self) {
        self.commands |= RAGE;
    }
    /// Drop only uncommitted ordinary-attack follow-ups at the next rule tick.
    /// This latch survives a paused frame with no FixedUpdate.
    pub fn cancel_melee_chain(&mut self) {
        self.cancel_melee_chain = true;
    }
    pub fn clear(&mut self) {
        *self = Self {
            cancel_melee_chain: true,
            ..Self::default()
        };
    }
}

/// The outcome of the player's latest combat command; presentation words it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CombatStatus {
    #[default]
    Ready,
    PairSwitched,
    /// The equipped pair wore out and was rotated out for the next one.
    WeaponExhausted,
    /// The rage skill was released.
    Enraged,
    Started(ActionKind),
    /// The pull combo's target is too far above or below.
    TargetOutOfReach,
    /// Rejected: another action is still running.
    Busy,
    /// Rejected: defending.
    Guarding,
    /// Rejected: no valid target or window, or still cooling down.
    Unavailable,
}

#[derive(Default)]
pub(super) struct MeleeChain {
    pub next_stage: MeleeStage,
    remaining: f32,
    pub buffered: bool,
    /// Only this committed action may open/extend the current chain. Clearing
    /// it on damage prevents its later retirement from reviving an old chain.
    eligible_action: Option<u64>,
}
impl MeleeChain {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn tick(&mut self, dt: f32) {
        if self.remaining > 0.0 {
            self.remaining = decay(self.remaining, dt);
            if self.remaining == 0.0 {
                self.clear();
            }
        }
    }
    pub fn begin(&mut self, action_id: u64, stage: MeleeStage) {
        self.clear();
        if let Some(next) = stage.next() {
            self.next_stage = next;
            self.eligible_action = Some(action_id);
        }
    }
    pub fn buffer(&mut self, action_id: u64) -> bool {
        if self.eligible_action == Some(action_id) {
            self.buffered = true;
            true
        } else {
            false
        }
    }
    pub fn finish(&mut self, action_id: u64) {
        if self.eligible_action == Some(action_id) {
            self.eligible_action = None;
            self.remaining = CHAIN_GRACE;
        }
    }
}
#[derive(Component)]
#[require(
    DefenseState,
    Stamina,
    Energy,
    Arsenal,
    crate::simulation::motion_feedback::MotionFeedbackState
)]
pub struct Combatant {
    pub(super) pair: WeaponPair,
    pub(super) target: Option<Entity>,
    /// Whether an empty target is filled with the nearest visible enemy.
    /// On by default; the lock toggle switches it with the lock.
    pub(super) auto_target: bool,
    pub(super) attack_cooldown: f32,
    pub(super) melee_cooldown: f32,
    pub(super) ranged_cooldown: f32,
    pub(super) combo: Option<ComboWindow>,
    pub(super) action: Option<ActionRuntime>,
    pub(super) melee_chain: MeleeChain,
    pub(super) companion: Option<Entity>,
    pub(super) axe_away: bool,
    /// The pair whose melee weapon is in flight as a projectile.
    pub(super) thrown: Option<WeaponPair>,
    pub(super) status: CombatStatus,
    pub(super) availability: ActionAvailability,
    pub(super) pull_velocity: Option<Vec2>,
}
impl Combatant {
    pub fn new(pair: WeaponPair) -> Self {
        Self {
            pair,
            target: None,
            auto_target: true,
            attack_cooldown: 0.0,
            melee_cooldown: 0.0,
            ranged_cooldown: 0.0,
            combo: None,
            action: None,
            melee_chain: MeleeChain::default(),
            companion: None,
            axe_away: false,
            thrown: None,
            status: CombatStatus::Ready,
            availability: default(),
            pull_velocity: None,
        }
    }
    pub fn weapon_pair(&self) -> WeaponPair {
        self.pair
    }
    pub fn target(&self) -> Option<Entity> {
        self.target
    }
    /// Whether targeting is automatic. Unlocking turns it off, locking again
    /// turns it back on.
    pub fn auto_target(&self) -> bool {
        self.auto_target
    }
    /// The same combatant with targeting left to the lock toggle alone.
    pub fn without_auto_target(mut self) -> Self {
        self.auto_target = false;
        self
    }
    pub fn melee_cooldown(&self) -> f32 {
        self.melee_cooldown
    }
    pub fn ranged_cooldown(&self) -> f32 {
        self.ranged_cooldown
    }
    pub fn attack_cooldown(&self) -> f32 {
        self.attack_cooldown
    }
    pub fn combo_remaining(&self) -> f32 {
        self.combo.map_or(0.0, |c| c.remaining)
    }
    pub fn combo_window(&self) -> Option<ComboWindow> {
        self.combo
    }
    pub fn active_action(&self) -> Option<ActionSnapshot> {
        self.action.as_ref().map(|a| a.snapshot)
    }
    /// The committed action's timeline, shared with enemies and the Boss.
    pub fn attack_clock(&self) -> Option<crate::simulation::AttackClock> {
        self.active_action().map(|action| action.clock())
    }
    pub fn sequence(&self) -> Option<SequenceSnapshot> {
        self.action.as_ref().and_then(ActionRuntime::sequence)
    }
    pub fn is_combo_active(&self) -> bool {
        self.sequence().is_some()
    }
    pub fn companion(&self) -> Option<Entity> {
        self.companion
    }
    /// The equipped melee weapon is out of the hand: thrown and not yet
    /// back, or holding the chain of a pull.
    pub fn axe_away(&self) -> bool {
        self.axe_away || self.thrown == Some(self.pair)
    }
    /// Whether the committed action pulls its actor along the chain.
    pub(super) fn pulls(&self) -> bool {
        self.action
            .as_ref()
            .is_some_and(|action| !action.cancelled && action.snapshot.kind.spec().pull().is_some())
    }
    pub fn status(&self) -> CombatStatus {
        self.status
    }
    pub fn availability(&self) -> ActionAvailability {
        self.availability
    }
    pub fn clear_targets(&mut self) {
        self.target = None;
        self.combo = None;
        self.action = None;
        self.melee_chain.clear();
        self.axe_away = false;
        self.thrown = None;
        self.pull_velocity = None;
        self.availability = ActionAvailability::default();
    }
    /// Gameplay interruption clears pending follow-ups, not the current hit.
    pub(in crate::simulation) fn interrupt_melee_chain(&mut self) {
        self.melee_chain.clear();
    }
    pub(in crate::simulation) fn movement_locked(&self) -> bool {
        self.is_combo_active()
    }
    pub(in crate::simulation) fn pull_velocity(&self) -> Option<Vec2> {
        self.pull_velocity
    }
    pub(in crate::simulation) fn busy(&self) -> bool {
        self.action.is_some()
    }
    pub(super) fn reject(
        &mut self,
        status: CombatStatus,
        entity: Entity,
        position: Vec3,
        feedback: &mut CombatFeedback,
    ) {
        self.status = status;
        feedback.push(entity, CombatEffect::Rejected, position, position, 0.0);
    }
}
