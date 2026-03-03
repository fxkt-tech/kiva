use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use bevy::prelude::*;
use crate::game::events::GameAction;
use crate::game::player::PlayerId;
use crate::net::protocol::{self, NetAction, NetMessage, NET_PORT};
use crate::states::WorldState;

/// Host 模式下的网络资源
#[derive(Resource)]
pub struct HostNet {
    pub socket: Arc<UdpSocket>,
    pub clients: Arc<Mutex<Vec<Option<String>>>>,
}

pub struct HostPlugin;

impl Plugin for HostPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WorldState::Lobby), start_host)
            .add_systems(
                Update,
                (receive_client_actions, broadcast_beacon)
                    .run_if(resource_exists::<HostNet>),
            )
            .add_systems(OnExit(WorldState::Game), stop_host);
    }
}

fn start_host(mut commands: Commands) {
    let addr = format!("0.0.0.0:{}", NET_PORT);
    match UdpSocket::bind(&addr) {
        Ok(socket) => {
            socket.set_nonblocking(true).ok();
            info!("Host listening on {}", addr);
            commands.insert_resource(HostNet {
                socket: Arc::new(socket),
                clients: Arc::new(Mutex::new(vec![None; 4])),
            });
        }
        Err(e) => error!("Failed to bind host socket: {}", e),
    }
}

fn stop_host(mut commands: Commands) {
    commands.remove_resource::<HostNet>();
}

fn receive_client_actions(host: Res<HostNet>, mut actions: MessageWriter<GameAction>) {
    let mut buf = [0u8; 1024];
    loop {
        match host.socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(msg) = protocol::decode(&buf[..len]) {
                    match msg {
                        NetMessage::JoinRequest { name } => {
                            info!("Join request from {} ({})", name, src);
                        }
                        NetMessage::PlayerAction { player_id, action_type } => {
                            let action = match action_type {
                                NetAction::Draw => GameAction::Draw { player_id },
                                NetAction::Stand => GameAction::Stand { player_id },
                            };
                            actions.write(action);
                        }
                        _ => {}
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                error!("Host recv error: {}", e);
                break;
            }
        }
    }
}

fn broadcast_beacon(host: Res<HostNet>, time: Res<Time>, mut timer: Local<f32>) {
    *timer += time.delta_secs();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;

    let beacon = protocol::RoomBeacon {
        room_name: "四方诛杀".to_string(),
        host_addr: format!("0.0.0.0:{}", NET_PORT),
        player_count: 1,
        max_players: 4,
    };
    // 将 beacon 包装成 NetMessage 广播
    let msg = NetMessage::JoinRequest { name: format!("beacon:{}", beacon.room_name) };
    let bytes = protocol::encode(&msg);
    let broadcast_addr = format!("255.255.255.255:{}", protocol::BEACON_PORT);
    host.socket.send_to(&bytes, &broadcast_addr).ok();
}

/// 广播 GameAction 给所有已连接客户端
#[allow(dead_code)]
pub fn broadcast_action(host: &HostNet, player_id: PlayerId, action: &NetAction) {
    let msg = NetMessage::ActionBroadcast {
        player_id,
        action_type: action.clone(),
    };
    let bytes = protocol::encode(&msg);
    if let Ok(clients) = host.clients.lock() {
        for addr in clients.iter().flatten() {
            host.socket.send_to(&bytes, addr).ok();
        }
    }
}
