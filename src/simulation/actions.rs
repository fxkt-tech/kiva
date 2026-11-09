//! Fixed-step action state. Device input only submits held states and latched requests.

use bevy::prelude::*;

pub(super) const GRAVITY: f32 = 18.0;
const JUMP_VELOCITY: f32 = 6.4;
const SPRINT_MULTIPLIER: f32 = 1.6;
const BLOCK_MULTIPLIER: f32 = 0.35;
const CROUCH_SPEED: f32 = 1.6;
const SLIDE_DURATION: f32 = 0.55;
const SLIDE_SPEED: f32 = 5.0;
const SPRINT_HOLD_TIME: f32 = 0.16;
const DODGE_DURATION: f32 = 0.42;
const SPRINT_DODGE_DURATION: f32 = 0.26;
const DODGE_DISTANCE: f32 = 2.4;
const DODGE_PROTECTION_START: f32 = 0.06;
const DODGE_PROTECTION_END: f32 = 0.28;
const DODGE_COOLDOWN: f32 = 0.8;

#[derive(Component, Default)]
pub struct ActionIntent {
    shift_held: bool,
    shift_released: bool,
    movement_held: bool,
    movement_released: bool,
    block: bool,
    jump: bool,
    dodge: bool,
    crouch_toggle: bool,
    cancel_stances: bool,
}

impl ActionIntent {
    pub fn set_held(&mut self, physical_shift: bool, block: bool) {
        self.shift_held = physical_shift;
        if !physical_shift {
            self.shift_released = true;
        }
        self.block = block;
    }

    pub fn set_movement_held(&mut self, held: bool) {
        self.movement_held = held;
        if !held {
            self.movement_released = true;
        }
    }

    pub fn request_jump(&mut self) {
        self.jump = true;
    }

    pub fn request_dodge(&mut self) {
        self.dodge = true;
    }

    pub fn request_crouch_toggle(&mut self) {
        self.crouch_toggle = !self.crouch_toggle;
    }

    pub fn clear(&mut self) {
        *self = Self {
            cancel_stances: true,
            ..Self::default()
        };
    }
}

#[derive(Component)]
pub struct Facing(Vec2);

impl Default for Facing {
    fn default() -> Self {
        Self(Vec2::NEG_Y)
    }
}

impl Facing {
    pub fn set(&mut self, direction: Vec2) {
        let normalized = direction.normalize_or_zero();
        if direction.is_finite() && normalized.length_squared() > 0.5 {
            self.0 = normalized;
        }
    }

    pub fn direction(&self) -> Vec2 {
        self.0
    }
}

struct Leap {
    velocity: Vec3,
    elapsed: f32,
    duration: f32,
}

struct Dodge {
    elapsed: f32,
    direction: Vec2,
    held: bool,
}

struct Slide {
    elapsed: f32,
    direction: Vec2,
}

impl Dodge {
    fn duration(&self) -> f32 {
        if self.held {
            SPRINT_DODGE_DURATION
        } else {
            DODGE_DURATION
        }
    }
}

fn dodge_distance(elapsed: f32) -> f32 {
    let progress = (elapsed / DODGE_DURATION).clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress) * DODGE_DISTANCE
}

#[derive(Component)]
pub struct MotionState {
    previous_height: f32,
    height: f32,
    vertical_velocity: f32,
    jumps_used: u8,
    dodge_cooldown: f32,
    dodge: Option<Dodge>,
    dodge_protected: bool,
    blocking: bool,
    sprinting: bool,
    sprint_hold_time: f32,
    sprint_latched: bool,
    crouched: bool,
    slide: Option<Slide>,
    leap: Option<Leap>,
    pub(super) support: Option<super::physics::Support>,
    pub(super) pending_vertical_velocity: Option<f32>,
    pub(super) pending_planar_velocity: Option<Vec2>,
    pub(super) capsule_height: f32,
    pub(super) steep_normal: Option<Vec3>,
}

impl Default for MotionState {
    fn default() -> Self {
        Self {
            previous_height: 0.0,
            height: 0.0,
            vertical_velocity: 0.0,
            jumps_used: 0,
            dodge_cooldown: 0.0,
            dodge: None,
            dodge_protected: false,
            blocking: false,
            sprinting: false,
            sprint_hold_time: 0.0,
            sprint_latched: false,
            crouched: false,
            slide: None,
            leap: None,
            support: None,
            pending_vertical_velocity: None,
            pending_planar_velocity: None,
            capsule_height: super::physics::STANDING_HEIGHT,
            steep_normal: None,
        }
    }
}

impl MotionState {
    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn interpolated_height(&self, alpha: f32) -> f32 {
        self.previous_height
            .lerp(self.height, alpha.clamp(0.0, 1.0))
    }

    pub fn is_grounded(&self) -> bool {
        self.support.is_some() && self.pending_vertical_velocity.is_none()
    }

    pub fn surface(&self) -> Option<super::SurfaceHit> {
        self.support.map(|support| support.hit)
    }

    pub fn support_normal(&self) -> Vec3 {
        self.support.map_or(Vec3::Y, |support| support.hit.normal)
    }

    pub fn is_dodging(&self) -> bool {
        self.dodge.is_some()
    }

    pub fn is_invulnerable(&self) -> bool {
        self.dodge_protected
    }

    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    pub fn is_sprinting(&self) -> bool {
        self.sprinting
    }

    pub fn is_crouched(&self) -> bool {
        self.crouched
    }

    pub fn is_sliding(&self) -> bool {
        self.slide.is_some()
    }

    pub fn slide_elapsed(&self) -> f32 {
        self.slide.as_ref().map_or(0.0, |slide| slide.elapsed)
    }

    pub fn slide_direction(&self) -> Vec2 {
        self.slide
            .as_ref()
            .map_or(Vec2::ZERO, |slide| slide.direction)
    }

    pub fn sprint_latched(&self) -> bool {
        self.sprint_latched
    }

    pub(super) fn cancel_sprint(&mut self) {
        self.sprint_hold_time = 0.0;
        self.sprint_latched = false;
        self.sprinting = false;
    }

    fn exit_crouch(&mut self) {
        self.crouched = false;
        self.slide = None;
    }

    pub fn is_leaping(&self) -> bool {
        self.leap.is_some()
    }

    pub(super) fn leap_drive_ended(&self) -> bool {
        self.leap
            .as_ref()
            .is_some_and(|leap| leap.elapsed >= leap.duration)
    }

    pub fn dodge_remaining(&self) -> f32 {
        self.dodge
            .as_ref()
            .map_or(0.0, |dodge| (dodge.duration() - dodge.elapsed).max(0.0))
    }

    pub fn dodge_elapsed(&self) -> f32 {
        self.dodge.as_ref().map_or(0.0, |dodge| dodge.elapsed)
    }

    pub fn dodge_direction(&self) -> Vec2 {
        self.dodge
            .as_ref()
            .map_or(Vec2::ZERO, |dodge| dodge.direction)
    }

    pub fn dodge_cooldown(&self) -> f32 {
        self.dodge_cooldown
    }

    pub fn vertical_velocity(&self) -> f32 {
        self.vertical_velocity
    }

    pub fn jumps_used(&self) -> u8 {
        self.jumps_used
    }

    /// Legacy planar adapter. The former peak parameter is validated for
    /// compatibility; the ballistic arc uses flight duration and shared gravity.
    pub fn start_leap(&mut self, origin: Vec2, target: Vec2, duration: f32, peak: f32) -> bool {
        if !peak.is_finite() || peak <= 0.0 {
            return false;
        }
        self.start_leap_to(
            Vec3::new(origin.x, self.height, origin.y),
            Vec3::new(target.x, self.height, target.y),
            duration,
        )
    }

    /// Launch toward a world-space destination using the shared physical gravity.
    pub fn start_leap_to(&mut self, origin: Vec3, target: Vec3, duration: f32) -> bool {
        if !self.is_grounded()
            || self.is_dodging()
            || !origin.is_finite()
            || !target.is_finite()
            || !duration.is_finite()
            || duration <= 0.0
        {
            return false;
        }
        let mut velocity = (target - origin) / duration;
        velocity.y += 0.5 * GRAVITY * duration;
        if !velocity.is_finite() {
            return false;
        }
        self.blocking = false;
        self.sprinting = false;
        self.exit_crouch();
        self.pending_vertical_velocity = Some(velocity.y);
        self.pending_planar_velocity = Some(velocity.xz());
        self.support = None;
        self.leap = Some(Leap {
            velocity,
            elapsed: 0.0,
            duration,
        });
        true
    }

    pub fn launch_up(&mut self, velocity: f32) -> bool {
        if !velocity.is_finite() || velocity <= 0.0 {
            return false;
        }
        self.vertical_velocity = self.vertical_velocity.max(velocity);
        self.pending_vertical_velocity = Some(self.vertical_velocity);
        self.support = None;
        self.dodge = None;
        self.dodge_protected = false;
        self.leap = None;
        self.blocking = false;
        self.sprinting = false;
        self.exit_crouch();
        true
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn reset_at(&mut self, height: f32) {
        self.reset();
        self.height = height;
        self.previous_height = height;
    }

    /// Publish the result of the sole physical solver, never integrate height here.
    pub(super) fn publish(
        &mut self,
        height: f32,
        velocity: f32,
        support: Option<super::physics::Support>,
    ) {
        let landed = self.support.is_none() && support.is_some();
        self.previous_height = self.height;
        self.height = height;
        self.vertical_velocity = velocity;
        self.support = support;
        if landed {
            self.jumps_used = 0;
            self.leap = None;
        }
    }

    pub(super) fn keep_crouched(&mut self) {
        self.crouched = true;
    }

    pub(super) fn advance(
        &mut self,
        intent: &mut ActionIntent,
        direction: Vec2,
        facing: Vec2,
        speed: f32,
        dt: f32,
    ) -> Vec2 {
        if std::mem::take(&mut intent.cancel_stances) {
            self.cancel_sprint();
            self.exit_crouch();
        }
        self.dodge_cooldown = (self.dodge_cooldown - dt).max(0.0);
        let jump = std::mem::take(&mut intent.jump);
        let dodge = std::mem::take(&mut intent.dodge);
        let crouch_toggle = std::mem::take(&mut intent.crouch_toggle);
        self.blocking = false;
        self.sprinting = false;
        self.dodge_protected = false;
        // Physical direction presence survives opposing keys that cancel the vector.
        let movement_held = intent.movement_held || direction != Vec2::ZERO;
        if std::mem::take(&mut intent.movement_released) || !movement_held {
            self.sprint_latched = false;
        }
        let shift_released = std::mem::take(&mut intent.shift_released);
        if shift_released || dodge {
            self.sprint_hold_time = 0.0;
        }
        if intent.shift_held {
            self.sprint_hold_time += dt;
            if self.sprint_hold_time >= SPRINT_HOLD_TIME && movement_held {
                self.sprint_latched = true;
            }
        } else {
            self.sprint_hold_time = 0.0;
        }

        if let Some(leap) = &mut self.leap {
            leap.elapsed += dt;
            // Flight ends only on a real landing; the committed direction is bounded
            // to its requested travel time, then horizontal momentum is released.
            return if leap.elapsed <= leap.duration {
                leap.velocity.xz() * dt
            } else {
                Vec2::ZERO
            };
        }

        if self.is_grounded() && !self.is_dodging() {
            if crouch_toggle {
                self.crouched = !self.crouched;
                self.slide = if self.crouched
                    && self.sprint_latched
                    && !intent.block
                    && direction != Vec2::ZERO
                {
                    Some(Slide {
                        elapsed: 0.0,
                        direction: direction.normalize_or_zero(),
                    })
                } else {
                    None
                };
            }
            if dodge && self.dodge_cooldown == 0.0 {
                self.exit_crouch();
                self.dodge = Some(Dodge {
                    elapsed: 0.0,
                    direction: if direction.length_squared() > 0.0001 {
                        direction.normalize_or_zero()
                    } else {
                        -facing
                    },
                    held: false,
                });
                self.dodge_cooldown = DODGE_COOLDOWN;
            }
        }

        // act's second jump replaces falling speed; only landing refills the pair.
        if jump && !self.is_dodging() && self.jumps_used < 2 {
            self.exit_crouch();
            self.jumps_used += 1;
            self.vertical_velocity = JUMP_VELOCITY;
            self.pending_vertical_velocity =
                Some(JUMP_VELOCITY + self.support.map_or(0.0, |support| support.velocity.y));
            self.support = None;
        }

        if let Some(dodge) = &mut self.dodge {
            if intent.shift_held && self.sprint_hold_time >= SPRINT_HOLD_TIME {
                dodge.held = true;
            }
            let previous = dodge.elapsed;
            dodge.elapsed += dt;
            self.dodge_protected =
                previous < DODGE_PROTECTION_END && dodge.elapsed >= DODGE_PROTECTION_START;
            let delta =
                dodge.direction * (dodge_distance(dodge.elapsed) - dodge_distance(previous));
            if dodge.elapsed >= dodge.duration() {
                self.dodge = None;
            }
            delta
        } else if let Some(slide) = &mut self.slide {
            let previous = slide.elapsed;
            slide.elapsed = (slide.elapsed + dt).min(SLIDE_DURATION);
            // Integrate the reference's linearly decaying velocity over this tick.
            let distance = SLIDE_SPEED
                * ((slide.elapsed - previous)
                    - (slide.elapsed * slide.elapsed - previous * previous)
                        / (2.0 * SLIDE_DURATION));
            let delta = slide.direction * distance;
            if slide.elapsed >= SLIDE_DURATION {
                self.slide = None;
            }
            delta
        } else {
            self.blocking = intent.block && self.is_grounded();
            self.sprinting = self.sprint_latched
                && self.is_grounded()
                && !self.blocking
                && !self.crouched
                && direction != Vec2::ZERO;
            let move_speed = if self.crouched {
                if self.blocking {
                    CROUCH_SPEED.min(speed * BLOCK_MULTIPLIER)
                } else {
                    CROUCH_SPEED
                }
            } else if self.blocking {
                speed * BLOCK_MULTIPLIER
            } else if self.sprinting {
                speed * SPRINT_MULTIPLIER
            } else {
                speed
            };
            direction * move_speed * dt
        }
    }
}
