use bevy::prelude::*;
use crate::game::events::GameAction;
use crate::states::WorldState;
use super::table::OnGameScreen;

#[derive(Component, Clone)]
pub enum ActionButton {
    Draw,
    Stand,
    Quit,
}

pub fn spawn_action_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let btn_node = Node {
        width: Val::Px(160.0),
        height: Val::Px(50.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let btn_color = Color::srgb(0.8, 0.8, 0.8);
    let text_color = Color::srgb(0.2, 0.2, 0.2);
    let text_font = TextFont {
        font: font.clone(),
        font_size: 28.0,
        ..default()
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        OnGameScreen,
        children![
            (
                Button,
                btn_node.clone(),
                BackgroundColor(btn_color),
                ActionButton::Draw,
                children![(
                    Text::new("抽牌"),
                    text_font.clone(),
                    TextColor(text_color)
                )]
            ),
            (
                Button,
                btn_node.clone(),
                BackgroundColor(btn_color),
                ActionButton::Stand,
                children![(
                    Text::new("不要牌"),
                    text_font.clone(),
                    TextColor(text_color)
                )]
            ),
            (
                Button,
                btn_node.clone(),
                BackgroundColor(btn_color),
                ActionButton::Quit,
                children![(
                    Text::new("退出"),
                    text_font.clone(),
                    TextColor(text_color)
                )]
            ),
        ],
    ));
}

/// 将按钮点击转换为 GameAction 事件
/// player_id 固定为 0（本地玩家），多人联网时从 NetRole resource 读取
pub fn handle_button_input(
    query: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
    mut actions: MessageWriter<GameAction>,
    mut world_state: ResMut<NextState<WorldState>>,
) {
    for (interaction, button) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            ActionButton::Draw => {
                actions.write(GameAction::Draw { player_id: 0 });
            }
            ActionButton::Stand => {
                actions.write(GameAction::Stand { player_id: 0 });
            }
            ActionButton::Quit => {
                world_state.set(WorldState::Menu);
            }
        }
    }
}
