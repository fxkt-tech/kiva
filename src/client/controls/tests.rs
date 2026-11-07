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
        .init_resource::<ClientFeedback>()
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

#[test]
fn picking_uses_the_displayed_body_transform_and_local_offset() {
    let (mut app, _) = app();
    let player = app.world().resource::<PlayerController>().player;
    app.world_mut().entity_mut(player).insert((
        GlobalTransform::from(Transform::from_xyz(4.0, 3.0, 5.0)),
        Pickable {
            offset: Vec3::Y,
            half_size: Vec3::splat(0.4),
        },
    ));
    let visible_body = Ray3d::new(Vec3::new(4.0, 4.0, 15.0), Dir3::NEG_Z);
    assert_eq!(hovered_unit(app.world_mut(), visible_body), Some(player));
    let old_ground_body = Ray3d::new(Vec3::new(4.0, 1.0, 15.0), Dir3::NEG_Z);
    assert_eq!(hovered_unit(app.world_mut(), old_ground_body), None);
}

#[test]
fn enemy_skill_keeps_aim_on_empty_ground_or_friendly_body_without_submitting() {
    let (mut app, _) = app();
    let player = app.world().resource::<PlayerController>().player;
    select_ability(app.world_mut(), 0);
    let mut events = bevy::ecs::message::MessageCursor::<SimulationEvent>::default();
    for hovered in [None, Some(player)] {
        *app.world_mut().resource_mut::<PointerState>() = PointerState {
            ground: Some(Vec2::new(1.0, 6.0)),
            hovered,
            ..default()
        };
        mouse_pressed(app.world_mut(), MouseButton::Left);
        assert!(matches!(
            app.world().resource::<PlayerController>().aim,
            Some(Aim::Ability(0))
        ));
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(
            events
                .read(app.world().resource::<Messages<SimulationEvent>>())
                .count(),
            0,
            "an invalid aim click must not enqueue even a rejected cast"
        );
    }
}

fn control_caster(app: &mut App, ability: AbilityId) -> Entity {
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Control test caster".into(),
            team: Team::Enemy,
            position: Vec2::new(1.0, 6.0),
            stats: UnitStats::default(),
            abilities: [Some(ability), None, None, None],
            respawn_delay: None,
        },
    )
    .expect("valid control caster")
}

#[test]
fn a_real_stun_clears_existing_aim_without_replaying_the_cast() {
    let (mut app, _) = app();
    let player = app.world().resource::<PlayerController>().player;
    let caster = control_caster(&mut app, AbilityId(5));
    select_ability(app.world_mut(), 0);
    submit(
        app.world_mut(),
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(player),
        },
    );
    for _ in 0..20 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(!app.world().get::<ControlState>(player).unwrap().can_cast());
    refresh_aim(app.world_mut());
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    select_ability(app.world_mut(), 0);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    for _ in 0..60 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(app.world().get::<ControlState>(player).unwrap().can_cast());
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().cooldown(0),
        Some(0.0),
        "the interrupted aim never becomes a deferred cast"
    );
    assert!(app.world().resource::<PlayerController>().aim.is_none());
}

#[test]
fn root_keeps_nonmovement_aim_but_rejects_selecting_a_dash() {
    let (mut app, _) = app();
    let player = app.world().resource::<PlayerController>().player;
    let caster = control_caster(&mut app, AbilityId(6));
    submit(
        app.world_mut(),
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Point(Vec2::new(0.0, 6.0)),
        },
    );
    for _ in 0..60 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let control = app.world().get::<ControlState>(player).unwrap();
    assert!(control.can_cast());
    assert!(!control.can_dash());
    select_ability(app.world_mut(), 0);
    refresh_aim(app.world_mut());
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(0))
    ));
    select_ability(app.world_mut(), 2);
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(0))
    ));
    assert!(
        app.world()
            .resource::<ClientFeedback>()
            .message
            .contains("位移")
    );
}
