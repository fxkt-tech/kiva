use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use rand::rng;
use rand::seq::SliceRandom;

// 普通牌
#[derive(Component, Clone, Copy)]
pub struct Card {
    rank: u8,

    pub entity: Option<Entity>,
}

impl Card {
    pub fn get_rank(&self) -> u8 {
        self.rank
    }
}

// 卡组
#[derive(Component, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

// 牌桌
impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for rank in 1..=11 {
            cards.push(Card { rank, entity: None });
        }
        // 打乱顺序
        let mut rng = rng();
        cards.shuffle(&mut rng);
        Deck { cards }
    }

    // 抽卡。需要考虑卡组是否为空
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn reset(&mut self) {
        self.cards.clear();
        for rank in 1..=11 {
            self.cards.push(Card { rank, entity: None });
        }
        // 打乱顺序
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }
}
