pub mod ai;
pub mod card;
pub mod events;
pub mod player;
pub mod room;
pub mod rules;

use bevy::prelude::*;
use events::*;
use room::{GamePhase, Room};

/// 系统集合标签：逻辑层必须在此集合内运行完后，渲染层才能响应消息
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameLogicSet;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Room>()
            .add_message::<CardDealt>()
            .add_message::<TurnChanged>()
            .add_message::<PlayerStood>()
            .add_message::<RoundOver>()
            .add_message::<RoundReset>()
            .add_message::<GameAction>()
            .add_plugins(ai::AiPlugin)
            .add_systems(Update, process_game_actions.in_set(GameLogicSet));
    }
}

/// 核心逻辑系统：处理 GameAction，更新 Room，发出逻辑事件
fn process_game_actions(
    mut actions: MessageReader<GameAction>,
    mut room: ResMut<Room>,
    mut card_dealt: MessageWriter<CardDealt>,
    mut turn_changed: MessageWriter<TurnChanged>,
    mut player_stood: MessageWriter<PlayerStood>,
    mut round_over: MessageWriter<RoundOver>,
    mut round_reset: MessageWriter<RoundReset>,
) {
    for action in actions.read() {
        match action {
            GameAction::Draw { player_id } => {
                if room.phase != GamePhase::Playing {
                    continue;
                }
                let pid = *player_id;
                // 验证是否轮到该玩家
                if room.current_turn != pid as usize {
                    continue;
                }
                if let Some(card) = room.deck.pop() {
                    if let Some(player) = room.seats[pid as usize].as_mut() {
                        player.take_card(card);
                        card_dealt.write(CardDealt {
                            player_id: pid,
                            card,
                        });
                    }
                    if rules::is_round_over(&room.seats, room.deck.remaining() == 0) {
                        emit_round_over(&room, &mut round_over);
                        room.phase = GamePhase::GameOver;
                    } else {
                        let next = room.advance_turn();
                        turn_changed.write(TurnChanged { player_id: next });
                    }
                }
            }
            GameAction::Stand { player_id } => {
                if room.phase != GamePhase::Playing {
                    continue;
                }
                let pid = *player_id;
                if room.current_turn != pid as usize {
                    continue;
                }
                if let Some(player) = room.seats[pid as usize].as_mut() {
                    player.is_stand = true;
                    player_stood.write(PlayerStood { player_id: pid });
                }
                if rules::is_round_over(&room.seats, room.deck.remaining() == 0) {
                    emit_round_over(&room, &mut round_over);
                    room.phase = GamePhase::GameOver;
                } else {
                    let next = room.advance_turn();
                    turn_changed.write(TurnChanged { player_id: next });
                }
            }
            GameAction::StartGame => {
                room.reset_round();
                let player_ids: Vec<u8> = room
                    .seats
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, s)| s.as_ref().map(|_| idx as u8))
                    .collect();
                // 先发出 RoundReset，让渲染层清除旧牌、重置计数，
                // 再写 CardDealt，确保新牌生成时旧状态已清理
                round_reset.write(RoundReset);
                // 每人轮流发两张牌
                for _ in 0..2 {
                    for &pid in &player_ids {
                        if let Some(card) = room.deck.pop() {
                            if let Some(player) = room.seats[pid as usize].as_mut() {
                                player.take_card(card);
                                card_dealt.write(CardDealt {
                                    player_id: pid,
                                    card,
                                });
                            }
                        }
                    }
                }
                let first = room.first_occupied_seat() as u8;
                turn_changed.write(TurnChanged { player_id: first });
            }
            GameAction::Quit => {
                // 由渲染层处理 WorldState 切换，逻辑层不感知
            }
        }
    }
}

fn emit_round_over(room: &Room, writer: &mut MessageWriter<RoundOver>) {
    let winners = rules::judge_winner(&room.seats);
    let mut scores = [0u8; 4];
    for (i, seat) in room.seats.iter().enumerate() {
        if let Some(p) = seat {
            scores[i] = rules::calc_score(&p.hand);
        }
    }
    writer.write(RoundOver { winners, scores });
}
