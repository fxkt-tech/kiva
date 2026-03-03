use bevy::prelude::*;
use crate::game::card::Card;
use crate::game::events::{CardDealt, RoundReset};
use super::table::PlayerEntityMap;

const CARD_FACE_BG: Color = Color::srgb(0.98, 0.97, 0.94);
const CARD_FACE_BORDER: Color = Color::srgb(0.70, 0.70, 0.76);
const CARD_BACK_BG: Color = Color::srgb(0.14, 0.22, 0.52);
const CARD_BACK_BORDER: Color = Color::srgb(0.28, 0.35, 0.65);

/// 每位玩家本局已渲染的牌数，用于判断第一张牌是否需要扣着。
/// 使用 Resource 而非查询现有实体，避免 Bevy 延迟 spawn/despawn
/// 导致同帧内计数不准的问题。
#[derive(Resource, Default)]
pub struct CardCount(pub [u32; 4]);

/// 标记卡牌UI实体，存储所属座位，用于 RoundReset 时批量清除
#[derive(Component)]
pub struct CardUiMarker {
    #[allow(dead_code)]
    pub seat: usize,
}

/// 必须在 on_card_dealt 之前运行，以便先重置计数再处理新牌
pub fn on_round_reset(
    mut ev: MessageReader<RoundReset>,
    card_query: Query<Entity, With<CardUiMarker>>,
    mut card_count: ResMut<CardCount>,
    mut commands: Commands,
) {
    for _ in ev.read() {
        for entity in &card_query {
            commands.entity(entity).despawn();
        }
        *card_count = CardCount::default();
    }
}

pub fn on_card_dealt(
    mut ev: MessageReader<CardDealt>,
    map: Res<PlayerEntityMap>,
    mut card_count: ResMut<CardCount>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    for ev in ev.read() {
        let seat = ev.player_id as usize;
        let Some(seat_entity) = map.seat_entities[seat] else {
            continue;
        };

        // 本局该座位的第一张牌，且不是本地玩家 → 扣着
        let face_down = seat != 0 && card_count.0[seat] == 0;
        card_count.0[seat] += 1;

        commands.entity(seat_entity).with_children(|parent| {
            if face_down {
                parent.spawn(back_card(seat, &font));
            } else {
                parent.spawn(face_card(ev.card, seat, &font));
            }
        });
    }
}

fn card_node() -> Node {
    Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(Val::Px(4.0)),
        width: Val::Px(54.0),
        height: Val::Px(76.0),
        border: UiRect::all(Val::Px(1.5)),
        ..default()
    }
}

fn face_card(card: Card, seat: usize, font: &Handle<Font>) -> impl Bundle {
    let rank_text = match card.rank {
        1 => "A",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "J",
        _ => "?",
    };
    let text_color = match card.rank {
        1 | 11 => Color::srgb(0.78, 0.10, 0.10),
        _ => Color::srgb(0.10, 0.10, 0.18),
    };
    (
        card_node(),
        BackgroundColor(CARD_FACE_BG),
        BorderColor::all(CARD_FACE_BORDER),
        CardUiMarker { seat },
        children![(
            Text::new(rank_text),
            TextFont { font: font.clone(), font_size: 38.0, ..default() },
            TextColor(text_color),
        )],
    )
}

fn back_card(seat: usize, font: &Handle<Font>) -> impl Bundle {
    (
        card_node(),
        BackgroundColor(CARD_BACK_BG),
        BorderColor::all(CARD_BACK_BORDER),
        CardUiMarker { seat },
        children![(
            Text::new("?"),
            TextFont { font: font.clone(), font_size: 38.0, ..default() },
            TextColor(Color::srgb(0.65, 0.75, 1.0)),
        )],
    )
}
