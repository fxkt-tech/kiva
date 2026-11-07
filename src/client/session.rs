//! Owns the transition from a confirmed character choice to one active player.
//! Device capture is an input fact; simulation pause and gameplay input remain
//! the controls adapter's responsibility.
use super::{
    content::{self, DemoSettings},
    web,
};
use crate::simulation::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CharacterChoice {
    pub character_id: CharacterId,
    pub revision: u64,
}

#[derive(Clone, Copy, Serialize)]
pub(super) struct SelectionAck {
    pub request_id: u32,
    pub accepted: bool,
}

pub(super) struct Selection {
    pub selected: Option<CharacterChoice>,
    pub ack: SelectionAck,
    pub error: Option<String>,
}
impl Selection {
    pub fn can_start(&self) -> bool {
        self.selected.is_some() && self.ack.accepted
    }
}

#[derive(Resource)]
pub(super) enum ClientSession {
    Selecting(Selection),
    Active {
        player: Entity,
        selection_revision: u64,
    },
}
impl ClientSession {
    pub fn player(&self) -> Option<Entity> {
        match self {
            Self::Active { player, .. } => Some(*player),
            Self::Selecting(_) => None,
        }
    }

    pub fn selection(&self) -> Option<&Selection> {
        match self {
            Self::Selecting(selection) => Some(selection),
            Self::Active { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum SessionRequest {
    SelectCharacter {
        character_id: u16,
        request_id: u32,
    },
    Launch {
        character_id: u16,
        selection_revision: u64,
    },
    ReturnToSelection {
        player_id: String,
    },
}

#[derive(Clone, Copy)]
struct CaptureFacts {
    focused: bool,
    capture: web::PointerCapture,
    pause_requested: bool,
}

pub(super) fn setup(world: &mut World) {
    let default = world.resource::<DemoSettings>().default_character;
    let valid = world.resource::<CharacterCatalog>().get(default).is_some();
    world.insert_resource(ClientSession::Selecting(Selection {
        selected: valid.then_some(CharacterChoice {
            character_id: default,
            revision: 1,
        }),
        ack: SelectionAck {
            request_id: 0,
            accepted: valid,
        },
        error: (!valid).then(|| "默认角色不可用，请重新选择。".into()),
    }));
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn start_native(world: &mut World) {
    let choice = world
        .resource::<ClientSession>()
        .selection()
        .and_then(|s| s.selected)
        .expect("registered default demo character");
    apply_request(
        world,
        SessionRequest::Launch {
            character_id: choice.character_id.0,
            selection_revision: choice.revision,
        },
        CaptureFacts {
            focused: true,
            capture: web::PointerCapture {
                actual: true,
                event_confirmed: true,
            },
            pause_requested: false,
        },
    )
    .expect("valid default demo character spawn");
}

/// The whole bounded batch must be structurally valid before any intent runs.
fn parse_requests(raw: &str) -> Result<Vec<SessionRequest>, &'static str> {
    let values: Vec<serde_json::Value> =
        serde_json::from_str(raw).map_err(|_| "无法读取选角请求，请重试。")?;
    if values.len() > 32 {
        return Err("选角请求过多，请重试。");
    }
    values
        .into_iter()
        .map(|value| serde_json::from_value(value).map_err(|_| "选角请求格式无效，请重试。"))
        .collect()
}

fn set_error(world: &mut World, error: String) {
    if let ClientSession::Selecting(selection) = &mut *world.resource_mut::<ClientSession>() {
        selection.error = Some(error);
    }
}

fn apply_request(
    world: &mut World,
    request: SessionRequest,
    facts: CaptureFacts,
) -> Result<(), String> {
    if let SessionRequest::ReturnToSelection { player_id } = &request {
        let result = return_to_selection(world, player_id, facts);
        world.resource_mut::<ClientSession>().set_changed();
        return result;
    }
    if world.resource::<ClientSession>().player().is_some() {
        // An Active rejection still acknowledges the consumed request with a
        // fresh projection, without changing the committed player.
        world.resource_mut::<ClientSession>().set_changed();
        return Err("本局已经开始。".into());
    }
    match request {
        SessionRequest::SelectCharacter {
            character_id,
            request_id,
        } => {
            let character_id = CharacterId(character_id);
            let valid = world
                .resource::<CharacterCatalog>()
                .get(character_id)
                .is_some();
            let ClientSession::Selecting(selection) = &mut *world.resource_mut::<ClientSession>()
            else {
                unreachable!()
            };
            selection.ack = SelectionAck {
                request_id,
                accepted: false,
            };
            if !valid {
                selection.error = Some("该角色不可用，请重新选择。".into());
                return Err(selection.error.clone().unwrap());
            }
            let Some(revision) = selection
                .selected
                .map_or(Some(1), |choice| choice.revision.checked_add(1))
            else {
                selection.error = Some("选择版本已耗尽，请重新加载。".into());
                return Err(selection.error.clone().unwrap());
            };
            selection.selected = Some(CharacterChoice {
                character_id,
                revision,
            });
            selection.ack.accepted = true;
            selection.error = None;
            Ok(())
        }
        SessionRequest::Launch {
            character_id,
            selection_revision,
        } => {
            let selection = world
                .resource::<ClientSession>()
                .selection()
                .expect("selecting");
            let choice = CharacterChoice {
                character_id: CharacterId(character_id),
                revision: selection_revision,
            };
            let error = if !selection.can_start() || selection.selected != Some(choice) {
                Some("角色选择已变更，请确认后重试。")
            } else if !facts.focused || !facts.capture.actual {
                Some("鼠标捕获已取消，请重新开始。")
            } else {
                None
            };
            if let Some(error) = error {
                set_error(world, error.into());
                return Err(error.into());
            }
            let settings = world.resource::<DemoSettings>();
            let spec = CharacterSpawnSpec {
                character: choice.character_id,
                team: settings.player_team,
                position: settings.player_spawn,
                loadout: LoadoutSelection::CharacterDefault,
                respawn_delay: settings.respawn_delay,
            };
            match spawn_character(world, spec) {
                Ok(player) => {
                    *world.resource_mut::<ClientSession>() = ClientSession::Active {
                        player,
                        selection_revision,
                    };
                    Ok(())
                }
                Err(error) => {
                    let error = format!("无法开始试炼：{error}");
                    set_error(world, error.clone());
                    Err(error)
                }
            }
        }
        SessionRequest::ReturnToSelection { .. } => unreachable!("handled before the Active gate"),
    }
}

fn return_to_selection(
    world: &mut World,
    player_id: &str,
    facts: CaptureFacts,
) -> Result<(), String> {
    let session = world.resource::<ClientSession>();
    let ClientSession::Active {
        player,
        selection_revision,
    } = *session
    else {
        return Ok(());
    };
    if player.to_bits().to_string() != player_id {
        return Err("这场试炼已经结束。".into());
    }
    let paused =
        world.resource::<SimulationControl>().paused || facts.pause_requested || !facts.focused;
    if facts.capture.actual || !paused {
        return Err("请先暂停试炼并释放鼠标。".into());
    }
    let character_id = world
        .get::<CharacterIdentity>(player)
        .map(CharacterIdentity::id)
        .ok_or("当前角色不可用。")?;
    let revision = selection_revision
        .checked_add(1)
        .ok_or("选择版本已耗尽，请重新加载。")?;
    reset_simulation(world);
    content::spawn_training_units(world);
    *world.resource_mut::<ClientSession>() = ClientSession::Selecting(Selection {
        selected: Some(CharacterChoice {
            character_id,
            revision,
        }),
        ack: SelectionAck {
            request_id: 0,
            accepted: true,
        },
        error: None,
    });
    Ok(())
}

/// Runs before gameplay input, including while the simulation is paused.
pub(super) fn process_requests(world: &mut World) {
    let raw = web::take_session_requests();
    // This synchronous batch cannot interleave browser focus callbacks.
    let facts = CaptureFacts {
        focused: web::document_focused().unwrap_or(false),
        capture: web::pointer_capture().unwrap_or_default(),
        pause_requested: web::paused().unwrap_or(world.resource::<SimulationControl>().paused),
    };
    if process_frame(world, raw.as_deref(), facts) {
        web::release_pointer();
    }
}

fn process_frame(world: &mut World, raw: Option<&str>, facts: CaptureFacts) -> bool {
    if let Some(raw) = raw {
        process_batch(world, raw, facts);
    }
    // Let pointerlockchange acknowledge a newly acquired lock and submit its
    // Launch first. An acknowledged lock without a committed player is stray
    // or rejected and must be released.
    world.resource::<ClientSession>().player().is_none() && facts.capture.confirmed()
}

fn process_batch(world: &mut World, raw: &str, facts: CaptureFacts) {
    match parse_requests(raw) {
        Ok(requests) => {
            for request in requests {
                let previous_player = world.resource::<ClientSession>().player();
                let _ = apply_request(world, request, facts);
                if previous_player.is_some() && world.resource::<ClientSession>().player().is_none()
                {
                    // Returning ends this round's input batch. Later queued
                    // requests cannot act on the new selection screen.
                    break;
                }
            }
        }
        Err(error) => set_error(world, error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPTURED: CaptureFacts = CaptureFacts {
        focused: true,
        capture: web::PointerCapture {
            actual: true,
            event_confirmed: true,
        },
        pause_requested: false,
    };
    const PAUSED_UNLOCKED: CaptureFacts = CaptureFacts {
        focused: true,
        capture: web::PointerCapture {
            actual: false,
            event_confirmed: false,
        },
        pause_requested: true,
    };

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
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
    fn content_and_preselection_create_only_npcs_until_the_same_native_creation_path_commits() {
        let mut app = app();
        assert_eq!(unit_count(&mut app), 2);
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
        assert_eq!(unit_count(&mut app), 3);
        assert_eq!(
            app.world().get::<CharacterIdentity>(player).unwrap().id(),
            CharacterId(1)
        );
        assert_eq!(app.world().get::<Unit>(player).unwrap().name(), "旅者");
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
        assert_eq!(unit_count(&mut app), 2);
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
        assert_eq!(unit_count(&mut app), 3);
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
        assert_eq!(unit.name(), "秘术师");
        assert_eq!(unit.attributes().ability_power(), 40.0);
        let slots = app.world().get::<AbilitySlots>(player).unwrap();
        for (slot, id) in [1, 2, 3, 4].into_iter().enumerate() {
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
            },
            CaptureFacts {
                focused: true,
                capture: web::PointerCapture {
                    actual: false,
                    event_confirmed: true,
                },
                pause_requested: false,
            },
        ] {
            assert!(apply_request(app.world_mut(), launch(selected), facts).is_err());
            assert_eq!(unit_count(&mut app), 2);
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
            2,
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
        app.world_mut().resource_mut::<DemoSettings>().player_spawn = Vec2::new(-10.0, 0.0);
        assert!(apply_request(app.world_mut(), launch(selected), CAPTURED).is_err());
        assert_eq!(unit_count(&mut app), 2);
        assert_eq!(choice(&app), selected);
        assert!(
            app.world()
                .resource::<ClientSession>()
                .selection()
                .unwrap()
                .can_start()
        );
        app.world_mut().resource_mut::<DemoSettings>().player_spawn = Vec2::new(0.0, 6.0);
        apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
        assert_eq!(unit_count(&mut app), 3);
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
        assert_eq!(unit_count(&mut app), 3);
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
            assert_eq!(unit_count(&mut app), 2);
        }
        let oversized = format!(
            "[{}]",
            vec![r#"{"type":"SelectCharacter","character_id":2,"request_id":1}"#; 33].join(",")
        );
        process_batch(app.world_mut(), &oversized, CAPTURED);
        assert_eq!(choice(&app), initial);
        assert_eq!(unit_count(&mut app), 2);
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
        assert_eq!(unit_count(&mut app), 2);
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
        assert_eq!(unit_count(&mut app), 3);
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
        };
        for _ in 0..4 {
            assert!(
                !process_frame(app.world_mut(), None, acquired_before_event),
                "a lock whose event has not arrived must remain available to Launch"
            );
            assert!(app.world().resource::<ClientSession>().player().is_none());
            assert_eq!(unit_count(&mut app), 2);
        }
        assert!(!process_frame(
            app.world_mut(),
            Some(r#"[{"type":"Launch","character_id":1,"selection_revision":1}]"#),
            CAPTURED
        ));
        assert!(app.world().resource::<ClientSession>().player().is_some());
        assert_eq!(unit_count(&mut app), 3);
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
        assert_eq!(unit_count(&mut app), 2);
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
    fn return_rebuilds_the_trial_after_real_damage_without_recreating_static_content() {
        let mut app = app();
        select(&mut app, 2, 1).unwrap();
        let previous_choice = choice(&app);
        apply_request(app.world_mut(), launch(previous_choice), CAPTURED).unwrap();
        let player = app.world().resource::<ClientSession>().player().unwrap();
        let dummy = app
            .world_mut()
            .query::<(Entity, &Unit)>()
            .iter(app.world())
            .find(|(_, unit)| unit.name() == "训练哨兵")
            .unwrap()
            .0;
        app.world_mut().resource_mut::<CommandQueue>().submit(
            player,
            UnitCommand::CastAbility {
                slot: 1,
                target: AbilityTarget::Point(Vec2::new(3.0, -2.0)),
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
                position: Vec2::new(1.0, 6.0),
                stats: UnitStats::default(),
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
        assert!(app.world().get::<UnitState>(player).unwrap().health() < 480.0);
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
        assert_eq!(unit_count(&mut app), 2);
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
        for (unit, state, pose) in app
            .world_mut()
            .query::<(&Unit, &UnitState, &UnitPose)>()
            .iter(app.world())
        {
            assert_eq!(state.health(), 400.0);
            assert_eq!(state.mana(), unit.definition().max_mana);
            assert!(
                matches!(pose.position(), point if point == Vec2::new(3.0, -2.0) || point == Vec2::new(-5.0, -6.0))
            );
        }
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
        assert_eq!(unit_count(&mut app), 3);
        assert!(!app.world().resource::<SimulationControl>().paused);
        // Pointer Lock's event already requested pause, before collect_input
        // has copied that request into SimulationControl this frame.
        apply_request(
            app.world_mut(),
            SessionRequest::ReturnToSelection { player_id: id },
            PAUSED_UNLOCKED,
        )
        .unwrap();
        assert_eq!(unit_count(&mut app), 2);
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
        assert_eq!(unit_count(&mut app), 3);
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
        assert_eq!(unit_count(&mut app), 2);
    }

    #[test]
    fn return_clears_input_feedback_and_restarts_patrol_when_tick_zero_is_reused() {
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
            .add_systems(
                Update,
                (controls::collect_input, controls::patrol_units).chain(),
            );
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        let selected = choice(&app);
        apply_request(app.world_mut(), launch(selected), CAPTURED).unwrap();
        let player = app.world().resource::<ClientSession>().player().unwrap();
        app.world_mut().run_schedule(Update); // patrol records tick 0
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
        let runner = app
            .world_mut()
            .query::<(&Unit, &UnitPose, &UnitOrder)>()
            .iter(app.world())
            .find(|(unit, _, _)| unit.name() == "巡游哨兵")
            .unwrap();
        assert!(matches!(runner.2, UnitOrder::MoveTo(_)));
        assert!(
            runner.1.position().x > -5.0,
            "new NPC patrol cannot inherit the old round's tick cache"
        );
    }
}
