//! Rendering-independent timing and hit rules. All durations are in seconds.
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackAxis {
    Horizontal,
    Vertical,
}
impl AttackAxis {
    pub fn label(self) -> &'static str {
        match self {
            Self::Horizontal => "HORIZONTAL",
            Self::Vertical => "VERTICAL",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackKind {
    Light(AttackAxis, usize),
    Focus(AttackAxis),
    Crouch(AttackAxis),
    Air(AttackAxis),
    Riposte,
    Training(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Armor {
    Normal,
    Blue,
    Gold,
}

impl AttackKind {
    pub fn axis(self) -> AttackAxis {
        match self {
            Self::Light(axis, _) | Self::Focus(axis) | Self::Crouch(axis) | Self::Air(axis) => axis,
            _ => AttackAxis::Vertical,
        }
    }
    pub fn armor(self) -> Armor {
        match self {
            Self::Light(_, 2) | Self::Focus(_) | Self::Training(true) => Armor::Blue,
            Self::Riposte => Armor::Gold,
            _ => Armor::Normal,
        }
    }
    pub fn label(self) -> String {
        match self {
            Self::Light(axis, i) => format!(
                "{} {}{}",
                axis.label(),
                i + 1,
                if i == 2 { " / BLUE" } else { "" }
            ),
            Self::Focus(axis) => format!("{} SWORD ENERGY / BLUE", axis.label()),
            Self::Crouch(AttackAxis::Horizontal) => "CROUCH SLASH".into(),
            Self::Crouch(AttackAxis::Vertical) => "UPPERCUT".into(),
            Self::Air(AttackAxis::Horizontal) => "AIR SLASH".into(),
            Self::Air(AttackAxis::Vertical) => "AIR PLUNGE".into(),
            Self::Riposte => "RIPOSTE / GOLD".into(),
            Self::Training(blue) => format!("TRAINING {}", if blue { "BLUE" } else { "NORMAL" }),
        }
    }
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
pub const CHARGE_READY: f32 = 0.55;
pub const CHARGE_AUTO_RELEASE: f32 = 1.45;
pub const PARRY_DURATION: f32 = 0.60;
pub const PARRY_START: f32 = 0.05;
pub const PARRY_END: f32 = 0.28;

#[derive(Debug)]
pub struct Charge {
    pub axis: AttackAxis,
    pub elapsed: f32,
    pub combo: usize,
}
impl Charge {
    pub fn release(&self) -> AttackKind {
        if self.elapsed >= CHARGE_READY {
            AttackKind::Focus(self.axis)
        } else {
            AttackKind::Light(self.axis, self.combo)
        }
    }
}

#[derive(Debug)]
pub struct Stamina {
    pub value: f32,
    delay: f32,
}
impl Default for Stamina {
    fn default() -> Self {
        Self {
            value: 100.0,
            delay: 0.0,
        }
    }
}
impl Stamina {
    pub fn spend(&mut self, cost: f32) -> bool {
        if self.value < cost {
            return false;
        }
        self.value -= cost;
        self.delay = 0.65;
        true
    }
    pub fn tick(&mut self, dt: f32) {
        let recovering = (dt - self.delay).max(0.0);
        self.delay = (self.delay - dt).max(0.0);
        self.value = (self.value + recovering * 24.0).min(100.0);
    }
}

#[derive(Clone, Copy)]
pub enum Defense {
    Open,
    Blue,
    Gold,
    Parry,
    Evade,
    Striking(Armor),
}
#[derive(Debug, PartialEq, Eq)]
pub enum HitResult {
    Damage { stagger: bool },
    Counter,
    Evade,
    Clash,
}
pub fn resolve_hit(attack: Armor, defense: Defense) -> HitResult {
    match (attack, defense) {
        (_, Defense::Evade) => HitResult::Evade,
        (Armor::Blue, Defense::Parry) => HitResult::Counter,
        (a, Defense::Striking(b)) if can_clash(a, b) => HitResult::Clash,
        (Armor::Normal, Defense::Striking(Armor::Blue | Armor::Gold))
        | (Armor::Blue, Defense::Striking(Armor::Gold)) => HitResult::Damage { stagger: false },
        (Armor::Normal, Defense::Blue | Defense::Gold) | (Armor::Blue, Defense::Gold) => {
            HitResult::Damage { stagger: false }
        }
        _ => HitResult::Damage { stagger: true },
    }
}
pub fn can_clash(a: Armor, b: Armor) -> bool {
    a == b && a != Armor::Gold
}
pub fn parry_window(previous: f32, current: f32) -> bool {
    previous < PARRY_END && current >= PARRY_START
}

#[derive(Debug)]
pub struct Attack {
    pub kind: AttackKind,
    pub elapsed: f32,
    queued: Option<AttackAxis>,
    buffered_until: Option<(f32, AttackAxis)>,
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
            queued: None,
            buffered_until: None,
            hit_targets: HashSet::new(),
        }
    }

    pub fn spec(&self) -> AttackSpec {
        match self.kind {
            AttackKind::Light(axis, index) => {
                let mut spec = LIGHT_ATTACKS[index];
                if axis == AttackAxis::Vertical {
                    spec.half_arc = 0.48;
                    spec.range += 0.12;
                }
                spec
            }
            AttackKind::Focus(axis) => AttackSpec {
                windup: 0.16,
                active: 0.10,
                recovery: 0.40,
                damage: 30.0,
                range: 1.9,
                half_arc: if axis == AttackAxis::Horizontal {
                    1.1
                } else {
                    0.48
                },
                step: 0.20,
            },
            AttackKind::Crouch(axis) => AttackSpec {
                windup: 0.14,
                active: 0.12,
                recovery: 0.25,
                damage: 14.0,
                range: 1.8,
                half_arc: if axis == AttackAxis::Horizontal {
                    1.2
                } else {
                    0.50
                },
                step: 0.2,
            },
            AttackKind::Air(axis) => AttackSpec {
                windup: 0.12,
                active: 0.16,
                recovery: 0.24,
                damage: if axis == AttackAxis::Horizontal {
                    12.0
                } else {
                    18.0
                },
                range: 1.9,
                half_arc: 0.8,
                step: 0.12,
            },
            AttackKind::Riposte => AttackSpec {
                windup: 0.10,
                active: 0.16,
                recovery: 0.35,
                damage: 35.0,
                range: 2.5,
                half_arc: 0.9,
                step: 0.55,
            },
            AttackKind::Training(_) => TRAINING_ATTACK,
        }
    }

    pub fn can_chain(&self) -> bool {
        matches!(self.kind, AttackKind::Light(_, 0 | 1))
            && self.elapsed >= self.spec().duration() - COMBO_WINDOW
            && self.elapsed < self.spec().duration()
    }

    pub fn buffer_next(&mut self, axis: AttackAxis) -> bool {
        if matches!(self.kind, AttackKind::Light(_, 0 | 1))
            && self.elapsed >= self.spec().windup
            && self.elapsed < self.spec().duration()
        {
            self.buffered_until = Some((self.elapsed + COMBO_WINDOW, axis));
            if self.can_chain() {
                self.queued = Some(axis);
            }
            true
        } else {
            false
        }
    }

    pub fn next_combo(&self) -> Option<AttackKind> {
        match self.kind {
            AttackKind::Light(_, index) if index < 2 => {
                self.queued.map(|axis| AttackKind::Light(axis, index + 1))
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
        if let Some((until, axis)) = self.buffered_until
            && until >= chain_start
            && until >= previous
            && self.elapsed >= chain_start
        {
            self.queued = Some(axis);
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

/// Swept wave collision in local coordinates: forward distance, lateral offset,
/// and height relative to the defender's torso. Tests the whole frame interval.
pub fn wave_intersects(
    previous: f32,
    current: f32,
    forward: f32,
    lateral: f32,
    height: f32,
    axis: AttackAxis,
) -> bool {
    let (width, tall) = match axis {
        AttackAxis::Horizontal => (1.3, 0.4),
        AttackAxis::Vertical => (0.42, 1.35),
    };
    forward >= previous - BODY_RADIUS
        && forward <= current + BODY_RADIUS
        && lateral.abs() <= width + BODY_RADIUS
        && height.abs() <= tall + BODY_RADIUS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windup_cannot_hit_but_a_slow_frame_crossing_the_strike_can() {
        let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 0));
        assert!(!attack.tick(0.05).active);
        assert!(attack.tick(0.3).active);
        assert!(!attack.tick(0.01).active);
    }

    #[test]
    fn one_swing_hits_each_target_only_once() {
        let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 0));
        assert!(attack.claim_target(42));
        assert!(!attack.claim_target(42));
        assert!(attack.claim_target(43));
        assert!(Attack::new(AttackKind::Light(AttackAxis::Horizontal, 1)).claim_target(42));
    }

    #[test]
    fn early_click_is_rejected_and_repeated_late_clicks_queue_only_one_step() {
        let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 0));
        assert!(!attack.buffer_next(AttackAxis::Horizontal));
        assert_eq!(attack.next_combo(), None);
        attack.tick(0.35);
        for _ in 0..10 {
            assert!(attack.buffer_next(AttackAxis::Horizontal));
        }
        assert_eq!(
            attack.next_combo(),
            Some(AttackKind::Light(AttackAxis::Horizontal, 1))
        );
        attack.tick(1.0);
        assert!(!attack.buffer_next(AttackAxis::Horizontal));
    }

    #[test]
    fn strike_input_is_buffered_until_the_chain_window_opens() {
        let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 0));
        attack.tick(0.16);
        assert!(attack.buffer_next(AttackAxis::Horizontal));
        assert_eq!(attack.next_combo(), None);
        attack.tick(0.16);
        assert_eq!(
            attack.next_combo(),
            Some(AttackKind::Light(AttackAxis::Horizontal, 1))
        );
    }

    #[test]
    fn final_and_training_attacks_never_chain() {
        for kind in [
            AttackKind::Light(AttackAxis::Horizontal, 2),
            AttackKind::Training(true),
        ] {
            let mut attack = Attack::new(kind);
            attack.tick(attack.spec().duration() - 0.1);
            assert!(!attack.buffer_next(AttackAxis::Horizontal));
            assert_eq!(attack.next_combo(), None);
        }
    }

    #[test]
    fn lunge_distance_is_independent_of_tick_size() {
        for dt in [0.005, 0.016, 0.1, 0.7] {
            let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 2));
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
    fn charge_release_chooses_a_combo_or_a_blue_wave() {
        let mut charge = Charge {
            axis: AttackAxis::Vertical,
            elapsed: 0.05,
            combo: 1,
        };
        assert_eq!(charge.release(), AttackKind::Light(AttackAxis::Vertical, 1));
        charge.elapsed = CHARGE_READY;
        assert_eq!(charge.release(), AttackKind::Focus(AttackAxis::Vertical));
    }

    #[test]
    fn mixed_combo_keeps_its_stage_but_uses_the_new_axis() {
        let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 0));
        attack.tick(0.35);
        attack.buffer_next(AttackAxis::Vertical);
        assert_eq!(
            attack.next_combo(),
            Some(AttackKind::Light(AttackAxis::Vertical, 1))
        );
        assert_eq!(
            AttackKind::Light(AttackAxis::Vertical, 2).armor(),
            Armor::Blue
        );
    }

    #[test]
    fn normal_beats_parry_parry_beats_blue_and_blue_resists_normal_stagger() {
        assert_eq!(
            resolve_hit(Armor::Normal, Defense::Parry),
            HitResult::Damage { stagger: true }
        );
        assert_eq!(resolve_hit(Armor::Blue, Defense::Parry), HitResult::Counter);
        assert_eq!(
            resolve_hit(Armor::Normal, Defense::Blue),
            HitResult::Damage { stagger: false }
        );
        assert_eq!(
            resolve_hit(Armor::Gold, Defense::Parry),
            HitResult::Damage { stagger: true }
        );
        assert_eq!(resolve_hit(Armor::Blue, Defense::Evade), HitResult::Evade);
        assert_eq!(
            resolve_hit(Armor::Normal, Defense::Striking(Armor::Normal)),
            HitResult::Clash
        );
        assert!(can_clash(Armor::Normal, Armor::Normal));
        assert!(can_clash(Armor::Blue, Armor::Blue));
        assert!(!can_clash(Armor::Normal, Armor::Blue));
        assert!(!can_clash(Armor::Gold, Armor::Gold));
    }

    #[test]
    fn parry_is_a_short_window_not_a_held_guard() {
        assert!(!parry_window(0.0, 0.03));
        assert!(parry_window(0.04, 0.08));
        assert!(parry_window(0.0, 0.40));
        assert!(!parry_window(0.30, 0.40));
    }

    #[test]
    fn stamina_limits_dodges_and_recovers_only_after_a_delay() {
        let mut stamina = Stamina::default();
        for _ in 0..5 {
            assert!(stamina.spend(20.0));
        }
        assert!(!stamina.spend(20.0));
        stamina.tick(0.6);
        assert_eq!(stamina.value, 0.0);
        stamina.tick(1.05);
        assert!((stamina.value - 24.0).abs() < 0.001);
        stamina.tick(10.0);
        assert_eq!(stamina.value, 100.0);
    }

    #[test]
    fn waves_hit_across_slow_frames_and_have_different_shapes() {
        assert!(wave_intersects(
            0.0,
            5.0,
            3.0,
            1.0,
            0.0,
            AttackAxis::Horizontal
        ));
        assert!(!wave_intersects(
            0.0,
            5.0,
            3.0,
            1.0,
            0.0,
            AttackAxis::Vertical
        ));
        assert!(wave_intersects(
            0.0,
            5.0,
            3.0,
            0.0,
            1.0,
            AttackAxis::Vertical
        ));
        assert!(!wave_intersects(
            0.0,
            5.0,
            3.0,
            0.0,
            1.0,
            AttackAxis::Horizontal
        ));
        assert!(!wave_intersects(
            0.0,
            1.0,
            3.0,
            0.0,
            0.0,
            AttackAxis::Horizontal
        ));
    }
}
