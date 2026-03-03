use serde::{Deserialize, Serialize};
use crate::game::card::Card;
use crate::game::player::PlayerId;
use crate::game::room::GamePhase;

pub const NET_PORT: u16 = 7777;
pub const BEACON_PORT: u16 = 7778;

/// 玩家快照（可序列化，不含 Entity）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub name: String,
    pub hand: Vec<Card>,
    pub is_ai: bool,
    pub is_stand: bool,
}

/// 房间快照（全量状态，用于新客户端加入时同步）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomSnapshot {
    pub seats: [Option<PlayerSnapshot>; 4],
    pub deck_remaining: usize,
    pub current_turn: usize,
    pub phase: GamePhase,
}

/// 局域网发现 beacon
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomBeacon {
    pub room_name: String,
    pub host_addr: String,
    pub player_count: usize,
    pub max_players: usize,
}

/// 动作类型（可序列化）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetAction {
    Draw,
    Stand,
}

/// 网络消息枚举
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMessage {
    // 客户端 → Host
    JoinRequest { name: String },
    PlayerAction { player_id: PlayerId, action_type: NetAction },

    // Host → 客户端
    JoinAccepted { assigned_id: PlayerId, snapshot: RoomSnapshot },
    JoinRejected { reason: String },
    ActionBroadcast { player_id: PlayerId, action_type: NetAction },
    FullSync { snapshot: RoomSnapshot },
}

/// 序列化消息为字节
pub fn encode(msg: &NetMessage) -> Vec<u8> {
    bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap_or_default()
}

/// 反序列化字节为消息
pub fn decode(bytes: &[u8]) -> Option<NetMessage> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .ok()
        .map(|(msg, _)| msg)
}
