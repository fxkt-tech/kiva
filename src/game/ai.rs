use bevy::prelude::*;
use rand::random;
use crate::game::events::{GameAction, TurnChanged};
use crate::game::room::Room;
use crate::game::rules::calc_score;

/// AI 待执行动作：决策后延迟 1-2 秒再发出
#[derive(Resource, Default)]
struct AiPending {
    action: Option<GameAction>,
    timer: Option<Timer>,
}

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiPending>()
            .add_systems(Update, ai_schedule)
            .add_systems(Update, ai_execute);
    }
}

/// 监听 TurnChanged，若当前玩家是 AI，将决策结果存入 AiPending 并启动随机延迟计时器
/// 策略：分数 < 17 则抽牌，否则不要牌
fn ai_schedule(
    mut turn_ev: MessageReader<TurnChanged>,
    room: Res<Room>,
    mut pending: ResMut<AiPending>,
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

        // 随机延迟 1.0 ~ 2.0 秒
        let delay = 1.0 + random::<f32>();
        pending.action = Some(action);
        pending.timer = Some(Timer::from_seconds(delay, TimerMode::Once));
    }
}

/// 每帧推进计时器，到期后发出 GameAction
fn ai_execute(
    time: Res<Time>,
    mut pending: ResMut<AiPending>,
    mut actions: MessageWriter<GameAction>,
) {
    let Some(timer) = pending.timer.as_mut() else {
        return;
    };
    timer.tick(time.delta());
    if timer.just_finished() {
        if let Some(action) = pending.action.take() {
            actions.write(action);
        }
        pending.timer = None;
    }
}
