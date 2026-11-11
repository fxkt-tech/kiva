//! Ordered right-button gestures, independent from motion and device clocks.
use super::{MotionState, Stamina};
use bevy::prelude::*;
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefenseEdge {
    Pressed,
    Released,
    Cancelled,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DefenseKind {
    #[default]
    Idle,
    Deflect,
    Waiting,
    Guard,
    Parry,
}

/// A display can miss an entire short phase between frames. The last gesture
/// and its age remain available after a natural return to Idle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DefenseSnapshot {
    pub kind: DefenseKind,
    pub elapsed: f32,
    pub remaining: f32,
    pub gesture_id: u64,
    pub last_gesture_kind: DefenseKind,
    pub gesture_elapsed: f32,
}
#[derive(Component, Default, Debug)]
pub struct DefenseState {
    kind: DefenseKind,
    elapsed: f32,
    gesture_id: u64,
    last_gesture_kind: DefenseKind,
    gesture_elapsed: f32,
    held: bool,
    held_elapsed: f32,
    remaining: f32,
    deflect_cooldown: f32,
    parry_cooldown: f32,
    edges: VecDeque<DefenseEdge>,
    cancel: bool,
}
impl DefenseState {
    pub fn push_edge(&mut self, edge: DefenseEdge) {
        if self.edges.len() >= 64 {
            self.edges.clear();
            self.cancel = true;
            return;
        }
        self.edges.push_back(edge);
    }
    pub fn clear(&mut self) {
        self.edges.clear();
        self.cancel = true;
    }
    pub fn kind(&self) -> DefenseKind {
        self.kind
    }
    pub fn snapshot(&self) -> DefenseSnapshot {
        DefenseSnapshot {
            kind: self.kind,
            elapsed: self.elapsed,
            remaining: self.remaining,
            gesture_id: self.gesture_id,
            last_gesture_kind: self.last_gesture_kind,
            gesture_elapsed: self.gesture_elapsed,
        }
    }
    pub fn is_active(&self) -> bool {
        self.kind != DefenseKind::Idle
    }
    pub fn is_deflecting(&self) -> bool {
        self.kind == DefenseKind::Deflect
    }
    pub fn is_guarding(&self) -> bool {
        self.kind == DefenseKind::Guard
    }
    pub fn is_parrying(&self) -> bool {
        self.kind == DefenseKind::Parry
    }
    pub fn remaining(&self) -> f32 {
        self.remaining
    }
    pub fn deflect_cooldown(&self) -> f32 {
        self.deflect_cooldown
    }
    pub fn parry_cooldown(&self) -> f32 {
        self.parry_cooldown
    }
    pub(super) fn cancel(&mut self) {
        self.transition(DefenseKind::Idle);
        self.held = false;
        self.held_elapsed = 0.0;
        self.remaining = 0.0;
        self.last_gesture_kind = DefenseKind::Idle;
        self.gesture_elapsed = 0.0;
    }
    pub(super) fn reset(&mut self) {
        let gesture_id = self.gesture_id;
        *self = Self {
            gesture_id,
            ..Self::default()
        };
    }
    fn transition(&mut self, kind: DefenseKind) {
        if self.kind != kind {
            self.kind = kind;
            self.elapsed = 0.0;
        }
    }
    fn begin_gesture(&mut self) {
        self.gesture_id += 1;
        self.last_gesture_kind = self.kind;
        self.gesture_elapsed = 0.0;
    }
    pub(super) fn advance(
        &mut self,
        dt: f32,
        admit: bool,
        exit: bool,
        motion: &MotionState,
        stamina: &Stamina,
    ) {
        self.elapsed += dt;
        if self.gesture_id != 0 {
            self.gesture_elapsed += dt;
        }
        self.deflect_cooldown = decay(self.deflect_cooldown, dt);
        self.parry_cooldown = decay(self.parry_cooldown, dt);
        if std::mem::take(&mut self.cancel)
            || exit
            || !motion.is_grounded()
            || motion.is_dodging()
            || motion.is_sliding()
        {
            self.cancel();
        }
        if self.held {
            self.held_elapsed += dt;
        }
        self.remaining = decay(self.remaining, dt);
        if matches!(self.kind, DefenseKind::Deflect | DefenseKind::Parry) && self.remaining == 0.0 {
            self.transition(if self.held {
                DefenseKind::Waiting
            } else {
                DefenseKind::Idle
            });
        }
        if self.held && self.held_elapsed + 1e-6 >= 0.20 {
            self.transition(if stamina.can_block() {
                DefenseKind::Guard
            } else {
                DefenseKind::Waiting
            });
            self.remaining = 0.0;
        }
        while let Some(edge) = self.edges.pop_front() {
            match edge {
                DefenseEdge::Cancelled => self.cancel(),
                DefenseEdge::Pressed => {
                    if self.held || !admit || exit {
                        continue;
                    }
                    self.held = true;
                    self.held_elapsed = 0.0;
                    if self.deflect_cooldown == 0.0 {
                        self.transition(DefenseKind::Deflect);
                        self.remaining = 0.20;
                        self.deflect_cooldown = 0.65;
                    } else {
                        self.transition(DefenseKind::Waiting);
                    }
                    self.begin_gesture();
                }
                DefenseEdge::Released => {
                    if !self.held {
                        continue;
                    }
                    self.held = false;
                    if self.kind == DefenseKind::Guard
                        && stamina.can_block()
                        && self.parry_cooldown == 0.0
                    {
                        self.transition(DefenseKind::Parry);
                        self.remaining = 0.18;
                        self.parry_cooldown = 0.65;
                        self.begin_gesture();
                    } else if self.kind != DefenseKind::Deflect {
                        self.transition(DefenseKind::Idle);
                        self.remaining = 0.0;
                    }
                }
            }
        }
    }
}
pub(super) fn decay(value: f32, dt: f32) -> f32 {
    if value <= dt + f32::EPSILON * 8.0 {
        0.0
    } else {
        value - dt
    }
}
