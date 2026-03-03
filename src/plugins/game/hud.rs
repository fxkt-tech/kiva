use bevy::prelude::*;
use crate::game::events::TurnChanged;
use crate::game::room::Room;
use crate::game::rules::calc_score;
use super::table::{OnGameScreen, ScoreLabel};

/// 回合指示器
#[derive(Component)]
pub struct TurnIndicator;

/// 顶部居中的回合提示栏
pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(14.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
                    BorderColor::all(Color::srgba(0.72, 0.55, 0.14, 0.5)),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        Text::new("等待开始..."),
                        TextFont {
                            font: font.clone(),
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.90, 0.35)),
                        TurnIndicator,
                    ));
                });
        });
}

/// 分数更新：
/// - 本地玩家（seat 0）：显示真实总分
/// - 其他玩家：第一张牌扣着，显示 "? + X"（X 为可见牌之和）
pub fn update_scores(room: Res<Room>, mut query: Query<(&mut Text, &ScoreLabel)>) {
    if !room.is_changed() {
        return;
    }
    for (mut text, label) in &mut query {
        let seat_idx = label.0;
        *text = match room.seats[seat_idx].as_ref() {
            None => Text::new("—"),
            Some(p) if seat_idx == 0 => {
                // 本地玩家：显示真实分数
                Text::new(format!("{} 分", calc_score(&p.hand)))
            }
            Some(p) => {
                // 其他玩家：第一张牌隐藏
                match p.hand.len() {
                    0 => Text::new("—"),
                    1 => Text::new("?"),
                    _ => {
                        let visible = calc_score(&p.hand[1..]);
                        Text::new(format!("? + {}", visible))
                    }
                }
            }
        };
    }
}

/// 回合切换：更新顶部指示器文字
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
