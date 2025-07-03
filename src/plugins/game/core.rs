use crate::{WorldState, plugins::game::room::Room};
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const CARD_COLOR: Color = Color::srgb(0.95, 0.95, 0.95); // 扑克牌牌面的颜色（白色/米白色）
const TABLE_COLOR: Color = Color::srgb(0.05, 0.4, 0.3);
const HALF_TABLE_COLOR: Color = Color::srgba(1.0, 0.4, 0.3, 0.5);
const BTN_TEXT_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

/// This plugin handles the game world and its states
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // As this plugin is managing the game screen, it will focus on the state `WorldState::Game`
            .init_state::<GameState>()
            .init_resource::<Room>()
            // When entering the state, spawn everything needed for this screen
            .add_systems(OnEnter(WorldState::Game), setup)
            // While in this state, run the game systems
            .add_systems(Update, player_action.run_if(in_state(WorldState::Game)))
            .add_systems(
                Update,
                computer_auto_play.run_if(in_state(WorldState::Game)),
            )
            .add_systems(Update, check_game_over.run_if(in_state(WorldState::Game)))
            // When exiting the state, despawn everything that was spawned for this screen
            .add_systems(OnExit(WorldState::Game), cleanup_game);
    }
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Disabled,
}

#[derive(Component)]
struct OnGameScreen;

// All actions that can be triggered from a button click
#[derive(Component, Clone)]
enum PlayerAction {
    Draw,     // 抽牌
    Stand,    // 不要牌
    Quit,     // 退出房间
    Continue, // 继续游戏（关闭结束弹窗）
}

// 用于标记游戏结束弹窗
#[derive(Component)]
struct GameOverPopup;

fn setup(mut commands: Commands, mut room: ResMut<Room>, asset_server: Res<AssetServer>) {
    prepare_table(&mut commands, &mut room);
    take_seat(&mut commands, &mut room);
    operate_bar(&mut commands, &asset_server, &mut room);
    deal_cards(&mut commands, &asset_server, &mut room);
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/table_bgm.ogg")),
        OnGameScreen,
    ));
}

fn player_action(
    interaction_query: Query<(&Interaction, &PlayerAction), (Changed<Interaction>, With<Button>)>,
    mut room: ResMut<Room>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<NextState<GameState>>,
    mut world_state: ResMut<NextState<WorldState>>,
    popup_query: Query<Entity, With<GameOverPopup>>,
) {
    for (interaction, player_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match player_action {
                PlayerAction::Draw => {
                    if let Some(mut new_card) = room.deck.pop() {
                        let p = room.current_turn_player();
                        p.give_me(new_card);
                        if let Some(entity) = p.entity {
                            let card_node = Node {
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                width: Val::Percent(12.5),
                                height: Val::Percent(90.0),
                                ..default()
                            };
                            commands.entity(entity).with_children(|parent| {
                                let card_entity = parent
                                    .spawn((
                                        card_node,
                                        BackgroundColor(CARD_COLOR),
                                        children![(
                                            Text::new(new_card.get_rank().to_string()),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/WenCangShuFang-2.ttf"),
                                                font_size: 80.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_COLOR)
                                        ),],
                                    ))
                                    .id();
                                new_card.entity = Some(card_entity);
                            });
                        }
                        room.set_current_player_stand(false);
                        room.turn();
                    }
                }
                PlayerAction::Stand => {
                    room.set_current_player_stand(true);
                    room.turn();
                }
                PlayerAction::Quit => {
                    world_state.set(WorldState::Menu);
                    game_state.set(GameState::Disabled);
                }

                PlayerAction::Continue => {
                    // 删除游戏结束弹窗
                    for entity in popup_query.iter() {
                        commands.entity(entity).despawn();
                    }

                    // 重置游戏状态，准备新一轮游戏
                    room.reset_game();

                    // 重新发初始牌
                    let card1 = room.deck.pop().unwrap();
                    room.player1.give_me(card1);
                    let card2 = room.deck.pop().unwrap();
                    room.player2.give_me(card2);
                    let card3 = room.deck.pop().unwrap();
                    room.player1.give_me(card3);
                    let card4 = room.deck.pop().unwrap();
                    room.player2.give_me(card4);

                    // 更新UI显示新牌
                    // update_cards_ui(&mut commands, &asset_server, &mut room);
                }
            }
        }
    }
}

// 电脑自动决策系统
fn computer_auto_play(
    mut room: ResMut<Room>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if !room.is_computer_turn() || room.is_game_over() {
        return;
    }
    // 先获取分数，避免多重可变借用
    let score = {
        let computer = room.current_turn_player();
        computer.get_score()
    };
    if score < 17 && !room.deck.cards.is_empty() {
        // 电脑要牌
        if let Some(mut new_card) = room.deck.pop() {
            let computer = room.current_turn_player();
            computer.give_me(new_card);
            if let Some(entity) = computer.entity {
                let card_node = Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    width: Val::Percent(12.5),
                    height: Val::Percent(90.0),
                    ..default()
                };
                commands.entity(entity).with_children(|parent| {
                    let card_entity = parent
                        .spawn((
                            card_node,
                            BackgroundColor(CARD_COLOR),
                            children![(
                                Text::new(new_card.get_rank().to_string()),
                                TextFont {
                                    font: asset_server.load("fonts/WenCangShuFang-2.ttf"),
                                    font_size: 80.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR)
                            ),],
                        ))
                        .id();
                    new_card.entity = Some(card_entity);
                });
            }
            room.set_current_player_stand(false);
        }
    } else {
        // 电脑不要牌
        room.set_current_player_stand(true);
    }
    room.turn();
}

// 准备桌子
fn prepare_table(commands: &mut Commands, room: &mut ResMut<Room>) {
    let table_node = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };

    let table_entity = commands
        .spawn((table_node, BackgroundColor(TABLE_COLOR), OnGameScreen))
        .id();
    room.entity = Some(table_entity);
}

// 都给我 坐下！
fn take_seat(commands: &mut Commands, room: &mut ResMut<Room>) {
    let half_table_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(42.0),
        ..default()
    };

    if let Some(entity) = room.entity {
        // player2：对手
        commands.entity(entity).with_children(|parent| {
            let player2_entity = parent
                .spawn((half_table_node.clone(), BackgroundColor(HALF_TABLE_COLOR)))
                .id();
            room.player2.entity = Some(player2_entity);
        });

        // player1：自己
        commands.entity(entity).with_children(|parent| {
            let player1_entity = parent
                .spawn((half_table_node.clone(), BackgroundColor(HALF_TABLE_COLOR)))
                .id();
            room.player1.entity = Some(player1_entity);
        });
    }
}

fn operate_bar(commands: &mut Commands, asset_server: &Res<AssetServer>, room: &mut ResMut<Room>) {
    let oper_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(10.0),
        ..default()
    };
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(50.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font: asset_server.load("fonts/WenCangShuFang-2.ttf"),
        font_size: 33.0,
        ..default()
    };

    if let Some(entity) = room.entity {
        commands.entity(entity).with_children(|parent| {
            // 操作栏
            parent.spawn((
                oper_node.clone(),
                children![
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(BTN_TEXT_COLOR),
                        PlayerAction::Draw,
                        children![(
                            Text::new("抽牌"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),],
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(BTN_TEXT_COLOR),
                        PlayerAction::Stand,
                        children![(
                            Text::new("不要牌"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),],
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(BTN_TEXT_COLOR),
                        PlayerAction::Quit,
                        children![(
                            Text::new("退出房间"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),],
                    ),
                ],
            ));
        });
    }
}

// 初始发牌
fn deal_cards(commands: &mut Commands, asset_server: &Res<AssetServer>, room: &mut ResMut<Room>) {
    let card_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(12.5),
        height: Val::Percent(90.0),
        ..default()
    };

    let card_font = TextFont {
        font: asset_server.load("fonts/WenCangShuFang-2.ttf"),
        font_size: 80.0,
        ..default()
    };

    let card1 = room.deck.pop().unwrap();
    room.player1.give_me(card1);
    let card2 = room.deck.pop().unwrap();
    room.player2.give_me(card2);
    let card3 = room.deck.pop().unwrap();
    room.player1.give_me(card3);
    let card4 = room.deck.pop().unwrap();
    room.player2.give_me(card4);

    // Get cards from players to render in the UI
    let mut player1_cards = room.player1.cards.clone();
    let mut player2_cards = room.player2.cards.clone();

    if let Some(entity) = room.player1.entity {
        commands.entity(entity).with_children(|parent| {
            for card in player1_cards.iter_mut() {
                let card_entity = parent
                    .spawn((
                        card_node.clone(),
                        BackgroundColor(CARD_COLOR),
                        children![(
                            Text::new(card.get_rank().to_string()),
                            card_font.clone(),
                            TextColor(TEXT_COLOR)
                        )],
                    ))
                    .id();
                card.entity = Some(card_entity);
            }
        });
    }

    if let Some(entity) = room.player2.entity {
        commands.entity(entity).with_children(|parent| {
            for card in player2_cards.iter_mut() {
                let card_entity = parent
                    .spawn((
                        card_node.clone(),
                        BackgroundColor(CARD_COLOR),
                        children![(
                            Text::new(card.get_rank().to_string()),
                            card_font.clone(),
                            TextColor(TEXT_COLOR)
                        )],
                    ))
                    .id();
                card.entity = Some(card_entity);
            }
        });
    }
}

// 游戏结束判定与结算
fn check_game_over(mut commands: Commands, room: Res<Room>, asset_server: Res<AssetServer>) {
    if !room.is_game_over() {
        return;
    }
    let player_score = room.player1.get_score();
    let computer_score = room.player2.get_score();
    let result = if player_score > 21 && computer_score > 21 {
        "双方都爆了，平局！"
    } else if player_score > 21 {
        "你爆了，电脑获胜！"
    } else if computer_score > 21 {
        "电脑爆了，你获胜！"
    } else if player_score > computer_score {
        "你获胜！"
    } else if player_score < computer_score {
        "电脑获胜！"
    } else {
        "平局！"
    };

    let default_font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(50.0),
        margin: UiRect::top(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    // 弹窗或文本显示结果
    commands.spawn((
        GameOverPopup,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        OnGameScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![
                (
                    Text::new(result),
                    TextFont {
                        font: default_font.clone(),
                        font_size: 60.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(BTN_TEXT_COLOR),
                    PlayerAction::Continue,
                    children![(
                        Text::new("继续"),
                        TextFont {
                            font: default_font.clone(),
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ),],
                ),
            ],
        ),],
    ));
}

fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<OnGameScreen>>,
    mut room: ResMut<Room>,
) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
    room.clear(commands);
    // commands.remove_resource::<Room>();
}
