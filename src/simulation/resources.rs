//! Stamina and control effects have one fixed-step authority.
use bevy::prelude::*;
#[derive(Component, Debug)]
pub struct Stamina {
    current: f32,
    quiet: f32,
}
impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            quiet: 0.0,
        }
    }
}
impl Stamina {
    pub fn current(&self) -> f32 {
        self.current
    }
    pub fn maximum(&self) -> f32 {
        100.0
    }
    pub fn quiet_elapsed(&self) -> f32 {
        self.quiet
    }
    pub(super) fn can_block(&self) -> bool {
        self.current >= 20.0
    }
    pub(super) fn block(&mut self) {
        self.current = (self.current - 20.0).max(0.0);
    }
    pub(super) fn disturb(&mut self) {
        self.quiet = 0.0;
    }
    pub(super) fn refill(&mut self) {
        *self = Self::default();
    }
    pub(super) fn recover(&mut self, dt: f32) {
        let previous = self.quiet;
        self.quiet += dt;
        let eligible = (self.quiet - 1.0).max(0.0) - (previous - 1.0).max(0.0);
        self.current = (self.current + 15.0 * eligible).min(100.0);
    }
}
#[derive(Component, Clone, Copy, Debug)]
pub struct SlowEffect {
    factor: f32,
    remaining: f32,
}
impl Default for SlowEffect {
    fn default() -> Self {
        Self {
            factor: 1.0,
            remaining: 0.0,
        }
    }
}
impl SlowEffect {
    pub fn factor(&self) -> f32 {
        self.factor
    }
    pub fn remaining(&self) -> f32 {
        self.remaining
    }
    pub(super) fn apply(&mut self, factor: f32, duration: f32) {
        self.factor = self.factor.min(factor);
        self.remaining = self.remaining.max(duration);
    }
    pub(super) fn tick(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.0);
        if self.remaining <= f32::EPSILON * 8.0 {
            *self = Self::default();
        }
    }
}
