use crate::plugins::game::deck::Card;
use bevy::ecs::{component::Component, entity::Entity, system::Commands};

// 玩家
#[derive(Component, Clone)]
pub struct Player {
    pub cards: Vec<Card>,

    pub entity: Option<Entity>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            cards: Vec::new(),
            entity: None,
        }
    }
}

impl Player {
    // 要一张牌
    pub fn give_me(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// 计算手牌点数（21点规则，A算1或11，JQK算10，普通牌按点数）
    pub fn get_score(&self) -> u8 {
        // 这里只用1-11，直接累加
        self.cards.iter().map(|c| c.get_rank()).sum()
    }

    // 离开房间
    pub fn leave(&mut self, commands: &mut Commands) {
        if let Some(entity) = self.entity {
            commands.entity(entity).despawn();
        }
        self.cards = Vec::new();
        self.entity = None;
    }
}
