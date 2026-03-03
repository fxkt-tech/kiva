use bevy::prelude::*;
use crate::game::events::{GameAction, RoundOver};
use crate::game::room::Room;
use super::table::OnGameScreen;

#[derive(Component)]
pub struct GameOverPopup;

#[derive(Component)]
pub struct ContinueButton;

pub fn on_round_over(
    mut ev: MessageReader<RoundOver>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    room: Res<Room>,
) {
    for ev in ev.read() {
        let font = asset_server.load("fonts/WenCangShuFang-2.ttf");

        // 用玩家真实名字展示结果
        let result_text = if ev.winners.len() > 1 {
            let names: Vec<String> = ev
                .winners
                .iter()
                .map(|&w| {
                    room.seats[w as usize]
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| format!("P{}", w + 1))
                })
                .collect();
            format!("平局！{}", names.join(" & "))
        } else if let Some(&w) = ev.winners.first() {
            let name = room.seats[w as usize]
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("P{}", w + 1));
            format!("{} 获胜！", name)
        } else {
            "游戏结束".to_string()
        };

        // 全屏半透明遮罩
        commands
            .spawn((
                GameOverPopup,
                OnGameScreen,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            ))
            .with_children(|parent| {
                // 弹窗卡片
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(64.0), Val::Px(52.0)),
                            row_gap: Val::Px(28.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.06, 0.20, 0.09)),
                        BorderColor::all(Color::srgba(1.0, 0.88, 0.35, 0.5)),
                    ))
                    .with_children(|modal| {
                        // 结果文字 — 金色
                        modal.spawn((
                            Text::new(result_text),
                            TextFont {
                                font: font.clone(),
                                font_size: 52.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.88, 0.32)),
                        ));

                        // 再来一局按钮
                        modal
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(190.0),
                                    height: Val::Px(56.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.18, 0.62, 0.26)),
                                ContinueButton,
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("再来一局"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 30.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
            });
    }
}

pub fn handle_continue(
    query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    popup_query: Query<Entity, With<GameOverPopup>>,
    mut commands: Commands,
    mut actions: MessageWriter<GameAction>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
            actions.write(GameAction::StartGame);
        }
    }
}
