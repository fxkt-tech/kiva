use crate::game::card::Card;

pub type PlayerId = u8; // 0-3

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub hand: Vec<Card>,
    pub is_ai: bool,
    pub is_stand: bool,
    #[allow(dead_code)]
    pub is_connected: bool,
}

impl Player {
    pub fn new(id: PlayerId, name: impl Into<String>, is_ai: bool) -> Self {
        Player {
            id,
            name: name.into(),
            hand: vec![],
            is_ai,
            is_stand: false,
            is_connected: true,
        }
    }

    pub fn take_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    #[allow(dead_code)]
    pub fn clear_hand(&mut self) {
        self.hand.clear();
    }

    pub fn reset_for_new_round(&mut self) {
        self.hand.clear();
        self.is_stand = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_has_empty_hand() {
        let p = Player::new(0, "Alice", false);
        assert!(p.hand.is_empty());
        assert!(!p.is_stand);
    }

    #[test]
    fn take_card_adds_to_hand() {
        let mut p = Player::new(0, "Alice", false);
        p.take_card(Card { rank: 5 });
        assert_eq!(p.hand.len(), 1);
        assert_eq!(p.hand[0].rank, 5);
    }

    #[test]
    fn reset_clears_hand_and_stand() {
        let mut p = Player::new(0, "Alice", false);
        p.take_card(Card { rank: 7 });
        p.is_stand = true;
        p.reset_for_new_round();
        assert!(p.hand.is_empty());
        assert!(!p.is_stand);
    }
}
