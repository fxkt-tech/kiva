use bevy::prelude::*;
use crate::game::events::TurnChanged;
use crate::game::room::Room;
use crate::game::rules::calc_score;
use super::table::{ScoreLabel, TurnArrow, ARROW_ACTIVE, ARROW_DIM};

/// 分数更新：
/// - 本地玩家（seat 0）：显示真实总分
/// - 其他玩家：第一张牌扣着，显示 "? + X"（X 为可见牌之和）
pub fn update_scores(room: Res<Room>, mut query: Query<(&mut Text, &ScoreLabel)>) {
    if !room.is_changed() {
        return;
    }
    for (mut text, label) in &mut query {
        let seat_idx = label.0;
        *text = match room.seats[seat_idx].as_ref() {
            None => Text::new("—"),
            Some(p) if seat_idx == 0 => Text::new(format!("{} 分", calc_score(&p.hand))),
            Some(p) => match p.hand.len() {
                0 => Text::new("—"),
                1 => Text::new("?"),
                _ => {
                    let visible = calc_score(&p.hand[1..]);
                    Text::new(format!("? + {}", visible))
                }
            },
        };
    }
}

/// 回合切换：高亮铜钱上指向当前出牌玩家的方向箭头
pub fn on_turn_changed(
    mut ev: MessageReader<TurnChanged>,
    mut query: Query<(&mut TextColor, &TurnArrow)>,
) {
    for ev in ev.read() {
        let active_seat = ev.player_id as usize;
        for (mut color, arrow) in &mut query {
            *color = if arrow.seat == active_seat {
                TextColor(ARROW_ACTIVE)
            } else {
                TextColor(ARROW_DIM)
            };
        }
    }
}
