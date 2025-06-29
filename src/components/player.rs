use crate::components::deck::Card;
use bevy::ecs::{component::Component, entity::Entity, system::Commands};

// 玩家
#[derive(Component, Clone)]
pub struct Player {
    pub cards: Vec<Card>,

    pub entity: Option<Entity>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            cards: Vec::new(),
            entity: None,
        }
    }

    pub fn insert_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn leave(&mut self, commands: &mut Commands) {
        if let Some(entity) = self.entity {
            commands.entity(entity).despawn();
        }
        self.cards = Vec::new();
        self.entity = None;
    }
}
