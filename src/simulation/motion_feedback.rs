//! Accepted movement and post-solver contact/stride facts. No audio clock or
//! asset names live here; consumers each keep their own sequence cursor.
use super::{Body, MotionState};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionKind {
    Jump,
    Dodge,
    Land,
    Footstep,
}

#[derive(Clone, Copy, Debug)]
pub struct MotionEvent {
    pub sequence: u64,
    pub actor: Entity,
    pub kind: MotionKind,
    pub position: Vec3,
    pub jump_count: u8,
    pub sprinting: bool,
}

#[derive(Resource, Default)]
pub struct MotionFeedback {
    next_sequence: u64,
    events: VecDeque<MotionEvent>,
}
impl MotionFeedback {
    pub fn events(&self) -> impl Iterator<Item = &MotionEvent> {
        self.events.iter()
    }
    pub(super) fn push(
        &mut self,
        actor: Entity,
        kind: MotionKind,
        body: &Body,
        motion: &MotionState,
    ) {
        self.next_sequence += 1;
        self.events.push_back(MotionEvent {
            sequence: self.next_sequence,
            actor,
            kind,
            position: body.world_position(),
            jump_count: motion.jumps_used(),
            sprinting: motion.is_sprinting(),
        });
        if self.events.len() > 128 {
            self.events.pop_front();
        }
    }
}

/// Both motor acceptance and post-solver sampling use the same fixed clock
/// and bounded fact stream, including its common simulation pause gate.
#[derive(SystemParam)]
pub(super) struct MotionRecorder<'w> {
    time: Res<'w, Time<Fixed>>,
    feedback: ResMut<'w, MotionFeedback>,
}
impl MotionRecorder<'_> {
    pub(super) fn delta_secs(&self) -> f32 {
        self.time.delta_secs()
    }
    pub(super) fn push(
        &mut self,
        actor: Entity,
        kind: MotionKind,
        body: &Body,
        motion: &MotionState,
    ) {
        self.feedback.push(actor, kind, body, motion);
    }
}

/// Contact/stride sampling history, not an alternative movement authority.
/// Installed only for controlled combatants and reset with physical teleports.
#[derive(Component, Default)]
pub(super) struct MotionFeedbackState {
    initialized: bool,
    grounded: bool,
    airborne_time: f32,
    last_jump_count: u8,
    distance: f32,
}
impl MotionFeedbackState {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(super) fn publish(
        &mut self,
        dt: f32,
        body: &Body,
        motion: &MotionState,
        moving: bool,
    ) -> (bool, bool) {
        if dt <= 0.0 {
            return (false, false);
        }
        let grounded = motion.is_grounded();
        if !self.initialized {
            self.initialized = true;
            self.grounded = grounded;
            self.last_jump_count = motion.jumps_used();
            return (false, false);
        }
        // One missing contact tick from solver skin/jitter is not a landing.
        // Real accepted jumps and falls have an observable airborne interval.
        let landed = grounded
            && !self.grounded
            && (self.last_jump_count > 0 || self.airborne_time + 1e-6 >= 0.05);
        self.airborne_time = if grounded {
            0.0
        } else {
            self.airborne_time + dt
        };
        let continuously_grounded = self.grounded && grounded;
        self.grounded = grounded;
        self.last_jump_count = motion.jumps_used();
        let support = motion
            .support
            .map_or(Vec2::ZERO, |support| support.velocity.xz() * dt);
        let distance = (body.step_displacement() - support).length();
        let speed = distance / dt;
        let walking = continuously_grounded
            && moving
            && !motion.is_dodging()
            && !motion.is_sliding()
            && speed > 0.15
            && speed <= body.speed * 2.0 + 1.0;
        if !walking {
            self.distance = 0.0;
            return (landed, false);
        }
        // Kiva stride adaptation, measured from actual displacement. Crouched
        // steps are shorter; sprinting naturally crosses the same stride faster.
        let stride = if motion.is_crouched() { 0.8 } else { 2.0 } * body.scale();
        self.distance += distance;
        let step = self.distance >= stride;
        if step {
            self.distance %= stride;
        }
        (landed, step)
    }
}
