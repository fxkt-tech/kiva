use std::net::UdpSocket;
use bevy::prelude::*;
use crate::game::events::GameAction;
use crate::game::player::PlayerId;
use crate::net::protocol::{self, NetAction, NetMessage};

#[derive(Resource)]
#[allow(dead_code)]
pub struct ClientNet {
    pub socket: UdpSocket,
    pub host_addr: String,
    pub local_player_id: PlayerId,
}

#[allow(dead_code)]
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            receive_from_host.run_if(resource_exists::<ClientNet>),
        );
    }
}

#[allow(dead_code)]
pub fn connect_to_host(commands: &mut Commands, host_addr: &str, player_name: &str) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("bind client socket");
    socket.set_nonblocking(true).ok();

    let join_msg = NetMessage::JoinRequest {
        name: player_name.to_string(),
    };
    let bytes = protocol::encode(&join_msg);
    socket.send_to(&bytes, host_addr).ok();

    commands.insert_resource(ClientNet {
        socket,
        host_addr: host_addr.to_string(),
        local_player_id: 0,
    });
}

fn receive_from_host(mut client: ResMut<ClientNet>, mut actions: MessageWriter<GameAction>) {
    let mut buf = [0u8; 4096];
    loop {
        match client.socket.recv(&mut buf) {
            Ok(len) => {
                if let Some(msg) = protocol::decode(&buf[..len]) {
                    match msg {
                        NetMessage::JoinAccepted { assigned_id, .. } => {
                            client.local_player_id = assigned_id;
                            info!("Joined as player {}", assigned_id);
                        }
                        NetMessage::ActionBroadcast { player_id, action_type } => {
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
                error!("Client recv error: {}", e);
                break;
            }
        }
    }
}

/// Client 发送本地玩家动作给 Host
#[allow(dead_code)]
pub fn send_action(client: &ClientNet, action: &NetAction) {
    let msg = NetMessage::PlayerAction {
        player_id: client.local_player_id,
        action_type: action.clone(),
    };
    let bytes = protocol::encode(&msg);
    client.socket.send_to(&bytes, &client.host_addr).ok();
}
