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
#[derive(Component, Default, Debug)]
pub struct DefenseState {
    kind: DefenseKind,
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
        self.kind = DefenseKind::Idle;
        self.held = false;
        self.held_elapsed = 0.0;
        self.remaining = 0.0;
    }
    pub(super) fn advance(
        &mut self,
        dt: f32,
        admit: bool,
        exit: bool,
        motion: &MotionState,
        stamina: &Stamina,
    ) {
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
            self.kind = if self.held {
                DefenseKind::Waiting
            } else {
                DefenseKind::Idle
            };
        }
        if self.held && self.held_elapsed + 1e-6 >= 0.20 {
            self.kind = if stamina.can_block() {
                DefenseKind::Guard
            } else {
                DefenseKind::Waiting
            };
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
                        self.kind = DefenseKind::Deflect;
                        self.remaining = 0.20;
                        self.deflect_cooldown = 0.65;
                    } else {
                        self.kind = DefenseKind::Waiting;
                    }
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
                        self.kind = DefenseKind::Parry;
                        self.remaining = 0.18;
                        self.parry_cooldown = 0.65;
                    } else if self.kind != DefenseKind::Deflect {
                        self.kind = DefenseKind::Idle;
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
