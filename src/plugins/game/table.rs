use bevy::prelude::*;
use crate::game::room::Room;

const TABLE_COLOR: Color = Color::srgb(0.07, 0.20, 0.07);
const SEAT_BG: Color = Color::srgba(0.0, 0.04, 0.02, 0.50);
const SEAT_BORDER: Color = Color::srgba(0.72, 0.55, 0.14, 0.85);
const INFO_BAR_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.42);
const NAME_COLOR: Color = Color::srgb(1.0, 0.85, 0.32);
const SCORE_COLOR: Color = Color::srgb(0.72, 0.96, 0.84);

/// 标记所有游戏界面实体，用于退出时批量 despawn
#[derive(Component)]
pub struct OnGameScreen;

/// 渲染层维护的 player_id → seat Entity 映射
#[derive(Resource, Default)]
pub struct PlayerEntityMap {
    pub seat_entities: [Option<Entity>; 4],
}

/// 标记座位名称文字
#[derive(Component)]
pub struct SeatLabel {
    #[allow(dead_code)]
    pub seat: usize,
}

/// 标记座位分数文字（由 hud.rs 的 update_scores 更新）
#[derive(Component)]
pub struct ScoreLabel(pub usize);

pub fn setup_table(
    mut commands: Commands,
    room: Res<Room>,
    mut map: ResMut<PlayerEntityMap>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let name_font = TextFont { font: font.clone(), font_size: 21.0, ..default() };
    let score_font = TextFont { font: font.clone(), font_size: 21.0, ..default() };

    // 全屏绿色桌面
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(TABLE_COLOR),
            OnGameScreen,
        ))
        .id();

    let top_row = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(50.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();

    let bottom_row = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(50.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();

    commands.entity(root).add_children(&[top_row, bottom_row]);

    // 四个座位
    let seat_rows = [(0, top_row), (1, top_row), (2, bottom_row), (3, bottom_row)];
    for (seat_idx, parent_row) in seat_rows {
        let player_name = room.seats[seat_idx]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("座位 {}", seat_idx + 1));

        let seat_entity = commands
            .spawn((
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_content: AlignContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(SEAT_BG),
                BorderColor::all(SEAT_BORDER),
            ))
            .with_children(|parent| {
                // 信息栏：深色底，占满宽度，推动牌到下一行
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(INFO_BAR_BG),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(player_name),
                            name_font.clone(),
                            TextColor(NAME_COLOR),
                            SeatLabel { seat: seat_idx },
                        ));
                        row.spawn((
                            Text::new("—"),
                            score_font.clone(),
                            TextColor(SCORE_COLOR),
                            ScoreLabel(seat_idx),
                        ));
                    });
            })
            .id();

        commands.entity(parent_row).add_children(&[seat_entity]);
        map.seat_entities[seat_idx] = Some(seat_entity);
    }

    // 中央装饰（绝对定位，叠在桌面中心）
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
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
                        width: Val::Px(110.0),
                        height: Val::Px(110.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                    BorderColor::all(Color::srgba(0.72, 0.55, 0.14, 0.65)),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        Text::new("四方\n诛杀"),
                        TextFont { font: font.clone(), font_size: 26.0, ..default() },
                        TextColor(Color::srgb(0.90, 0.74, 0.22)),
                    ));
                });
        });
}

pub fn cleanup_table(
    to_despawn: Query<Entity, With<OnGameScreen>>,
    mut commands: Commands,
    mut map: ResMut<PlayerEntityMap>,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
    *map = PlayerEntityMap::default();
}
