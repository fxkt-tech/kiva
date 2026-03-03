use bevy::prelude::*;
use crate::game::events::{GameAction, TurnChanged};
use crate::game::room::Room;
use crate::game::rules::calc_score;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ai_decision);
    }
}

/// AI 决策系统：监听 TurnChanged，若当前玩家是 AI，自动产生 GameAction
/// 策略：分数 < 17 则抽牌，否则不要牌
fn ai_decision(
    mut turn_ev: MessageReader<TurnChanged>,
    room: Res<Room>,
    mut actions: MessageWriter<GameAction>,
) {
    for ev in turn_ev.read() {
        let Some(player) = &room.seats[ev.player_id as usize] else {
            continue;
        };
        if !player.is_ai {
            continue;
        }

        let score = calc_score(&player.hand);
        let action = if score < 17 {
            GameAction::Draw {
                player_id: ev.player_id,
            }
        } else {
            GameAction::Stand {
                player_id: ev.player_id,
            }
        };
        actions.write(action);
    }
}
