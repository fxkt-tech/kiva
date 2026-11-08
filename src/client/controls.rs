//! FPS-style local input. Mouse displacement is an angle, not an angular speed.
use crate::{gameplay::Player, simulation::MoveIntent};
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::{CursorGrabMode, WindowFocused};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};

const LOOK_SENSITIVITY: f32 = 0.0025;

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
}

#[cfg(not(target_arch = "wasm32"))]
impl CaptureState {
    fn update(&mut self, focused: bool, lost_focus: bool, escape: bool, clicked: bool) {
        if !focused || lost_focus || escape {
            self.active = false;
        } else if clicked {
            self.active = true;
        }
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
    mouse: (Res<ButtonInput<MouseButton>>, Res<AccumulatedMouseMotion>),
    mut window: Single<(Entity, &Window, &mut CursorOptions), With<PrimaryWindow>>,
    #[cfg(not(target_arch = "wasm32"))] mut focus_events: MessageReader<WindowFocused>,
    mut capture: ResMut<CaptureState>,
    mut yaw: ResMut<ViewYaw>,
    mut players: Query<&mut MoveIntent, With<Player>>,
) {
    let (buttons, motion) = mouse;
    #[cfg(target_arch = "wasm32")]
    let (active, look, axis) = {
        let _ = (&buttons, &motion, &mut window, &mut capture);
        let keys = super::web::keys();
        (
            super::web::active(),
            super::web::take_look(),
            movement_axis([keys & 1 != 0, keys & 2 != 0, keys & 4 != 0, keys & 8 != 0]),
        )
    };
    #[cfg(not(target_arch = "wasm32"))]
    let (active, look, axis) = {
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
        (
            capture.active,
            if was_active { motion.delta.x } else { 0.0 },
            movement_axis(
                [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD]
                    .map(|key| keys.pressed(key)),
            ),
        )
    };
    if !active {
        keys.reset_all();
        for mut intent in &mut players {
            intent.stop();
        }
        return;
    }
    yaw.turn(look);
    for mut intent in &mut players {
        intent.set(movement_direction(axis, yaw.yaw()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
