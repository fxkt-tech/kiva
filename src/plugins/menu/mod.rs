pub mod lobby;

use crate::states::WorldState;
use crate::despawn_screen;
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const RED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.3, 0.4);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lobby::LobbyPlugin)
            .add_systems(OnEnter(WorldState::Menu), setup)
            .add_systems(Update, menu_action.run_if(in_state(WorldState::Menu)))
            .add_systems(OnExit(WorldState::Menu), despawn_screen::<OnMenuScreen>);
    }
}

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                )]
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
                )]
            ),
        ],
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/menu_bgm.ogg")),
        OnMenuScreen,
    ));
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: MessageWriter<AppExit>,
    mut world_state: ResMut<NextState<WorldState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                // 暂时跳 Game，Lobby 在 Task 18 实现后改为 Lobby
                MenuButtonAction::Play => {
                    world_state.set(WorldState::Lobby);
                }
            }
        }
    }
}
