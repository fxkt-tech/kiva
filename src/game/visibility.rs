use crate::game::domain::{PlayerId, Role};
use crate::game::events::{EventLog, EventVisibility, LoggedEvent};
use crate::game::session::GameSession;

#[allow(dead_code)]
pub fn visible_events<'a>(
    session: &GameSession,
    viewer: PlayerId,
    log: &'a EventLog,
) -> Vec<&'a LoggedEvent> {
    log.events()
        .iter()
        .filter(|event| can_see(session, viewer, event.visibility))
        .collect()
}

#[allow(dead_code)]
fn can_see(session: &GameSession, viewer: PlayerId, visibility: EventVisibility) -> bool {
    match visibility {
        EventVisibility::Public => true,
        EventVisibility::Wolves => is_werewolf(session, viewer),
        EventVisibility::ActorOnly(actor) => viewer == actor,
        EventVisibility::WerewolfAndActor(actor) => viewer == actor || is_werewolf(session, viewer),
        EventVisibility::System => false,
    }
}

#[allow(dead_code)]
fn is_werewolf(session: &GameSession, viewer: PlayerId) -> bool {
    session
        .player(viewer)
        .map(|player| player.role == Role::Werewolf)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{Camp, PlayerId, Role};
    use crate::game::events::{EventLog, EventVisibility, GameEvent};
    use crate::game::session::GameSession;

    #[test]
    fn werewolf_sees_wolf_chat_but_villager_does_not() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut log = EventLog::default();
        log.append(
            GameEvent::WolfChat {
                night: 1,
                speaker: PlayerId(1),
                content: "刀 4。".to_string(),
            },
            EventVisibility::Wolves,
        );

        assert_eq!(visible_events(&session, PlayerId(1), &log).len(), 1);
        assert_eq!(visible_events(&session, PlayerId(7), &log).len(), 0);
    }

    #[test]
    fn seer_only_sees_own_check_result() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut log = EventLog::default();
        log.append(
            GameEvent::SeerChecked {
                night: 1,
                seer: PlayerId(4),
                target: PlayerId(1),
                camp: Camp::Werewolf,
            },
            EventVisibility::ActorOnly(PlayerId(4)),
        );

        assert_eq!(visible_events(&session, PlayerId(4), &log).len(), 1);
        assert_eq!(visible_events(&session, PlayerId(5), &log).len(), 0);
    }
}
