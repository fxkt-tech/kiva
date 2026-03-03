use crate::game::card::Card;
use crate::game::player::{Player, PlayerId};

pub fn calc_score(hand: &[Card]) -> u8 {
    hand.iter().map(|c| c.rank).sum()
}

#[allow(dead_code)]
pub fn is_bust(hand: &[Card]) -> bool {
    calc_score(hand) > 21
}

/// 返回胜者的 PlayerId 列表（可能平局多人）
/// 规则：爆牌者淘汰，剩余者中点数最高者胜，同分平局
pub fn judge_winner(seats: &[Option<Player>; 4]) -> Vec<PlayerId> {
    let scores: Vec<(PlayerId, u8)> = seats
        .iter()
        .enumerate()
        .filter_map(|(_i, seat)| {
            seat.as_ref().map(|p| (p.id, calc_score(&p.hand)))
        })
        .filter(|(_, score)| *score <= 21)
        .collect();

    if scores.is_empty() {
        // 所有人都爆牌，返回所有参与者（平局）
        return seats
            .iter()
            .filter_map(|s| s.as_ref().map(|p| p.id))
            .collect();
    }

    let max_score = scores.iter().map(|(_, s)| *s).max().unwrap_or(0);
    scores
        .into_iter()
        .filter(|(_, s)| *s == max_score)
        .map(|(id, _)| id)
        .collect()
}

/// 游戏结束条件：所有在座玩家都 stand，或牌组耗尽
pub fn is_round_over(seats: &[Option<Player>; 4], deck_empty: bool) -> bool {
    if deck_empty {
        return true;
    }
    seats.iter().filter_map(|s| s.as_ref()).all(|p| p.is_stand)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::Card;
    use crate::game::player::Player;

    fn card(rank: u8) -> Card {
        Card { rank }
    }
    fn player_with_hand(id: u8, ranks: &[u8]) -> Player {
        let mut p = Player::new(id, "test", false);
        for &r in ranks {
            p.take_card(card(r));
        }
        p
    }

    #[test]
    fn calc_score_sums_ranks() {
        assert_eq!(calc_score(&[card(5), card(7)]), 12);
    }

    #[test]
    fn is_bust_over_21() {
        assert!(is_bust(&[card(11), card(11)]));
        assert!(!is_bust(&[card(10), card(11)]));
    }

    #[test]
    fn judge_winner_highest_wins() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        seats[0] = Some(player_with_hand(0, &[10, 8])); // 18
        seats[1] = Some(player_with_hand(1, &[9, 9])); // 18
        seats[2] = Some(player_with_hand(2, &[7, 7])); // 14
        let winners = judge_winner(&seats);
        assert_eq!(winners.len(), 2); // 平局
        assert!(winners.contains(&0));
        assert!(winners.contains(&1));
    }

    #[test]
    fn judge_winner_bust_excluded() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        seats[0] = Some(player_with_hand(0, &[11, 11])); // 22 爆牌
        seats[1] = Some(player_with_hand(1, &[9, 8])); // 17
        let winners = judge_winner(&seats);
        assert_eq!(winners, vec![1]);
    }

    #[test]
    fn judge_winner_all_bust_returns_all() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        seats[0] = Some(player_with_hand(0, &[11, 11]));
        seats[1] = Some(player_with_hand(1, &[11, 11]));
        let winners = judge_winner(&seats);
        assert_eq!(winners.len(), 2);
    }

    #[test]
    fn round_over_when_all_stand() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        let mut p0 = Player::new(0, "a", false);
        let mut p1 = Player::new(1, "b", false);
        p0.is_stand = true;
        p1.is_stand = true;
        seats[0] = Some(p0);
        seats[1] = Some(p1);
        assert!(is_round_over(&seats, false));
    }
}
