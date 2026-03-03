use bevy::prelude::*;
use crate::game::card::Card;
use crate::game::events::{CardDealt, RoundReset};
use super::table::PlayerEntityMap;

const CARD_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
const TEXT_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

/// 标记卡牌UI实体，存储所属座位，用于 RoundReset 时批量清除
#[derive(Component)]
pub struct CardUiMarker {
    #[allow(dead_code)]
    pub seat: usize,
}

pub fn on_card_dealt(
    mut ev: MessageReader<CardDealt>,
    map: Res<PlayerEntityMap>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in ev.read() {
        let seat = ev.player_id as usize;
        let Some(seat_entity) = map.seat_entities[seat] else {
            continue;
        };
        commands.entity(seat_entity).with_children(|parent| {
            parent.spawn(card_bundle(ev.card, seat, &asset_server));
        });
    }
}

pub fn on_round_reset(
    mut ev: MessageReader<RoundReset>,
    card_query: Query<Entity, With<CardUiMarker>>,
    mut commands: Commands,
) {
    for _ in ev.read() {
        for entity in &card_query {
            commands.entity(entity).despawn();
        }
    }
}

fn card_bundle(card: Card, seat: usize, asset_server: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Px(4.0)),
            width: Val::Px(60.0),
            height: Val::Px(90.0),
            ..default()
        },
        BackgroundColor(CARD_COLOR),
        CardUiMarker { seat },
        children![(
            Text::new(card.rank.to_string()),
            TextFont {
                font: asset_server.load("fonts/WenCangShuFang-2.ttf"),
                font_size: 48.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    )
}
