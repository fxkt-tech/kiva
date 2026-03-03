use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::card::Deck;
use crate::game::player::{Player, PlayerId};

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum GamePhase {
    #[default]
    WaitingForPlayers,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct Room {
    pub seats: [Option<Player>; 4],
    pub deck: Deck,
    pub current_turn: usize, // 当前回合的 seat index
    pub phase: GamePhase,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            seats: [None, None, None, None],
            deck: Deck::new(),
            current_turn: 0,
            phase: GamePhase::WaitingForPlayers,
        }
    }
}

impl Room {
    /// 在指定座位坐下一个玩家
    pub fn seat_player(&mut self, seat: usize, player: Player) {
        self.seats[seat] = Some(player);
    }

    /// 返回当前回合玩家的可变引用
    #[allow(dead_code)]
    pub fn current_player_mut(&mut self) -> Option<&mut Player> {
        self.seats[self.current_turn].as_mut()
    }

    /// 推进到下一个有人的座位，返回新的 current_turn（seat index）
    pub fn advance_turn(&mut self) -> PlayerId {
        let total = self.seats.len();
        for i in 1..=total {
            let next = (self.current_turn + i) % total;
            if self.seats[next].is_some() {
                self.current_turn = next;
                return next as PlayerId;
            }
        }
        self.current_turn as PlayerId
    }

    /// 重置为新一局（保留玩家，重置手牌和牌组）
    pub fn reset_round(&mut self) {
        self.deck.reset();
        for seat in self.seats.iter_mut().flatten() {
            seat.reset_for_new_round();
        }
        self.current_turn = self.first_occupied_seat();
        self.phase = GamePhase::Playing;
    }

    pub fn first_occupied_seat(&self) -> usize {
        self.seats.iter().position(|s| s.is_some()).unwrap_or(0)
    }

    pub fn active_player_count(&self) -> usize {
        self.seats.iter().filter(|s| s.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player::Player;

    #[test]
    fn seat_player_fills_slot() {
        let mut room = Room::default();
        room.seat_player(0, Player::new(0, "Alice", false));
        assert!(room.seats[0].is_some());
    }

    #[test]
    fn advance_turn_skips_empty_seats() {
        let mut room = Room::default();
        room.seat_player(0, Player::new(0, "Alice", false));
        room.seat_player(2, Player::new(2, "Bob", false));
        room.current_turn = 0;
        let next = room.advance_turn();
        assert_eq!(next, 2); // 跳过空的 seat 1
    }

    #[test]
    fn reset_round_clears_hands() {
        let mut room = Room::default();
        let mut p = Player::new(0, "Alice", false);
        p.take_card(crate::game::card::Card { rank: 5 });
        room.seat_player(0, p);
        room.reset_round();
        assert!(room.seats[0].as_ref().unwrap().hand.is_empty());
    }
}
