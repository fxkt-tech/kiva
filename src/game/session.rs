use rand::seq::SliceRandom;

use crate::game::domain::{Camp, Player, PlayerId, PlayerKind, PlayerProfile, Role};
use crate::game::player_pool::PlayerPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winner {
    Good,
    Werewolf,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub players: Vec<Player>,
}

impl GameSession {
    #[allow(dead_code)]
    pub fn new_with_roles(roles: Vec<Role>) -> Self {
        let profiles = (1..=roles.len())
            .map(|seat| PlayerProfile {
                name: format!("AI-{seat:02}"),
                personality_preference: "基础 AI 玩家".to_string(),
                avatar: "AI".to_string(),
                ai: crate::game::domain::PlayerAiConfig {
                    base_url: String::new(),
                    api_key: String::new(),
                    model: String::new(),
                    system_prompt: "你是基础 AI 玩家。".to_string(),
                },
            })
            .collect();

        Self::new_with_roles_and_profiles(roles, profiles)
    }

    pub fn new_random_from_pool<R: rand::Rng + ?Sized>(mut roles: Vec<Role>, rng: &mut R) -> Self {
        roles.shuffle(rng);
        let profiles = PlayerPool::default()
            .draw_nine(rng)
            .expect("default player pool must contain at least nine players");

        Self::new_with_roles_and_profiles(roles, profiles)
    }

    pub fn new_with_roles_and_profiles(roles: Vec<Role>, profiles: Vec<PlayerProfile>) -> Self {
        assert_eq!(
            roles.len(),
            profiles.len(),
            "roles and player profiles must have the same length"
        );

        let players = roles
            .into_iter()
            .zip(profiles)
            .enumerate()
            .map(|(index, (role, profile))| {
                let seat = index + 1;
                Player {
                    id: PlayerId(seat),
                    name: profile.name,
                    personality_preference: profile.personality_preference,
                    avatar: profile.avatar,
                    ai: profile.ai,
                    role,
                    kind: PlayerKind::Ai,
                    alive: true,
                }
            })
            .collect();

        Self { players }
    }

    pub fn alive_players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|player| player.alive)
    }

    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players.iter().find(|player| player.id == id)
    }

    pub fn player_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        self.players.iter_mut().find(|player| player.id == id)
    }

    pub fn winner(&self) -> Option<Winner> {
        let living_wolves = self
            .alive_players()
            .filter(|player| player.role == Role::Werewolf)
            .count();
        let living_good = self
            .alive_players()
            .filter(|player| player.role.camp() == Camp::Good)
            .count();

        if living_wolves == 0 {
            Some(Winner::Good)
        } else if living_wolves >= living_good {
            Some(Winner::Werewolf)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{PlayerKind, Role};

    #[test]
    fn session_creates_nine_ai_players() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        assert_eq!(session.players.len(), 9);
        assert_eq!(
            session
                .players
                .iter()
                .filter(|player| player.kind == PlayerKind::Ai)
                .count(),
            9
        );
    }

    #[test]
    fn good_wins_when_all_wolves_are_dead() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());
        for player in &mut session.players {
            if player.role == Role::Werewolf {
                player.alive = false;
            }
        }
        assert_eq!(session.winner(), Some(Winner::Good));
    }

    #[test]
    fn wolves_win_when_wolves_equal_good_count() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut living_good = 0;
        for player in &mut session.players {
            if player.role == Role::Werewolf {
                player.alive = true;
            } else if living_good < 3 {
                player.alive = true;
                living_good += 1;
            } else {
                player.alive = false;
            }
        }
        assert_eq!(session.winner(), Some(Winner::Werewolf));
    }
}
