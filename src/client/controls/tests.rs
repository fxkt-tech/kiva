//! Adapter regressions use real ordered window messages, without a GPU/window runner.
use super::*;
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    window::WindowEvent,
};

fn selecting_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        SimulationPlugin,
        crate::gameplay::GameplayPlugin,
    ))
    .add_message::<WindowEvent>()
    .init_resource::<PointerState>()
    .init_resource::<GamePointer>()
    .init_resource::<ClientFeedback>()
    .init_resource::<PlayerController>()
    .add_systems(Update, collect_input);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    super::super::content::setup(app.world_mut());
    super::super::test_support::generic_control_roster(app.world_mut());
    super::super::session::setup(app.world_mut());
    (app, window)
}

fn app() -> (App, Entity) {
    let (mut app, window) = selecting_app();
    super::super::session::start_native(app.world_mut());
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
    let player = app.world().resource::<ClientSession>().player().unwrap();
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
    let player = app.world().resource::<ClientSession>().player().unwrap();
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

fn out_of_range_enemy_click(equipment: bool) {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let origin = app.world().get::<UnitPose>(player).unwrap().position();
    let position = origin + Vec2::X * 14.0;
    // A real allied observer reveals the distant enemy; the caster's own
    // vision is shorter than this fixture's targeted ability range.
    for (team, point) in [(Team::Ally, position + Vec2::Y), (Team::Enemy, position)] {
        spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Distant click fixture".into(),
                team,
                position: point,
                stats: UnitStats {
                    vision_radius: 3.0,
                    ..default()
                },
                abilities: [None; 4],
                respawn_delay: None,
            },
        )
        .unwrap();
    }
    let enemy = app
        .world_mut()
        .query::<(Entity, &Unit, &UnitPose)>()
        .iter(app.world())
        .find(|(_, unit, pose)| unit.team() == Team::Enemy && pose.position() == position)
        .map(|(entity, _, _)| entity)
        .unwrap();
    let source = if equipment {
        let instance = equip_equipment(app.world_mut(), player, 0, EquipmentId(7)).unwrap();
        CastSource::EquipmentSlot { slot: 0, instance }
    } else {
        CastSource::AbilitySlot(0)
    };
    assert_eq!(
        preview_cast(app.world(), player, source, AbilityTarget::Unit(enemy)),
        Err(Rejection::OutOfRange),
        "AI's immediate preflight stays strict"
    );
    select_cast(app.world_mut(), source);
    *app.world_mut().resource_mut::<PointerState>() = PointerState {
        ground: Some(position),
        hovered: Some(enemy),
        ..default()
    };
    mouse_pressed(app.world_mut(), MouseButton::Left);
    assert!(
        app.world().resource::<PlayerController>().aim.is_none(),
        "a valid distant enemy click must submit its intent and leave aim"
    );
    app.world_mut().run_schedule(FixedUpdate);
    assert!(
        app.world()
            .get::<UnitPose>(player)
            .unwrap()
            .position()
            .distance(origin)
            > 0.0,
        "the submitted targeted ability must start walking into range"
    );
}

#[test]
fn distant_enemy_click_submits_hero_skill_instead_of_remaining_in_aim() {
    out_of_range_enemy_click(false);
}

#[test]
fn distant_enemy_click_submits_equipment_skill_through_the_same_adapter() {
    out_of_range_enemy_click(true);
}

fn control_caster(app: &mut App, ability: AbilityId) -> Entity {
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let position = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X;
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Control test caster".into(),
            team: Team::Enemy,
            position,
            stats: UnitStats {
                vision_radius: 12.0,
                ..default()
            },
            abilities: [Some(ability), None, None, None],
            respawn_delay: None,
        },
    )
    .expect("valid control caster")
}

#[test]
fn a_real_stun_clears_existing_aim_without_replaying_the_cast() {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
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
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let caster = control_caster(&mut app, AbilityId(6));
    let position = app.world().get::<UnitPose>(player).unwrap().position();
    submit(
        app.world_mut(),
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Point(position),
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

#[test]
fn selecting_discards_game_input_and_npc_commands_and_cannot_be_unpaused() {
    let (mut app, window) = selecting_app();
    let npc = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Selecting command fixture".into(),
            team: Team::Enemy,
            position: Vec2::ZERO,
            stats: UnitStats::default(),
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    let positions = [(npc, Vec2::ZERO)];
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(npc, UnitCommand::MoveTo(Vec2::new(5.0, 5.0)));
    for key in [KeyCode::KeyQ, KeyCode::KeyA, KeyCode::Escape, KeyCode::KeyS] {
        press(&mut app, window, key);
    }
    set_paused(app.world_mut(), false);
    app.world_mut().run_schedule(Update);
    for _ in 0..20 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(app.world().resource::<SimulationControl>().paused);
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    assert!(app.world().resource::<ClientSession>().player().is_none());
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    for (id, position) in &positions {
        assert_eq!(
            app.world().get::<UnitPose>(*id).unwrap().position(),
            *position
        );
    }

    // Starting later must not replay any of the selection screen's input or
    // the NPC command that was queued before the no-player gate ran.
    super::super::session::start_native(app.world_mut());
    set_paused(app.world_mut(), false);
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(FixedUpdate);
    let player = app.world().resource::<ClientSession>().player().unwrap();
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    assert!(matches!(
        app.world().get::<UnitOrder>(player),
        Some(UnitOrder::Idle)
    ));
    assert!(matches!(
        app.world().get::<UnitOrder>(npc),
        Some(UnitOrder::Idle)
    ));
}

#[test]
fn pause_and_resume_keep_the_same_player_loadout_and_committed_cooldown() {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    submit(
        app.world_mut(),
        player,
        UnitCommand::CastAbility {
            slot: 2,
            target: AbilityTarget::Point(Vec2::new(1.0, 6.0)),
        },
    );
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let cooldown = app
        .world()
        .get::<AbilitySlots>(player)
        .unwrap()
        .cooldown(2)
        .unwrap();
    let mana = app.world().get::<UnitState>(player).unwrap().mana();
    let tick = app.world().resource::<SimulationTick>().0;
    set_paused(app.world_mut(), true);
    for _ in 0..15 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(app.world().resource::<SimulationTick>().0, tick);
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().cooldown(2),
        Some(cooldown)
    );
    assert_eq!(app.world().get::<UnitState>(player).unwrap().mana(), mana);
    set_paused(app.world_mut(), false);
    app.world_mut().run_schedule(FixedUpdate);
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(player)
    );
    assert_eq!(
        app.world().get::<CharacterIdentity>(player).unwrap().id(),
        CharacterId(1)
    );
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().ability(2),
        Some(AbilityId(3))
    );
    assert!(
        app.world()
            .get::<AbilitySlots>(player)
            .unwrap()
            .cooldown(2)
            .unwrap()
            < cooldown
    );
    assert_eq!(
        app.world_mut()
            .query::<&CharacterIdentity>()
            .iter(app.world())
            .count(),
        2
    );
}

#[test]
fn a_skill_queued_before_capture_cannot_arm_or_cast_after_launch_or_resume() {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    queued_skill_request(app.world_mut(), Some(0), true);
    queued_skill_request(app.world_mut(), Some(2), true);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    app.world_mut().run_schedule(FixedUpdate);
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().cooldown(2),
        Some(0.0)
    );
    assert_eq!(app.world().get::<UnitState>(player).unwrap().mana(), 300.0);
    set_paused(app.world_mut(), true);
    queued_skill_request(app.world_mut(), Some(0), false);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    set_paused(app.world_mut(), false);
    queued_skill_request(app.world_mut(), Some(0), true);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    queued_skill_request(app.world_mut(), Some(0), false);
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(0))
    ));
}

#[test]
fn delayed_capture_event_keeps_resume_paused_then_recenters_and_discards_queued_skill() {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let size = Vec2::new(1280.0, 800.0);
    let confirmed = Some(web::PointerCapture {
        actual: true,
        event_confirmed: true,
    });
    assert!(sync_pointer(app.world_mut(), confirmed, false, size, None).1);
    app.world_mut()
        .resource_mut::<GamePointer>()
        .observe(Vec2::new(840.0, 300.0));
    set_paused(app.world_mut(), true);
    // DOM pause can precede native unlock. It still ends this input session.
    assert!(!sync_pointer(app.world_mut(), confirmed, true, size, None).1);
    let before_event = Some(web::PointerCapture {
        actual: true,
        event_confirmed: false,
    });
    for _ in 0..4 {
        let (captured, just_captured) =
            sync_pointer(app.world_mut(), before_event, true, size, None);
        assert_eq!(captured, Some(false));
        assert!(
            !just_captured,
            "native acquisition cannot consume the event boundary"
        );
        queued_skill_request(app.world_mut(), Some(0), just_captured);
        assert!(app.world().resource::<PlayerController>().aim.is_none());
    }
    set_paused(app.world_mut(), false);
    let (captured, just_captured) = sync_pointer(app.world_mut(), confirmed, false, size, None);
    assert_eq!(captured, Some(true));
    assert!(
        just_captured,
        "the event's unpause must start a fresh input session"
    );
    assert_eq!(
        app.world().resource::<GamePointer>().position(),
        Some(size * 0.5)
    );
    queued_skill_request(app.world_mut(), Some(0), just_captured);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(player)
    );
    assert!(
        !sync_pointer(app.world_mut(), None, false, size, None).1,
        "native input keeps its absent-browser capture branch"
    );
}

#[test]
fn keyboard_and_mouse_item_selection_share_target_validation_and_commit_the_item_source() {
    let (mut app, window) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let instance = equip_equipment(app.world_mut(), player, 1, EquipmentId(7)).unwrap();
    let enemy = control_caster(&mut app, AbilityId(5));
    let health = app.world().get::<UnitState>(enemy).unwrap().health();
    press(&mut app, window, KeyCode::Digit2);
    app.update();
    assert!(matches!(app.world().resource::<PlayerController>().aim,
        Some(Aim::Equipment { slot:1, instance:id }) if id == instance));
    for hovered in [None, Some(player)] {
        *app.world_mut().resource_mut::<PointerState>() = PointerState {
            ground: Some(Vec2::new(1.0, 6.0)),
            hovered,
            ..default()
        };
        mouse_pressed(app.world_mut(), MouseButton::Left);
        assert!(matches!(
            app.world().resource::<PlayerController>().aim,
            Some(Aim::Equipment { slot: 1, .. })
        ));
    }
    app.world_mut().resource_mut::<PlayerController>().aim = None;
    let raw = serde_json::json!({"slot":1,"instance_id":instance.to_string()}).to_string();
    queued_equipment_request(app.world_mut(), Some(&raw), false);
    assert!(matches!(app.world().resource::<PlayerController>().aim,
        Some(Aim::Equipment { slot:1, instance:id }) if id == instance));
    app.world_mut().resource_mut::<PointerState>().hovered = Some(enemy);
    mouse_pressed(app.world_mut(), MouseButton::Left);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    for _ in 0..20 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(
        app.world()
            .get::<EquipmentSlots>(player)
            .unwrap()
            .item(1)
            .unwrap()
            .cooldown()
            > 0.0
    );
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().cooldown(0),
        Some(0.0)
    );
    assert!(app.world().get::<UnitState>(enemy).unwrap().health() < health);
}

#[test]
fn queued_item_requests_are_discarded_at_capture_pause_and_stale_instance_boundaries() {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let instance = equip_equipment(app.world_mut(), player, 0, EquipmentId(7)).unwrap();
    let raw = serde_json::json!({"slot":0,"instance_id":instance.to_string()}).to_string();
    queued_equipment_request(app.world_mut(), Some(&raw), true);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    queued_equipment_request(app.world_mut(), Some(&raw), false);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    queued_equipment_request(app.world_mut(), Some(&raw), false);
    assert!(app.world().resource::<PlayerController>().aim.is_some());
    unequip_equipment(app.world_mut(), player, 0, instance).unwrap();
    let replacement = equip_equipment(app.world_mut(), player, 0, EquipmentId(7)).unwrap();
    assert_ne!(instance, replacement);
    refresh_aim(app.world_mut());
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    queued_equipment_request(app.world_mut(), Some(&raw), false);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    assert!(
        app.world()
            .resource::<ClientFeedback>()
            .message
            .contains("已经变更")
    );
    app.world_mut().run_schedule(FixedUpdate);
    assert_eq!(
        app.world()
            .get::<EquipmentSlots>(player)
            .unwrap()
            .item(0)
            .unwrap()
            .cooldown(),
        0.0
    );
}

#[test]
fn a_self_cast_item_submits_immediately_and_consumes_its_own_cooldown() {
    let (mut app, window) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    equip_equipment(app.world_mut(), player, 5, EquipmentId(9)).unwrap();
    press(&mut app, window, KeyCode::Digit6);
    app.update();
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    for _ in 0..20 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(matches!(
        app.world().get::<UnitAction>(player),
        Some(UnitAction::Channeling { .. })
    ));
    assert!(
        app.world()
            .get::<EquipmentSlots>(player)
            .unwrap()
            .item(5)
            .unwrap()
            .cooldown()
            > 0.0
    );
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().cooldown(3),
        Some(0.0)
    );
}

#[test]
fn terminal_round_cannot_resume_or_accept_new_gameplay_input() {
    let (mut app, window) = app();
    super::super::test_support::finish_match(&mut app, Team::Ally);
    let tick = app.world().resource::<SimulationTick>().0;
    set_paused(app.world_mut(), false);
    press(&mut app, window, KeyCode::KeyQ);
    app.world_mut().run_schedule(Update);
    assert!(app.world().resource::<SimulationControl>().paused);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(app.world().resource::<SimulationTick>().0, tick);
}

#[test]
fn enemy_skill_and_item_keep_aim_when_clicking_a_structure() {
    let (mut app, _) = app();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let base = app
        .world()
        .resource::<crate::gameplay::MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let instance = equip_equipment(app.world_mut(), player, 0, EquipmentId(7)).unwrap();
    let base_position = app.world().get::<UnitPose>(base).unwrap().position();
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Visible structure observer".into(),
            team: Team::Ally,
            position: base_position + Vec2::X * 3.0,
            stats: UnitStats {
                vision_radius: 12.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    for aim in [Aim::Ability(0), Aim::Equipment { slot: 0, instance }] {
        app.world_mut().resource_mut::<PlayerController>().aim = Some(aim);
        *app.world_mut().resource_mut::<PointerState>() = PointerState {
            hovered: Some(base),
            ground: Some(Vec2::new(4.0, -12.0)),
            ..default()
        };
        mouse_pressed(app.world_mut(), MouseButton::Left);
        assert!(app.world().resource::<PlayerController>().aim.is_some());
        assert!(!app.world().resource::<ClientFeedback>().message.is_empty());
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(
            app.world().get::<AbilitySlots>(player).unwrap().cooldown(0),
            Some(0.0)
        );
        assert_eq!(
            app.world()
                .get::<EquipmentSlots>(player)
                .unwrap()
                .item(0)
                .unwrap()
                .cooldown(),
            0.0
        );
    }
}

fn champion_app(character: CharacterId) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        SimulationPlugin,
        crate::gameplay::GameplayPlugin,
    ))
    .add_message::<WindowEvent>()
    .init_resource::<PointerState>()
    .init_resource::<GamePointer>()
    .init_resource::<ClientFeedback>()
    .init_resource::<PlayerController>()
    .add_systems(Update, collect_input);
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    super::super::content::setup(app.world_mut());
    app.world_mut()
        .resource_mut::<super::super::content::SelectionSettings>()
        .default_character = character;
    super::super::session::setup(app.world_mut());
    super::super::session::start_native(app.world_mut());
    let player = app.world().resource::<ClientSession>().player().unwrap();
    (app, player)
}
fn step(app: &mut App, ticks: usize) {
    for _ in 0..ticks {
        app.world_mut().run_schedule(FixedUpdate);
    }
}

#[test]
fn actual_shield_and_clamped_standard_share_the_real_skill_input_path() {
    let (mut app, player) = champion_app(CharacterId(1));
    select_ability(app.world_mut(), 1);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    step(&mut app, 1);
    assert!(app.world().get::<UnitMechanics>(player).unwrap().shield() > 0.0);
    let origin = app.world().get::<UnitPose>(player).unwrap().position();
    let destination = origin + Vec2::X * 20.0;
    select_ability(app.world_mut(), 2);
    app.world_mut().resource_mut::<PointerState>().ground = Some(destination);
    mouse_pressed(app.world_mut(), MouseButton::Left);
    step(&mut app, 1);
    let standard = app
        .world_mut()
        .query::<&BattlefieldObject>()
        .iter(app.world())
        .find(|object| object.kind() == ObjectKind::Standard)
        .unwrap();
    let range = app
        .world()
        .resource::<AbilityCatalog>()
        .get(AbilityId(13))
        .unwrap()
        .range;
    assert!((standard.position().distance(origin) - range).abs() < 0.01);
    assert!(
        app.world()
            .get::<AbilitySlots>(player)
            .unwrap()
            .cooldown(2)
            .unwrap()
            > 0.0
    );
}

#[test]
fn real_hero_target_leap_keeps_aim_on_minions_and_recasts_without_enemy_or_second_cost() {
    let (mut app, player) = champion_app(CharacterId(1));
    let origin = app.world().get::<UnitPose>(player).unwrap().position();
    let minion = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Leap type fixture".into(),
            team: Team::Enemy,
            position: origin + Vec2::X * 2.0,
            stats: UnitStats {
                kind: UnitKind::Minion,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    select_ability(app.world_mut(), 3);
    app.world_mut().resource_mut::<PointerState>().hovered = Some(minion);
    mouse_pressed(app.world_mut(), MouseButton::Left);
    assert!(matches!(
        app.world().resource::<PlayerController>().aim,
        Some(Aim::Ability(3))
    ));
    let enemy = spawn_character(
        app.world_mut(),
        CharacterSpawnSpec {
            character: CharacterId(2),
            team: Team::Enemy,
            position: origin + Vec2::X * 4.0,
            loadout: LoadoutSelection::CharacterDefault,
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut().resource_mut::<PointerState>().hovered = Some(enemy);
    mouse_pressed(app.world_mut(), MouseButton::Left);
    step(&mut app, 25);
    assert!(app.world().get::<UnitMechanics>(player).unwrap().recast(3));
    assert!(
        app.world()
            .get::<AbilitySlots>(player)
            .unwrap()
            .cooldown(3)
            .unwrap()
            > 100.0
    );
    step(&mut app, 46);
    app.world_mut().resource_mut::<PointerState>().hovered = None;
    let mana = app.world().get::<UnitState>(player).unwrap().mana();
    select_ability(app.world_mut(), 3);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    step(&mut app, 1);
    assert!(!app.world().get::<UnitMechanics>(player).unwrap().recast(3));
    assert!((app.world().get::<UnitState>(player).unwrap().mana() - mana).abs() < 1.0);
    assert!(
        app.world()
            .get::<AbilitySlots>(player)
            .unwrap()
            .cooldown(3)
            .unwrap()
            > 100.0
    );
}

#[test]
fn real_focus_rejects_zero_stacks_then_activates_and_scout_uses_actual_charge_state() {
    let (mut app, player) = champion_app(CharacterId(2));
    select_ability(app.world_mut(), 0);
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    assert!(
        app.world()
            .resource::<ClientFeedback>()
            .message
            .contains("专注")
    );
    let origin = app.world().get::<UnitPose>(player).unwrap().position();
    let target = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Focus target".into(),
            team: Team::Enemy,
            position: origin + Vec2::X * 3.0,
            stats: UnitStats {
                max_health: 10000.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    submit(app.world_mut(), player, UnitCommand::AttackTarget(target));
    step(&mut app, 330);
    assert_eq!(app.world().get::<UnitMechanics>(player).unwrap().focus(), 4);
    select_ability(app.world_mut(), 0);
    step(&mut app, 1);
    assert!(
        app.world()
            .get::<UnitMechanics>(player)
            .unwrap()
            .empowered()
    );
    assert_eq!(
        app.world().get::<UnitMechanics>(player).unwrap().arrows(),
        5
    );
    assert_eq!(
        app.world().get::<UnitMechanics>(player).unwrap().charges(2),
        Some(1)
    );
    // Empower resets the attack interval and the persistent attack order can
    // immediately enter a new windup. Stop it before testing Scout separately.
    submit(app.world_mut(), player, UnitCommand::Stop);
    step(&mut app, 1);
    select_ability(app.world_mut(), 2);
    app.world_mut().resource_mut::<PointerState>().ground = Some(Vec2::ZERO);
    mouse_pressed(app.world_mut(), MouseButton::Left);
    // Scout has a real 250ms windup; charges are consumed only on commit.
    step(&mut app, 20);
    assert_eq!(
        app.world().get::<UnitMechanics>(player).unwrap().charges(2),
        Some(0)
    );
    assert!(
        app.world_mut()
            .query::<&BattlefieldObject>()
            .iter(app.world())
            .any(|object| object.kind() == ObjectKind::Scout)
    );
}

#[test]
fn picking_rejects_a_hidden_enemy_body_then_accepts_it_when_shared_team_vision_reveals_it() {
    let (mut app, player) = champion_app(CharacterId(1));
    let enemy = app
        .world()
        .resource::<crate::gameplay::MatchState>()
        .base(Team::Enemy)
        .unwrap();
    let point = app.world().get::<UnitPose>(enemy).unwrap().position();
    app.world_mut().entity_mut(enemy).insert((
        GlobalTransform::from_translation(Vec3::new(point.x, 0.0, point.y)),
        Pickable {
            offset: Vec3::Y,
            half_size: Vec3::splat(0.5),
        },
    ));
    let ray = Ray3d::new(Vec3::new(point.x, 1.0, point.y + 10.0), Dir3::NEG_Z);
    assert!(!team_can_see(
        app.world(),
        app.world().get::<Unit>(player).unwrap().team(),
        enemy
    ));
    assert_eq!(hovered_unit(app.world_mut(), ray), None);
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Team observer".into(),
            team: Team::Ally,
            position: point + Vec2::X * 4.0,
            stats: UnitStats {
                vision_radius: 8.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    assert_eq!(hovered_unit(app.world_mut(), ray), Some(enemy));
}
