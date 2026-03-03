use bevy::prelude::*;
use crate::game::card::Card;
use crate::game::player::PlayerId;

// ── 逻辑层 → 渲染层 / 网络层 ──────────────────────────────

/// 一张牌被发给某玩家
#[derive(Message, Clone, Debug)]
pub struct CardDealt {
    pub player_id: PlayerId,
    pub card: Card,
}

/// 回合切换
#[derive(Message, Clone, Debug)]
pub struct TurnChanged {
    pub player_id: PlayerId,
}

/// 某玩家选择不要牌
#[derive(Message, Clone, Debug)]
pub struct PlayerStood {
    #[allow(dead_code)]
    pub player_id: PlayerId,
}

/// 本局游戏结束
#[derive(Message, Clone, Debug)]
pub struct RoundOver {
    pub winners: Vec<PlayerId>,
    #[allow(dead_code)]
    pub scores: [u8; 4], // 每个座位的得分，空座位为 0
}

/// 新一局开始（牌已重置）
#[derive(Message, Clone, Debug)]
pub struct RoundReset;

// ── 渲染层 / 网络层 → 逻辑层 ──────────────────────────────

/// 玩家动作指令（人类输入或AI决策或网络收到）
#[derive(Message, Clone, Debug)]
pub enum GameAction {
    Draw { player_id: PlayerId },
    Stand { player_id: PlayerId },
    #[allow(dead_code)]
    Quit,
    StartGame,
}
