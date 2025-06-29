use crate::WorldState;
use crate::components::deck::Deck;
use crate::components::player::Player;
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const CARD_COLOR: Color = Color::srgb(0.95, 0.95, 0.95); // 扑克牌牌面的颜色（白色/米白色）
const TABLE_COLOR: Color = Color::srgb(0.05, 0.4, 0.3);
const HALF_TABLE_COLOR: Color = Color::srgba(1.0, 0.4, 0.3, 0.5);
const BTN_TEXT_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub fn game_plugin(app: &mut App) {
    // As this plugin is managing the splash screen, it will focus on the state `::Splash`
    app
        // Current screen in the menu is handled by an independent state from ``
        .init_state::<GameState>()
        .insert_resource(Room::new())
        // When entering the state, spawn everything needed for this screen
        .add_systems(OnEnter(WorldState::Game), setup)
        // While in this state, run the `countdown` system
        // .add_systems(Update, countdown.run_if(in_state(WorldState::Splash)))
        .add_systems(Update, player_action.run_if(in_state(WorldState::Game)))
        // When exiting the state, despawn everything that was spawned for this screen
        .add_systems(OnExit(WorldState::Game), cleanup_game);
}

#[derive(Resource, Clone)]
pub struct Room {
    player1: Player,
    player2: Player,
    pub deck: Deck,
    my_turn: bool,
}

impl Room {
    pub fn new() -> Self {
        Self {
            player1: Player::new(),
            player2: Player::new(),
            deck: Deck::new(),
            my_turn: true,
        }
    }

    pub fn current_player(&mut self) -> &mut Player {
        if self.my_turn {
            &mut self.player1
        } else {
            &mut self.player2
        }
    }

    pub fn turn(&mut self) {
        self.my_turn = !self.my_turn;
    }

    pub fn clear(&mut self, mut commands: Commands) {
        self.player1.leave(&mut commands);
        self.player2.leave(&mut commands);
        self.deck.reset();
    }
}

#[derive(Component)]
struct OnGameScreen;

// All actions that can be triggered from a button click
#[derive(Component, Clone)]
enum PlayerAction {
    Draw, // 抽牌
    Quit, // 退出房间
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Disabled,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut room: ResMut<Room>) {
    println!("init game setup");
    let default_font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let table_node = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    let half_table_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(42.0),
        ..default()
    };
    let oper_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(10.0),
        ..default()
    };
    let card_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(12.5),
        height: Val::Percent(90.0),
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
        font: default_font.clone(),
        font_size: 33.0,
        ..default()
    };

    let card_font = TextFont {
        font: default_font.clone(),
        font_size: 80.0,
        ..default()
    };

    let card1 = room.deck.draw().unwrap();
    room.player1.insert_card(card1);
    let card2 = room.deck.draw().unwrap();
    room.player2.insert_card(card2);
    let card3 = room.deck.draw().unwrap();
    room.player1.insert_card(card3);
    let card4 = room.deck.draw().unwrap();
    room.player2.insert_card(card4);

    // Get cards from players to render in the UI
    let mut player1_cards = room.player1.cards.clone();
    let mut player2_cards = room.player2.cards.clone();

    commands
        .spawn((table_node, BackgroundColor(TABLE_COLOR), OnGameScreen))
        .with_children(|commands| {
            // player2：对手
            let player2_entity = commands
                .spawn((half_table_node.clone(), BackgroundColor(HALF_TABLE_COLOR)))
                .with_children(|commands| {
                    for card in player2_cards.iter_mut() {
                        let card_entity = commands
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
                })
                .id();

            // player1：自己
            let player1_entity = commands
                .spawn((half_table_node.clone(), BackgroundColor(HALF_TABLE_COLOR)))
                .with_children(|commands| {
                    for card in player1_cards.iter_mut() {
                        let card_entity = commands
                            .spawn((
                                card_node.clone(),
                                BackgroundColor(CARD_COLOR),
                                children![(
                                    Text::new(card.get_rank().to_string()),
                                    card_font.clone(),
                                    TextColor(TEXT_COLOR)
                                ),],
                            ))
                            .id();
                        card.entity = Some(card_entity);
                    }
                })
                .id();

            // 保存player1容器实体到Room资源中
            room.player1.entity = Some(player1_entity);
            room.player2.entity = Some(player2_entity);

            // 操作栏
            commands.spawn((
                oper_node.clone(),
                // BackgroundColor(HALF_TABLE_COLOR),
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
) {
    for (interaction, player_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match player_action {
                PlayerAction::Draw => {
                    // 抽一张新卡
                    let mut new_card = room.deck.draw().unwrap();
                    let p = room.current_player();
                    p.insert_card(new_card);

                    // 渲染新卡到UI
                    if let Some(entity) = p.entity {
                        // 创建卡片节点
                        let card_node = Node {
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::all(Val::Px(5.0)),
                            width: Val::Percent(12.5),
                            height: Val::Percent(90.0),
                            ..default()
                        };

                        // 向当前玩家容器添加新卡片
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

                    room.turn();
                }
                PlayerAction::Quit => {
                    world_state.set(WorldState::Menu);
                    game_state.set(GameState::Disabled);
                }
            }
        }
    }
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
