//! Adapter regressions use real ordered window messages, without a GPU/window runner.
use super::*;
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    window::WindowEvent,
};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin))
        .add_message::<WindowEvent>()
        .init_resource::<PointerState>()
        .init_resource::<GamePointer>()
        .add_systems(Update, collect_input);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    super::super::content::setup(app.world_mut());
    (app, window)
}

fn press(app: &mut App, window: Entity, key: KeyCode) {
    app.world_mut()
        .write_message(WindowEvent::KeyboardInput(KeyboardInput {
            key_code: key,
            logical_key: Key::Character("test".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        }));
}

#[test]
fn stop_then_skill_in_one_frame_preserves_the_later_skill_selection() {
    let (mut app, window) = app();
    press(&mut app, window, KeyCode::KeyS);
    press(&mut app, window, KeyCode::KeyQ);
    app.update();
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(0))
    ));
}

#[test]
fn skill_then_escape_in_one_frame_cancels_aim_without_pausing() {
    let (mut app, window) = app();
    press(&mut app, window, KeyCode::KeyQ);
    press(&mut app, window, KeyCode::Escape);
    app.update();
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    assert!(!app.world().resource::<SimulationControl>().paused);
}

#[test]
fn selecting_two_skills_in_one_frame_keeps_the_last_selection() {
    let (mut app, window) = app();
    press(&mut app, window, KeyCode::KeyQ);
    press(&mut app, window, KeyCode::KeyW);
    app.update();
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(1))
    ));
}

#[test]
fn entering_pause_discards_later_game_inputs_from_the_same_frame() {
    let (mut app, window) = app();
    press(&mut app, window, KeyCode::Escape);
    press(&mut app, window, KeyCode::KeyQ);
    app.update();
    assert!(app.world().resource::<SimulationControl>().paused);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
}

#[test]
fn an_earlier_focus_event_cannot_override_explicit_resume() {
    use bevy::window::WindowFocused;
    let (mut app, window) = app();
    // The resume button has focused the canvas again. Both focus transitions
    // can still be present in the event batch, while Window reports final focus.
    app.world_mut()
        .write_message(WindowEvent::WindowFocused(WindowFocused {
            window,
            focused: false,
        }));
    app.world_mut()
        .write_message(WindowEvent::WindowFocused(WindowFocused {
            window,
            focused: true,
        }));
    press(&mut app, window, KeyCode::KeyQ);
    app.update();
    assert!(!app.world().resource::<SimulationControl>().paused);
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(0))
    ));
}
