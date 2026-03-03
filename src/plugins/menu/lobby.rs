use bevy::prelude::*;
use crate::states::WorldState;
use crate::game::room::Room;
use crate::game::player::Player;

const BG_COLOR: Color = Color::srgb(0.05, 0.1, 0.2);
const BTN_COLOR: Color = Color::srgb(0.2, 0.5, 0.8);
const BTN_START_COLOR: Color = Color::srgb(0.2, 0.6, 0.3);
const BTN_BACK_COLOR: Color = Color::srgb(0.35, 0.35, 0.35);
const SEAT_PLAYER_COLOR: Color = Color::srgb(0.15, 0.3, 0.6);
const SEAT_AI_COLOR: Color = Color::srgb(0.15, 0.45, 0.2);
const SEAT_EMPTY_COLOR: Color = Color::srgb(0.1, 0.12, 0.22);
const TEXT_COLOR: Color = Color::WHITE;
const DIM_TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.6);

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WorldState::Lobby), setup_lobby)
            .add_systems(
                Update,
                (lobby_action, update_seat_display).run_if(in_state(WorldState::Lobby)),
            )
            .add_systems(OnExit(WorldState::Lobby), cleanup_lobby);
    }
}

#[derive(Component)]
struct OnLobbyScreen;

/// 标记大厅座位卡片容器（用于动态更新背景色）
#[derive(Component)]
struct LobbySeatBox(usize);

/// 标记大厅座位内的文字（用于动态更新内容）
#[derive(Component)]
struct LobbySeatText(usize);

#[derive(Component)]
enum LobbyAction {
    AddAi,
    StartGame,
    Back,
}

fn setup_lobby(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut room: ResMut<Room>,
) {
    // 进入大厅时自动将本地玩家落座到座位 1
    if room.seats[0].is_none() {
        room.seat_player(0, Player::new(0, "你", false));
    }

    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let title_font = TextFont { font: font.clone(), font_size: 56.0, ..default() };
    let hint_font = TextFont { font: font.clone(), font_size: 20.0, ..default() };
    let btn_font = TextFont { font: font.clone(), font_size: 26.0, ..default() };
    let seat_font = TextFont { font: font.clone(), font_size: 22.0, ..default() };

    let btn_node = Node {
        width: Val::Px(180.0),
        height: Val::Px(56.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    // 根节点
    let root = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            OnLobbyScreen,
        ))
        .id();

    // 标题
    let title = commands
        .spawn((Text::new("游戏大厅"), title_font, TextColor(TEXT_COLOR)))
        .id();

    // 提示文字
    let hint = commands
        .spawn((
            Text::new("最多 4 名玩家，空位可填入 AI"),
            hint_font,
            TextColor(DIM_TEXT_COLOR),
        ))
        .id();

    // 两行座位（每行 2 个）
    let seat_row1 = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            ..default()
        })
        .id();
    let seat_row2 = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            ..default()
        })
        .id();

    for seat_idx in 0..4usize {
        let (label, bg) = seat_label_and_color(seat_idx, room.seats[seat_idx].as_ref());

        let seat_box = commands
            .spawn((
                Node {
                    width: Val::Px(210.0),
                    height: Val::Px(90.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(bg),
                LobbySeatBox(seat_idx),
            ))
            .id();

        let seat_text = commands
            .spawn((
                Text::new(label),
                seat_font.clone(),
                TextColor(TEXT_COLOR),
                LobbySeatText(seat_idx),
            ))
            .id();

        commands.entity(seat_box).add_child(seat_text);

        if seat_idx < 2 {
            commands.entity(seat_row1).add_child(seat_box);
        } else {
            commands.entity(seat_row2).add_child(seat_box);
        }
    }

    // 按钮行
    let btn_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        })
        .id();

    let add_ai_btn = commands
        .spawn((
            Button,
            btn_node.clone(),
            BackgroundColor(BTN_COLOR),
            LobbyAction::AddAi,
        ))
        .with_children(|p| {
            p.spawn((Text::new("添加 AI"), btn_font.clone(), TextColor(TEXT_COLOR)));
        })
        .id();

    let start_btn = commands
        .spawn((
            Button,
            btn_node.clone(),
            BackgroundColor(BTN_START_COLOR),
            LobbyAction::StartGame,
        ))
        .with_children(|p| {
            p.spawn((Text::new("开始游戏"), btn_font.clone(), TextColor(TEXT_COLOR)));
        })
        .id();

    let back_btn = commands
        .spawn((
            Button,
            btn_node.clone(),
            BackgroundColor(BTN_BACK_COLOR),
            LobbyAction::Back,
        ))
        .with_children(|p| {
            p.spawn((Text::new("返回"), btn_font.clone(), TextColor(TEXT_COLOR)));
        })
        .id();

    commands.entity(btn_row).add_children(&[add_ai_btn, start_btn, back_btn]);
    commands
        .entity(root)
        .add_children(&[title, hint, seat_row1, seat_row2, btn_row]);
}

/// 根据座位状态返回显示文字和背景颜色
fn seat_label_and_color(idx: usize, player: Option<&Player>) -> (String, Color) {
    match player {
        Some(p) if p.is_ai => (
            format!("座位 {}\nAI — {}", idx + 1, p.name),
            SEAT_AI_COLOR,
        ),
        Some(p) => (
            format!("座位 {}\n玩家 — {}", idx + 1, p.name),
            SEAT_PLAYER_COLOR,
        ),
        None => (format!("座位 {}\n（空位）", idx + 1), SEAT_EMPTY_COLOR),
    }
}

/// 当 Room 数据变化时刷新座位卡片的文字和颜色
fn update_seat_display(
    room: Res<Room>,
    mut text_query: Query<(&LobbySeatText, &mut Text)>,
    mut box_query: Query<(&LobbySeatBox, &mut BackgroundColor)>,
) {
    if !room.is_changed() {
        return;
    }
    for (lbl, mut text) in &mut text_query {
        let (label, _) = seat_label_and_color(lbl.0, room.seats[lbl.0].as_ref());
        *text = Text::new(label);
    }
    for (lbl, mut bg) in &mut box_query {
        let (_, color) = seat_label_and_color(lbl.0, room.seats[lbl.0].as_ref());
        *bg = BackgroundColor(color);
    }
}

fn lobby_action(
    query: Query<(&Interaction, &LobbyAction), (Changed<Interaction>, With<Button>)>,
    mut world_state: ResMut<NextState<WorldState>>,
    mut room: ResMut<Room>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            LobbyAction::AddAi => {
                for i in 0..4 {
                    if room.seats[i].is_none() {
                        room.seat_player(
                            i,
                            Player::new(i as u8, format!("AI {}", i + 1), true),
                        );
                        info!("Added AI to seat {}", i);
                        break;
                    }
                }
            }
            LobbyAction::StartGame => {
                if room.active_player_count() >= 1 {
                    world_state.set(WorldState::Game);
                } else {
                    info!("需要至少一个玩家才能开始");
                }
            }
            LobbyAction::Back => {
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
