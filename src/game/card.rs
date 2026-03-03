use rand::seq::SliceRandom;
use rand::rng;
use serde::{Deserialize, Serialize};

/// 一张牌，只含点数，不含任何渲染信息
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8, // 1-11
}

/// 牌组
pub struct Deck {
    cards: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    pub fn new() -> Self {
        let mut cards: Vec<Card> = (1..=11).map(|rank| Card { rank }).collect();
        let mut rng = rng();
        cards.shuffle(&mut rng);
        Deck { cards }
    }

    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_11_cards() {
        let deck = Deck::new();
        assert_eq!(deck.cards.len(), 11);
    }

    #[test]
    fn deck_contains_ranks_1_to_11() {
        let deck = Deck::new();
        let mut ranks: Vec<u8> = deck.cards.iter().map(|c| c.rank).collect();
        ranks.sort();
        assert_eq!(ranks, (1u8..=11).collect::<Vec<_>>());
    }

    #[test]
    fn pop_reduces_count() {
        let mut deck = Deck::new();
        deck.pop();
        assert_eq!(deck.remaining(), 10);
    }

    #[test]
    fn reset_restores_11_cards() {
        let mut deck = Deck::new();
        deck.pop();
        deck.pop();
        deck.reset();
        assert_eq!(deck.remaining(), 11);
    }
}
