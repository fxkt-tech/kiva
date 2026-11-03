//! Rendering-independent timing and hit rules. All durations are in seconds.
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackKind {
    Light(usize),
    Training,
}

#[derive(Clone, Copy, Debug)]
pub struct AttackSpec {
    pub windup: f32,
    pub active: f32,
    pub recovery: f32,
    pub damage: f32,
    pub range: f32,
    pub half_arc: f32,
    pub step: f32,
}

impl AttackSpec {
    pub fn duration(self) -> f32 {
        self.windup + self.active + self.recovery
    }
}

pub const LIGHT_ATTACKS: [AttackSpec; 3] = [
    AttackSpec {
        windup: 0.14,
        active: 0.10,
        recovery: 0.28,
        damage: 10.0,
        range: 1.75,
        half_arc: 1.05,
        step: 0.18,
    },
    AttackSpec {
        windup: 0.18,
        active: 0.12,
        recovery: 0.32,
        damage: 15.0,
        range: 1.85,
        half_arc: 1.10,
        step: 0.22,
    },
    AttackSpec {
        windup: 0.30,
        active: 0.14,
        recovery: 0.48,
        damage: 25.0,
        range: 2.00,
        half_arc: 0.85,
        step: 0.28,
    },
];
pub const TRAINING_ATTACK: AttackSpec = AttackSpec {
    windup: 0.65,
    active: 0.14,
    recovery: 0.55,
    damage: 20.0,
    range: 2.65,
    half_arc: 1.05,
    step: 0.12,
};
pub const COMBO_WINDOW: f32 = 0.22;
pub const BODY_RADIUS: f32 = 0.32;
pub const GUARD_HALF_ARC: f32 = std::f32::consts::FRAC_PI_3;

#[derive(Debug)]
pub struct Attack {
    pub kind: AttackKind,
    pub elapsed: f32,
    queued: bool,
    buffered_until: Option<f32>,
    hit_targets: HashSet<u64>,
}

pub struct AttackStep {
    pub active: bool,
    pub forward_step: f32,
    pub finished: bool,
}

impl Attack {
    pub fn new(kind: AttackKind) -> Self {
        Self {
            kind,
            elapsed: 0.0,
            queued: false,
            buffered_until: None,
            hit_targets: HashSet::new(),
        }
    }

    pub fn spec(&self) -> AttackSpec {
        match self.kind {
            AttackKind::Light(index) => LIGHT_ATTACKS[index],
            AttackKind::Training => TRAINING_ATTACK,
        }
    }

    pub fn can_chain(&self) -> bool {
        matches!(self.kind, AttackKind::Light(0 | 1))
            && self.elapsed >= self.spec().duration() - COMBO_WINDOW
            && self.elapsed < self.spec().duration()
    }

    pub fn buffer_next(&mut self) -> bool {
        if matches!(self.kind, AttackKind::Light(0 | 1))
            && self.elapsed >= self.spec().windup
            && self.elapsed < self.spec().duration()
        {
            self.buffered_until = Some(self.elapsed + COMBO_WINDOW);
            if self.can_chain() {
                self.queued = true;
            }
            true
        } else {
            false
        }
    }

    pub fn next_combo(&self) -> Option<AttackKind> {
        match self.kind {
            AttackKind::Light(index) if self.queued && index < 2 => {
                Some(AttackKind::Light(index + 1))
            }
            _ => None,
        }
    }

    pub fn has_hit(&self) -> bool {
        !self.hit_targets.is_empty()
    }

    pub fn claim_target(&mut self, target: u64) -> bool {
        self.hit_targets.insert(target)
    }

    pub fn tick(&mut self, dt: f32) -> AttackStep {
        let spec = self.spec();
        let previous = self.elapsed;
        self.elapsed += dt.max(0.0);
        let chain_start = spec.duration() - COMBO_WINDOW;
        if self
            .buffered_until
            .is_some_and(|until| until >= chain_start && until >= previous)
            && self.elapsed >= chain_start
        {
            self.queued = true;
            self.buffered_until = None;
        }
        let end = spec.windup + spec.active;
        let step_start = spec.windup * 0.85;
        // Test the crossed interval, not just the current phase: a slow frame
        // must not skip an entire damage window or apply the lunge twice.
        AttackStep {
            active: previous < end && self.elapsed >= spec.windup,
            forward_step: (ease((self.elapsed - step_start) / (end - step_start))
                - ease((previous - step_start) / (end - step_start)))
                * spec.step,
            finished: self.elapsed >= spec.duration(),
        }
    }

    pub fn phase(&self) -> &'static str {
        if self.elapsed < self.spec().windup {
            "WINDUP"
        } else if self.elapsed < self.spec().windup + self.spec().active {
            "STRIKE"
        } else {
            "RECOVERY"
        }
    }
}

/// Integrate exponential damping exactly, so slow frames do not increase knockback.
pub fn damped_motion(dt: f32, rate: f32) -> (f32, f32) {
    let retention = (-rate * dt).exp();
    ((1.0 - retention) / rate, retention)
}

pub const DODGE_DURATION: f32 = 0.42;
pub const DODGE_DISTANCE: f32 = 2.4;
pub const DODGE_COOLDOWN: f32 = 0.72;
pub const JUMP_SPEED: f32 = 6.4;
pub const GRAVITY: f32 = 18.0;

pub fn ease(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(Debug, Default)]
pub struct JumpState {
    used: u8,
}

impl JumpState {
    pub fn try_jump(&mut self) -> Option<f32> {
        if self.used >= 2 {
            return None;
        }
        self.used += 1;
        Some(JUMP_SPEED)
    }

    pub fn used(&self) -> u8 {
        self.used
    }

    pub fn land(&mut self) {
        self.used = 0;
    }
}

pub fn jump_step(height: f32, speed: f32, dt: f32) -> (f32, f32) {
    let next_height = height + speed * dt - 0.5 * GRAVITY * dt * dt;
    let next_speed = speed - GRAVITY * dt;
    if next_height <= 0.0 && next_speed <= 0.0 {
        (0.0, 0.0)
    } else {
        (next_height.max(0.0), next_speed)
    }
}

pub fn dodge_step(previous: f32, current: f32) -> f32 {
    (ease(current / DODGE_DURATION) - ease(previous / DODGE_DURATION)) * DODGE_DISTANCE
}

pub fn dodge_protected(previous: f32, current: f32) -> bool {
    previous < 0.28 && current >= 0.06
}

pub fn inside_attack(distance: f32, forward_dot: f32, spec: AttackSpec) -> bool {
    // Expand the angular test by the target's radius as well as the range test.
    let allowance = (BODY_RADIUS / distance.max(BODY_RADIUS)).asin();
    distance <= spec.range + BODY_RADIUS
        && forward_dot >= (spec.half_arc + allowance).min(std::f32::consts::PI).cos()
}

pub fn guarded_damage(damage: f32, guarding: bool, incoming_dot: f32) -> (f32, bool) {
    let blocked = guarding && incoming_dot >= GUARD_HALF_ARC.cos();
    (if blocked { damage * 0.2 } else { damage }, blocked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windup_cannot_hit_but_a_slow_frame_crossing_the_strike_can() {
        let mut attack = Attack::new(AttackKind::Light(0));
        assert!(!attack.tick(0.05).active);
        assert!(attack.tick(0.3).active);
        assert!(!attack.tick(0.01).active);
    }

    #[test]
    fn one_swing_hits_each_target_only_once() {
        let mut attack = Attack::new(AttackKind::Light(0));
        assert!(attack.claim_target(42));
        assert!(!attack.claim_target(42));
        assert!(attack.claim_target(43));
        assert!(Attack::new(AttackKind::Light(1)).claim_target(42));
    }

    #[test]
    fn early_click_is_rejected_and_repeated_late_clicks_queue_only_one_step() {
        let mut attack = Attack::new(AttackKind::Light(0));
        assert!(!attack.buffer_next());
        assert_eq!(attack.next_combo(), None);
        attack.tick(0.35);
        for _ in 0..10 {
            assert!(attack.buffer_next());
        }
        assert_eq!(attack.next_combo(), Some(AttackKind::Light(1)));
        attack.tick(1.0);
        assert!(!attack.buffer_next());
    }

    #[test]
    fn strike_input_is_buffered_until_the_chain_window_opens() {
        let mut attack = Attack::new(AttackKind::Light(0));
        attack.tick(0.16);
        assert!(attack.buffer_next());
        assert_eq!(attack.next_combo(), None);
        attack.tick(0.16);
        assert_eq!(attack.next_combo(), Some(AttackKind::Light(1)));
    }

    #[test]
    fn final_and_training_attacks_never_chain() {
        for kind in [AttackKind::Light(2), AttackKind::Training] {
            let mut attack = Attack::new(kind);
            attack.tick(attack.spec().duration() - 0.1);
            assert!(!attack.buffer_next());
            assert_eq!(attack.next_combo(), None);
        }
    }

    #[test]
    fn lunge_distance_is_independent_of_tick_size() {
        for dt in [0.005, 0.016, 0.1, 0.7] {
            let mut attack = Attack::new(AttackKind::Light(2));
            let mut distance = 0.0;
            while attack.elapsed < attack.spec().duration() {
                distance += attack.tick(dt).forward_step;
            }
            assert!((distance - LIGHT_ATTACKS[2].step).abs() < 0.0001);
        }
    }

    #[test]
    fn hits_require_range_and_facing() {
        let spec = LIGHT_ATTACKS[0];
        assert!(inside_attack(1.6, 1.0, spec));
        assert!(!inside_attack(3.0, 1.0, spec));
        assert!(!inside_attack(1.0, -1.0, spec));
    }

    #[test]
    fn knockback_distance_is_independent_of_frame_rate() {
        for dt in [0.004, 0.016, 0.1, 0.25] {
            let mut speed = 2.2;
            let mut distance = 0.0;
            let mut time = 0.0;
            while time < 4.0 {
                let (movement, retention) = damped_motion(dt, 12.0);
                distance += speed * movement;
                speed *= retention;
                time += dt;
            }
            assert!(
                (distance - 2.2 / 12.0).abs() < 0.001,
                "dt={dt}, distance={distance}"
            );
        }
    }

    #[test]
    fn two_jumps_are_available_and_only_landing_refills_them() {
        let mut jumps = JumpState::default();
        assert_eq!(jumps.try_jump(), Some(JUMP_SPEED));
        assert_eq!(jumps.used(), 1);
        assert_eq!(jumps.try_jump(), Some(JUMP_SPEED));
        assert_eq!(jumps.used(), 2);
        assert_eq!(jumps.try_jump(), None);
        assert_eq!(jumps.try_jump(), None);
        jumps.land();
        assert_eq!(jumps.used(), 0);
        assert_eq!(jumps.try_jump(), Some(JUMP_SPEED));
        assert_eq!(jumps.try_jump(), Some(JUMP_SPEED));
        assert_eq!(jumps.try_jump(), None);
    }

    #[test]
    fn second_jump_restores_upward_speed_while_falling() {
        let mut jumps = JumpState::default();
        let first = jumps.try_jump().unwrap();
        let (height, falling_speed) = jump_step(0.0, first, 0.4);
        assert!(height > 0.0 && falling_speed < 0.0);
        let second = jumps.try_jump().unwrap();
        let (next_height, next_speed) = jump_step(height, second, 0.016);
        assert!(next_height > height && next_speed > 0.0);
        assert_eq!(jumps.try_jump(), None);
    }

    #[test]
    fn jump_has_consistent_apex_and_lands_without_sinking() {
        for dt in [0.004, 0.016, 0.05] {
            let (mut height, mut speed, mut apex) = (0.0_f32, JUMP_SPEED, 0.0_f32);
            for _ in 0..300 {
                (height, speed) = jump_step(height, speed, dt);
                apex = apex.max(height);
                if speed == 0.0 {
                    break;
                }
            }
            assert!((apex - JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY)).abs() < 0.02);
            assert_eq!((height, speed), (0.0, 0.0));
        }
    }

    #[test]
    fn dodge_travel_and_invulnerability_have_bounded_windows() {
        for dt in [0.004, 0.016, 0.1] {
            let (mut time, mut distance) = (0.0, 0.0);
            while time < DODGE_DURATION {
                distance += dodge_step(time, time + dt);
                time += dt;
            }
            assert!((distance - DODGE_DISTANCE).abs() < 0.001);
        }
        assert!(!dodge_protected(0.0, 0.03));
        assert!(dodge_protected(0.05, 0.10));
        assert!(dodge_protected(0.0, 0.4));
        assert!(!dodge_protected(0.3, 0.42));
    }

    #[test]
    fn only_frontal_guard_reduces_damage() {
        assert_eq!(guarded_damage(20.0, true, 1.0), (4.0, true));
        assert_eq!(guarded_damage(20.0, true, -1.0), (20.0, false));
        assert_eq!(guarded_damage(20.0, false, 1.0), (20.0, false));
    }
}
