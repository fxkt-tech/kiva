//! The committed action's private timeline: marker de-duplication and cancellation.
use super::skills::{RETURN_DURATION, Shape};
use crate::simulation::{ActionSnapshot, SequenceSnapshot};
use bevy::prelude::Entity;

const CANCELLED_ELAPSED: f32 = 0.65;

pub(super) struct ActionRuntime {
    pub snapshot: ActionSnapshot,
    pub previous: f32,
    pub markers: u8,
    pub cancelled: bool,
    pub return_remaining: f32,
    pub pull_stalled: f32,
    pub pull_stopped: bool,
    /// Fixed when the action starts: empowerment or the enraged attack bonus.
    pub damage_scale: f32,
    /// Enemies this action's blade or shockwave has already hit; each is hit
    /// at most once per action.
    pub struck: Vec<Entity>,
}
impl ActionRuntime {
    pub fn new(snapshot: ActionSnapshot, damage_scale: f32) -> Self {
        Self {
            snapshot,
            previous: 0.0,
            markers: 0,
            cancelled: false,
            return_remaining: 0.0,
            pull_stalled: 0.0,
            pull_stopped: false,
            damage_scale,
            struck: Vec::new(),
        }
    }
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
            self.return_remaining = RETURN_DURATION;
            let start = match self.snapshot.kind.spec().shape {
                Shape::Volley(volley) => volley.return_from,
                // Other actions retire on the next rule tick; until then
                // their snapshot keeps this established past-the-end time.
                _ => CANCELLED_ELAPSED,
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
