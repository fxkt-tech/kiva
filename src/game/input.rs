#[cfg(not(target_arch = "wasm32"))]
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Clone, Copy, Default)]
pub(super) struct ButtonFrame {
    pub pressed: bool,
    pub held: bool,
    pub released: bool,
}

#[derive(Resource, Default)]
pub(super) struct Controls {
    pub active: bool,
    pub just_entered: bool,
    pub movement: Vec2,
    pub movement_held: bool,
    pub look_delta: Vec2,
    pub zoom: f32,
    pub attacks: [ButtonFrame; 2],
    pub parry: bool,
    pub crouch_pressed: bool,
    /// Physical Shift state; combat latches sprint after a long press.
    pub sprint: bool,
    pub walk: bool,
    pub free_look: bool,
    pub reset: bool,
    pub training_attack: bool,
    pub training_normal: bool,
    pub training_parry: bool,
    pub jump: bool,
    pub dodge: bool,
}

pub(super) fn collect_input(
    mut controls: ResMut<Controls>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    #[cfg(not(target_arch = "wasm32"))] mut cursor: Single<&mut CursorOptions>,
    #[cfg(not(target_arch = "wasm32"))] window: Single<&Window>,
) {
    let previously_active = controls.active;
    #[cfg(target_arch = "wasm32")]
    {
        controls.active = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.pointer_lock_element())
            .is_some_and(|element| element.id() == "game");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if keys.just_pressed(KeyCode::Escape) || !window.focused {
            controls.active = false;
        } else if mouse.just_pressed(MouseButton::Left) {
            controls.active = true;
        }
        cursor.grab_mode = if controls.active {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !controls.active;
    }
    if previously_active && !controls.active {
        keys.reset_all();
        mouse.reset_all();
    }
    controls.just_entered = controls.active && !previously_active;
    let receiving = controls.active && !controls.just_entered;
    controls.look_delta = if receiving { motion.delta } else { Vec2::ZERO };
    controls.zoom = wheel.read().map(|event| event.y).sum();
    if !receiving {
        controls.zoom = 0.0;
    }
    controls.movement = Vec2::new(
        (keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) as i32
            - keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) as i32) as f32,
        (keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) as i32
            - keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) as i32) as f32,
    )
    .normalize_or_zero();
    if !receiving {
        controls.movement = Vec2::ZERO;
    }
    controls.movement_held = receiving
        && keys.any_pressed([
            KeyCode::KeyW,
            KeyCode::KeyA,
            KeyCode::KeyS,
            KeyCode::KeyD,
            KeyCode::ArrowUp,
            KeyCode::ArrowLeft,
            KeyCode::ArrowDown,
            KeyCode::ArrowRight,
        ]);
    controls.attacks = [MouseButton::Left, MouseButton::Right].map(|button| ButtonFrame {
        pressed: receiving && mouse.just_pressed(button),
        held: receiving && mouse.pressed(button),
        released: receiving && mouse.just_released(button),
    });
    controls.parry = receiving
        && (keys.just_pressed(KeyCode::KeyG)
            || (controls.attacks.iter().all(|button| button.held)
                && controls.attacks.iter().any(|button| button.pressed)));
    controls.crouch_pressed = receiving && keys.just_pressed(KeyCode::KeyC);
    controls.sprint = receiving && keys.pressed(KeyCode::ShiftLeft);
    controls.walk = receiving && keys.pressed(KeyCode::ControlLeft);
    controls.free_look = receiving && keys.pressed(KeyCode::AltLeft);
    controls.reset = receiving && keys.just_pressed(KeyCode::F8);
    controls.jump = receiving && keys.just_pressed(KeyCode::Space);
    controls.dodge = receiving && keys.just_pressed(KeyCode::ShiftLeft);
    controls.training_attack = receiving && keys.just_pressed(KeyCode::F6);
    controls.training_normal = receiving && keys.just_pressed(KeyCode::F7);
    controls.training_parry = receiving && keys.just_pressed(KeyCode::F5);
}
