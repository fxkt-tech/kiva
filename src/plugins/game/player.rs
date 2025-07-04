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

    /// 计算手牌点数
    pub fn get_score(&self) -> u8 {
        self.cards.iter().map(|c| c.get_rank()).sum()
    }

    pub fn clear_cards(&mut self, commands: &mut Commands) {
        println!("clear_cards");
        for card in self.cards.iter() {
            println!("card: {}", card.get_rank());
            if let Some(entity) = card.entity {
                println!("despawn card: {:?}", entity);
                commands.entity(entity).despawn();
            }
        }
        self.cards.clear();
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
