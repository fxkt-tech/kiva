//! Local input records device edges; fixed simulation owns action timing.
use crate::{
    gameplay::Player,
    simulation::{
        ActionIntent, Body, CombatIntent, Combatant, DefenseEdge, DefenseState, Enemy, Facing,
        MoveIntent, SimulationActive,
    },
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::input::{ButtonState, keyboard::KeyboardInput, mouse::MouseButtonInput};
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::{CursorGrabMode, WindowFocused};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;

const LOOK_SENSITIVITY: f32 = 0.0025;
const SPRINT: u32 = 16;
const JUMP: u32 = 1;
const DODGE: u32 = 2;
const ATTACK: u32 = 4;
const MELEE_SKILL: u32 = 8;
const COMBO: u32 = 16;
const TOGGLE_TARGET: u32 = 32;
const NEXT_PAIR: u32 = 64;
const RAGE: u32 = 128;
const RANGED_SKILL: u32 = 256;
const CROUCH_TOGGLE: u32 = 512;
const MOVEMENT_RELEASED: u32 = 1024;
const SHIFT_RELEASED: u32 = 2048;
#[cfg(not(target_arch = "wasm32"))]
const MAX_DEFENSE_EDGES: usize = 64;
/// While a target is locked the view turns to keep it straight ahead, closing
/// the gap with this time constant; movement is then relative to the target,
/// so sideways keys circle it.
const LOCK_TURN_TIME: f32 = 0.08;
/// Closer than this the bearing to the target is too unstable to follow.
const LOCK_MIN_DISTANCE: f32 = 0.6;

type PlayerIntents = (
    &'static Body,
    &'static mut MoveIntent,
    &'static mut ActionIntent,
    &'static mut Facing,
    &'static mut CombatIntent,
    &'static Combatant,
    &'static mut DefenseState,
);

/// Enemy bodies a locked view turns toward.
type LockTargets<'w, 's> = Query<'w, 's, &'static Body, (With<Enemy>, Without<Player>)>;

#[derive(Resource, Default)]
pub(super) struct ViewYaw(f32);
impl ViewYaw {
    pub fn yaw(&self) -> f32 {
        self.0
    }
    /// Turn toward the world planar `direction`, closing `share` of the gap.
    fn face(&mut self, direction: Vec2, share: f32) {
        if direction.length_squared() < 1e-6 || !share.is_finite() {
            return;
        }
        let wanted = (-direction.x).atan2(-direction.y);
        let gap = (wanted - self.0 + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        self.0 = (self.0 + gap * share.clamp(0.0, 1.0) + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
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
    cancellation_epoch: u64,
    #[cfg(not(target_arch = "wasm32"))]
    active: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pressed: HashSet<KeyCode>,
    #[cfg(not(target_arch = "wasm32"))]
    mouse_pressed: HashSet<MouseButton>,
    #[cfg(not(target_arch = "wasm32"))]
    defense_edges: Vec<DefenseEdge>,
    #[cfg(not(target_arch = "wasm32"))]
    right_blocked: bool,
}

impl CaptureState {
    pub(super) fn cancellation_epoch(&self) -> u64 {
        self.cancellation_epoch
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CaptureState {
    fn clear_inputs(&mut self) {
        self.pressed.clear();
        self.mouse_pressed.clear();
        self.defense_edges.clear();
        self.defense_edges.push(DefenseEdge::Cancelled);
        self.right_blocked = false;
    }

    fn update(&mut self, focused: bool, lost_focus: bool, escape: bool, clicked: bool) {
        if !focused || lost_focus || escape {
            self.active = false;
            self.clear_inputs();
        } else if clicked && !self.active {
            self.clear_inputs();
            self.active = true;
        }
    }

    fn held(&self) -> u32 {
        let mut held = 0;
        for (key, bit) in [
            (KeyCode::KeyW, 1),
            (KeyCode::KeyS, 2),
            (KeyCode::KeyA, 4),
            (KeyCode::KeyD, 8),
            (KeyCode::ShiftLeft, SPRINT),
            (KeyCode::ShiftRight, SPRINT),
        ] {
            if self.pressed.contains(&key) {
                held |= bit;
            }
        }
        held
    }

    fn keyboard_event(&mut self, event: &KeyboardInput, window: Entity, receiving: bool) -> u32 {
        if !receiving || !self.active || event.window != window || event.repeat {
            return 0;
        }
        if event.state == ButtonState::Released {
            let previously_held = self.held();
            self.pressed.remove(&event.key_code);
            let held = self.held();
            let mut released = 0;
            if previously_held & 15 != 0 && held & 15 == 0 {
                released |= MOVEMENT_RELEASED;
            }
            if previously_held & SPRINT != 0 && held & SPRINT == 0 {
                released |= SHIFT_RELEASED;
            }
            return released;
        }
        let shift_held = self.held() & SPRINT != 0;
        if !self.pressed.insert(event.key_code) {
            return 0;
        }
        match event.key_code {
            KeyCode::ShiftLeft | KeyCode::ShiftRight if !shift_held => DODGE,
            KeyCode::Space => JUMP,
            KeyCode::KeyE => MELEE_SKILL,
            KeyCode::KeyQ => RANGED_SKILL,
            KeyCode::Tab => COMBO,
            KeyCode::KeyZ => NEXT_PAIR,
            KeyCode::KeyX => RAGE,
            KeyCode::KeyC => CROUCH_TOGGLE,
            _ => 0,
        }
    }

    fn queue_defense(&mut self, edge: DefenseEdge) {
        if self.defense_edges.len() >= MAX_DEFENSE_EDGES {
            self.defense_edges.clear();
            self.defense_edges.push(DefenseEdge::Cancelled);
            self.right_blocked = self.mouse_pressed.contains(&MouseButton::Right);
        } else {
            self.defense_edges.push(edge);
        }
    }

    fn mouse_event(&mut self, event: &MouseButtonInput, window: Entity, receiving: bool) -> u32 {
        if !receiving || !self.active || event.window != window {
            return 0;
        }
        if event.state == ButtonState::Released {
            if self.mouse_pressed.remove(&event.button) && event.button == MouseButton::Right {
                if self.right_blocked {
                    self.right_blocked = false;
                } else {
                    self.queue_defense(DefenseEdge::Released);
                }
            }
            return 0;
        }
        if !self.mouse_pressed.insert(event.button) {
            return 0;
        }
        match event.button {
            MouseButton::Left => ATTACK,
            MouseButton::Middle => TOGGLE_TARGET,
            MouseButton::Right if !self.right_blocked => {
                self.queue_defense(DefenseEdge::Pressed);
                0
            }
            _ => 0,
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
    mouse: (
        ResMut<ButtonInput<MouseButton>>,
        Res<AccumulatedMouseMotion>,
    ),
    mut window: Single<(Entity, &Window, &mut CursorOptions), With<PrimaryWindow>>,
    #[cfg(not(target_arch = "wasm32"))] native_events: (
        MessageReader<WindowFocused>,
        MessageReader<KeyboardInput>,
        MessageReader<MouseButtonInput>,
    ),
    capture: (
        ResMut<CaptureState>,
        ResMut<SimulationActive>,
        Res<Time>,
        LockTargets,
    ),
    mut yaw: ResMut<ViewYaw>,
    mut players: Query<PlayerIntents, With<Player>>,
) {
    let (mut buttons, motion) = mouse;
    let (mut capture, mut simulation_active, time, enemies) = capture;
    #[cfg(target_arch = "wasm32")]
    let (active, look, held, commands, defense_edges) = {
        let _ = (&buttons, &motion, &mut window);
        (
            super::web::active(),
            super::web::take_look(),
            super::web::keys(),
            super::web::take_actions(),
            super::web::take_defense_edges()
                .into_iter()
                .filter_map(|edge| match edge {
                    1 => Some(DefenseEdge::Pressed),
                    2 => Some(DefenseEdge::Released),
                    3 => Some(DefenseEdge::Cancelled),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )
    };
    #[cfg(not(target_arch = "wasm32"))]
    let (active, look, held, commands, defense_edges) = {
        let (mut focus_events, mut keyboard_events, mut mouse_events) = native_events;
        let was_active = capture.active;
        let mut lost_focus = false;
        for event in focus_events.read() {
            lost_focus |= event.window == window.0 && !event.focused;
        }
        let keyboard_events = keyboard_events.read().collect::<Vec<_>>();
        let mouse_events = mouse_events.read().collect::<Vec<_>>();
        capture.update(
            window.1.focused,
            lost_focus,
            keyboard_events.iter().any(|event| {
                event.window == window.0
                    && event.key_code == KeyCode::Escape
                    && event.state == ButtonState::Pressed
            }),
            mouse_events.iter().any(|event| {
                event.window == window.0
                    && event.button == MouseButton::Left
                    && event.state == ButtonState::Pressed
            }),
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
        // Capture clicks and all inputs from that frame are consumed without play.
        if !was_active {
            keys.reset_all();
            buttons.reset_all();
        }
        let receiving = was_active && capture.active;
        let mut commands = 0;
        for event in keyboard_events {
            let command = capture.keyboard_event(event, window.0, receiving);
            if command == CROUCH_TOGGLE {
                commands ^= CROUCH_TOGGLE;
            } else {
                commands |= command;
            }
        }
        for event in mouse_events {
            let command = capture.mouse_event(event, window.0, receiving);
            if command == TOGGLE_TARGET {
                commands ^= TOGGLE_TARGET;
            } else {
                commands |= command;
            }
        }
        (
            capture.active,
            if receiving { motion.delta.x } else { 0.0 },
            capture.held(),
            commands,
            std::mem::take(&mut capture.defense_edges),
        )
    };
    // Keep cancellation observable even when blur and recapture coalesce into
    // an active sample. Audio may still have a future authoritative cue.
    if defense_edges.contains(&DefenseEdge::Cancelled) {
        capture.cancellation_epoch += 1;
    }
    simulation_active.0 = active;
    if !active {
        keys.reset_all();
        buttons.reset_all();
        for (_, mut intent, mut actions, _, mut combat, _, mut defense) in &mut players {
            intent.stop();
            actions.clear();
            combat.clear();
            defense.clear();
        }
        return;
    }
    // Both combo cameras discard look samples and retain the pre-combo yaw.
    // A locked target owns the view: it turns to keep the target ahead, which
    // makes the camera orbit it as the player moves.
    if !players
        .iter()
        .any(|(_, _, _, _, _, loadout, _)| loadout.is_combo_active())
    {
        let bearing = players.iter().find_map(|(body, _, _, _, _, loadout, _)| {
            let target = enemies.get(loadout.target()?).ok()?;
            let offset = (target.world_position() - body.world_position()).xz();
            (offset.length() >= LOCK_MIN_DISTANCE).then_some(offset)
        });
        match bearing {
            Some(offset) => yaw.face(offset, 1.0 - (-time.delta_secs() / LOCK_TURN_TIME).exp()),
            None => yaw.turn(look),
        }
    }
    let axis = movement_axis([held & 1 != 0, held & 2 != 0, held & 4 != 0, held & 8 != 0]);
    let forward = movement_direction(Vec2::Y, yaw.yaw());
    for (body, mut intent, mut actions, mut facing, mut combat, loadout, mut defense) in
        &mut players
    {
        intent.set(movement_direction(axis, yaw.yaw()));
        facing.set(loadout.sequence().map_or(forward, |sequence| {
            (sequence.aim_position() - body.world_position()).xz()
        }));
        // A release followed by a new press between samples still ends the old
        // physical hold; Intent keeps that fact until a real fixed tick.
        if commands & SHIFT_RELEASED != 0 {
            actions.set_shift_held(false);
        }
        if commands & MOVEMENT_RELEASED != 0 {
            actions.set_movement_held(false);
        }
        actions.set_shift_held(held & SPRINT != 0);
        actions.set_movement_held(held & 15 != 0);
        for edge in &defense_edges {
            // A blur/capture cancellation can arrive in a frame which has
            // already regained focus. Latch it until the next fixed tick.
            if *edge == DefenseEdge::Cancelled {
                combat.cancel_melee_chain();
            }
            defense.push_edge(*edge);
        }
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
        if commands & MELEE_SKILL != 0 {
            combat.request_melee_skill();
        }
        if commands & RANGED_SKILL != 0 {
            combat.request_ranged_skill();
        }
        if commands & COMBO != 0 {
            combat.request_combo();
        }
        if commands & NEXT_PAIR != 0 {
            combat.request_next_pair();
        }
        if commands & RAGE != 0 {
            combat.request_rage();
        }
        if commands & TOGGLE_TARGET != 0 {
            combat.request_toggle_target();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    fn captured() -> CaptureState {
        let mut capture = CaptureState::default();
        capture.update(true, false, false, true);
        capture.defense_edges.clear();
        capture
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn key(key_code: KeyCode, state: ButtonState) -> KeyboardInput {
        KeyboardInput {
            key_code,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn mouse(button: MouseButton, state: ButtonState) -> MouseButtonInput {
        MouseButtonInput {
            button,
            state,
            window: Entity::PLACEHOLDER,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_skill_keydowns_are_distinct_and_hold_repeat_release_do_not_attack() {
        let mut capture = captured();
        for (code, command) in [
            (KeyCode::KeyE, MELEE_SKILL),
            (KeyCode::KeyQ, RANGED_SKILL),
            (KeyCode::KeyZ, NEXT_PAIR),
            (KeyCode::KeyX, RAGE),
        ] {
            let mut press = key(code, ButtonState::Pressed);
            assert_eq!(capture.keyboard_event(&press, press.window, true), command);
            assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
            press.repeat = true;
            assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
            let release = key(code, ButtonState::Released);
            assert_eq!(capture.keyboard_event(&release, release.window, true), 0);
            press.repeat = false;
            assert_eq!(capture.keyboard_event(&press, press.window, true), command);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_keyboard_events_preserve_short_shift_taps_repress_and_handoff() {
        let mut capture = captured();
        let left_down = key(KeyCode::ShiftLeft, ButtonState::Pressed);
        let right_down = key(KeyCode::ShiftRight, ButtonState::Pressed);
        let left_up = key(KeyCode::ShiftLeft, ButtonState::Released);
        let right_up = key(KeyCode::ShiftRight, ButtonState::Released);
        assert_eq!(
            capture.keyboard_event(&left_down, left_down.window, true),
            DODGE
        );
        assert_eq!(
            capture.keyboard_event(&right_down, right_down.window, true),
            0
        );
        assert_eq!(capture.keyboard_event(&left_up, left_up.window, true), 0);
        assert_eq!(capture.held(), SPRINT);
        assert_eq!(
            capture.keyboard_event(&left_down, left_down.window, true),
            0
        );
        assert_eq!(capture.keyboard_event(&left_up, left_up.window, true), 0);
        assert_eq!(
            capture.keyboard_event(&right_up, right_up.window, true),
            SHIFT_RELEASED
        );
        assert_eq!(capture.held(), 0);
        assert_eq!(
            capture.keyboard_event(&left_down, left_down.window, true),
            DODGE
        );
        assert_eq!(
            capture.keyboard_event(&left_up, left_up.window, true),
            SHIFT_RELEASED
        );
        assert_eq!(
            capture.keyboard_event(&left_down, left_down.window, true),
            DODGE
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_crouch_taps_keep_parity_without_changing_other_queued_actions() {
        let mut capture = captured();
        let mut commands = JUMP;
        for _ in 0..2 {
            let mut press = key(KeyCode::KeyC, ButtonState::Pressed);
            commands ^= capture.keyboard_event(&press, press.window, true);
            assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
            press.repeat = true;
            assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
            let release = key(KeyCode::KeyC, ButtonState::Released);
            assert_eq!(capture.keyboard_event(&release, release.window, true), 0);
        }
        assert_eq!(commands, JUMP);
        let press = key(KeyCode::KeyC, ButtonState::Pressed);
        commands ^= capture.keyboard_event(&press, press.window, true);
        assert_eq!(commands, JUMP | CROUCH_TOGGLE);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_movement_uses_physical_held_keys_even_when_their_axes_cancel() {
        let mut capture = captured();
        for code in [KeyCode::KeyW, KeyCode::KeyS] {
            let press = key(code, ButtonState::Pressed);
            capture.keyboard_event(&press, press.window, true);
        }
        assert_eq!(capture.held(), 3);
        assert_eq!(movement_axis([true, true, false, false]), Vec2::ZERO);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_mouse_taps_preserve_defense_order_and_deduplicate_left_and_middle() {
        let mut capture = captured();
        for (button, command) in [
            (MouseButton::Left, ATTACK),
            (MouseButton::Middle, TOGGLE_TARGET),
        ] {
            let press = mouse(button, ButtonState::Pressed);
            assert_eq!(capture.mouse_event(&press, press.window, true), command);
            assert_eq!(capture.mouse_event(&press, press.window, true), 0);
            let release = mouse(button, ButtonState::Released);
            assert_eq!(capture.mouse_event(&release, release.window, true), 0);
            assert_eq!(capture.mouse_event(&press, press.window, true), command);
        }
        let press = mouse(MouseButton::Right, ButtonState::Pressed);
        let release = mouse(MouseButton::Right, ButtonState::Released);
        for _ in 0..2 {
            assert_eq!(capture.mouse_event(&press, press.window, true), 0);
            assert_eq!(capture.mouse_event(&press, press.window, true), 0);
            assert_eq!(capture.mouse_event(&release, release.window, true), 0);
            assert_eq!(capture.mouse_event(&release, release.window, true), 0);
        }
        assert_eq!(
            std::mem::take(&mut capture.defense_edges),
            [
                DefenseEdge::Pressed,
                DefenseEdge::Released,
                DefenseEdge::Pressed,
                DefenseEdge::Released,
            ]
        );
        assert!(capture.defense_edges.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_defense_overflow_cancels_and_requires_release_before_another_press() {
        let mut capture = captured();
        let press = mouse(MouseButton::Right, ButtonState::Pressed);
        let release = mouse(MouseButton::Right, ButtonState::Released);
        for _ in 0..32 {
            capture.mouse_event(&press, press.window, true);
            capture.mouse_event(&release, release.window, true);
        }
        assert_eq!(capture.defense_edges.len(), MAX_DEFENSE_EDGES);
        capture.mouse_event(&press, press.window, true);
        capture.mouse_event(&press, press.window, true);
        assert_eq!(
            std::mem::take(&mut capture.defense_edges),
            [DefenseEdge::Cancelled]
        );
        capture.mouse_event(&release, release.window, true);
        assert!(capture.defense_edges.is_empty());
        capture.mouse_event(&press, press.window, true);
        capture.mouse_event(&release, release.window, true);
        assert_eq!(
            capture.defense_edges,
            [DefenseEdge::Pressed, DefenseEdge::Released]
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_focus_loss_and_escape_cancel_instead_of_releasing_and_require_a_capture_click() {
        for escape in [false, true] {
            let mut capture = captured();
            let press = mouse(MouseButton::Right, ButtonState::Pressed);
            let release = mouse(MouseButton::Right, ButtonState::Released);
            capture.mouse_event(&press, press.window, true);
            // Final Window.focused may already be true; the intermediate event matters.
            capture.update(true, !escape, escape, true);
            assert!(!capture.active);
            assert_eq!(capture.defense_edges, [DefenseEdge::Cancelled]);
            assert_eq!(capture.mouse_event(&release, release.window, true), 0);
            capture.update(true, false, false, false);
            assert!(!capture.active);
            capture.update(true, false, false, true);
            assert!(capture.active);
            assert_eq!(capture.mouse_event(&release, release.window, true), 0);
            assert_eq!(capture.defense_edges, [DefenseEdge::Cancelled]);
            capture.mouse_event(&press, press.window, true);
            assert_eq!(
                capture.defense_edges,
                [DefenseEdge::Cancelled, DefenseEdge::Pressed]
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_capture_frame_other_windows_and_refocus_repeats_cannot_create_inputs() {
        let mut capture = CaptureState::default();
        let mut press = key(KeyCode::KeyQ, ButtonState::Pressed);
        let mouse_press = mouse(MouseButton::Right, ButtonState::Pressed);
        assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
        assert_eq!(
            capture.mouse_event(&mouse_press, mouse_press.window, true),
            0
        );
        capture.update(true, false, false, true);
        assert_eq!(capture.keyboard_event(&press, press.window, false), 0);
        assert_eq!(
            capture.mouse_event(&mouse_press, mouse_press.window, false),
            0
        );
        let other = World::new().spawn_empty().id();
        assert_eq!(capture.keyboard_event(&press, other, true), 0);
        assert_eq!(capture.mouse_event(&mouse_press, other, true), 0);
        assert!(capture.pressed.is_empty());
        assert!(capture.mouse_pressed.is_empty());
        press.repeat = true;
        assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
        assert!(capture.pressed.is_empty());
        press.repeat = false;
        assert_eq!(
            capture.keyboard_event(&press, press.window, true),
            RANGED_SKILL
        );
        capture.update(false, true, false, false);
        assert!(capture.pressed.is_empty());
        capture.update(true, false, false, true);
        press.repeat = true;
        assert_eq!(capture.keyboard_event(&press, press.window, true), 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn input_app() -> (App, Entity, Entity) {
        use crate::simulation::SimulationPlugin;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .add_message::<WindowFocused>()
            .add_message::<KeyboardInput>()
            .add_message::<MouseButtonInput>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<CaptureState>()
            .init_resource::<ViewYaw>()
            .add_systems(Update, sample);
        app.finish();
        crate::gameplay::setup(app.world_mut());
        crate::gameplay::manual_targeting(app.world_mut());
        let window = app
            .world_mut()
            .spawn((Window::default(), CursorOptions::default(), PrimaryWindow))
            .id();
        let actor = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        fixed(&mut app, 30);
        // The capture click is deliberately sent through the real sample system.
        send_mouse(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        send_mouse(&mut app, window, MouseButton::Left, ButtonState::Released);
        app.world_mut().run_schedule(Update);
        (app, actor, window)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fixed(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn send_mouse(app: &mut App, window: Entity, button: MouseButton, state: ButtonState) {
        let mut event = mouse(button, state);
        event.window = window;
        app.world_mut().write_message(event);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn send_key(app: &mut App, window: Entity, code: KeyCode, state: ButtonState) {
        let mut event = key(code, state);
        event.window = window;
        app.world_mut().write_message(event);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sample_preserves_a_tap_through_zero_fixed_frames_and_consumes_it_once() {
        let (mut app, actor, window) = input_app();
        send_mouse(&mut app, window, MouseButton::Right, ButtonState::Pressed);
        send_mouse(&mut app, window, MouseButton::Right, ButtonState::Released);
        app.world_mut().run_schedule(Update);
        for _ in 0..3 {
            app.world_mut().run_schedule(Update);
        }
        assert!(!app.world().get::<DefenseState>(actor).unwrap().is_active());
        fixed(&mut app, 1);
        assert!(
            app.world()
                .get::<DefenseState>(actor)
                .unwrap()
                .is_deflecting()
        );
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 20);
        let defense = app.world().get::<DefenseState>(actor).unwrap();
        assert!(!defense.is_active());
        assert!(defense.deflect_cooldown() > 0.0);
        fixed(&mut app, 25);
        assert!(!app.world().get::<DefenseState>(actor).unwrap().is_active());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sample_focus_loss_cancels_guard_without_a_release_parry_after_recapture() {
        let (mut app, actor, window) = input_app();
        send_mouse(&mut app, window, MouseButton::Right, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 14);
        assert!(
            app.world()
                .get::<DefenseState>(actor)
                .unwrap()
                .is_guarding()
        );
        app.world_mut().write_message(WindowFocused {
            window,
            focused: false,
        });
        app.world_mut().write_message(WindowFocused {
            window,
            focused: true,
        });
        send_mouse(&mut app, window, MouseButton::Right, ButtonState::Released);
        app.world_mut().run_schedule(Update);
        assert!(!app.world().resource::<SimulationActive>().0);
        fixed(&mut app, 3);
        send_mouse(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 1);
        let defense = app.world().get::<DefenseState>(actor).unwrap();
        assert!(!defense.is_active());
        assert_eq!(defense.parry_cooldown(), 0.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn frame(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().run_schedule(Update);
        fixed(app, 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn shipped_input_app() -> (App, Entity, Entity) {
        let (mut app, actor, window) = input_app();
        // `input_app` keeps manual targeting for the toggle tests; the
        // shipped player starts automatic, as gameplay spawns it.
        let mut entity = app.world_mut().entity_mut(actor);
        let combat = entity.take::<Combatant>().unwrap();
        entity.insert(Combatant::new(combat.weapon_pair()));
        (app, actor, window)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn bearing(app: &App, from: Entity, to: Entity) -> Vec2 {
        let body = |entity| app.world().get::<Body>(entity).unwrap().world_position();
        (body(to) - body(from)).xz()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn view_forward(app: &App) -> Vec2 {
        movement_direction(Vec2::Y, app.world().resource::<ViewYaw>().yaw())
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_shipped_player_targets_automatically_and_the_middle_button_unlocks_and_relocks() {
        let (mut app, actor, window) = shipped_input_app();
        frame(&mut app);
        let dummy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .single(app.world())
            .unwrap();
        fn combat(app: &App, actor: Entity) -> &Combatant {
            app.world().get::<Combatant>(actor).unwrap()
        }
        assert_eq!(combat(&app, actor).target(), Some(dummy));
        assert!(combat(&app, actor).auto_target());
        // Middle button: unlock, and stay unlocked.
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Pressed);
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Released);
        frame(&mut app);
        for _ in 0..30 {
            frame(&mut app);
        }
        assert_eq!(combat(&app, actor).target(), None);
        assert!(!combat(&app, actor).auto_target());
        // Middle again: lock, and target automatically from then on.
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Pressed);
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Released);
        frame(&mut app);
        assert_eq!(combat(&app, actor).target(), Some(dummy));
        assert!(combat(&app, actor).auto_target());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_locked_view_keeps_the_target_ahead_and_sideways_keys_circle_it() {
        let (mut app, actor, window) = shipped_input_app();
        let dummy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .single(app.world())
            .unwrap();
        // The target is off to the side; the view turns to it within a few
        // frames even though the mouse never moved.
        crate::simulation::teleport_actor(app.world_mut(), dummy, Vec3::new(5.0, 0.0, -2.0));
        for _ in 0..30 {
            frame(&mut app);
        }
        assert_eq!(
            app.world().get::<Combatant>(actor).unwrap().target(),
            Some(dummy)
        );
        let ahead = view_forward(&app).dot(bearing(&app, actor, dummy).normalize());
        assert!(ahead > 0.999, "the target is not straight ahead: {ahead}");
        // Mouse look does not steal the view from the lock.
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(400.0, 0.0);
        frame(&mut app);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::ZERO;
        let ahead = view_forward(&app).dot(bearing(&app, actor, dummy).normalize());
        assert!(ahead > 0.99, "mouse look turned a locked view: {ahead}");
        // Holding D circles the target: the bearing sweeps round while the
        // distance stays about the same and the target stays ahead.
        let start = bearing(&app, actor, dummy);
        send_key(&mut app, window, KeyCode::KeyD, ButtonState::Pressed);
        let mut swept = 0.0;
        let mut previous = start;
        for _ in 0..90 {
            frame(&mut app);
            let now = bearing(&app, actor, dummy);
            swept += previous.angle_to(now);
            previous = now;
            let ahead = view_forward(&app).dot(now.normalize());
            assert!(ahead > 0.97, "the target drifted out of view: {ahead}");
        }
        send_key(&mut app, window, KeyCode::KeyD, ButtonState::Released);
        frame(&mut app);
        assert!(
            swept.abs() > 1.0,
            "the player did not circle the target: {swept} rad"
        );
        let (from, to) = (start.length(), previous.length());
        assert!(
            (to - from).abs() < from * 0.35,
            "circling changed the distance from {from} to {to}"
        );
        // Unlocked, the mouse turns the view again.
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Pressed);
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Released);
        frame(&mut app);
        let before = app.world().resource::<ViewYaw>().yaw();
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(100.0, 0.0);
        frame(&mut app);
        assert!((app.world().resource::<ViewYaw>().yaw() - before).abs() > 0.1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sample_middle_toggles_lock_and_focus_clear_preserves_the_committed_q_action() {
        use crate::simulation::ActionKind;
        let (mut app, actor, window) = input_app();
        assert!(
            app.world()
                .get::<Combatant>(actor)
                .unwrap()
                .target()
                .is_none()
        );
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Pressed);
        send_mouse(&mut app, window, MouseButton::Middle, ButtonState::Released);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 1);
        let target = app.world().get::<Combatant>(actor).unwrap().target();
        assert!(target.is_some());
        send_key(&mut app, window, KeyCode::KeyQ, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 1);
        let before = app
            .world()
            .get::<Combatant>(actor)
            .unwrap()
            .active_action()
            .unwrap();
        assert_eq!(before.kind, ActionKind::MusketShot);
        assert_eq!(
            app.world().get::<Combatant>(actor).unwrap().target(),
            target
        );
        app.world_mut().write_message(WindowFocused {
            window,
            focused: false,
        });
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 5);
        let paused = app
            .world()
            .get::<Combatant>(actor)
            .unwrap()
            .active_action()
            .unwrap();
        assert_eq!(paused.action_id, before.action_id);
        assert_eq!(paused.elapsed, before.elapsed);
        send_mouse(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 1);
        let resumed = app
            .world()
            .get::<Combatant>(actor)
            .unwrap()
            .active_action()
            .unwrap();
        assert_eq!(resumed.action_id, before.action_id);
        assert!(resumed.elapsed > before.elapsed);
        // No left attack from the click used to resume capture.
        assert_eq!(resumed.kind, ActionKind::MusketShot);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_direction_release_fact_requires_the_entire_physical_group_to_be_up() {
        let mut capture = captured();
        for code in [KeyCode::KeyW, KeyCode::KeyS] {
            let press = key(code, ButtonState::Pressed);
            capture.keyboard_event(&press, press.window, true);
        }
        let w_up = key(KeyCode::KeyW, ButtonState::Released);
        let s_up = key(KeyCode::KeyS, ButtonState::Released);
        assert_eq!(capture.keyboard_event(&w_up, w_up.window, true), 0);
        assert_eq!(
            capture.keyboard_event(&s_up, s_up.window, true),
            MOVEMENT_RELEASED
        );
        assert_eq!(capture.keyboard_event(&s_up, s_up.window, true), 0);
        let w_down = key(KeyCode::KeyW, ButtonState::Pressed);
        capture.keyboard_event(&w_down, w_down.window, true);
        assert_eq!(capture.held(), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sample_direction_release_and_repress_end_sprint_even_with_zero_fixed_frames() {
        use crate::simulation::MotionState;
        let (mut app, actor, window) = input_app();
        send_key(&mut app, window, KeyCode::KeyW, ButtonState::Pressed);
        send_key(&mut app, window, KeyCode::ShiftLeft, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 30);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        send_key(&mut app, window, KeyCode::ShiftLeft, ButtonState::Released);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 1);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        send_key(&mut app, window, KeyCode::KeyW, ButtonState::Released);
        send_key(&mut app, window, KeyCode::KeyW, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        // The final device snapshot is unchanged and more render frames may have no tick.
        assert_eq!(app.world().resource::<CaptureState>().held(), 1);
        app.world_mut().run_schedule(Update);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        fixed(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(!motion.sprint_latched());
        assert!(!motion.is_sprinting());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_sample_shift_release_and_repress_restart_hold_when_new_dodges_are_rejected() {
        use crate::simulation::{ActionKind, MotionState};
        let (mut app, actor, window) = input_app();
        // A committed melee action rejects new dodge commands, so only the
        // physical Shift release can reset continuous-hold timing in this case.
        send_mouse(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 1);
        assert_eq!(
            app.world()
                .get::<Combatant>(actor)
                .unwrap()
                .active_action()
                .unwrap()
                .kind,
            ActionKind::Melee
        );
        send_key(&mut app, window, KeyCode::KeyW, ButtonState::Pressed);
        send_key(&mut app, window, KeyCode::ShiftLeft, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 6);
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_dodging());
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        send_key(&mut app, window, KeyCode::ShiftLeft, ButtonState::Released);
        send_key(&mut app, window, KeyCode::ShiftLeft, ButtonState::Pressed);
        app.world_mut().run_schedule(Update);
        assert_eq!(app.world().resource::<CaptureState>().held(), 1 | SPRINT);
        app.world_mut().run_schedule(Update);
        fixed(&mut app, 4);
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        fixed(&mut app, 6);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_dodging());
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn sampled_focus_cancellation_clears_buffered_melee_but_ordinary_active_frames_preserve_it() {
        use crate::simulation::{CombatEffect, CombatFeedback, Combatant, MeleeStage};
        // The third case represents a blur/capture cancellation coalesced with
        // recapture in one browser sample: final active=true, Cancelled retained.
        for mode in 0..3 {
            let (mut app, actor, window) = input_app();
            for _ in 0..2 {
                send_mouse(&mut app, window, MouseButton::Left, ButtonState::Pressed);
                send_mouse(&mut app, window, MouseButton::Left, ButtonState::Released);
                app.world_mut().run_schedule(Update);
                fixed(&mut app, 1);
            }
            assert_eq!(
                app.world()
                    .get::<Combatant>(actor)
                    .unwrap()
                    .active_action()
                    .unwrap()
                    .melee_stage,
                Some(MeleeStage::One)
            );
            if mode == 1 {
                app.world_mut().write_message(WindowFocused {
                    window,
                    focused: false,
                });
                app.world_mut().write_message(WindowFocused {
                    window,
                    focused: true,
                });
            } else if mode == 2 {
                app.world_mut()
                    .resource_mut::<CaptureState>()
                    .queue_defense(DefenseEdge::Cancelled);
            }
            // Cancellation must survive any number of display frames with no
            // FixedUpdate, just like a buffered click must survive normal frames.
            for _ in 0..3 {
                app.world_mut().run_schedule(Update);
            }
            if mode == 1 {
                assert!(!app.world().resource::<SimulationActive>().0);
                fixed(&mut app, 5);
                send_mouse(&mut app, window, MouseButton::Left, ButtonState::Pressed);
                app.world_mut().run_schedule(Update);
                assert!(app.world().resource::<SimulationActive>().0);
            }
            fixed(&mut app, 40);
            let stages: Vec<_> = app
                .world()
                .resource::<CombatFeedback>()
                .events()
                .filter(|event| event.actor == actor && event.kind == CombatEffect::MeleeStart)
                .map(|event| event.melee_stage.unwrap())
                .collect();
            if mode == 0 {
                assert_eq!(stages, [MeleeStage::One, MeleeStage::Two]);
                assert_eq!(
                    app.world()
                        .get::<Combatant>(actor)
                        .unwrap()
                        .active_action()
                        .unwrap()
                        .melee_stage,
                    Some(MeleeStage::Two)
                );
            } else {
                assert_eq!(stages, [MeleeStage::One]);
                assert!(
                    app.world()
                        .get::<Combatant>(actor)
                        .unwrap()
                        .active_action()
                        .is_none()
                );
            }
        }
    }
}
