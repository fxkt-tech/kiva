use crate::game::ai::{
    NoopLlmClient, choose_seer_check, choose_vote_target, choose_witch_medicine, choose_wolf_kill,
    generate_day_speech, generate_vote,
};
use crate::game::app_state::{
    AppScreen, FlowPhase, FlowState, NeedsGameRedraw, PendingInput, PlayerAction, SelectedPlayer,
    SessionResource, SpeechPlayback, WitchIntent,
};
use crate::game::decision::WitchActionDecision;
use crate::game::domain::{Player, PlayerId, PlayerKind, Role};
use crate::game::rules::{
    DeathReason, NightActions, check_camp, resolve_hunter_shot, resolve_night, tally_votes,
};
use crate::game::session::{GameSession, Winner};
use bevy::{
    ecs::hierarchy::ChildSpawnerCommands,
    input::keyboard::{Key, KeyboardInput},
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};
use bevy_ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};

const BG: Color = Color::srgb(0.018, 0.022, 0.030);
const PANEL: Color = Color::srgb(0.045, 0.052, 0.067);
const PANEL_DARK: Color = Color::srgb(0.030, 0.035, 0.047);
const SURFACE: Color = Color::srgb(0.070, 0.079, 0.095);
const GOLD: Color = Color::srgb(0.930, 0.730, 0.330);
const RED: Color = Color::srgb(0.720, 0.110, 0.120);
const BLUE: Color = Color::srgb(0.230, 0.570, 0.760);
const GREEN: Color = Color::srgb(0.260, 0.680, 0.490);
const TEXT: Color = Color::srgb(0.900, 0.885, 0.835);
const MUTED: Color = Color::srgb(0.620, 0.655, 0.710);
const DEAD_BG: Color = Color::srgb(0.050, 0.052, 0.056);
const DEAD_SURFACE: Color = Color::srgb(0.078, 0.080, 0.085);
const DEAD_BORDER: Color = Color::srgb(0.155, 0.158, 0.165);
const DEAD_TEXT: Color = Color::srgb(0.455, 0.465, 0.485);
const CHINESE_FONT: &str = "fonts/chinese/STHeiti-Medium.ttc";
const LOG_BUBBLE_LIMIT: usize = 32;
const SCROLL_LINE_HEIGHT: f32 = 21.0;

#[derive(Resource, Clone)]
pub struct UiAssets {
    font: Handle<Font>,
}

impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            font: asset_server.load(CHINESE_FONT),
        }
    }
}

#[derive(Component)]
pub struct ScreenRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonAction {
    StartGame,
    EnterNight,
    AdvanceFlow,
    WitchSave,
    WitchPoison,
    WitchNoUse,
    HunterShoot,
    HunterSkip,
}

#[derive(Component, Clone, Copy)]
pub struct SeatButton {
    pub player: PlayerId,
}

#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct UiScroll {
    entity: Entity,
    delta: Vec2,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn enable_ime(mut window: Single<&mut Window>) {
    window.ime_enabled = true;
}

pub fn cleanup_screen(mut commands: Commands, roots: Query<Entity, With<ScreenRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

pub fn spawn_start_screen(mut commands: Commands, assets: Res<UiAssets>) {
    commands.spawn((
        screen_root(),
        children![
            text(&assets, "AI 狼人杀观战台", 50.0, TEXT),
            text(&assets, "9 人局 / 9 个 AI / 观战视角", 24.0, MUTED),
            text(
                &assets,
                "左侧玩家身份公开，右侧同步记录发言、夜晚与投票。",
                20.0,
                GOLD
            ),
            action_button(&assets, "开始观战", ButtonAction::StartGame),
        ],
    ));
}

pub fn spawn_game_placeholder(
    mut commands: Commands,
    session: Res<SessionResource>,
    flow: Res<FlowState>,
    selected_player: Res<SelectedPlayer>,
    assets: Res<UiAssets>,
    pending_input: Res<PendingInput>,
    speech: Res<SpeechPlayback>,
) {
    spawn_game_screen(
        &mut commands,
        &session,
        &flow,
        &selected_player,
        &assets,
        &pending_input,
        &speech,
    );
}

pub fn redraw_game_screen(
    mut commands: Commands,
    screen: Res<State<AppScreen>>,
    mut redraw: ResMut<NeedsGameRedraw>,
    roots: Query<Entity, With<ScreenRoot>>,
    session: Res<SessionResource>,
    flow: Res<FlowState>,
    selected_player: Res<SelectedPlayer>,
    assets: Res<UiAssets>,
    pending_input: Res<PendingInput>,
    speech: Res<SpeechPlayback>,
) {
    if screen.get() != &AppScreen::Game || !redraw.value {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }
    spawn_game_screen(
        &mut commands,
        &session,
        &flow,
        &selected_player,
        &assets,
        &pending_input,
        &speech,
    );
    redraw.value = false;
}

fn spawn_game_screen(
    commands: &mut Commands,
    session: &SessionResource,
    flow: &FlowState,
    selected_player: &SelectedPlayer,
    assets: &UiAssets,
    _pending_input: &PendingInput,
    speech: &SpeechPlayback,
) {
    let Some(session) = session.session.as_ref() else {
        commands.spawn((
            screen_root(),
            children![text(assets, "缺少对局数据", 32.0, TEXT)],
        ));
        return;
    };

    let alive = session.alive_players().count();
    let selected = selected_player.player;
    let current_speaker = speech.current_speaker.filter(|_| speech.active);
    let (phase_label, judge_hint) = phase_copy(flow.phase);

    commands
        .spawn((
            ScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(18)),
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(BG),
        ))
        .with_children(|root| {
            root.spawn(judge_bar(
                assets,
                phase_label,
                judge_hint,
                &format!("存活 {alive}/9"),
            ));

            root.spawn((Node {
                width: percent(100),
                height: percent(100),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                column_gap: px(14),
                overflow: Overflow::clip_y(),
                ..default()
            },))
                .with_children(|body| {
                    body.spawn(player_list(
                        assets,
                        &session.players,
                        selected,
                        current_speaker,
                    ));
                    body.spawn(observer_panel(assets, session, flow));
                    spawn_log_panel(body, assets, &flow.public_records);
                });

            root.spawn(observer_console(assets, flow.phase));
        });
}

pub fn spawn_review_screen(
    mut commands: Commands,
    session: Res<SessionResource>,
    flow: Res<FlowState>,
    assets: Res<UiAssets>,
) {
    let winner = flow.winner.as_deref().unwrap_or("未分出胜负");
    commands.spawn((screen_root(),)).with_children(|root| {
        root.spawn(text(&assets, "复盘", 44.0, TEXT));
        root.spawn(text(&assets, format!("结果：{winner}"), 30.0, GOLD));
        if let Some(session) = session.session.as_ref() {
            for player in &session.players {
                root.spawn(text(
                    &assets,
                    format!(
                        "{}：{} / {}",
                        player.name,
                        player.role.label(),
                        status_label(player.alive)
                    ),
                    20.0,
                    if player.role == Role::Werewolf {
                        RED
                    } else {
                        TEXT
                    },
                ));
            }
        }
    });
}

pub fn button_action_system(
    mut interactions: Query<(&Interaction, &ButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_screen: ResMut<NextState<AppScreen>>,
    mut session: ResMut<SessionResource>,
    mut flow: ResMut<FlowState>,
    mut action_state: ResMut<PlayerAction>,
    mut pending_input: ResMut<PendingInput>,
    mut redraw: ResMut<NeedsGameRedraw>,
    mut speech: ResMut<SpeechPlayback>,
) {
    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            ButtonAction::StartGame => {
                let roles = Role::nine_player_deck();
                session.session = Some(GameSession::new_random_from_pool(roles, &mut rand::rng()));
                *flow = FlowState::default();
                *action_state = PlayerAction::default();
                speech.reset();
                pending_input.text.clear();
                next_screen.set(AppScreen::Game);
            }
            ButtonAction::EnterNight
            | ButtonAction::WitchSave
            | ButtonAction::WitchPoison
            | ButtonAction::WitchNoUse
            | ButtonAction::HunterShoot
            | ButtonAction::HunterSkip => {}
            ButtonAction::AdvanceFlow => {
                if flow.phase == FlowPhase::DaySpeech {
                    start_speech_playback(&session, &mut speech, &mut redraw);
                    continue;
                }

                speech.reset();
                advance_flow(
                    &mut session,
                    &mut flow,
                    &mut action_state,
                    &mut pending_input,
                    &mut next_screen,
                );
                redraw.value = true;
            }
        }
    }
}

pub fn speech_playback_system(
    time: Res<Time>,
    session: Res<SessionResource>,
    mut flow: ResMut<FlowState>,
    mut pending_input: ResMut<PendingInput>,
    mut speech: ResMut<SpeechPlayback>,
    mut redraw: ResMut<NeedsGameRedraw>,
) {
    if !speech.active || flow.phase != FlowPhase::DaySpeech {
        return;
    }

    speech.timer.tick(time.delta());
    if !speech.timer.is_finished() {
        return;
    }

    let Some(session) = session.session.as_ref() else {
        speech.reset();
        return;
    };

    if let Some(actor) = speech.current_speaker {
        let day = flow.day;
        let client = NoopLlmClient;
        let _ = generate_day_speech(&client, session, &mut flow.event_log, actor, day);
        if let Some(record) = flow.event_log.public_projection().last().cloned() {
            flow.public_records.push(record);
        }
    }

    if let Some(next_speaker) = speech.queue.pop() {
        speech.current_speaker = Some(next_speaker);
        speech.timer.reset();
    } else {
        speech.reset();
        pending_input.text.clear();
        flow.phase = FlowPhase::Vote;
    }

    redraw.value = true;
}

pub fn text_input_system(
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut ime_reader: MessageReader<Ime>,
    mut pending: ResMut<PendingInput>,
    screen: Res<State<AppScreen>>,
    mut redraw: ResMut<NeedsGameRedraw>,
) {
    if screen.get() != &AppScreen::Game {
        return;
    }

    let mut changed = false;
    for ime in ime_reader.read() {
        if let Ime::Commit { value, .. } = ime {
            pending.text.push_str(value);
            changed = true;
        }
    }

    for keyboard_input in keyboard_input_reader.read() {
        if !keyboard_input.state.is_pressed() {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Backspace, _) => {
                pending.text.pop();
                changed = true;
            }
            (Key::Enter, _) => {}
            (_, Some(inserted_text)) => {
                if inserted_text.chars().all(is_printable_char) {
                    pending.text.push_str(inserted_text);
                    changed = true;
                }
            }
            _ => {}
        }
    }

    if changed {
        redraw.value = true;
    }
}

pub fn seat_selection_system(
    mut interactions: Query<(&Interaction, &SeatButton), (Changed<Interaction>, With<Button>)>,
    mut selected: ResMut<SelectedPlayer>,
    mut action_state: ResMut<PlayerAction>,
    mut redraw: ResMut<NeedsGameRedraw>,
) {
    for (interaction, seat) in &mut interactions {
        if *interaction == Interaction::Pressed {
            selected.player = Some(seat.player);
            action_state.selected_target = Some(seat.player);
            redraw.value = true;
        }
    }
}

pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= SCROLL_LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(UiScroll { entity, delta });
            }
        }
    }
}

pub fn on_scroll_handler(
    mut scroll: On<UiScroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
    let delta = &mut scroll.delta;

    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let at_limit = if delta.x > 0.0 {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.0
        };

        if !at_limit {
            scroll_position.x = (scroll_position.x + delta.x).clamp(0.0, max_offset.x.max(0.0));
            delta.x = 0.0;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let at_limit = if delta.y > 0.0 {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.0
        };

        if !at_limit {
            scroll_position.y = (scroll_position.y + delta.y).clamp(0.0, max_offset.y.max(0.0));
            delta.y = 0.0;
        }
    }

    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}

fn screen_root() -> impl Bundle {
    (
        ScreenRoot,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(22),
            padding: UiRect::all(px(32)),
            ..default()
        },
        BackgroundColor(BG),
    )
}

fn action_button(assets: &UiAssets, label: &'static str, action: ButtonAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: px(220),
            height: px(58),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            ..default()
        },
        BorderColor::all(GOLD),
        BackgroundColor(RED),
        children![text(assets, label, 22.0, TEXT)],
    )
}

fn player_list(
    assets: &UiAssets,
    players: &[Player],
    selected: Option<PlayerId>,
    current_speaker: Option<PlayerId>,
) -> impl Bundle {
    (
        Node {
            width: percent(27),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(12)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            text(assets, "玩家", 25.0, GOLD),
            player_row(assets, &players[0], selected, current_speaker),
            player_row(assets, &players[1], selected, current_speaker),
            player_row(assets, &players[2], selected, current_speaker),
            player_row(assets, &players[3], selected, current_speaker),
            player_row(assets, &players[4], selected, current_speaker),
            player_row(assets, &players[5], selected, current_speaker),
            player_row(assets, &players[6], selected, current_speaker),
            player_row(assets, &players[7], selected, current_speaker),
            player_row(assets, &players[8], selected, current_speaker),
        ],
    )
}

fn player_row(
    assets: &UiAssets,
    player: &Player,
    selected: Option<PlayerId>,
    current_speaker: Option<PlayerId>,
) -> impl Bundle {
    let selected = selected == Some(player.id);
    let speaking = current_speaker == Some(player.id);
    let role_color = role_color(player.role);
    let bg = if !player.alive {
        DEAD_BG
    } else if speaking {
        Color::srgb(0.180, 0.130, 0.035)
    } else if selected {
        Color::srgb(0.120, 0.105, 0.065)
    } else {
        SURFACE
    };
    let border = if !player.alive {
        DEAD_BORDER
    } else if speaking {
        Color::srgb(1.000, 0.830, 0.250)
    } else if selected {
        GOLD
    } else {
        Color::srgb(0.145, 0.160, 0.190)
    };
    let primary_text = if player.alive { TEXT } else { DEAD_TEXT };
    let secondary_text = if player.alive { role_color } else { DEAD_TEXT };

    (
        Button,
        SeatButton { player: player.id },
        Node {
            width: percent(100),
            height: px(58),
            align_items: AlignItems::Center,
            column_gap: px(10),
            border: UiRect::all(px(1)),
            padding: UiRect::horizontal(px(10)),
            ..default()
        },
        BorderColor::all(border),
        BackgroundColor(bg),
        children![
            avatar(assets, player),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: px(2),
                    ..default()
                },
                children![
                    text(
                        assets,
                        format!("{}  {}", player.id.0, player.name),
                        16.0,
                        primary_text
                    ),
                    text(
                        assets,
                        player_status_line(player, speaking),
                        14.0,
                        secondary_text
                    ),
                    text(
                        assets,
                        format!(
                            "{} / {} / {}",
                            player_kind_label(&player.kind),
                            player.ai.model,
                            player.personality_preference
                        ),
                        11.0,
                        MUTED
                    ),
                ],
            ),
        ],
    )
}

fn player_status_line(player: &Player, speaking: bool) -> String {
    if speaking {
        format!("{} / 发言中", player.role.label())
    } else {
        format!("{} / {}", player.role.label(), status_label(player.alive))
    }
}

fn player_kind_label(kind: &PlayerKind) -> &'static str {
    match kind {
        PlayerKind::Ai => "AI",
    }
}

fn avatar(assets: &UiAssets, player: &Player) -> impl Bundle {
    let color = if player.alive {
        role_color(player.role)
    } else {
        DEAD_TEXT
    };
    (
        Node {
            width: px(38),
            height: px(38),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            ..default()
        },
        BorderColor::all(if player.alive { color } else { DEAD_BORDER }),
        BackgroundColor(if player.alive {
            Color::srgb(0.105, 0.112, 0.132)
        } else {
            DEAD_SURFACE
        }),
        children![text(assets, player.avatar.clone(), 18.0, color)],
    )
}

fn observer_panel(assets: &UiAssets, session: &GameSession, flow: &FlowState) -> impl Bundle {
    let wolves = session
        .alive_players()
        .filter(|player| player.role == Role::Werewolf)
        .count();
    let good = session
        .alive_players()
        .filter(|player| player.role != Role::Werewolf)
        .count();
    let selected_hint = "观战模式：身份公开，点击左侧玩家可高亮，点击底部按钮推进 AI 行动。";

    (
        Node {
            width: percent(35),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(px(16)),
            ..default()
        },
        BackgroundColor(PANEL_DARK),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(16),
                    ..default()
                },
                children![
                    text(assets, "观战面板", 27.0, TEXT),
                    text(assets, format!("第 {} 天", flow.day), 40.0, GOLD),
                    stat_strip(assets, wolves, good),
                    text(assets, selected_hint, 17.0, MUTED),
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    padding: UiRect::all(px(14)),
                    ..default()
                },
                BackgroundColor(SURFACE),
                children![
                    text(assets, "当前局势", 20.0, GOLD),
                    text(assets, phase_detail(flow.phase), 17.0, TEXT),
                    text(
                        assets,
                        "狼人全灭则好人胜；狼人数量不少于好人则狼人胜。",
                        15.0,
                        MUTED
                    ),
                ],
            ),
        ],
    )
}

fn stat_strip(assets: &UiAssets, wolves: usize, good: usize) -> impl Bundle {
    (
        Node {
            width: percent(100),
            column_gap: px(10),
            ..default()
        },
        children![
            stat_box(assets, "狼人", wolves, RED),
            stat_box(assets, "好人", good, GREEN),
        ],
    )
}

fn stat_box(assets: &UiAssets, label: &'static str, value: usize, color: Color) -> impl Bundle {
    (
        Node {
            width: percent(50),
            height: px(82),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(4),
            ..default()
        },
        BackgroundColor(SURFACE),
        children![
            text(assets, label, 16.0, MUTED),
            text(assets, value.to_string(), 30.0, color),
        ],
    )
}

#[allow(dead_code)]
fn old_log_panel(assets: &UiAssets, records: &[String]) -> impl Bundle {
    (
        Node {
            width: percent(38),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            padding: UiRect::all(px(14)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            text(assets, "日志", 25.0, GOLD),
            text(assets, display_log(records, "暂无公开记录。"), 15.0, MUTED),
        ],
    )
}

#[allow(dead_code)]
fn fixed_log_panel(assets: &UiAssets, records: &[String]) -> impl Bundle {
    let latest = recent_records(records);
    (
        Node {
            width: percent(38),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            padding: UiRect::all(px(14)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            text(assets, "日志", 25.0, GOLD),
            log_bubble(assets, log_record(&latest, 0), 0),
            log_bubble(assets, log_record(&latest, 1), 1),
            log_bubble(assets, log_record(&latest, 2), 2),
            log_bubble(assets, log_record(&latest, 3), 3),
            log_bubble(assets, log_record(&latest, 4), 4),
            log_bubble(assets, log_record(&latest, 5), 5),
            log_bubble(assets, log_record(&latest, 6), 6),
            log_bubble(assets, log_record(&latest, 7), 7),
            log_bubble(assets, log_record(&latest, 8), 8),
        ],
    )
}

#[allow(dead_code)]
fn log_panel(assets: &UiAssets, records: &[String]) -> impl Bundle {
    let visible_records = visible_log_records(records);
    let has_scroll = visible_records.len() > 9;
    (
        Node {
            width: percent(38),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            padding: UiRect::all(px(14)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            text(assets, "日志", 25.0, GOLD),
            (
                Node {
                    width: percent(100),
                    height: px(0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    column_gap: px(8),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition(Vec2::ZERO),
                children![
                    (
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(10),
                            padding: UiRect::right(px(4)),
                            ..default()
                        },
                        children![
                            log_bubble(assets, log_record(&visible_records, 0), 0),
                            log_bubble(assets, log_record(&visible_records, 1), 1),
                            log_bubble(assets, log_record(&visible_records, 2), 2),
                            log_bubble(assets, log_record(&visible_records, 3), 3),
                            log_bubble(assets, log_record(&visible_records, 4), 4),
                            log_bubble(assets, log_record(&visible_records, 5), 5),
                            log_bubble(assets, log_record(&visible_records, 6), 6),
                            log_bubble(assets, log_record(&visible_records, 7), 7),
                            log_bubble(assets, log_record(&visible_records, 8), 8),
                            log_bubble(assets, log_record(&visible_records, 9), 9),
                            log_bubble(assets, log_record(&visible_records, 10), 10),
                            log_bubble(assets, log_record(&visible_records, 11), 11),
                            log_bubble(assets, log_record(&visible_records, 12), 12),
                            log_bubble(assets, log_record(&visible_records, 13), 13),
                            log_bubble(assets, log_record(&visible_records, 14), 14),
                            log_bubble(assets, log_record(&visible_records, 15), 15),
                            log_bubble(assets, log_record(&visible_records, 16), 16),
                            log_bubble(assets, log_record(&visible_records, 17), 17),
                            log_bubble(assets, log_record(&visible_records, 18), 18),
                            log_bubble(assets, log_record(&visible_records, 19), 19),
                            log_bubble(assets, log_record(&visible_records, 20), 20),
                            log_bubble(assets, log_record(&visible_records, 21), 21),
                            log_bubble(assets, log_record(&visible_records, 22), 22),
                            log_bubble(assets, log_record(&visible_records, 23), 23),
                            log_bubble(assets, log_record(&visible_records, 24), 24),
                            log_bubble(assets, log_record(&visible_records, 25), 25),
                            log_bubble(assets, log_record(&visible_records, 26), 26),
                            log_bubble(assets, log_record(&visible_records, 27), 27),
                            log_bubble(assets, log_record(&visible_records, 28), 28),
                            log_bubble(assets, log_record(&visible_records, 29), 29),
                            log_bubble(assets, log_record(&visible_records, 30), 30),
                            log_bubble(assets, log_record(&visible_records, 31), 31),
                        ],
                    ),
                    scroll_bar(assets, has_scroll),
                ],
            ),
        ],
    )
}

fn spawn_log_panel(parent: &mut ChildSpawnerCommands, assets: &UiAssets, records: &[String]) {
    let visible_records = visible_log_records(records);
    parent
        .spawn((
            Node {
                width: percent(38),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                padding: UiRect::all(px(14)),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|panel| {
            panel.spawn(text(assets, "日志", 25.0, GOLD));
            panel
                .spawn((Node {
                    width: percent(100),
                    height: px(0),
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: vec![
                        RepeatedGridTrack::flex(1, 1.0),
                        RepeatedGridTrack::auto(1),
                    ],
                    column_gap: px(8),
                    ..default()
                },))
                .with_children(|frame| {
                    let scroll_area_id = frame
                        .spawn((
                            Node {
                                width: percent(100),
                                height: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10),
                                padding: UiRect::right(px(4)),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            ScrollPosition(Vec2::ZERO),
                        ))
                        .with_children(|scroll_area| {
                            for (index, record) in visible_records.iter().enumerate() {
                                scroll_area.spawn(log_bubble(assets, record.clone(), index));
                            }
                        })
                        .id();

                    frame.spawn((
                        Node {
                            width: px(8),
                            height: percent(100),
                            grid_column: GridPlacement::start(2),
                            ..default()
                        },
                        Scrollbar::new(scroll_area_id, ControlOrientation::Vertical, 18.0),
                        children![(
                            Node {
                                position_type: PositionType::Absolute,
                                border_radius: BorderRadius::all(px(4)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.360, 0.385, 0.430)),
                            CoreScrollbarThumb,
                        )],
                    ));
                });
        });
}

fn recent_records(records: &[String]) -> Vec<String> {
    if records.is_empty() {
        return vec!["暂无公开记录。".to_string()];
    }

    records.iter().rev().take(9).cloned().collect()
}

fn log_record(records: &[String], index: usize) -> String {
    records.get(index).cloned().unwrap_or_default()
}

fn visible_log_records(records: &[String]) -> Vec<String> {
    if records.is_empty() {
        return vec!["暂无公开记录。".to_string()];
    }

    records
        .iter()
        .rev()
        .cloned()
        .take(LOG_BUBBLE_LIMIT)
        .collect()
}

#[allow(dead_code)]
fn scroll_bar(assets: &UiAssets, visible: bool) -> impl Bundle {
    (
        Node {
            width: px(5),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::vertical(px(4)),
            ..default()
        },
        if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        BackgroundColor(Color::srgb(0.035, 0.040, 0.052)),
        children![(
            Node {
                width: px(5),
                height: percent(28),
                ..default()
            },
            BackgroundColor(Color::srgb(0.360, 0.385, 0.430)),
            children![text(assets, "", 1.0, Color::NONE)],
        )],
    )
}

fn log_bubble(assets: &UiAssets, record: String, index: usize) -> impl Bundle {
    let visible = !record.is_empty();
    let height = if visible { Val::Auto } else { px(0) };
    let padding = if visible {
        UiRect::all(px(10))
    } else {
        UiRect::ZERO
    };
    let accent = if visible {
        log_accent(&record)
    } else {
        Color::NONE
    };
    let background = if !visible {
        Color::NONE
    } else if index + 1 == LOG_BUBBLE_LIMIT {
        Color::srgb(0.090, 0.095, 0.112)
    } else {
        SURFACE
    };

    (
        Node {
            width: percent(100),
            height,
            align_items: AlignItems::FlexStart,
            column_gap: px(10),
            padding,
            border: UiRect::left(px(3)),
            ..default()
        },
        if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        BorderColor::all(accent),
        BackgroundColor(background),
        children![
            (
                Node {
                    width: px(26),
                    height: px(26),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(accent),
                children![text(
                    assets,
                    if visible { log_icon(&record) } else { "" },
                    14.0,
                    Color::srgb(0.980, 0.970, 0.930)
                )],
            ),
            (
                Node {
                    width: px(0),
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    row_gap: px(3),
                    ..default()
                },
                children![
                    text(
                        assets,
                        if visible { log_title(&record) } else { "" },
                        13.0,
                        accent
                    ),
                    wrapped_text(assets, record, 14.0, TEXT),
                ],
            ),
        ],
    )
}

fn observer_console(assets: &UiAssets, phase: FlowPhase) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(96),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(22)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    ..default()
                },
                children![
                    text(assets, console_title(phase), 22.0, TEXT),
                    text(assets, console_hint(phase), 16.0, MUTED),
                ],
            ),
            action_button(assets, advance_label(phase), ButtonAction::AdvanceFlow),
        ],
    )
}

fn judge_bar(
    assets: &UiAssets,
    phase: &'static str,
    hint: &'static str,
    alive: &str,
) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(72),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(22)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            text(assets, phase, 28.0, GOLD),
            text(assets, hint, 20.0, TEXT),
            text(assets, alive.to_string(), 20.0, MUTED),
        ],
    )
}

#[allow(dead_code)]
fn display_log(records: &[String], empty: &'static str) -> String {
    if records.is_empty() {
        empty.to_string()
    } else {
        records
            .iter()
            .enumerate()
            .map(|(index, record)| format!("{:02}. {}", index + 1, record))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn advance_flow(
    session: &mut SessionResource,
    flow: &mut FlowState,
    action: &mut PlayerAction,
    pending_input: &mut PendingInput,
    next_screen: &mut NextState<AppScreen>,
) {
    let Some(session) = session.session.as_mut() else {
        return;
    };

    let review = advance_flow_state(session, flow, action, pending_input);
    if review {
        next_screen.set(AppScreen::Review);
    } else {
        next_screen.set(AppScreen::Game);
    }
}

fn advance_flow_state(
    session: &mut GameSession,
    flow: &mut FlowState,
    action: &mut PlayerAction,
    pending_input: &mut PendingInput,
) -> bool {
    match flow.phase {
        FlowPhase::Night => {
            let wolf_actor = session
                .players
                .iter()
                .find(|player| player.alive && player.role == Role::Werewolf)
                .map(|player| player.id);
            let client = NoopLlmClient;
            let wolf_target = wolf_actor
                .and_then(|actor| {
                    choose_wolf_kill(&client, session, &mut flow.event_log, actor, flow.day).ok()
                })
                .flatten();
            let seer_target = session
                .players
                .iter()
                .find(|player| player.alive && player.role == Role::Seer)
                .and_then(|seer| {
                    choose_seer_check(&client, session, &mut flow.event_log, seer.id, flow.day, &[])
                        .ok()
                })
                .flatten();
            if let Some(target) = seer_target
                && let Some(camp) = check_camp(session, target)
            {
                flow.public_records.push(format!(
                    "第 {} 夜：预言家查验 {} 号，结果为 {}。",
                    flow.day,
                    target.0,
                    camp_label(camp)
                ));
            }
            let witch_decision = session
                .players
                .iter()
                .find(|player| player.alive && player.role == Role::Witch)
                .and_then(|witch| {
                    choose_witch_medicine(
                        &client,
                        session,
                        &mut flow.event_log,
                        witch.id,
                        flow.day,
                        wolf_target,
                        true,
                        true,
                    )
                    .ok()
                });

            let result = resolve_night(
                session,
                NightActions {
                    wolf_target,
                    witch_save: matches!(
                        witch_decision.as_ref().map(|decision| decision.action),
                        Some(WitchActionDecision::Save)
                    ),
                    witch_poison_target: witch_decision.and_then(|decision| {
                        if decision.action == WitchActionDecision::Poison {
                            decision.target
                        } else {
                            None
                        }
                    }),
                },
            );

            let mut hunter_target = None;
            if result.deaths.is_empty() {
                flow.public_records
                    .push(format!("第 {} 夜：平安夜。", flow.day));
            } else {
                for death in &result.deaths {
                    flow.public_records.push(format!(
                        "第 {} 夜：{} 号死亡（{}）。",
                        flow.day,
                        death.player.0,
                        death_reason_label(death.reason)
                    ));
                    if hunter_can_auto_shoot(session, death.player, death.reason) {
                        hunter_target = choose_vote_target(session, death.player);
                    }
                }
            }

            if let Some(target) = hunter_target {
                action.selected_target = Some(target);
                action.hunter_shot_pending = true;
                flow.phase = FlowPhase::HunterShot;
                return false;
            }

            action.selected_target = None;
            action.witch_intent = WitchIntent::None;
            pending_input.text.clear();
            flow.phase = FlowPhase::DaySpeech;
            false
        }
        FlowPhase::HunterShot => {
            let target = action.selected_target;
            if let Some(target) = target {
                let death = resolve_hunter_shot(session, target);
                flow.public_records
                    .push(format!("猎人开枪带走 {} 号。", death.player.0));
            }
            action.selected_target = None;
            action.hunter_shot_pending = false;
            if end_if_winner(session, flow) {
                true
            } else {
                flow.phase = FlowPhase::DaySpeech;
                false
            }
        }
        FlowPhase::DaySpeech => {
            pending_input.text.clear();
            false
        }
        FlowPhase::Vote => {
            let client = NoopLlmClient;
            let votes = session
                .alive_players()
                .map(|player| player.id)
                .collect::<Vec<_>>()
                .into_iter()
                .filter_map(|actor| {
                    generate_vote(&client, session, &mut flow.event_log, actor, flow.day).ok()
                })
                .collect::<Vec<_>>();
            for record in flow.event_log.public_projection() {
                if !flow.public_records.contains(&record) {
                    flow.public_records.push(record);
                }
            }

            let target = tally_votes(&votes);
            if let Some(target) = target {
                if let Some(player) = session.player_mut(target) {
                    player.alive = false;
                }
                flow.event_log.append(
                    crate::game::events::GameEvent::PlayerExiled {
                        day: flow.day,
                        player: target,
                    },
                    crate::game::events::EventVisibility::Public,
                );
                flow.public_records
                    .push(format!("第 {} 天：{} 号被放逐。", flow.day, target.0));

                if hunter_can_auto_shoot(session, target, DeathReason::Exile)
                    && let Some(shot_target) = choose_vote_target(session, target)
                {
                    action.selected_target = Some(shot_target);
                    action.hunter_shot_pending = true;
                    flow.phase = FlowPhase::HunterShot;
                    return false;
                }
            }
            action.selected_target = None;

            if end_if_winner(session, flow) {
                true
            } else {
                flow.day += 1;
                flow.phase = FlowPhase::Night;
                false
            }
        }
        FlowPhase::Review => true,
    }
}

fn start_speech_playback(
    session: &SessionResource,
    speech: &mut SpeechPlayback,
    redraw: &mut NeedsGameRedraw,
) {
    if speech.active {
        return;
    }

    let Some(session) = session.session.as_ref() else {
        return;
    };

    let mut queue = session
        .alive_players()
        .map(|player| player.id)
        .collect::<Vec<_>>();
    queue.reverse();

    let Some(first_speaker) = queue.pop() else {
        return;
    };

    speech.queue = queue;
    speech.current_speaker = Some(first_speaker);
    speech.timer = Timer::from_seconds(0.5, TimerMode::Once);
    speech.active = true;
    redraw.value = true;
}

fn hunter_can_auto_shoot(session: &GameSession, hunter: PlayerId, reason: DeathReason) -> bool {
    matches!(reason, DeathReason::WolfKill | DeathReason::Exile)
        && session
            .player(hunter)
            .map(|player| player.role == Role::Hunter)
            .unwrap_or(false)
}

fn end_if_winner(session: &GameSession, flow: &mut FlowState) -> bool {
    if let Some(winner) = session.winner() {
        flow.winner = Some(match winner {
            Winner::Good => "好人胜利".to_string(),
            Winner::Werewolf => "狼人胜利".to_string(),
        });
        flow.phase = FlowPhase::Review;
        true
    } else {
        false
    }
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn text(assets: &UiAssets, label: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font: assets.font.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
    )
}

fn wrapped_text(
    assets: &UiAssets,
    label: impl Into<String>,
    font_size: f32,
    color: Color,
) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font: assets.font.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
        TextLayout::new_with_linebreak(LineBreak::WordOrCharacter),
    )
}

fn role_color(role: Role) -> Color {
    match role {
        Role::Werewolf => RED,
        Role::Seer => BLUE,
        Role::Witch => Color::srgb(0.720, 0.410, 0.800),
        Role::Hunter => GOLD,
        Role::Villager => GREEN,
    }
}

#[allow(dead_code)]
#[allow(dead_code)]
fn role_icon(role: Role) -> &'static str {
    match role {
        Role::Werewolf => "狼",
        Role::Seer => "预",
        Role::Witch => "巫",
        Role::Hunter => "猎",
        Role::Villager => "民",
    }
}

fn status_label(alive: bool) -> &'static str {
    if alive { "存活" } else { "死亡" }
}

fn log_accent(record: &str) -> Color {
    if record.contains("死亡") || record.contains("被放逐") || record.contains("开枪") {
        RED
    } else if record.contains("查验") {
        BLUE
    } else if record.contains("夜") {
        GOLD
    } else {
        GREEN
    }
}

fn log_icon(record: &str) -> &'static str {
    if record.contains("死亡") || record.contains("被放逐") {
        "!"
    } else if record.contains("查验") {
        "?"
    } else if record.contains("夜") {
        "N"
    } else {
        "言"
    }
}

fn log_title(record: &str) -> &'static str {
    if record.contains("死亡") || record.contains("被放逐") || record.contains("开枪") {
        "结算"
    } else if record.contains("查验") {
        "查验"
    } else if record.contains("夜") {
        "夜晚"
    } else {
        "发言"
    }
}

fn phase_copy(phase: FlowPhase) -> (&'static str, &'static str) {
    match phase {
        FlowPhase::Night => ("夜晚", "AI 狼人、预言家与女巫依次行动。"),
        FlowPhase::DaySpeech => ("白天发言", "所有存活 AI 依次发言。"),
        FlowPhase::Vote => ("投票", "AI 自动归票并放逐一名玩家。"),
        FlowPhase::HunterShot => ("猎人开枪", "猎人死亡后自动选择目标。"),
        FlowPhase::Review => ("复盘", "本局已经结束。"),
    }
}

fn phase_detail(phase: FlowPhase) -> &'static str {
    match phase {
        FlowPhase::Night => "等待夜晚结算。",
        FlowPhase::DaySpeech => "下一步将写入所有 AI 发言。",
        FlowPhase::Vote => "下一步将结算集体投票。",
        FlowPhase::HunterShot => "下一步将结算猎人开枪。",
        FlowPhase::Review => "查看胜负与全员身份。",
    }
}

fn console_title(phase: FlowPhase) -> &'static str {
    match phase {
        FlowPhase::Night => "推进夜晚",
        FlowPhase::DaySpeech => "推进发言",
        FlowPhase::Vote => "推进投票",
        FlowPhase::HunterShot => "推进猎人",
        FlowPhase::Review => "查看复盘",
    }
}

fn console_hint(phase: FlowPhase) -> &'static str {
    match phase {
        FlowPhase::Night => "结算 AI 夜间行动，并把结果写入右侧日志。",
        FlowPhase::DaySpeech => "生成本轮所有存活 AI 的发言。",
        FlowPhase::Vote => "结算 AI 投票与放逐。",
        FlowPhase::HunterShot => "结算猎人带人。",
        FlowPhase::Review => "本局结束。",
    }
}

fn advance_label(phase: FlowPhase) -> &'static str {
    match phase {
        FlowPhase::Night => "结算夜晚",
        FlowPhase::DaySpeech => "生成发言",
        FlowPhase::Vote => "结算投票",
        FlowPhase::HunterShot => "结算开枪",
        FlowPhase::Review => "复盘",
    }
}

fn camp_label(camp: crate::game::domain::Camp) -> &'static str {
    match camp {
        crate::game::domain::Camp::Good => "好人",
        crate::game::domain::Camp::Werewolf => "狼人",
    }
}

fn death_reason_label(reason: DeathReason) -> &'static str {
    match reason {
        DeathReason::WolfKill => "狼刀",
        DeathReason::WitchPoison => "毒药",
        DeathReason::Exile => "放逐",
        DeathReason::HunterShot => "猎枪",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_can_advance_from_night_to_day_speech_for_ai_observer() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut flow = FlowState::default();
        let mut action = PlayerAction::default();
        let mut pending = PendingInput::default();

        assert_eq!(flow.phase, FlowPhase::Night);

        assert!(!advance_flow_state(
            &mut session,
            &mut flow,
            &mut action,
            &mut pending
        ));
        assert_eq!(flow.phase, FlowPhase::DaySpeech);
        assert!(
            flow.public_records
                .iter()
                .any(|record| record.contains("第 1 夜"))
        );
    }

    #[test]
    fn speech_playback_starts_with_first_alive_player() {
        let session = SessionResource {
            session: Some(GameSession::new_with_roles(Role::nine_player_deck())),
        };
        let mut speech = SpeechPlayback::default();
        let mut redraw = NeedsGameRedraw::default();

        start_speech_playback(&session, &mut speech, &mut redraw);

        assert!(speech.active);
        assert_eq!(speech.current_speaker, Some(PlayerId(1)));
        assert_eq!(speech.queue.len(), 8);
        assert!(redraw.value);
    }

    #[test]
    fn vote_target_is_exiled_without_human_input() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut flow = FlowState {
            phase: FlowPhase::Vote,
            ..default()
        };
        let mut action = PlayerAction::default();
        let mut pending = PendingInput::default();

        assert!(!advance_flow_state(
            &mut session,
            &mut flow,
            &mut action,
            &mut pending
        ));

        assert!(session.players.iter().any(|player| !player.alive));
        assert!(
            flow.public_records
                .iter()
                .any(|record| record.contains("被放逐"))
        );
    }

    #[test]
    fn display_log_keeps_all_records_in_order() {
        let records = vec![
            "第 1 夜：游戏开始。".to_string(),
            "第 1 夜：平安夜。".to_string(),
            "1 号：我先听发言。".to_string(),
        ];

        let display = display_log(&records, "暂无公开记录。");

        assert!(display.contains("01. 第 1 夜：游戏开始。"));
        assert!(display.contains("02. 第 1 夜：平安夜。"));
        assert!(display.contains("03. 1 号：我先听发言。"));
    }
}
