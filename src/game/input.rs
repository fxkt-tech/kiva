#[cfg(not(target_arch = "wasm32"))]
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Resource, Default)]
pub(super) struct Controls {
    pub active: bool,
    pub just_entered: bool,
    pub movement: Vec2,
    pub look_delta: Vec2,
    pub zoom: f32,
    pub attack: bool,
    pub guard: bool,
    pub reset: bool,
    pub training_attack: bool,
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
    controls.attack = receiving && mouse.just_pressed(MouseButton::Left);
    controls.guard = receiving && mouse.pressed(MouseButton::Right);
    controls.reset = receiving && keys.just_pressed(KeyCode::KeyR);
    controls.jump = receiving && keys.just_pressed(KeyCode::Space);
    controls.dodge = receiving && keys.just_pressed(KeyCode::ShiftLeft);
    controls.training_attack = receiving && keys.just_pressed(KeyCode::KeyT);
}
