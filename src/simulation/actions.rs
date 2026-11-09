//! Fixed-step action state. Device input only submits held states and latched requests.

use bevy::prelude::*;

const GRAVITY: f32 = 18.0;
const JUMP_VELOCITY: f32 = 6.0;
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
    velocity: Vec2,
    elapsed: f32,
    duration: f32,
    peak: f32,
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

#[derive(Component, Default)]
pub struct MotionState {
    previous_height: f32,
    height: f32,
    vertical_velocity: f32,
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
        self.height == 0.0 && self.vertical_velocity <= 0.0 && self.leap.is_none()
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

    pub fn start_leap(&mut self, origin: Vec2, target: Vec2, duration: f32, peak: f32) -> bool {
        if !self.is_grounded()
            || self.is_dodging()
            || !origin.is_finite()
            || !target.is_finite()
            || !duration.is_finite()
            || duration <= 0.0
            || !peak.is_finite()
            || peak <= 0.0
        {
            return false;
        }
        let velocity = (target - origin) / duration;
        if !velocity.is_finite() || !(4.0 * peak / duration).is_finite() {
            return false;
        }
        self.blocking = false;
        self.sprinting = false;
        self.exit_crouch();
        self.leap = Some(Leap {
            velocity,
            elapsed: 0.0,
            duration,
            peak,
        });
        true
    }

    pub fn launch_up(&mut self, velocity: f32) -> bool {
        if !velocity.is_finite() || velocity <= 0.0 {
            return false;
        }
        self.vertical_velocity = self.vertical_velocity.max(velocity);
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
        self.previous_height = self.height;
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
            let elapsed = (leap.elapsed + dt).min(leap.duration);
            let delta = leap.velocity * (elapsed - leap.elapsed);
            leap.elapsed = elapsed;
            let progress = elapsed / leap.duration;
            self.height = 4.0 * leap.peak * progress * (1.0 - progress);
            self.vertical_velocity = 4.0 * leap.peak * (1.0 - 2.0 * progress) / leap.duration;
            if elapsed >= leap.duration {
                self.leap = None;
                self.height = 0.0;
                self.vertical_velocity = 0.0;
            }
            return delta;
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
            } else if jump {
                self.exit_crouch();
                self.vertical_velocity = JUMP_VELOCITY;
            }
        }

        let delta = if let Some(dodge) = &mut self.dodge {
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
        };

        if !self.is_grounded() {
            self.height += self.vertical_velocity * dt - 0.5 * GRAVITY * dt * dt;
            self.vertical_velocity -= GRAVITY * dt;
            if self.height <= 0.0 {
                self.height = 0.0;
                self.vertical_velocity = 0.0;
            }
        }
        delta
    }
}
