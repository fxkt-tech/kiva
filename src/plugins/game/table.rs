use bevy::prelude::*;
use crate::game::room::Room;

const TABLE_COLOR: Color = Color::srgb(0.05, 0.4, 0.3);
const SEAT_COLOR: Color = Color::srgba(1.0, 0.4, 0.3, 0.3);

/// 标记所有游戏界面实体，用于退出时批量 despawn
#[derive(Component)]
pub struct OnGameScreen;

/// 渲染层维护的 player_id → seat Entity 映射
#[derive(Resource, Default)]
pub struct PlayerEntityMap {
    pub seat_entities: [Option<Entity>; 4],
}

/// 标记座位名称文字，供 hud.rs 查询
#[derive(Component)]
pub struct SeatLabel {
    #[allow(dead_code)]
    pub seat: usize,
}

pub fn setup_table(
    mut commands: Commands,
    room: Res<Room>,
    mut map: ResMut<PlayerEntityMap>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");

    // 根节点：全屏绿色桌面，列布局
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

    // 上半行（seat 0 和 seat 1）
    let top_row = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(45.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();

    // 下半行（seat 2 和 seat 3）
    let bottom_row = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(45.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();

    commands.entity(root).add_children(&[top_row, bottom_row]);

    // 生成4个座位
    let seat_rows = [(0, top_row), (1, top_row), (2, bottom_row), (3, bottom_row)];
    for (seat_idx, parent_row) in seat_rows {
        let player_name = room.seats[seat_idx]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("空位 {}", seat_idx + 1));

        let seat_entity = commands
            .spawn((
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(SEAT_COLOR),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(player_name),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    SeatLabel { seat: seat_idx },
                ));
            })
            .id();

        commands.entity(parent_row).add_children(&[seat_entity]);
        map.seat_entities[seat_idx] = Some(seat_entity);
    }
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
