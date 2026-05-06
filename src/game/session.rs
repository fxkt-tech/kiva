use crate::game::domain::{Camp, Player, PlayerId, PlayerKind, Role};

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
    pub fn new_with_roles(roles: Vec<Role>, human_seat: usize) -> Self {
        let players = roles
            .into_iter()
            .enumerate()
            .map(|(index, role)| {
                let seat = index + 1;
                Player {
                    id: PlayerId(seat),
                    name: if seat == 1 {
                        "你".to_string()
                    } else {
                        format!("{seat} 号")
                    },
                    role,
                    kind: if seat == human_seat {
                        PlayerKind::Human
                    } else {
                        PlayerKind::Ai
                    },
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
    fn session_creates_nine_players_with_one_human() {
        let session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        assert_eq!(session.players.len(), 9);
        assert_eq!(
            session
                .players
                .iter()
                .filter(|player| player.kind == PlayerKind::Human)
                .count(),
            1
        );
    }

    #[test]
    fn good_wins_when_all_wolves_are_dead() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        for player in &mut session.players {
            if player.role == Role::Werewolf {
                player.alive = false;
            }
        }
        assert_eq!(session.winner(), Some(Winner::Good));
    }

    #[test]
    fn wolves_win_when_wolves_equal_good_count() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
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
