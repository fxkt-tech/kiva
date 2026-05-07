use crate::game::domain::{Camp, PlayerId};
use crate::game::session::GameSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathReason {
    WolfKill,
    WitchPoison,
    Exile,
    HunterShot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Death {
    pub player: PlayerId,
    pub reason: DeathReason,
}

impl Death {
    pub fn new(player: PlayerId, reason: DeathReason) -> Self {
        Self { player, reason }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NightActions {
    pub wolf_target: Option<PlayerId>,
    pub witch_save: bool,
    pub witch_poison_target: Option<PlayerId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NightResult {
    pub deaths: Vec<Death>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteCast {
    pub voter: PlayerId,
    pub target: PlayerId,
    pub reason: String,
}

pub fn tally_votes(votes: &[VoteCast]) -> Option<PlayerId> {
    let mut counts = std::collections::HashMap::<PlayerId, usize>::new();
    for vote in votes {
        *counts.entry(vote.target).or_default() += 1;
    }

    counts
        .into_iter()
        .max_by(|(left_target, left_count), (right_target, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_target.0.cmp(&left_target.0))
        })
        .map(|(target, _)| target)
}

pub fn resolve_night(session: &mut GameSession, actions: NightActions) -> NightResult {
    let mut deaths = Vec::new();

    if let Some(target) = actions.wolf_target
        && !actions.witch_save
    {
        kill(session, target);
        deaths.push(Death::new(target, DeathReason::WolfKill));
    }

    if let Some(target) = actions.witch_poison_target {
        kill(session, target);
        deaths.push(Death::new(target, DeathReason::WitchPoison));
    }

    NightResult { deaths }
}

pub fn check_camp(session: &GameSession, target: PlayerId) -> Option<Camp> {
    session.player(target).map(|player| player.role.camp())
}

#[allow(dead_code)]
pub fn hunter_can_shoot(reason: DeathReason) -> bool {
    matches!(reason, DeathReason::WolfKill | DeathReason::Exile)
}

pub fn resolve_hunter_shot(session: &mut GameSession, target: PlayerId) -> Death {
    kill(session, target);
    Death::new(target, DeathReason::HunterShot)
}

fn kill(session: &mut GameSession, target: PlayerId) {
    if let Some(player) = session.player_mut(target) {
        player.alive = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{Camp, PlayerId, Role};
    use crate::game::session::GameSession;

    #[test]
    fn night_kill_kills_target_unless_witch_saves() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());

        let killed = resolve_night(
            &mut session,
            NightActions {
                wolf_target: Some(PlayerId(4)),
                witch_save: false,
                ..NightActions::default()
            },
        );
        assert_eq!(
            killed.deaths,
            vec![Death::new(PlayerId(4), DeathReason::WolfKill)]
        );
        assert!(!session.player(PlayerId(4)).unwrap().alive);

        let mut saved_session = GameSession::new_with_roles(Role::nine_player_deck());
        let saved = resolve_night(
            &mut saved_session,
            NightActions {
                wolf_target: Some(PlayerId(4)),
                witch_save: true,
                ..NightActions::default()
            },
        );
        assert!(saved.deaths.is_empty());
        assert!(saved_session.player(PlayerId(4)).unwrap().alive);
    }

    #[test]
    fn seer_check_returns_target_camp() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());

        assert_eq!(check_camp(&session, PlayerId(1)), Some(Camp::Werewolf));
        assert_eq!(check_camp(&session, PlayerId(4)), Some(Camp::Good));
    }

    #[test]
    fn witch_poison_kills_target() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());

        let result = resolve_night(
            &mut session,
            NightActions {
                witch_poison_target: Some(PlayerId(5)),
                ..NightActions::default()
            },
        );

        assert_eq!(
            result.deaths,
            vec![Death::new(PlayerId(5), DeathReason::WitchPoison)]
        );
        assert!(!session.player(PlayerId(5)).unwrap().alive);
    }

    #[test]
    fn hunter_can_shoot_after_exile_or_wolf_kill_but_not_poison() {
        assert!(hunter_can_shoot(DeathReason::Exile));
        assert!(hunter_can_shoot(DeathReason::WolfKill));
        assert!(!hunter_can_shoot(DeathReason::WitchPoison));
    }

    #[test]
    fn hunter_shot_kills_target() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());

        let death = resolve_hunter_shot(&mut session, PlayerId(6));

        assert_eq!(death, Death::new(PlayerId(6), DeathReason::HunterShot));
        assert!(!session.player(PlayerId(6)).unwrap().alive);
    }

    #[test]
    fn vote_tally_exiles_highest_vote_target() {
        let votes = vec![
            VoteCast {
                voter: PlayerId(1),
                target: PlayerId(4),
                reason: "a".to_string(),
            },
            VoteCast {
                voter: PlayerId(2),
                target: PlayerId(4),
                reason: "b".to_string(),
            },
            VoteCast {
                voter: PlayerId(3),
                target: PlayerId(5),
                reason: "c".to_string(),
            },
        ];

        assert_eq!(tally_votes(&votes), Some(PlayerId(4)));
    }
}
