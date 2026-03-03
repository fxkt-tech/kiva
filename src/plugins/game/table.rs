use bevy::prelude::*;
use crate::game::room::Room;

const TABLE_COLOR: Color = Color::srgb(0.07, 0.20, 0.07);
const SEAT_BG: Color = Color::srgba(0.0, 0.04, 0.02, 0.50);
const SEAT_BORDER: Color = Color::srgba(0.72, 0.55, 0.14, 0.85);
const INFO_BAR_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.42);
const NAME_COLOR: Color = Color::srgb(1.0, 0.85, 0.32);
const SCORE_COLOR: Color = Color::srgb(0.72, 0.96, 0.84);
const COIN_COLOR: Color = Color::srgb(0.90, 0.74, 0.22);
const SCORE_PANEL_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);

/// 铜钱方向箭头高亮色（轮到该方向玩家出牌时）
pub const ARROW_ACTIVE: Color = Color::srgb(1.0, 0.88, 0.15);
/// 铜钱方向箭头暗色（非当前玩家）
pub const ARROW_DIM: Color = Color::srgba(0.72, 0.55, 0.14, 0.22);

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

/// 中央铜钱的方向箭头，seat 对应哪个玩家座位
/// 由 hud.rs 的 on_turn_changed 更新颜色
#[derive(Component)]
pub struct TurnArrow {
    pub seat: usize,
}

/// 麻将风格布局：
///   北(seat 1)在上，西(seat 2)在左，东(seat 3)在右，南(seat 0/本地)在下
///   中央铜钱"四方诛杀"带四向箭头指示当前出牌方
///   右侧面板显示四家分数
pub fn setup_table(
    mut commands: Commands,
    room: Res<Room>,
    mut map: ResMut<PlayerEntityMap>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let name_font = TextFont { font: font.clone(), font_size: 19.0, ..default() };
    let score_font = TextFont { font: font.clone(), font_size: 17.0, ..default() };
    let coin_char_font = TextFont { font: font.clone(), font_size: 27.0, ..default() };
    let arrow_font = TextFont { font: font.clone(), font_size: 22.0, ..default() };

    // ─── Root: Row (主区域 | 分数面板) ───────────────────────────────
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(TABLE_COLOR),
            OnGameScreen,
        ))
        .id();

    // ─── 右侧分数面板 ────────────────────────────────────────────────
    let score_panel = commands
        .spawn((
            Node {
                width: Val::Px(118.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(20.0)),
                row_gap: Val::Px(18.0),
                border: UiRect::left(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(SCORE_PANEL_BG),
            BorderColor::all(SEAT_BORDER),
        ))
        .id();

    // 分数条目：北→西→东→南顺序显示
    for &seat_idx in &[1usize, 2, 3, 0] {
        let player_name = room.seats[seat_idx]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("座位{}", seat_idx + 1));
        let dir = match seat_idx {
            0 => "南",
            1 => "北",
            2 => "西",
            3 => "东",
            _ => "?",
        };
        let entry = commands
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    Text::new(format!("[{}] {}", dir, player_name)),
                    name_font.clone(),
                    TextColor(NAME_COLOR),
                    SeatLabel { seat: seat_idx },
                ));
                p.spawn((
                    Text::new("—"),
                    score_font.clone(),
                    TextColor(SCORE_COLOR),
                    ScoreLabel(seat_idx),
                ));
            })
            .id();
        commands.entity(score_panel).add_children(&[entry]);
    }

    // ─── 主区域 (Column, flex:1) ────────────────────────────────────
    let main_area = commands
        .spawn(Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    commands.entity(root).add_children(&[main_area, score_panel]);

    // ════════════════════════════════════════════════════════════════
    //  上方区域 — Seat 1（北家/对面）
    // ════════════════════════════════════════════════════════════════
    let top_section = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(130.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .id();

    let seat1 = commands
        .spawn((
            Node {
                width: Val::Percent(65.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(SEAT_BG),
            BorderColor::all(SEAT_BORDER),
        ))
        .with_children(|parent| {
            let name = room.seats[1]
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "对面".to_string());
            // 信息栏（宽度100%，将牌挤到下一行）
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(INFO_BAR_BG),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(name),
                        name_font.clone(),
                        TextColor(NAME_COLOR),
                        SeatLabel { seat: 1 },
                    ));
                });
        })
        .id();

    commands.entity(top_section).add_children(&[seat1]);
    map.seat_entities[1] = Some(seat1);

    // ════════════════════════════════════════════════════════════════
    //  中间区域 — Seat 2（西家/左）| 铜钱中心 | Seat 3（东家/右）
    // ════════════════════════════════════════════════════════════════
    let middle_section = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();

    // Seat 2 — 左侧，牌纵向排列
    let seat2 = commands
        .spawn((
            Node {
                width: Val::Px(145.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(SEAT_BG),
            BorderColor::all(SEAT_BORDER),
        ))
        .with_children(|parent| {
            let name = room.seats[2]
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "左位".to_string());
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(INFO_BAR_BG),
                ))
                .with_children(|col| {
                    col.spawn((
                        Text::new(name),
                        name_font.clone(),
                        TextColor(NAME_COLOR),
                        SeatLabel { seat: 2 },
                    ));
                });
        })
        .id();

    // 中央区域（flex:1，放置铜钱）
    let center_area = commands
        .spawn(Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    // ── 铜钱装饰："四方诛杀"十字排列 + 四向指针箭头 ──────────────
    // 结构（Column）：
    //   上段：字"四" + 箭头▲（指向北家 seat 1）
    //   中段（Row）：◀+字"杀" | 方孔 | 字"方"+▶
    //   下段：箭头▼（指向南家 seat 0） + 字"诛"
    let coin = commands
        .spawn((
            Node {
                width: Val::Px(168.0),
                height: Val::Px(168.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(3.0)),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.04, 0.01, 0.92)),
            BorderColor::all(COIN_COLOR),
        ))
        .with_children(|coin| {
            // 上段："四" + ▲
            coin.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(1.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("四"),
                    coin_char_font.clone(),
                    TextColor(COIN_COLOR),
                ));
                col.spawn((
                    Text::new("▲"),
                    arrow_font.clone(),
                    TextColor(ARROW_DIM),
                    TurnArrow { seat: 1 },
                ));
            });

            // 中段：◀杀 | 方孔 | 方▶
            coin.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|row| {
                // 左：◀ + "杀"
                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|l| {
                    l.spawn((
                        Text::new("◀"),
                        arrow_font.clone(),
                        TextColor(ARROW_DIM),
                        TurnArrow { seat: 2 },
                    ));
                    l.spawn((
                        Text::new("杀"),
                        coin_char_font.clone(),
                        TextColor(COIN_COLOR),
                    ));
                });

                // 中心方孔
                row.spawn((
                    Node {
                        width: Val::Px(26.0),
                        height: Val::Px(26.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                    BorderColor::all(COIN_COLOR),
                ));

                // 右："方" + ▶
                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|r| {
                    r.spawn((
                        Text::new("方"),
                        coin_char_font.clone(),
                        TextColor(COIN_COLOR),
                    ));
                    r.spawn((
                        Text::new("▶"),
                        arrow_font.clone(),
                        TextColor(ARROW_DIM),
                        TurnArrow { seat: 3 },
                    ));
                });
            });

            // 下段：▼ + "诛"
            coin.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(1.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("▼"),
                    arrow_font.clone(),
                    TextColor(ARROW_DIM),
                    TurnArrow { seat: 0 },
                ));
                col.spawn((
                    Text::new("诛"),
                    coin_char_font.clone(),
                    TextColor(COIN_COLOR),
                ));
            });
        })
        .id();

    commands.entity(center_area).add_children(&[coin]);

    // Seat 3 — 右侧，牌纵向排列
    let seat3 = commands
        .spawn((
            Node {
                width: Val::Px(145.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(SEAT_BG),
            BorderColor::all(SEAT_BORDER),
        ))
        .with_children(|parent| {
            let name = room.seats[3]
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "右位".to_string());
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(INFO_BAR_BG),
                ))
                .with_children(|col| {
                    col.spawn((
                        Text::new(name),
                        name_font.clone(),
                        TextColor(NAME_COLOR),
                        SeatLabel { seat: 3 },
                    ));
                });
        })
        .id();

    commands
        .entity(middle_section)
        .add_children(&[seat2, center_area, seat3]);
    map.seat_entities[2] = Some(seat2);
    map.seat_entities[3] = Some(seat3);

    // ════════════════════════════════════════════════════════════════
    //  下方区域 — Seat 0（南家/本地玩家）
    // ════════════════════════════════════════════════════════════════
    let bottom_section = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(195.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();

    let seat0 = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(6.0),
                    bottom: Val::Px(76.0), // 为操作按钮留空间
                },
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(SEAT_BG),
            BorderColor::all(SEAT_BORDER),
        ))
        .with_children(|parent| {
            let name = room.seats[0]
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "玩家".to_string());
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(INFO_BAR_BG),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(name),
                        name_font.clone(),
                        TextColor(NAME_COLOR),
                        SeatLabel { seat: 0 },
                    ));
                });
        })
        .id();

    commands.entity(bottom_section).add_children(&[seat0]);
    commands
        .entity(main_area)
        .add_children(&[top_section, middle_section, bottom_section]);
    map.seat_entities[0] = Some(seat0);
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
