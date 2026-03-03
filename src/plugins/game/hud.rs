use bevy::prelude::*;
use crate::game::events::TurnChanged;
use crate::game::room::Room;
use crate::game::rules::calc_score;
use super::table::OnGameScreen;

/// 分数标签组件，标记哪个座位
#[derive(Component)]
pub struct ScoreLabel(pub usize);

/// 回合指示器
#[derive(Component)]
pub struct TurnIndicator;

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(70.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            for seat in 0..4 {
                parent.spawn((
                    Text::new(format!("P{}: 0分", seat + 1)),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    ScoreLabel(seat),
                ));
            }
            parent.spawn((
                Text::new("等待开始..."),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                TurnIndicator,
            ));
        });
}

pub fn update_scores(room: Res<Room>, mut query: Query<(&mut Text, &ScoreLabel)>) {
    if !room.is_changed() {
        return;
    }
    for (mut text, label) in &mut query {
        let score = room.seats[label.0]
            .as_ref()
            .map(|p| calc_score(&p.hand))
            .unwrap_or(0);
        let name = room.seats[label.0]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("P{}", label.0 + 1));
        *text = Text::new(format!("{}: {}分", name, score));
    }
}

pub fn on_turn_changed(
    mut ev: MessageReader<TurnChanged>,
    room: Res<Room>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
) {
    for ev in ev.read() {
        let name = room.seats[ev.player_id as usize]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default();
        for mut text in &mut query {
            *text = Text::new(format!("轮到 {} 出牌", name));
        }
    }
}
