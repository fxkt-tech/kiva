//! Owns the transition from a confirmed character choice to one active player.
//! Device capture is an input fact; simulation pause and gameplay input remain
//! the controls adapter's responsibility.
use super::{content::SelectionSettings, web};
use crate::{
    gameplay::{self, MatchId, MatchPhase, MatchState},
    simulation::*,
};
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
        match_id: MatchId,
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
    RestartMatch {
        match_id: String,
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
    let default = world.resource::<SelectionSettings>().default_character;
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
        .expect("registered default match character");
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
    .expect("valid default match character spawn");
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
    if let SessionRequest::RestartMatch {
        match_id,
        player_id,
    } = &request
    {
        let result = restart_match(world, match_id, player_id, facts);
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
            } else if !facts.focused || !facts.capture.confirmed() {
                Some("鼠标捕获已取消，请重新开始。")
            } else {
                None
            };
            if let Some(error) = error {
                set_error(world, error.into());
                return Err(error.into());
            }
            match gameplay::start_match(world, choice.character_id) {
                Ok(handle) => {
                    *world.resource_mut::<ClientSession>() = ClientSession::Active {
                        player: handle.player,
                        selection_revision,
                        match_id: handle.id,
                    };
                    world.resource_mut::<SimulationControl>().paused = false;
                    Ok(())
                }
                Err(error) => {
                    let error = format!("无法开始对局：{error}");
                    set_error(world, error.clone());
                    Err(error)
                }
            }
        }
        SessionRequest::ReturnToSelection { .. } | SessionRequest::RestartMatch { .. } => {
            unreachable!("handled before the Active gate")
        }
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
        ..
    } = *session
    else {
        return Ok(());
    };
    if player.to_bits().to_string() != player_id {
        return Err("这场对局已经结束。".into());
    }
    let paused =
        world.resource::<SimulationControl>().paused || facts.pause_requested || !facts.focused;
    if facts.capture.actual || !paused {
        return Err("请先暂停对局并释放鼠标。".into());
    }
    let character_id = world
        .get::<CharacterIdentity>(player)
        .map(CharacterIdentity::id)
        .ok_or("当前角色不可用。")?;
    let revision = selection_revision
        .checked_add(1)
        .ok_or("选择版本已耗尽，请重新加载。")?;
    gameplay::reset_match(world);
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

fn restart_match(
    world: &mut World,
    expected_match: &str,
    expected_player: &str,
    facts: CaptureFacts,
) -> Result<(), String> {
    let ClientSession::Active {
        player,
        match_id,
        selection_revision,
    } = *world.resource::<ClientSession>()
    else {
        return Err("这场对局已经结束。".into());
    };
    let state = world.resource::<MatchState>();
    if match_id.to_string() != expected_match
        || player.to_bits().to_string() != expected_player
        || state.id() != Some(match_id)
        || !matches!(state.phase(), MatchPhase::Finished(_))
    {
        return Err("对局已变更，请使用当前结果界面重开。".into());
    }
    if !facts.focused || !facts.capture.confirmed() {
        return Err("请重新点击再来一局并捕获鼠标。".into());
    }
    let character = state.player_character().ok_or("当前角色不可用。")?;
    let revision = selection_revision
        .checked_add(1)
        .ok_or("选择版本已耗尽。")?;
    gameplay::reset_match(world);
    match gameplay::start_match(world, character) {
        Ok(handle) => {
            *world.resource_mut::<ClientSession>() = ClientSession::Active {
                player: handle.player,
                match_id: handle.id,
                selection_revision: revision,
            };
            world.resource_mut::<SimulationControl>().paused = false;
            Ok(())
        }
        Err(error) => {
            // A failed new roster must never leave an Active binding pointing
            // at the old despawned hero. Keep the choice available for retry.
            *world.resource_mut::<ClientSession>() = ClientSession::Selecting(Selection {
                selected: Some(CharacterChoice {
                    character_id: character,
                    revision,
                }),
                ack: SelectionAck {
                    request_id: 0,
                    accepted: true,
                },
                error: Some(format!("无法重新开局：{error}")),
            });
            Err(error.to_string())
        }
    }
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
    (world.resource::<ClientSession>().player().is_none()
        || !world.resource::<MatchState>().can_advance())
        && facts.capture.confirmed()
}

fn process_batch(world: &mut World, raw: &str, facts: CaptureFacts) {
    match parse_requests(raw) {
        Ok(requests) => {
            for request in requests {
                let previous_player = world.resource::<ClientSession>().player();
                let _ = apply_request(world, request, facts);
                if previous_player.is_some()
                    && world.resource::<ClientSession>().player() != previous_player
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
mod tests;
