use crate::plugins::game::deck::Deck;
use crate::plugins::game::player::Player;
use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct Room {
    pub player1: Player,
    pub player2: Player,
    pub deck: Deck,
    my_turn: bool,
    pub player1_stand: bool,
    pub player2_stand: bool,

    pub entity: Option<Entity>,
}

impl Default for Room {
    fn default() -> Self {
        Self {
            player1: Player::default(),
            player2: Player::default(),
            deck: Deck::default(),
            my_turn: true,
            player1_stand: false,
            player2_stand: false,
            entity: None,
        }
    }
}

impl Room {
    pub fn current_turn_player(&mut self) -> &mut Player {
        if self.my_turn {
            &mut self.player1
        } else {
            &mut self.player2
        }
    }

    pub fn set_current_player_stand(&mut self, stand: bool) {
        if self.my_turn {
            self.player1_stand = stand;
        } else {
            self.player2_stand = stand;
        }
    }

    pub fn is_computer_turn(&self) -> bool {
        !self.my_turn
    }

    pub fn is_game_over(&self) -> bool {
        (self.player1_stand && self.player2_stand) || self.deck.cards.is_empty()
    }

    pub fn turn(&mut self) {
        self.my_turn = !self.my_turn;
    }

    pub fn clear(&mut self, mut commands: Commands) {
        self.player1.leave(&mut commands);
        self.player2.leave(&mut commands);
        self.deck.reset();
        self.player1_stand = false;
        self.player2_stand = false;
    }

    pub fn reset_game(&mut self) {
        // 清空玩家手牌但保留实体引用
        self.player1.cards.clear();
        self.player2.cards.clear();

        // 重置牌组
        self.deck.reset();

        // 重置玩家状态
        self.player1_stand = false;
        self.player2_stand = false;

        // 设置为玩家回合
        self.my_turn = true;
    }
}
