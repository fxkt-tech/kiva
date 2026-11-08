use super::*;

const CAPTURED: CaptureFacts = CaptureFacts {
    focused: true,
    capture: web::PointerCapture {
        actual: true,
        event_confirmed: true,
    },
    pause_requested: false,
    settings_open: false,
};
const PAUSED_UNLOCKED: CaptureFacts = CaptureFacts {
    focused: true,
    capture: web::PointerCapture {
        actual: false,
        event_confirmed: false,
    },
    pause_requested: true,
    settings_open: false,
};

fn change_request(app: &App, character_id: u16) -> SessionRequest {
    let ClientSession::Active {
        player, match_id, ..
    } = *app.world().resource::<ClientSession>()
    else {
        panic!("active test match")
    };
    SessionRequest::ChangeCharacter {
        match_id: match_id.to_string(),
        player_id: player.to_bits().to_string(),
        character_id,
    }
}

#[test]
fn inline_character_confirmation_replaces_the_whole_match_and_discards_old_batch_tail() {
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let old_player = app.world().resource::<ClientSession>().player().unwrap();
    let old_match = app.world().resource::<MatchState>().id().unwrap();
    let old_units: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Unit>>()
        .iter(app.world())
        .collect();
    let old_revision = match *app.world().resource::<ClientSession>() {
        ClientSession::Active {
            selection_revision, ..
        } => selection_revision,
        _ => unreachable!(),
    };
    equip_equipment(app.world_mut(), old_player, 0, EquipmentId(1)).unwrap();
    app.world_mut().run_schedule(FixedUpdate);
    let raw = serde_json::json!([
        {"type":"ChangeCharacter", "match_id":old_match.to_string(), "player_id":old_player.to_bits().to_string(), "character_id":2},
        {"type":"ReturnToSelection", "player_id":old_player.to_bits().to_string()}
    ]).to_string();
    process_batch(
        app.world_mut(),
        &raw,
        CaptureFacts {
            settings_open: true,
            ..CAPTURED
        },
    );
    let player = app.world().resource::<ClientSession>().player().unwrap();
    assert_ne!(player, old_player);
    assert_ne!(app.world().resource::<MatchState>().id(), Some(old_match));
    assert_eq!(
        app.world().resource::<MatchState>().player_character(),
        Some(CharacterId(2))
    );
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    assert!(!app.world().resource::<SimulationControl>().paused);
    assert!(
        old_units
            .iter()
            .all(|id| app.world().get::<Unit>(*id).is_none())
    );
    assert!(
        app.world()
            .get::<EquipmentSlots>(player)
            .unwrap()
            .item(0)
            .is_none()
    );
    assert_eq!(
        app.world().get::<CharacterIdentity>(player).unwrap().id(),
        CharacterId(2)
    );
    assert_eq!(
        match *app.world().resource::<ClientSession>() {
            ClientSession::Active {
                selection_revision, ..
            } => selection_revision,
            _ => unreachable!(),
        },
        old_revision + 1
    );
}

#[test]
fn inline_character_rejects_stale_identity_unknown_role_and_unconfirmed_capture_without_resetting()
{
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let initial_match = app.world().resource::<MatchState>().id();
    for facts in [
        CAPTURED,
        CaptureFacts {
            settings_open: true,
            capture: web::PointerCapture {
                actual: true,
                event_confirmed: false,
            },
            ..CAPTURED
        },
        CaptureFacts {
            settings_open: true,
            focused: false,
            ..CAPTURED
        },
        CaptureFacts {
            settings_open: true,
            capture: web::PointerCapture::default(),
            ..CAPTURED
        },
    ] {
        let request = change_request(&app, 2);
        assert!(apply_request(app.world_mut(), request, facts).is_err());
        assert_eq!(
            app.world().resource::<ClientSession>().player(),
            Some(player)
        );
    }
    let facts = CaptureFacts {
        settings_open: true,
        ..CAPTURED
    };
    let unknown = change_request(&app, 999);
    assert!(apply_request(app.world_mut(), unknown, facts).is_err());
    for (match_id, player_id) in [
        ("old-match".to_owned(), player.to_bits().to_string()),
        (initial_match.unwrap().to_string(), "old-player".into()),
    ] {
        assert!(
            apply_request(
                app.world_mut(),
                SessionRequest::ChangeCharacter {
                    match_id,
                    player_id,
                    character_id: 2
                },
                facts
            )
            .is_err()
        );
    }
    assert_eq!(app.world().resource::<MatchState>().id(), initial_match);
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(player)
    );
    assert!(!app.world().resource::<SimulationControl>().paused);
}

#[test]
fn result_character_selection_can_restart_with_the_same_or_another_hero() {
    for character in [1, 2] {
        let mut app = app();
        let selected = choice(&app);
        apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
        let old_player = app.world().resource::<ClientSession>().player();
        super::super::test_support::finish_match(&mut app, Team::Ally);
        let request = change_request(&app, character);
        apply_request(
            app.world_mut(),
            request,
            CaptureFacts {
                settings_open: true,
                ..CAPTURED
            },
        )
        .unwrap();
        assert_ne!(app.world().resource::<ClientSession>().player(), old_player);
        assert_eq!(
            app.world().resource::<MatchState>().phase(),
            MatchPhase::Running
        );
        assert_eq!(
            app.world().resource::<MatchState>().player_character(),
            Some(CharacterId(character))
        );
        assert_eq!(app.world().resource::<SimulationTick>().0, 0);
        assert!(!process_frame(app.world_mut(), None, CAPTURED));
    }
}

#[test]
fn settings_allow_current_player_to_reselect_while_the_match_is_running() {
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    assert!(!app.world().resource::<SimulationControl>().paused);
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection {
            player_id: player.to_bits().to_string(),
        },
        CaptureFacts {
            pause_requested: false,
            settings_open: true,
            ..PAUSED_UNLOCKED
        },
    )
    .unwrap();
    assert!(
        app.world()
            .resource::<ClientSession>()
            .selection()
            .is_some()
    );
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Waiting
    );
    assert_eq!(unit_count(&mut app), 0);
    assert!(app.world().resource::<SimulationControl>().paused);
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        SimulationPlugin,
        crate::gameplay::GameplayPlugin,
    ))
    .init_resource::<super::super::controls::GameplayInput>();
    super::super::content::setup(app.world_mut());
    setup(app.world_mut());
    app
}
fn choice(app: &App) -> CharacterChoice {
    app.world()
        .resource::<ClientSession>()
        .selection()
        .unwrap()
        .selected
        .unwrap()
}
fn launch(choice: CharacterChoice) -> SessionRequest {
    SessionRequest::Launch {
        character_id: choice.character_id.0,
        selection_revision: choice.revision,
    }
}
fn select(app: &mut App, character_id: u16, request_id: u32) -> Result<(), String> {
    apply_request(
        app.world_mut(),
        SessionRequest::SelectCharacter {
            character_id,
            request_id,
        },
        CAPTURED,
    )
}
fn unit_count(app: &mut App) -> usize {
    app.world_mut().query::<&Unit>().iter(app.world()).count()
}

#[test]
fn content_and_preselection_create_no_units_until_the_native_creation_path_commits() {
    let mut app = app();
    assert_eq!(unit_count(&mut app), 0);
    assert!(app.world().resource::<ClientSession>().player().is_none());
    assert!(
        app.world_mut()
            .query::<&CharacterIdentity>()
            .iter(app.world())
            .next()
            .is_none()
    );
    let selection = app.world().resource::<ClientSession>().selection().unwrap();
    assert_eq!(selection.ack.request_id, 0);
    assert!(selection.can_start());
    start_native(app.world_mut());
    let player = app.world().resource::<ClientSession>().player().unwrap();
    assert_eq!(unit_count(&mut app), 6);
    assert_eq!(
        app.world().get::<CharacterIdentity>(player).unwrap().id(),
        CharacterId(1)
    );
    assert_eq!(
        app.world().get::<Unit>(player).unwrap().name(),
        "德玛西亚皇子"
    );
    assert!(!app.world().resource::<SimulationControl>().paused);
}

#[test]
fn invalid_select_revokes_old_launch_until_a_new_accepted_selection() {
    let mut app = app();
    let initial = choice(&app);
    assert!(select(&mut app, 999, 1).is_err());
    assert_eq!(
        choice(&app),
        initial,
        "invalid choice preserves the valid preview"
    );
    let selection = app.world().resource::<ClientSession>().selection().unwrap();
    assert_eq!(selection.ack.request_id, 1);
    assert!(!selection.can_start());
    assert!(apply_request(app.world_mut(), launch(initial), CAPTURED).is_err());
    assert_eq!(unit_count(&mut app), 0);
    select(&mut app, 1, 2).unwrap();
    let accepted = choice(&app);
    assert!(
        accepted.revision > initial.revision,
        "same-ID selection must advance the version"
    );
    assert!(apply_request(app.world_mut(), launch(initial), CAPTURED).is_err());
    assert!(
        app.world()
            .resource::<ClientSession>()
            .selection()
            .unwrap()
            .can_start()
    );
    apply_request(app.world_mut(), launch(accepted), CAPTURED).unwrap();
    assert_eq!(unit_count(&mut app), 6);
}

#[test]
fn rapid_choices_use_the_last_rust_revision_and_create_the_selected_loadout() {
    let mut app = app();
    let initial = choice(&app);
    select(&mut app, 2, 1).unwrap();
    let arcanist = choice(&app);
    select(&mut app, 1, 2).unwrap();
    let traveller = choice(&app);
    select(&mut app, 2, 3).unwrap();
    let final_choice = choice(&app);
    assert!(
        initial.revision < arcanist.revision
            && arcanist.revision < traveller.revision
            && traveller.revision < final_choice.revision
    );
    for stale in [initial, arcanist, traveller] {
        assert!(apply_request(app.world_mut(), launch(stale), CAPTURED).is_err());
    }
    apply_request(app.world_mut(), launch(final_choice), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let unit = app.world().get::<Unit>(player).unwrap();
    assert_eq!(unit.name(), "寒冰射手");
    assert_eq!(unit.attributes().ability_power(), 0.0);
    let slots = app.world().get::<AbilitySlots>(player).unwrap();
    for (slot, id) in [21, 22, 23, 24].into_iter().enumerate() {
        assert_eq!(slots.ability(slot), Some(AbilityId(id)));
    }
}

#[test]
fn lost_capture_focus_and_spawn_failure_leave_a_valid_choice_retryable() {
    let mut app = app();
    let selected = choice(&app);
    for facts in [
        CaptureFacts {
            focused: false,
            capture: web::PointerCapture {
                actual: true,
                event_confirmed: true,
            },
            pause_requested: false,
            settings_open: false,
        },
        CaptureFacts {
            focused: true,
            capture: web::PointerCapture {
                actual: false,
                event_confirmed: true,
            },
            pause_requested: false,
            settings_open: false,
        },
    ] {
        assert!(apply_request(app.world_mut(), launch(selected), facts).is_err());
        assert_eq!(unit_count(&mut app), 0);
        assert!(
            app.world()
                .resource::<ClientSession>()
                .selection()
                .unwrap()
                .can_start()
        );
    }
    let abilities = app.world_mut().remove_resource::<AbilityCatalog>().unwrap();
    app.world_mut().insert_resource(AbilityCatalog::default());
    assert!(apply_request(app.world_mut(), launch(selected), CAPTURED).is_err());
    assert_eq!(
        unit_count(&mut app),
        0,
        "invalid loadout cannot bind a partial player"
    );
    assert!(
        app.world()
            .resource::<ClientSession>()
            .selection()
            .unwrap()
            .can_start()
    );
    app.world_mut().insert_resource(abilities);
    let navigation = app.world().resource::<NavigationMap>().clone();
    let spawn = app
        .world()
        .resource::<crate::gameplay::MapDefinition>()
        .hero_spawn(Team::Ally);
    app.world_mut().insert_resource(NavigationMap::new(
        navigation.half_size(),
        1.0,
        vec![Obstacle {
            center: spawn,
            half_size: Vec2::splat(0.8),
        }],
    ));
    assert!(apply_request(app.world_mut(), launch(selected), CAPTURED).is_err());
    assert_eq!(unit_count(&mut app), 0);
    assert_eq!(choice(&app), selected);
    assert!(
        app.world()
            .resource::<ClientSession>()
            .selection()
            .unwrap()
            .can_start()
    );
    app.world_mut().insert_resource(navigation);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    assert_eq!(unit_count(&mut app), 6);
}

#[test]
fn active_rejects_duplicate_launch_and_selection_without_resetting_a_committed_cast() {
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    app.world_mut().resource_mut::<CommandQueue>().submit(
        player,
        UnitCommand::CastAbility {
            slot: 2,
            target: AbilityTarget::Point(Vec2::new(1.0, 6.0)),
        },
    );
    for _ in 0..8 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let mana = app.world().get::<UnitState>(player).unwrap().mana();
    let cooldown = app.world().get::<AbilitySlots>(player).unwrap().cooldown(2);
    assert!(cooldown.unwrap() > 0.0);
    assert!(apply_request(app.world_mut(), launch(selected), CAPTURED).is_err());
    assert!(select(&mut app, 2, 9).is_err());
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(player)
    );
    assert_eq!(unit_count(&mut app), 6);
    assert_eq!(app.world().get::<UnitState>(player).unwrap().mana(), mana);
    assert_eq!(
        app.world().get::<AbilitySlots>(player).unwrap().cooldown(2),
        cooldown
    );
}

#[test]
fn malformed_or_oversized_batches_never_apply_a_valid_prefix() {
    let mut app = app();
    let initial = choice(&app);
    for raw in [
        r#"[{"type":"Launch","character_id":1,"selection_revision":1},{"type":"WorldWrite"}]"#,
        r#"[{"type":"SelectCharacter","character_id":2,"request_id":1},{"type":"Launch","character_id":2,"selection_revision":2,"stats":{}}]"#,
        r#"[{"type":"SelectCharacter","character_id":2.5,"request_id":1}]"#,
    ] {
        process_batch(app.world_mut(), raw, CAPTURED);
        assert_eq!(choice(&app), initial);
        assert_eq!(unit_count(&mut app), 0);
    }
    let oversized = format!(
        "[{}]",
        vec![r#"{"type":"SelectCharacter","character_id":2,"request_id":1}"#; 33].join(",")
    );
    process_batch(app.world_mut(), &oversized, CAPTURED);
    assert_eq!(choice(&app), initial);
    assert_eq!(unit_count(&mut app), 0);
}

#[test]
fn structurally_valid_batches_execute_fifo_without_rolling_back_semantic_successes() {
    let mut app = app();
    process_batch(
        app.world_mut(),
        r#"[{"type":"SelectCharacter","character_id":2,"request_id":1},{"type":"Launch","character_id":1,"selection_revision":1}]"#,
        CAPTURED,
    );
    assert_eq!(choice(&app).character_id, CharacterId(2));
    assert_eq!(unit_count(&mut app), 0);
    process_batch(
        app.world_mut(),
        r#"[{"type":"Launch","character_id":2,"selection_revision":2},{"type":"SelectCharacter","character_id":999,"request_id":2}]"#,
        CAPTURED,
    );
    let player = app.world().resource::<ClientSession>().player().unwrap();
    assert_eq!(
        app.world().get::<CharacterIdentity>(player).unwrap().id(),
        CharacterId(2)
    );
    assert_eq!(unit_count(&mut app), 6);
}

#[test]
fn native_capture_waits_for_its_event_before_stray_cleanup_or_launch() {
    let mut app = app();
    let acquired_before_event = CaptureFacts {
        focused: true,
        capture: web::PointerCapture {
            actual: true,
            event_confirmed: false,
        },
        pause_requested: false,
        settings_open: false,
    };
    for _ in 0..4 {
        assert!(
            !process_frame(app.world_mut(), None, acquired_before_event),
            "a lock whose event has not arrived must remain available to Launch"
        );
        assert!(app.world().resource::<ClientSession>().player().is_none());
        assert_eq!(unit_count(&mut app), 0);
    }
    assert!(!process_frame(
        app.world_mut(),
        Some(r#"[{"type":"Launch","character_id":1,"selection_revision":1}]"#),
        CAPTURED
    ));
    assert!(app.world().resource::<ClientSession>().player().is_some());
    assert_eq!(unit_count(&mut app), 6);
}

#[test]
fn confirmed_stray_or_rejected_capture_releases_without_creating_a_player() {
    let mut app = app();
    assert!(process_frame(app.world_mut(), None, CAPTURED));
    assert!(process_frame(
        app.world_mut(),
        Some(r#"[{"type":"Launch","character_id":2,"selection_revision":1}]"#),
        CAPTURED
    ));
    assert_eq!(unit_count(&mut app), 0);
    assert!(
        app.world()
            .resource::<ClientSession>()
            .selection()
            .unwrap()
            .can_start()
    );
    let lost_capture = CaptureFacts {
        focused: true,
        capture: web::PointerCapture {
            actual: false,
            event_confirmed: true,
        },
        pause_requested: false,
        settings_open: false,
    };
    assert!(!process_frame(
        app.world_mut(),
        Some(r#"[{"type":"Launch","character_id":1,"selection_revision":1}]"#),
        lost_capture
    ));
    assert!(
        app.world().resource::<ClientSession>().player().is_none(),
        "event acknowledgement cannot replace actual capture validation"
    );
}

#[test]
fn return_resets_the_match_after_real_damage_without_recreating_static_content() {
    let mut app = app();
    select(&mut app, 2, 1).unwrap();
    let previous_choice = choice(&app);
    apply_request(app.world_mut(), launch(previous_choice), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let player_position = app.world().get::<UnitPose>(player).unwrap().position();
    let target_position = player_position + Vec2::Y * 2.0;
    let dummy = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Reset test target".into(),
            team: Team::Enemy,
            position: target_position,
            stats: UnitStats {
                max_health: 400.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut().resource_mut::<CommandQueue>().submit(
        player,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::Direction(target_position - player_position),
        },
    );
    for _ in 0..80 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(app.world().get::<UnitState>(dummy).unwrap().health() < 400.0);
    let caster = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Reset test source".into(),
            team: Team::Enemy,
            position: player_position + Vec2::X,
            stats: UnitStats {
                vision_radius: 12.0,
                ..default()
            },
            abilities: [Some(AbilityId(5)), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut().resource_mut::<CommandQueue>().submit(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(player),
        },
    );
    for _ in 0..20 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(
        app.world().get::<UnitState>(player).unwrap().health()
            < app
                .world()
                .get::<Unit>(player)
                .unwrap()
                .definition()
                .max_health
    );
    assert!(
        app.world()
            .get::<AbilitySlots>(player)
            .unwrap()
            .cooldown(1)
            .unwrap()
            > 0.0
    );
    let previous_units: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Unit>>()
        .iter(app.world())
        .collect();
    let static_entity = app.world_mut().spawn_empty().id();
    let map_obstacles = app.world().resource::<NavigationMap>().obstacles().len();
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection {
            player_id: player.to_bits().to_string(),
        },
        PAUSED_UNLOCKED,
    )
    .unwrap();
    assert!(app.world().resource::<ClientSession>().player().is_none());
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    assert!(app.world().resource::<SimulationControl>().paused);
    assert_eq!(unit_count(&mut app), 0);
    assert!(
        previous_units
            .into_iter()
            .all(|id| app.world().get_entity(id).is_err())
    );
    assert!(app.world().get_entity(static_entity).is_ok());
    assert_eq!(
        app.world().resource::<NavigationMap>().obstacles().len(),
        map_obstacles
    );
    assert_eq!(app.world().resource::<CharacterCatalog>().iter().len(), 2);
    let selection = app.world().resource::<ClientSession>().selection().unwrap();
    assert_eq!(selection.selected.unwrap().character_id, CharacterId(2));
    assert!(selection.selected.unwrap().revision > previous_choice.revision);
    assert_eq!(selection.ack.request_id, 0);
    assert!(selection.can_start());
    let game = app.world().resource::<MatchState>();
    assert_eq!(game.phase(), MatchPhase::Waiting);
    assert_eq!(game.wave_number(), 0);
    assert!(game.id().is_none());
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
}

#[test]
fn return_requires_the_current_player_and_a_paused_unlocked_boundary() {
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let id = player.to_bits().to_string();
    assert!(
        apply_request(
            app.world_mut(),
            SessionRequest::ReturnToSelection {
                player_id: "stale".into()
            },
            PAUSED_UNLOCKED
        )
        .is_err()
    );
    assert!(
        apply_request(
            app.world_mut(),
            SessionRequest::ReturnToSelection {
                player_id: id.clone()
            },
            CaptureFacts {
                pause_requested: true,
                ..CAPTURED
            }
        )
        .is_err()
    );
    assert!(
        apply_request(
            app.world_mut(),
            SessionRequest::ReturnToSelection {
                player_id: id.clone()
            },
            CaptureFacts {
                pause_requested: false,
                ..PAUSED_UNLOCKED
            }
        )
        .is_err()
    );
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(player)
    );
    assert_eq!(unit_count(&mut app), 6);
    assert!(!app.world().resource::<SimulationControl>().paused);
    // Pointer Lock's event already requested pause, before collect_input
    // has copied that request into SimulationControl this frame.
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection { player_id: id },
        PAUSED_UNLOCKED,
    )
    .unwrap();
    assert_eq!(unit_count(&mut app), 0);
}

#[test]
fn return_is_idempotent_in_selection_and_old_round_requests_cannot_reset_the_next_player() {
    let mut app = app();
    let previous_choice = choice(&app);
    apply_request(app.world_mut(), launch(previous_choice), CAPTURED).unwrap();
    let previous_player = app.world().resource::<ClientSession>().player().unwrap();
    let previous_id = previous_player.to_bits().to_string();
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection {
            player_id: previous_id.clone(),
        },
        PAUSED_UNLOCKED,
    )
    .unwrap();
    let new_choice = choice(&app);
    let npcs: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Unit>>()
        .iter(app.world())
        .collect();
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection {
            player_id: previous_id.clone(),
        },
        PAUSED_UNLOCKED,
    )
    .unwrap();
    assert_eq!(choice(&app), new_choice);
    assert!(
        npcs.into_iter()
            .all(|id| app.world().get_entity(id).is_ok())
    );
    assert!(apply_request(app.world_mut(), launch(previous_choice), CAPTURED).is_err());
    apply_request(app.world_mut(), launch(new_choice), CAPTURED).unwrap();
    let next_player = app.world().resource::<ClientSession>().player().unwrap();
    assert_ne!(next_player, previous_player);
    assert!(
        apply_request(
            app.world_mut(),
            SessionRequest::ReturnToSelection {
                player_id: previous_id
            },
            PAUSED_UNLOCKED
        )
        .is_err()
    );
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(next_player)
    );
    assert_eq!(unit_count(&mut app), 6);
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection {
            player_id: next_player.to_bits().to_string(),
        },
        PAUSED_UNLOCKED,
    )
    .unwrap();
    assert!(choice(&app).revision > new_choice.revision);
}

#[test]
fn accepted_return_discards_the_old_batch_tail_but_malformed_batches_do_not_reset() {
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let malformed = serde_json::json!([
        {"type":"ReturnToSelection", "player_id":player.to_bits().to_string()},
        {"type":"Launch", "character_id":1, "selection_revision":1, "extra":true},
    ])
    .to_string();
    process_batch(app.world_mut(), &malformed, PAUSED_UNLOCKED);
    assert_eq!(
        app.world().resource::<ClientSession>().player(),
        Some(player)
    );
    let batch = serde_json::json!([
        {"type":"ReturnToSelection", "player_id":player.to_bits().to_string()},
        {"type":"SelectCharacter", "character_id":2, "request_id":11},
        {"type":"SelectCharacter", "character_id":999, "request_id":12},
        {"type":"Launch", "character_id":1, "selection_revision":selected.revision},
    ])
    .to_string();
    process_batch(app.world_mut(), &batch, PAUSED_UNLOCKED);
    let selection = app.world().resource::<ClientSession>().selection().unwrap();
    assert_eq!(selection.selected.unwrap().character_id, CharacterId(1));
    assert_eq!(selection.selected.unwrap().revision, selected.revision + 1);
    assert_eq!(selection.ack.request_id, 0);
    assert!(selection.can_start());
    assert!(selection.error.is_none());
    assert_eq!(unit_count(&mut app), 0);
}

#[test]
fn return_clears_input_feedback_and_restarts_a_clean_round_at_tick_zero() {
    use super::super::{
        controls::{self, Aim, ClientFeedback, PlayerController, PointerState},
        pointer::GamePointer,
    };
    let mut app = app();
    app.init_resource::<PlayerController>()
        .init_resource::<PointerState>()
        .init_resource::<GamePointer>()
        .init_resource::<ClientFeedback>()
        .add_message::<bevy::window::WindowEvent>()
        .add_systems(Update, controls::collect_input);
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    app.world_mut().run_schedule(Update);
    app.world_mut().resource_mut::<PlayerController>().selected = Some(player);
    app.world_mut().resource_mut::<PlayerController>().aim = Some(Aim::Ability(0));
    *app.world_mut().resource_mut::<ClientFeedback>() = ClientFeedback {
        message: "old trial".into(),
        expires: 100.0,
        marker: Some((Vec2::ZERO, false, 0.0)),
    };
    apply_request(
        app.world_mut(),
        SessionRequest::ReturnToSelection {
            player_id: player.to_bits().to_string(),
        },
        PAUSED_UNLOCKED,
    )
    .unwrap();
    app.world_mut().run_schedule(Update);
    assert!(
        app.world()
            .resource::<PlayerController>()
            .selected
            .is_none()
    );
    assert!(app.world().resource::<PlayerController>().aim.is_none());
    let feedback = app.world().resource::<ClientFeedback>();
    assert!(feedback.message.is_empty());
    assert_eq!(feedback.expires, 0.0);
    assert!(feedback.marker.is_none());
    let next = choice(&app);
    apply_request(app.world_mut(), launch(next), CAPTURED).unwrap();
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(FixedUpdate);
    let game = app.world().resource::<MatchState>();
    assert!(game.can_advance());
    assert_eq!(game.wave_number(), 1);
    assert_eq!(app.world().resource::<SimulationTick>().0, 1);
    assert_eq!(unit_count(&mut app), 6);
}

fn restart_request(app: &App) -> SessionRequest {
    SessionRequest::RestartMatch {
        match_id: app
            .world()
            .resource::<MatchState>()
            .id()
            .unwrap()
            .to_string(),
        player_id: app
            .world()
            .resource::<ClientSession>()
            .player()
            .unwrap()
            .to_bits()
            .to_string(),
    }
}

#[test]
fn restart_requires_current_finished_match_and_confirmed_capture_then_binds_one_fresh_round() {
    let mut app = app();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let request = restart_request(&app);
    assert!(apply_request(app.world_mut(), request.clone(), CAPTURED).is_err());
    super::super::test_support::finish_match(&mut app, Team::Ally);
    let old_match = app.world().resource::<MatchState>().id();
    assert!(
        process_frame(app.world_mut(), None, CAPTURED),
        "terminal capture must be released"
    );
    for facts in [
        PAUSED_UNLOCKED,
        CaptureFacts {
            focused: false,
            ..CAPTURED
        },
        CaptureFacts {
            capture: web::PointerCapture {
                actual: true,
                event_confirmed: false,
            },
            ..CAPTURED
        },
    ] {
        assert!(apply_request(app.world_mut(), request.clone(), facts).is_err());
        assert_eq!(
            app.world().resource::<ClientSession>().player(),
            Some(player)
        );
        assert_eq!(app.world().resource::<MatchState>().id(), old_match);
    }
    let old_units: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Unit>>()
        .iter(app.world())
        .collect();
    apply_request(app.world_mut(), request.clone(), CAPTURED).unwrap();
    let next = app.world().resource::<ClientSession>().player().unwrap();
    assert_ne!(next, player);
    assert_ne!(app.world().resource::<MatchState>().id(), old_match);
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Running
    );
    assert_eq!(app.world().resource::<MatchState>().wave_number(), 1);
    assert_eq!(app.world().resource::<SimulationTick>().0, 0);
    assert!(!app.world().resource::<SimulationControl>().paused);
    assert_eq!(unit_count(&mut app), 6);
    assert!(
        old_units
            .into_iter()
            .all(|id| app.world().get_entity(id).is_err())
    );
    let ClientSession::Active {
        selection_revision, ..
    } = *app.world().resource::<ClientSession>()
    else {
        panic!("active");
    };
    assert_eq!(selection_revision, selected.revision + 1);
    assert!(apply_request(app.world_mut(), request, CAPTURED).is_err());
    assert_eq!(app.world().resource::<ClientSession>().player(), Some(next));
}

#[test]
fn failed_restart_returns_a_retryable_choice_and_discards_the_old_batch_tail() {
    let mut app = app();
    select(&mut app, 2, 1).unwrap();
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    super::super::test_support::finish_match(&mut app, Team::Enemy);
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let match_id = app
        .world()
        .resource::<MatchState>()
        .id()
        .unwrap()
        .to_string();
    let catalog = app.world_mut().remove_resource::<AbilityCatalog>().unwrap();
    app.world_mut().insert_resource(AbilityCatalog::default());
    let batch = serde_json::json!([
        {"type":"RestartMatch", "match_id":match_id, "player_id":player.to_bits().to_string()},
        {"type":"SelectCharacter", "character_id":1, "request_id":9}
    ])
    .to_string();
    process_batch(app.world_mut(), &batch, CAPTURED);
    assert!(app.world().resource::<ClientSession>().player().is_none());
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Waiting
    );
    assert_eq!(unit_count(&mut app), 0);
    assert!(app.world().resource::<SimulationControl>().paused);
    let selection = app.world().resource::<ClientSession>().selection().unwrap();
    assert!(selection.can_start());
    assert!(selection.error.is_some());
    assert_eq!(selection.selected.unwrap().character_id, CharacterId(2));
    assert_eq!(selection.selected.unwrap().revision, selected.revision + 1);
    assert_eq!(
        selection.ack.request_id, 0,
        "old batch tail cannot select a different character"
    );
    app.world_mut().insert_resource(catalog);
    let next = choice(&app);
    apply_request(app.world_mut(), launch(next), CAPTURED).unwrap();
    assert_eq!(unit_count(&mut app), 6);
}

#[test]
fn first_launch_and_result_restart_center_once_while_ordinary_resume_keeps_free_camera() {
    use super::super::{
        camera::{self, CameraRig, GameCamera},
        controls::PointerState,
    };
    use bevy::input::mouse::MouseWheel;
    let mut app = app();
    app.init_resource::<CameraRig>()
        .init_resource::<PointerState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<MouseWheel>()
        .add_systems(Update, camera::update);
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut().spawn(Window::default());
    let selected = choice(&app);
    apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
    let player = app.world().resource::<ClientSession>().player().unwrap();
    let spawn = app.world().get::<UnitPose>(player).unwrap().position();
    app.world_mut().run_schedule(Update);
    let rig = app.world().resource::<CameraRig>();
    assert_eq!(rig.center, spawn, "first launch must show the real hero");
    assert!(
        !rig.locked,
        "initial positioning must preserve free-camera mode"
    );
    let free_center = Vec2::new(3.0, 7.0);
    *app.world_mut().resource_mut::<CameraRig>() = CameraRig {
        center: free_center,
        locked: false,
        zoom: 1.3,
    };
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    app.world_mut().run_schedule(Update);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    app.world_mut().run_schedule(Update);
    let rig = app.world().resource::<CameraRig>();
    assert_eq!(
        rig.center, free_center,
        "ordinary resume must preserve free pan"
    );
    assert_eq!(rig.zoom, 1.3);
    assert!(!rig.locked);
    super::super::test_support::finish_match(&mut app, Team::Ally);
    app.world_mut().resource_mut::<CameraRig>().locked = true;
    let restart = restart_request(&app);
    apply_request(app.world_mut(), restart, CAPTURED).unwrap();
    app.world_mut().run_schedule(Update);
    let next = app.world().resource::<ClientSession>().player().unwrap();
    assert_ne!(next, player);
    let rig = app.world().resource::<CameraRig>();
    assert_eq!(
        rig.center,
        app.world().get::<UnitPose>(next).unwrap().position()
    );
    assert!(!rig.locked, "result restart must enter a fresh free camera");
    assert_eq!(
        rig.zoom, 1.3,
        "new-player centering must not overwrite the zoom preference"
    );
    app.world_mut().resource_mut::<CameraRig>().center = free_center;
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world().resource::<CameraRig>().center,
        free_center,
        "new-player centering happens exactly once"
    );
}
