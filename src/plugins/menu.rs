use crate::{GameState, despawn_screen};
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const RED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.3, 0.4);

// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub fn menu_plugin(app: &mut App) {
    // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
    app
        // Current screen in the menu is handled by an independent state from `GameState`
        .init_state::<MenuState>()
        // When entering the state, spawn everything needed for this screen
        .add_systems(OnEnter(GameState::Menu), setup)
        // While in this state, run the `countdown` system
        // .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
        .add_systems(Update, menu_action.run_if(in_state(GameState::Menu)))
        // When exiting the state, despawn everything that was spawned for this screen
        .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMenuScreen>);
}

#[derive(Component)]
struct OnMenuScreen;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    Settings,
    SettingsDisplay,
    SettingsSound,
    #[default]
    Disabled,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let icon = asset_server.load("fxkt_logo.png");
    let default_font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let title_font = TextFont {
        font: default_font.clone(),
        font_size: 80.0,
        ..default()
    };
    let button_text_font = TextFont {
        font: default_font.clone(),
        font_size: 33.0,
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
        BackgroundColor(BACKGROUND_COLOR),
        OnMenuScreen,
        children![
            (
                Text::new("四方诛杀"),
                title_font.clone(),
                TextColor(TEXT_COLOR)
            ),
            (
                Button,
                button_node.clone(),
                BackgroundColor(RED_COLOR),
                MenuButtonAction::Play,
                children![(
                    Text::new("开始游戏"),
                    button_text_font.clone(),
                    TextColor(TEXT_COLOR),
                ),]
            ),
            (
                Button,
                button_node.clone(),
                BackgroundColor(RED_COLOR),
                MenuButtonAction::Quit,
                children![(
                    Text::new("退出"),
                    button_text_font.clone(),
                    TextColor(TEXT_COLOR),
                ),]
            ),
        ],
    ));
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/menu_bgm.ogg")));
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Table);
                    menu_state.set(MenuState::Disabled);
                }
            }
        }
    }
}
