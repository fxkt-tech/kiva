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
    let btn_font = TextFont { font: font.clone(), font_size: 26.0, ..default() };
    let btn_node = Node {
        width: Val::Px(120.0),
        height: Val::Px(52.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    // 全宽居中容器，底部固定
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            // 半透明深色面板
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(14.0)),
                        column_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                ))
                .with_children(|panel| {
                    // 抽牌 — 绿色
                    panel
                        .spawn((
                            Button,
                            btn_node.clone(),
                            BackgroundColor(Color::srgb(0.16, 0.62, 0.26)),
                            ActionButton::Draw,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("抽牌"),
                                btn_font.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 不要牌 — 琥珀色
                    panel
                        .spawn((
                            Button,
                            btn_node.clone(),
                            BackgroundColor(Color::srgb(0.82, 0.52, 0.08)),
                            ActionButton::Stand,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("不要牌"),
                                btn_font.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 退出 — 灰色
                    panel
                        .spawn((
                            Button,
                            btn_node.clone(),
                            BackgroundColor(Color::srgb(0.32, 0.32, 0.36)),
                            ActionButton::Quit,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("退出"),
                                btn_font.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
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
