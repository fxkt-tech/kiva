use crate::{GameState, despawn_screen};
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const CARD_COLOR: Color = Color::srgb(0.95, 0.95, 0.95); // 扑克牌牌面的颜色（白色/米白色）
const TABLE_COLOR: Color = Color::srgb(0.05, 0.4, 0.3);
const HALF_TABLE_COLOR: Color = Color::srgba(1.0, 0.4, 0.3, 0.5);

// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub fn table_plugin(app: &mut App) {
    // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
    app
        // Current screen in the menu is handled by an independent state from `GameState`
        .init_state::<TableState>()
        // When entering the state, spawn everything needed for this screen
        .add_systems(OnEnter(GameState::Table), setup)
        // While in this state, run the `countdown` system
        // .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
        // .add_systems(Update, table_action.run_if(in_state(GameState::Table)))
        // When exiting the state, despawn everything that was spawned for this screen
        .add_systems(OnExit(GameState::Menu), despawn_screen::<OnTableScreen>);
}

#[derive(Component)]
struct OnTableScreen;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum TableState {
    #[default]
    Disabled,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    let card_node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(12.5),
        height: Val::Percent(90.0),
        ..default()
    };

    let card_font = TextFont {
        font: default_font.clone(),
        font_size: 80.0,
        ..default()
    };

    commands.spawn((
        table_node,
        BackgroundColor(TABLE_COLOR),
        OnTableScreen,
        children![
            (
                half_table_node.clone(),
                BackgroundColor(HALF_TABLE_COLOR),
                children![
                    (
                        card_node.clone(),
                        BackgroundColor(CARD_COLOR),
                        children![(Text::new("1"), card_font.clone(), TextColor(TEXT_COLOR)),]
                    ),
                    (
                        card_node.clone(),
                        BackgroundColor(CARD_COLOR),
                        children![(Text::new("2"), card_font.clone(), TextColor(TEXT_COLOR)),]
                    )
                ]
            ),
            (
                half_table_node.clone(),
                BackgroundColor(HALF_TABLE_COLOR),
                children![
                    (
                        card_node.clone(),
                        BackgroundColor(CARD_COLOR),
                        children![(Text::new("11"), card_font.clone(), TextColor(TEXT_COLOR)),]
                    ),
                    (
                        card_node.clone(),
                        BackgroundColor(CARD_COLOR),
                        children![(Text::new("10"), card_font.clone(), TextColor(TEXT_COLOR)),]
                    )
                ]
            )
        ],
    ));
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/table_bgm.ogg")));
}
