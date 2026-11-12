//! Life, the brief hit flash and the respawn countdown.
use crate::simulation::defense::decay;
use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    current: f32,
    maximum: f32,
    hit_remaining: f32,
    respawn_remaining: f32,
}
impl Health {
    pub fn new(maximum: f32) -> Self {
        assert!(maximum.is_finite() && maximum > 0.0);
        Self {
            current: maximum,
            maximum,
            hit_remaining: 0.0,
            respawn_remaining: 0.0,
        }
    }
    pub fn current(&self) -> f32 {
        self.current
    }
    pub fn maximum(&self) -> f32 {
        self.maximum
    }
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }
    pub fn hit_remaining(&self) -> f32 {
        self.hit_remaining
    }
    pub fn respawn_remaining(&self) -> f32 {
        self.respawn_remaining
    }
    pub(in crate::simulation) fn refill(&mut self) {
        self.current = self.maximum;
        self.hit_remaining = 0.0;
        self.respawn_remaining = 0.0;
    }
    pub(in crate::simulation) fn tick(&mut self, dt: f32) -> bool {
        self.hit_remaining = decay(self.hit_remaining, dt);
        if self.is_alive() {
            return false;
        }
        self.respawn_remaining = decay(self.respawn_remaining, dt);
        if self.respawn_remaining == 0.0 {
            self.refill();
            true
        } else {
            false
        }
    }
    pub(in crate::simulation) fn damage(&mut self, amount: f32) -> bool {
        if !self.is_alive() {
            return false;
        }
        self.current = (self.current - amount).max(0.0);
        self.hit_remaining = 0.18;
        if !self.is_alive() {
            self.respawn_remaining = 3.0;
        }
        true
    }
}
#[derive(Component, Clone, Copy)]
pub struct SpawnPoint(pub Vec2);
