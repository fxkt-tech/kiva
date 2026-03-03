use bevy::prelude::*;
use crate::game::events::{GameAction, RoundOver};
use super::table::OnGameScreen;

#[derive(Component)]
pub struct GameOverPopup;

#[derive(Component)]
pub struct ContinueButton;

pub fn on_round_over(
    mut ev: MessageReader<RoundOver>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in ev.read() {
        let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
        let result_text = if ev.winners.len() > 1 {
            let ids: Vec<String> = ev.winners.iter().map(|w| format!("P{}", w + 1)).collect();
            format!("平局！胜者：{}", ids.join("、"))
        } else if let Some(&w) = ev.winners.first() {
            format!("P{} 获胜！", w + 1)
        } else {
            "游戏结束".to_string()
        };

        commands.spawn((
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                children![
                    (
                        Text::new(result_text),
                        TextFont {
                            font: font.clone(),
                            font_size: 60.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        Button,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(55.0),
                            margin: UiRect::top(Val::Px(30.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
                        ContinueButton,
                        children![(
                            Text::new("继续"),
                            TextFont {
                                font: font.clone(),
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.2, 0.2, 0.2)),
                        )],
                    ),
                ],
            )],
        ));
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
