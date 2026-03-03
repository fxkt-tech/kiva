use bevy::prelude::*;
use crate::states::WorldState;
use crate::game::room::Room;
use crate::game::player::Player;
use crate::game::events::GameAction;

const BG_COLOR: Color = Color::srgb(0.05, 0.1, 0.2);
const BTN_COLOR: Color = Color::srgb(0.2, 0.5, 0.8);
const BTN_START_COLOR: Color = Color::srgb(0.2, 0.6, 0.3);
const BTN_BACK_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const TEXT_COLOR: Color = Color::WHITE;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WorldState::Lobby), setup_lobby)
            .add_systems(Update, lobby_action.run_if(in_state(WorldState::Lobby)))
            .add_systems(OnExit(WorldState::Lobby), cleanup_lobby);
    }
}

#[derive(Component)]
struct OnLobbyScreen;

#[derive(Component)]
enum LobbyAction {
    HostGame,
    JoinGame,
    AddAi,
    StartGame,
    Back,
}

fn setup_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let btn = Node {
        width: Val::Px(280.0),
        height: Val::Px(60.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let tf = TextFont {
        font: font.clone(),
        font_size: 30.0,
        ..default()
    };

    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(BG_COLOR),
        OnLobbyScreen,
        children![
            (
                Text::new("游戏大厅"),
                TextFont { font: font.clone(), font_size: 60.0, ..default() },
                TextColor(TEXT_COLOR)
            ),
            (
                Button, btn.clone(), BackgroundColor(BTN_COLOR),
                LobbyAction::HostGame,
                children![(Text::new("创建房间（主机）"), tf.clone(), TextColor(TEXT_COLOR))]
            ),
            (
                Button, btn.clone(), BackgroundColor(BTN_COLOR),
                LobbyAction::JoinGame,
                children![(Text::new("加入房间"), tf.clone(), TextColor(TEXT_COLOR))]
            ),
            (
                Button, btn.clone(), BackgroundColor(BTN_COLOR),
                LobbyAction::AddAi,
                children![(Text::new("添加AI玩家"), tf.clone(), TextColor(TEXT_COLOR))]
            ),
            (
                Button, btn.clone(), BackgroundColor(BTN_START_COLOR),
                LobbyAction::StartGame,
                children![(Text::new("开始游戏"), tf.clone(), TextColor(TEXT_COLOR))]
            ),
            (
                Button, btn.clone(), BackgroundColor(BTN_BACK_COLOR),
                LobbyAction::Back,
                children![(Text::new("返回"), tf.clone(), TextColor(TEXT_COLOR))]
            ),
        ],
    ));
}

fn lobby_action(
    query: Query<(&Interaction, &LobbyAction), (Changed<Interaction>, With<Button>)>,
    mut world_state: ResMut<NextState<WorldState>>,
    mut room: ResMut<Room>,
    _actions: MessageWriter<GameAction>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            LobbyAction::HostGame => {
                // 本机作为 Host，seat 0 给本地玩家
                if room.seats[0].is_none() {
                    room.seat_player(0, Player::new(0, "玩家(主机)", false));
                }
                info!("Created room as host");
            }
            LobbyAction::AddAi => {
                // 找第一个空位填入AI
                for i in 0..4 {
                    if room.seats[i].is_none() {
                        room.seat_player(i, Player::new(i as u8, format!("AI {}", i), true));
                        info!("Added AI to seat {}", i);
                        break;
                    }
                }
            }
            LobbyAction::StartGame => {
                if room.active_player_count() > 0 {
                    world_state.set(WorldState::Game);
                } else {
                    info!("需要至少一个玩家才能开始");
                }
            }
            LobbyAction::JoinGame => {
                // TODO: 显示房间列表，选择后调用 net::client::connect_to_host
                info!("Join game - not yet implemented");
            }
            LobbyAction::Back => {
                // 清空房间再返回
                *room = Room::default();
                world_state.set(WorldState::Menu);
            }
        }
    }
}

fn cleanup_lobby(query: Query<Entity, With<OnLobbyScreen>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
