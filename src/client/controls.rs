//! FPS-style local input. Mouse displacement is an angle, not an angular speed.
use crate::{
    gameplay::Player,
    simulation::{
        ActionIntent, Body, CombatIntent, Combatant, Facing, MotionState, MoveIntent,
        SimulationActive, WeaponMode, WeaponSet,
    },
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::input::{ButtonState, keyboard::KeyboardInput};
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::{CursorGrabMode, WindowFocused};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};

const LOOK_SENSITIVITY: f32 = 0.0025;
const SPRINT: u32 = 16;
const BLOCK: u32 = 32;
const JUMP: u32 = 1;
const DODGE: u32 = 2;
const ATTACK: u32 = 4;
const SKILL: u32 = 8;
const COMBO: u32 = 16;
const SWITCH_WEAPON: u32 = 32;
const HAMMER_SNIPER: u32 = 64;
const AXE_CANNON: u32 = 128;
const CYCLE_TARGET: u32 = 256;
const CROUCH_TOGGLE: u32 = 512;

type PlayerIntents = (
    &'static Body,
    &'static mut MoveIntent,
    &'static mut ActionIntent,
    &'static mut Facing,
    &'static mut CombatIntent,
    &'static Combatant,
    &'static MotionState,
);

#[derive(Resource)]
pub(super) struct AimDistance(f32);
impl Default for AimDistance {
    fn default() -> Self {
        Self(6.0)
    }
}
impl AimDistance {
    fn adjust(&mut self, vertical_pixels: f32) {
        if vertical_pixels.is_finite() {
            self.0 = (self.0 - vertical_pixels * 0.012).clamp(1.0, 8.0);
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct ViewYaw(f32);
impl ViewYaw {
    pub fn yaw(&self) -> f32 {
        self.0
    }
    fn turn(&mut self, pixels: f32) {
        if pixels.is_finite() {
            self.0 = (self.0 - pixels * LOOK_SENSITIVITY + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct CaptureState {
    #[cfg(not(target_arch = "wasm32"))]
    active: bool,
    #[cfg(not(target_arch = "wasm32"))]
    last_wheel: Option<f64>,
    #[cfg(not(target_arch = "wasm32"))]
    shift_keys: [bool; 2],
    #[cfg(not(target_arch = "wasm32"))]
    crouch_key: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl CaptureState {
    fn update(&mut self, focused: bool, lost_focus: bool, escape: bool, clicked: bool) {
        if !focused || lost_focus || escape {
            self.active = false;
            self.last_wheel = None;
            self.shift_keys = [false; 2];
            self.crouch_key = false;
        } else if clicked {
            self.active = true;
        }
    }

    fn scroll(&mut self, delta: f32, now: f64) -> bool {
        if !delta.is_finite() || delta == 0.0 {
            return false;
        }
        let fresh = self.last_wheel.is_none_or(|last| now - last >= 0.16);
        self.last_wheel = Some(now);
        fresh
    }

    fn shift_event(&mut self, event: &KeyboardInput, window: Entity, receiving: bool) -> bool {
        if !self.accepts_event(event, window, receiving) {
            return false;
        }
        let index = match event.key_code {
            KeyCode::ShiftLeft => 0,
            KeyCode::ShiftRight => 1,
            _ => return false,
        };
        let previously_held = self.shift_keys.iter().any(|held| *held);
        self.shift_keys[index] = event.state == ButtonState::Pressed;
        event.state == ButtonState::Pressed && !previously_held
    }

    fn crouch_event(&mut self, event: &KeyboardInput, window: Entity, receiving: bool) -> bool {
        if !self.accepts_event(event, window, receiving) || event.key_code != KeyCode::KeyC {
            return false;
        }
        let pressed = event.state == ButtonState::Pressed;
        let toggle = pressed && !self.crouch_key;
        self.crouch_key = pressed;
        toggle
    }

    fn accepts_event(&self, event: &KeyboardInput, window: Entity, receiving: bool) -> bool {
        receiving && self.active && event.window == window && !event.repeat
    }
}

fn movement_axis([forward, back, left, right]: [bool; 4]) -> Vec2 {
    Vec2::new(
        f32::from(right) - f32::from(left),
        f32::from(forward) - f32::from(back),
    )
}

pub(super) fn movement_direction(axis: Vec2, yaw: f32) -> Vec2 {
    let rotation = Quat::from_rotation_y(yaw);
    let forward = rotation * Vec3::NEG_Z;
    let right = rotation * Vec3::X;
    (right * axis.x + forward * axis.y)
        .xz()
        .clamp_length_max(1.0)
}

pub(super) fn sample(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mouse: (
        ResMut<ButtonInput<MouseButton>>,
        Res<AccumulatedMouseMotion>,
        Res<AccumulatedMouseScroll>,
    ),
    mut window: Single<(Entity, &Window, &mut CursorOptions), With<PrimaryWindow>>,
    #[cfg(not(target_arch = "wasm32"))] native_events: (
        MessageReader<WindowFocused>,
        MessageReader<KeyboardInput>,
    ),
    capture: (
        ResMut<CaptureState>,
        Res<Time<Real>>,
        ResMut<SimulationActive>,
        ResMut<AimDistance>,
    ),
    mut yaw: ResMut<ViewYaw>,
    mut players: Query<PlayerIntents, With<Player>>,
) {
    let (mut buttons, motion, scroll) = mouse;
    let (mut capture, time, mut simulation_active, mut aim_distance) = capture;
    #[cfg(target_arch = "wasm32")]
    let (active, look, aim_delta, held, commands) = {
        let _ = (&buttons, &motion, &scroll, &mut window, &mut capture, &time);
        (
            super::web::active(),
            super::web::take_look(),
            super::web::take_aim(),
            super::web::keys(),
            super::web::take_actions(),
        )
    };
    #[cfg(not(target_arch = "wasm32"))]
    let (active, look, aim_delta, held, commands) = {
        let (mut focus_events, mut keyboard_events) = native_events;
        let was_active = capture.active;
        let mut lost_focus = false;
        for event in focus_events.read() {
            if event.window == window.0 && !event.focused {
                lost_focus = true;
            }
        }
        capture.update(
            window.1.focused,
            lost_focus,
            keys.just_pressed(KeyCode::Escape),
            buttons.just_pressed(MouseButton::Left),
        );
        window.2.visible = !capture.active;
        window.2.grab_mode = if capture.active {
            if cfg!(target_os = "windows") {
                CursorGrabMode::Confined
            } else {
                CursorGrabMode::Locked
            }
        } else {
            CursorGrabMode::None
        };
        // The click that captures the pointer is never also an attack, and held
        // state from outside the game must require a fresh press after capture.
        if !was_active {
            keys.reset_all();
            buttons.reset_all();
            capture.last_wheel = None;
            capture.shift_keys = [false; 2];
            capture.crouch_key = false;
        }
        let mut held = 0;
        let mut commands = 0;
        let receiving = was_active && capture.active;
        // Preserve every edge: release+repress can happen between two rendered
        // frames even when ButtonInput ends both frames in the held state.
        for event in keyboard_events.read() {
            if capture.shift_event(event, window.0, receiving) {
                commands |= DODGE;
            }
            if capture.crouch_event(event, window.0, receiving) {
                commands ^= CROUCH_TOGGLE;
            }
        }
        if was_active && capture.active {
            for (key, bit) in [
                (KeyCode::KeyW, 1),
                (KeyCode::KeyS, 2),
                (KeyCode::KeyA, 4),
                (KeyCode::KeyD, 8),
                (KeyCode::ShiftLeft, SPRINT),
                (KeyCode::ShiftRight, SPRINT),
            ] {
                if keys.pressed(key) {
                    held |= bit;
                }
            }
            if buttons.pressed(MouseButton::Right) {
                held |= BLOCK;
            }
            for (key, bit) in [
                (KeyCode::Space, JUMP),
                (KeyCode::KeyE, SKILL),
                (KeyCode::Tab, COMBO),
                (KeyCode::Digit1, HAMMER_SNIPER),
                (KeyCode::Digit2, AXE_CANNON),
                (KeyCode::KeyQ, CYCLE_TARGET),
            ] {
                if keys.just_pressed(key) {
                    commands |= bit;
                }
            }
            if buttons.just_pressed(MouseButton::Left) {
                commands |= ATTACK;
            }
            if capture.scroll(scroll.delta.y, time.elapsed_secs_f64()) {
                commands |= SWITCH_WEAPON;
            }
        }
        (
            capture.active,
            if was_active { motion.delta.x } else { 0.0 },
            if was_active { motion.delta.y } else { 0.0 },
            held,
            commands,
        )
    };
    simulation_active.0 = active;
    if !active {
        keys.reset_all();
        buttons.reset_all();
        for (_, mut intent, mut actions, _, mut combat, _, _) in &mut players {
            intent.stop();
            actions.clear();
            combat.clear();
        }
        return;
    }
    yaw.turn(look);
    let axis = movement_axis([held & 1 != 0, held & 2 != 0, held & 4 != 0, held & 8 != 0]);
    let forward = movement_direction(Vec2::Y, yaw.yaw());
    for (body, mut intent, mut actions, mut facing, mut combat, loadout, motion) in &mut players {
        intent.set(movement_direction(axis, yaw.yaw()));
        facing.set(forward);
        actions.set_held(held & SPRINT != 0, held & BLOCK != 0);
        actions.set_movement_held(held & 15 != 0);
        if loadout.weapon_set() == WeaponSet::AxeCannon
            && loadout.weapon_mode() == WeaponMode::Ranged
            && !motion.is_leaping()
        {
            aim_distance.adjust(aim_delta);
        }
        combat.set_aim(body.position() + forward * aim_distance.0);
        if commands & JUMP != 0 {
            actions.request_jump();
        }
        if commands & DODGE != 0 {
            actions.request_dodge();
        }
        if commands & CROUCH_TOGGLE != 0 {
            actions.request_crouch_toggle();
        }
        if commands & ATTACK != 0 {
            combat.request_attack();
        }
        if commands & SKILL != 0 {
            combat.request_skill();
        }
        if commands & COMBO != 0 {
            combat.request_combo();
        }
        if commands & SWITCH_WEAPON != 0 {
            combat.request_switch_weapon();
        }
        if commands & HAMMER_SNIPER != 0 {
            combat.request_set(WeaponSet::HammerSniper);
        }
        if commands & AXE_CANNON != 0 {
            combat.request_set(WeaponSet::AxeCannon);
        }
        if commands & CYCLE_TARGET != 0 {
            combat.request_cycle_target();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aiming_distance_stays_within_cast_range_and_rejects_invalid_motion() {
        let mut distance = AimDistance::default();
        distance.adjust(100.0);
        assert!(distance.0 < 6.0);
        distance.adjust(f32::NAN);
        distance.adjust(f32::INFINITY);
        assert!(distance.0.is_finite());
        distance.adjust(100_000.0);
        assert_eq!(distance.0, 1.0);
        distance.adjust(-100_000.0);
        assert_eq!(distance.0, 8.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn transient_focus_loss_requires_a_new_click_even_when_final_window_is_focused() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        assert!(capture.active);
        capture.update(true, true, false, false);
        assert!(!capture.active);
        capture.update(true, false, false, false);
        assert!(!capture.active);
        capture.update(true, false, false, true);
        assert!(capture.active);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn wheel_inertia_has_one_switch_until_quiet_and_capture_loss_resets_it() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        assert!(capture.scroll(1.0, 0.0));
        assert!(!capture.scroll(1.0, 0.05));
        assert!(!capture.scroll(-0.5, 0.1));
        assert!(capture.scroll(-1.0, 0.3));
        assert!(!capture.scroll(f32::NAN, 1.0));
        assert!(!capture.scroll(0.0, 1.0));
        capture.update(false, true, false, false);
        capture.update(true, false, false, true);
        assert!(capture.scroll(1.0, 0.31));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_shift_repress_between_frames_preserves_the_new_edge() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        let press = key_event(KeyCode::ShiftLeft, ButtonState::Pressed);
        let release = key_event(KeyCode::ShiftLeft, ButtonState::Released);
        assert!(capture.shift_event(&press, Entity::PLACEHOLDER, true));
        // Previous render ended held; next render receives release+press but
        // its final ButtonInput snapshot would also be held.
        assert!(!capture.shift_event(&release, Entity::PLACEHOLDER, true));
        assert!(capture.shift_event(&press, Entity::PLACEHOLDER, true));
        assert!(!capture.shift_event(&press, Entity::PLACEHOLDER, true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_shift_handoff_requires_all_shift_keys_released_before_another_dodge() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        let left_down = key_event(KeyCode::ShiftLeft, ButtonState::Pressed);
        let right_down = key_event(KeyCode::ShiftRight, ButtonState::Pressed);
        let left_up = key_event(KeyCode::ShiftLeft, ButtonState::Released);
        let right_up = key_event(KeyCode::ShiftRight, ButtonState::Released);
        assert!(capture.shift_event(&left_down, Entity::PLACEHOLDER, true));
        assert!(!capture.shift_event(&right_down, Entity::PLACEHOLDER, true));
        assert!(!capture.shift_event(&left_up, Entity::PLACEHOLDER, true));
        assert!(!capture.shift_event(&left_down, Entity::PLACEHOLDER, true));
        assert!(!capture.shift_event(&right_up, Entity::PLACEHOLDER, true));
        assert!(!capture.shift_event(&left_up, Entity::PLACEHOLDER, true));
        assert!(capture.shift_event(&right_down, Entity::PLACEHOLDER, true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_shift_short_tap_is_preserved_even_when_final_snapshot_is_released() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        let press = key_event(KeyCode::ShiftLeft, ButtonState::Pressed);
        let release = key_event(KeyCode::ShiftLeft, ButtonState::Released);
        let mut queued = false;
        for event in [&press, &release] {
            queued |= capture.shift_event(event, Entity::PLACEHOLDER, true);
        }
        assert!(queued);
        assert_eq!(capture.shift_keys, [false; 2]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_shift_ignores_inactive_capture_frame_repeats_and_other_windows() {
        let mut capture = CaptureState::default();
        let mut press = key_event(KeyCode::ShiftLeft, ButtonState::Pressed);
        assert!(!capture.shift_event(&press, Entity::PLACEHOLDER, true));
        capture.update(true, false, false, true);
        assert!(!capture.shift_event(&press, Entity::PLACEHOLDER, false));
        let other_window = World::new().spawn_empty().id();
        assert!(!capture.shift_event(&press, other_window, true));
        press.repeat = true;
        assert!(!capture.shift_event(&press, Entity::PLACEHOLDER, true));
        assert_eq!(capture.shift_keys, [false; 2]);
        press.repeat = false;
        assert!(capture.shift_event(&press, Entity::PLACEHOLDER, true));
        capture.update(false, true, false, false);
        assert_eq!(capture.shift_keys, [false; 2]);
        capture.update(true, false, false, true);
        assert!(!capture.shift_event(&press, Entity::PLACEHOLDER, false));
        assert!(capture.shift_event(&press, Entity::PLACEHOLDER, true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_crouch_short_tap_keeps_one_toggle_while_release_and_repeats_do_not_toggle() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        let mut press = key_event(KeyCode::KeyC, ButtonState::Pressed);
        let release = key_event(KeyCode::KeyC, ButtonState::Released);
        assert!(capture.crouch_event(&press, Entity::PLACEHOLDER, true));
        assert!(!capture.crouch_event(&press, Entity::PLACEHOLDER, true));
        press.repeat = true;
        assert!(!capture.crouch_event(&press, Entity::PLACEHOLDER, true));
        assert!(!capture.crouch_event(&release, Entity::PLACEHOLDER, true));
        assert!(!capture.crouch_key);
        press.repeat = false;
        assert!(capture.crouch_event(&press, Entity::PLACEHOLDER, true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_crouch_two_complete_taps_cancel_without_changing_a_queued_dodge() {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        let press = key_event(KeyCode::KeyC, ButtonState::Pressed);
        let release = key_event(KeyCode::KeyC, ButtonState::Released);
        let mut commands = DODGE;
        for event in [&press, &release, &press, &release] {
            if capture.crouch_event(event, Entity::PLACEHOLDER, true) {
                commands ^= CROUCH_TOGGLE;
            }
        }
        assert_eq!(commands, DODGE);
        assert!(!capture.crouch_key);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_crouch_clears_on_focus_loss_and_rejects_capture_frame_or_other_window() {
        let mut capture = CaptureState::default();
        let mut press = key_event(KeyCode::KeyC, ButtonState::Pressed);
        assert!(!capture.crouch_event(&press, Entity::PLACEHOLDER, true));
        capture.update(true, false, false, true);
        assert!(!capture.crouch_event(&press, Entity::PLACEHOLDER, false));
        let other_window = World::new().spawn_empty().id();
        assert!(!capture.crouch_event(&press, other_window, true));
        assert!(capture.crouch_event(&press, Entity::PLACEHOLDER, true));
        capture.update(false, true, false, false);
        assert!(!capture.crouch_key);
        capture.update(true, false, false, true);
        press.repeat = true;
        assert!(!capture.crouch_event(&press, Entity::PLACEHOLDER, true));
        assert!(!capture.crouch_key);
        press.repeat = false;
        assert!(capture.crouch_event(&press, Entity::PLACEHOLDER, true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn key_event(key_code: KeyCode, state: ButtonState) -> KeyboardInput {
        KeyboardInput {
            key_code,
            logical_key: if key_code == KeyCode::KeyC {
                bevy::input::keyboard::Key::Character("c".into())
            } else {
                bevy::input::keyboard::Key::Shift
            },
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    #[test]
    fn wasd_strafes_relative_to_the_view_and_diagonals_do_not_accelerate() {
        assert!(movement_direction(Vec2::Y, 0.0).distance(Vec2::NEG_Y) < 0.001);
        assert!(movement_direction(Vec2::X, 0.0).distance(Vec2::X) < 0.001);
        assert!(
            movement_direction(Vec2::Y, -std::f32::consts::FRAC_PI_2).distance(Vec2::X) < 0.001
        );
        assert!(
            movement_direction(Vec2::X, -std::f32::consts::FRAC_PI_2).distance(Vec2::Y) < 0.001
        );
        assert!((movement_direction(Vec2::ONE, 1.2).length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn rightward_mouse_turns_right_without_a_frame_time_multiplier() {
        let mut one = ViewYaw::default();
        one.turn(160.0);
        let mut several = ViewYaw::default();
        for _ in 0..16 {
            several.turn(10.0);
        }
        assert!(movement_direction(Vec2::Y, one.yaw()).x > 0.0);
        assert!((one.yaw() - several.yaw()).abs() < 0.0001);
        let before = one.yaw();
        one.turn(f32::NAN);
        assert_eq!(one.yaw(), before);
    }
}
