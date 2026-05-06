use crate::game::domain::{PlayerId, Role};
use crate::game::session::GameSession;

pub fn choose_wolf_target(session: &GameSession, actor: PlayerId) -> Option<PlayerId> {
    let actor_role = session.player(actor).map(|player| player.role);

    session
        .alive_players()
        .find(|player| Some(player.role) != actor_role && player.role != Role::Werewolf)
        .map(|player| player.id)
}

pub fn choose_seer_target(session: &GameSession, checked: &[PlayerId]) -> Option<PlayerId> {
    session
        .alive_players()
        .find(|player| !checked.contains(&player.id))
        .map(|player| player.id)
}

pub fn choose_vote_target(session: &GameSession, actor: PlayerId) -> Option<PlayerId> {
    session
        .alive_players()
        .find(|player| player.id != actor)
        .map(|player| player.id)
}

pub fn generate_speech(session: &GameSession, actor: PlayerId, day: u32) -> String {
    let Some(player) = session.player(actor) else {
        return "我先过。".to_string();
    };

    match player.role {
        Role::Werewolf => format!("{} 号：我觉得今天先听发言，不要太早定票。", actor.0),
        Role::Seer => format!("{} 号：第 {day} 天我会重点看投票和站边。", actor.0),
        Role::Witch => format!("{} 号：我先不跳身份，大家聊清楚怀疑点。", actor.0),
        Role::Hunter => format!("{} 号：我会看谁在强行带节奏。", actor.0),
        Role::Villager => format!("{} 号：目前信息还少，我先听后面发言。", actor.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{PlayerId, Role};
    use crate::game::session::GameSession;

    #[test]
    fn wolf_target_is_living_non_wolf() {
        let session = GameSession::new_with_roles(Role::nine_player_deck(), 1);

        let target = choose_wolf_target(&session, PlayerId(1)).unwrap();

        let player = session.player(target).unwrap();
        assert!(player.alive);
        assert_ne!(player.role, Role::Werewolf);
    }

    #[test]
    fn seer_target_is_living_and_unchecked() {
        let session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        let checked = vec![PlayerId(4), PlayerId(5)];

        let target = choose_seer_target(&session, &checked).unwrap();

        assert!(session.player(target).unwrap().alive);
        assert!(!checked.contains(&target));
    }

    #[test]
    fn vote_target_is_living_and_not_self() {
        let session = GameSession::new_with_roles(Role::nine_player_deck(), 1);

        let target = choose_vote_target(&session, PlayerId(2)).unwrap();

        assert!(session.player(target).unwrap().alive);
        assert_ne!(target, PlayerId(2));
    }

    #[test]
    fn speech_does_not_reveal_hidden_roles_for_villager() {
        let session = GameSession::new_with_roles(Role::nine_player_deck(), 1);

        let speech = generate_speech(&session, PlayerId(9), 1);

        assert!(!speech.contains("1号是狼人"));
        assert!(!speech.contains("1 号是狼人"));
        assert!(speech.contains("9 号"));
    }
}
