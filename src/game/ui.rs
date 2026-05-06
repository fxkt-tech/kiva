use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use rand::seq::SliceRandom;

use crate::game::ai::{choose_vote_target, choose_wolf_target, generate_speech};
use crate::game::app_state::{
    AppScreen, FlowPhase, FlowState, NeedsGameRedraw, PendingInput, PlayerAction,
    SelectedPlayer, SessionResource, TrustMark, WitchIntent,
};
use crate::game::domain::{PlayerId, PlayerKind, Role};
use crate::game::rules::{check_camp, resolve_hunter_shot, resolve_night, NightActions};
use crate::game::session::{GameSession, Winner};

const NIGHT_BG: Color = Color::srgb(0.018, 0.024, 0.040);
const PANEL_BG: Color = Color::srgb(0.042, 0.050, 0.074);
const TABLE_WOOD: Color = Color::srgb(0.135, 0.074, 0.040);
const TABLE_INNER: Color = Color::srgb(0.090, 0.050, 0.030);
const GOLD: Color = Color::srgb(0.950, 0.760, 0.350);
const WARM_TEXT: Color = Color::srgb(0.900, 0.875, 0.810);
const MUTED_TEXT: Color = Color::srgb(0.640, 0.675, 0.730);
const DANGER: Color = Color::srgb(0.760, 0.075, 0.070);
const CHINESE_FONT: &str = "fonts/chinese/STHeiti-Medium.ttc";

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
            text(&assets, "AI 狼人杀", 52.0, WARM_TEXT),
            text(&assets, "黑夜圆桌 / 9 人局 / 单人对抗 8 名 AI", 24.0, Color::srgb(0.74, 0.76, 0.80)),
            text(&assets, "3 狼人、预言家、女巫、猎人、3 村民", 22.0, GOLD),
            action_button(&assets, "开始游戏", ButtonAction::StartGame),
            text(&assets, "规则：狼人夜晚击杀，好人白天放逐，狼人全灭则好人胜。", 18.0, MUTED_TEXT),
        ],
    ));
}

pub fn spawn_role_reveal_screen(
    mut commands: Commands,
    mut session: ResMut<SessionResource>,
    mut flow: ResMut<FlowState>,
    assets: Res<UiAssets>,
) {
    if session.session.is_none() {
        let mut roles = Role::nine_player_deck();
        roles.shuffle(&mut rand::rng());
        session.session = Some(GameSession::new_with_roles(roles, 1));
    }

    let session = session.session.as_ref().expect("session initialized");
    let human = session
        .players
        .iter()
        .find(|player| player.kind == PlayerKind::Human)
        .expect("human player exists");
    flow.human_role = Some(human.role);
    let teammate_line = if human.role == Role::Werewolf {
        let teammates = session
            .players
            .iter()
            .filter(|player| player.role == Role::Werewolf && player.id != human.id)
            .map(|player| player.name.as_str())
            .collect::<Vec<_>>()
            .join("、");
        format!("狼队友：{teammates}")
    } else {
        "你的身份信息仅自己可见。".to_string()
    };

    commands.spawn((
        screen_root(),
        children![
            text(&assets, "身份揭示", 44.0, WARM_TEXT),
            text(&assets, format!("你是：{}", human.role.label()), 36.0, role_color(human.role)),
            text(&assets, role_goal(human.role), 22.0, WARM_TEXT),
            text(&assets, teammate_line, 20.0, MUTED_TEXT),
            action_button(&assets, "入夜", ButtonAction::EnterNight),
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
) {
    spawn_game_screen(&mut commands, &session, &flow, &selected_player, &assets, &pending_input);
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
) {
    if screen.get() != &AppScreen::Game || !redraw.value {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn();
    }
    spawn_game_screen(&mut commands, &session, &flow, &selected_player, &assets, &pending_input);
    redraw.value = false;
}

fn spawn_game_screen(
    commands: &mut Commands,
    session: &SessionResource,
    flow: &FlowState,
    selected_player: &SelectedPlayer,
    assets: &UiAssets,
    pending_input: &PendingInput,
) {
    let Some(session) = session.session.as_ref() else {
        commands.spawn((screen_root(), children![text(&assets, "缺少对局数据", 32.0, Color::WHITE)]));
        return;
    };

    let alive = session.alive_players().count();
    let selected = selected_player.player;
    let human_role = flow
        .human_role
        .or_else(|| session.player(PlayerId(1)).map(|player| player.role));
    let (phase_label, judge_hint, action_title, action_detail) = match flow.phase {
        FlowPhase::Night => (
            "夜晚",
            "法官：黑夜降临。按身份选择目标或直接继续。",
            "继续到天亮",
            night_action_hint(human_role),
        ),
        FlowPhase::DaySpeech => (
            "白天发言",
            "法官：所有存活玩家依次发言。",
            "进入投票",
            "系统会记录一轮默认发言并进入投票。",
        ),
        FlowPhase::Vote => (
            "投票",
            "法官：请选择或等待系统归票。",
            "结算投票",
            "系统会按 AI 投票放逐一名玩家。",
        ),
        FlowPhase::HunterShot => (
            "猎人开枪",
            "法官：猎人可以选择一名玩家带走。",
            "不开枪",
            "点击座位后选择开枪，或直接跳过。",
        ),
        FlowPhase::Review => (
            "复盘",
            "法官：本局已经结束。",
            "查看复盘",
            "揭示身份和关键记录。",
        ),
    };
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
            BackgroundColor(Color::srgb(0.025, 0.032, 0.055)),
        ))
        .with_children(|root| {
            root.spawn(judge_bar(
                &assets,
                phase_label,
                judge_hint,
                &format!("存活 {alive}/9"),
            ));
            root.spawn((
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    column_gap: px(14),
                    ..default()
                },
            ))
            .with_children(|body| {
                body.spawn((
                    Node {
                        width: percent(70),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: px(18),
                        ..default()
                    },
                    BackgroundColor(TABLE_WOOD),
                ))
                .with_children(|table_panel| {
                    table_panel.spawn(text(&assets, "黑夜圆桌", 28.0, GOLD));
                    table_panel
                        .spawn((
                            Node {
                                width: px(620),
                                height: px(420),
                                display: Display::Grid,
                                grid_template_columns: RepeatedGridTrack::flex(3, 1.0),
                                grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                                row_gap: px(14),
                                column_gap: px(14),
                                padding: UiRect::all(px(18)),
                                ..default()
                            },
                            BackgroundColor(TABLE_INNER),
                        ))
                        .with_children(|grid| {
                            for player in &session.players {
                                grid.spawn(seat_button(
                                    &assets,
                                    player.id,
                                    &player.name,
                                    player.role,
                                    player.alive,
                                    player.kind == PlayerKind::Human,
                                    selected == Some(player.id),
                                ));
                            }
                        });
                });
                body.spawn(info_drawer(&assets, &flow.public_records, &flow.my_clues));
            });
            root.spawn(action_console(
                &assets,
                action_title,
                action_detail,
                ButtonAction::AdvanceFlow,
                flow.phase,
                human_role,
                pending_input,
                selected,
            ));
        });
}

pub fn spawn_review_screen(
    mut commands: Commands,
    session: Res<SessionResource>,
    flow: Res<FlowState>,
    assets: Res<UiAssets>,
) {
    let winner = flow.winner.as_deref().unwrap_or("未分出胜负");
    commands
        .spawn((screen_root(),))
        .with_children(|root| {
            root.spawn(text(&assets, "复盘", 44.0, Color::srgb(0.96, 0.91, 0.82)));
            root.spawn(text(&assets, format!("结果：{winner}"), 30.0, GOLD));
            if let Some(session) = session.session.as_ref() {
                for player in &session.players {
                    root.spawn(text(
                        &assets,
                        format!("{}：{}", player.name, player.role.label()),
                        20.0,
                        if player.role == Role::Werewolf { DANGER } else { WARM_TEXT },
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
) {
    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            ButtonAction::StartGame => next_screen.set(AppScreen::RoleReveal),
            ButtonAction::EnterNight => next_screen.set(AppScreen::Game),
            ButtonAction::WitchSave => {
                action_state.witch_intent = WitchIntent::Save;
                redraw.value = true;
            }
            ButtonAction::WitchPoison => {
                action_state.witch_intent = WitchIntent::Poison;
                redraw.value = true;
            }
            ButtonAction::WitchNoUse => {
                action_state.selected_target = None;
                action_state.witch_intent = WitchIntent::None;
                redraw.value = true;
            }
            ButtonAction::HunterShoot => {
                advance_hunter_shot(&mut session, &mut flow, &mut action_state, &mut next_screen);
                redraw.value = true;
            }
            ButtonAction::HunterSkip => {
                flow.phase = FlowPhase::Vote;
                action_state.hunter_shot_pending = false;
                redraw.value = true;
            }
            ButtonAction::AdvanceFlow => {
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

pub fn text_input_system(
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut ime_reader: MessageReader<Ime>,
    mut pending: ResMut<PendingInput>,
    screen: Res<State<AppScreen>>,
    flow: Res<FlowState>,
    mut redraw: ResMut<NeedsGameRedraw>,
) {
    if screen.get() != &AppScreen::Game || flow.phase != FlowPhase::DaySpeech {
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
        BackgroundColor(NIGHT_BG),
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
        BackgroundColor(DANGER),
        children![text(assets, label, 22.0, WARM_TEXT)],
    )
}

fn seat_button(
    assets: &UiAssets,
    id: PlayerId,
    name: &str,
    role: Role,
    alive: bool,
    human: bool,
    selected: bool,
) -> impl Bundle {
    let label = if human {
        format!("{} 号\n{name}\n你 / {}", id.0, role.label())
    } else {
        format!("{} 号\n{name}\n{}", id.0, if alive { "存活" } else { "死亡" })
    };
    let background = if !alive {
        Color::srgb(0.025, 0.028, 0.036)
    } else if human {
        Color::srgb(0.135, 0.105, 0.055)
    } else if selected {
        Color::srgb(0.180, 0.135, 0.055)
    } else {
        Color::srgb(0.070, 0.086, 0.122)
    };
    let border = if selected {
        Color::srgb(0.98, 0.86, 0.42)
    } else if human {
        GOLD
    } else if !alive {
        Color::srgb(0.18, 0.18, 0.20)
    } else {
        Color::srgb(0.28, 0.32, 0.40)
    };

    (
        Button,
        SeatButton { player: id },
        Node {
            min_width: px(150),
            min_height: px(96),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            padding: UiRect::all(px(8)),
            ..default()
        },
        BorderColor::all(border),
        BackgroundColor(background),
        children![text(
            assets,
            label,
            16.0,
            if alive { WARM_TEXT } else { Color::srgb(0.42, 0.44, 0.48) }
        )],
    )
}

fn judge_bar(assets: &UiAssets, phase: &'static str, hint: &'static str, alive: &str) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(72),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(22)),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        children![
            text(assets, phase, 28.0, GOLD),
            text(assets, hint, 20.0, Color::srgb(0.86, 0.88, 0.95)),
            text(assets, alive.to_string(), 20.0, MUTED_TEXT),
        ],
    )
}

fn info_drawer(assets: &UiAssets, records: &[String], my_clues: &[String]) -> impl Bundle {
    let public_log = display_log(records, "暂无公开记录。");
    let private_log = display_log(my_clues, "暂无私有线索。");

    (
        Node {
            width: percent(30),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(16)),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        children![
            text(assets, "信息抽屉", 24.0, GOLD),
            text(assets, "公开记录", 19.0, WARM_TEXT),
            text(assets, public_log, 15.0, MUTED_TEXT),
            text(assets, "我的线索", 19.0, WARM_TEXT),
            text(assets, private_log, 15.0, MUTED_TEXT),
            text(assets, format!("默认标记：{:?}", TrustMark::Neutral), 16.0, Color::srgb(0.54, 0.58, 0.66)),
        ],
    )
}

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

fn action_console(
    assets: &UiAssets,
    title: &'static str,
    detail: &'static str,
    action: ButtonAction,
    phase: FlowPhase,
    human_role: Option<Role>,
    pending_input: &PendingInput,
    selected: Option<PlayerId>,
) -> impl Bundle {
    let target_text = selected
        .map(|player| format!("当前选择：{} 号", player.0))
        .unwrap_or_else(|| "当前未选择目标。".to_string());
    let input_text = if pending_input.text.is_empty() {
        "输入你的发言，或直接继续使用默认发言。".to_string()
    } else {
        format!("你的发言：{}", pending_input.text)
    };

    (
        Node {
            width: percent(100),
            height: px(116),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(7),
            padding: UiRect::horizontal(px(22)),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        children![
            text(assets, title, 21.0, GOLD),
            text(assets, detail, 17.0, MUTED_TEXT),
            text(
                assets,
                if phase == FlowPhase::DaySpeech {
                    input_text
                } else {
                    target_text
                },
                16.0,
                WARM_TEXT
            ),
            action_controls(assets, action, phase, human_role),
        ],
    )
}

fn action_controls(
    assets: &UiAssets,
    _action: ButtonAction,
    phase: FlowPhase,
    human_role: Option<Role>,
) -> impl Bundle {
    let actions = visible_actions(phase, human_role);
    let first = actions.first().copied();
    let second = actions.get(1).copied();
    let third = actions.get(2).copied();
    let fourth = actions.get(3).copied();
    (
        Node {
            column_gap: px(12),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            optional_action_button(assets, first),
            optional_action_button(assets, second),
            optional_action_button(assets, third),
            optional_action_button(assets, fourth),
        ],
    )
}

fn optional_action_button(
    assets: &UiAssets,
    action: Option<(&'static str, ButtonAction)>,
) -> impl Bundle {
    if let Some((label, action)) = action {
        (
            Button,
            action,
            Node {
                width: px(160),
                height: px(48),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            Visibility::Visible,
            BorderColor::all(GOLD),
            BackgroundColor(DANGER),
            children![text(assets, label, 19.0, WARM_TEXT)],
        )
    } else {
        (
            Button,
            ButtonAction::AdvanceFlow,
            Node {
                width: px(0),
                height: px(0),
                ..default()
            },
            Visibility::Hidden,
            BorderColor::all(Color::NONE),
            BackgroundColor(Color::NONE),
            children![text(assets, "", 1.0, Color::NONE)],
        )
    }
}

fn visible_actions(
    phase: FlowPhase,
    human_role: Option<Role>,
) -> Vec<(&'static str, ButtonAction)> {
    match phase {
        FlowPhase::Night => match human_role {
            Some(Role::Werewolf) => vec![("确认击杀", ButtonAction::AdvanceFlow)],
            Some(Role::Seer) => vec![("确认查验", ButtonAction::AdvanceFlow)],
            Some(Role::Witch) => vec![
                ("使用解药", ButtonAction::WitchSave),
                ("使用毒药", ButtonAction::WitchPoison),
                ("不用药", ButtonAction::WitchNoUse),
                ("确认", ButtonAction::AdvanceFlow),
            ],
            Some(Role::Hunter) | Some(Role::Villager) | None => {
                vec![("等待天亮", ButtonAction::AdvanceFlow)]
            }
        },
        FlowPhase::DaySpeech => vec![("提交发言", ButtonAction::AdvanceFlow)],
        FlowPhase::Vote => vec![("确认投票", ButtonAction::AdvanceFlow)],
        FlowPhase::HunterShot => vec![
            ("开枪", ButtonAction::HunterShoot),
            ("不开枪", ButtonAction::HunterSkip),
        ],
        FlowPhase::Review => vec![("查看复盘", ButtonAction::AdvanceFlow)],
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
    if !human_is_alive(session) && flow.phase != FlowPhase::Review {
        action.selected_target = None;
        action.witch_intent = WitchIntent::None;
        action.hunter_shot_pending = false;
        pending_input.text.clear();
        flow.winner = Some("你已死亡".to_string());
        flow.phase = FlowPhase::Review;
        return true;
    }

    match flow.phase {
        FlowPhase::Night => {
            let wolf_actor = session
                .players
                .iter()
                .find(|player| player.alive && player.role == Role::Werewolf)
                .map(|player| player.id);
            let human_role = flow
                .human_role
                .or_else(|| session.player(PlayerId(1)).map(|player| player.role));
            let wolf_target = if human_role == Some(Role::Werewolf) {
                action.selected_target
            } else {
                wolf_actor.and_then(|actor| choose_wolf_target(session, actor))
            };
            if human_role == Some(Role::Seer)
                && let Some(target) = action.selected_target
                && let Some(camp) = check_camp(session, target)
            {
                flow.my_clues
                    .push(format!("你查验了 {} 号，结果是 {:?}。", target.0, camp));
            }
            let poison_target = if human_role == Some(Role::Witch)
                && action.witch_intent == WitchIntent::Poison
            {
                action.selected_target
            } else {
                None
            };
            let result = resolve_night(
                session,
                NightActions {
                    wolf_target,
                    witch_save: human_role == Some(Role::Witch)
                        && action.witch_intent == WitchIntent::Save,
                    witch_poison_target: poison_target,
                },
            );
            let hunter_died = result.deaths.iter().any(|death| {
                session
                    .player(death.player)
                    .map(|player| player.role == Role::Hunter)
                    .unwrap_or(false)
            });
            if result.deaths.is_empty() {
                flow.public_records.push(format!("第 {} 夜：平安夜。", flow.day));
            } else {
                for death in result.deaths {
                    flow.public_records.push(format!(
                        "第 {} 夜：{} 号死亡（{:?}）。",
                        flow.day, death.player.0, death.reason
                    ));
                }
            }
            if hunter_died && human_role == Some(Role::Hunter) {
                flow.phase = FlowPhase::HunterShot;
                action.hunter_shot_pending = true;
                action.selected_target = None;
                action.witch_intent = WitchIntent::None;
                return false;
            }
            action.selected_target = None;
            action.witch_intent = WitchIntent::None;
            if !human_is_alive(session) {
                flow.winner = Some("你已死亡".to_string());
                flow.phase = FlowPhase::Review;
                return true;
            }
            flow.phase = FlowPhase::DaySpeech;
            false
        }
        FlowPhase::HunterShot => false,
        FlowPhase::DaySpeech => {
            let human_speech = if pending_input.text.trim().is_empty() {
                "1 号：我先听发言，投票前再给判断。".to_string()
            } else {
                format!("1 号：{}", pending_input.text.trim())
            };
            flow.public_records.push(human_speech);
            pending_input.text.clear();

            for player in session.alive_players() {
                if player.id == PlayerId(1) {
                    continue;
                }
                flow.public_records
                    .push(generate_speech(session, player.id, flow.day));
            }
            flow.phase = FlowPhase::Vote;
            false
        }
        FlowPhase::Vote => {
            let target = action
                .selected_target
                .or_else(|| choose_vote_target(session, PlayerId(1)));
            if let Some(target) = target {
                if let Some(player) = session.player_mut(target) {
                    player.alive = false;
                }
                flow.public_records
                    .push(format!("第 {} 天：{} 号被放逐。", flow.day, target.0));
            }
            action.selected_target = None;

            if !human_is_alive(session) {
                flow.winner = Some("你已死亡".to_string());
                flow.phase = FlowPhase::Review;
                return true;
            }

            if let Some(winner) = session.winner() {
                flow.winner = Some(match winner {
                    Winner::Good => "好人胜利".to_string(),
                    Winner::Werewolf => "狼人胜利".to_string(),
                });
                flow.phase = FlowPhase::Review;
                true
            } else {
                flow.day += 1;
                flow.phase = FlowPhase::Night;
                false
            }
        }
        FlowPhase::Review => {
            true
        }
    }
}

fn human_is_alive(session: &GameSession) -> bool {
    session
        .player(PlayerId(1))
        .map(|player| player.alive)
        .unwrap_or(false)
}

fn advance_hunter_shot(
    session: &mut SessionResource,
    flow: &mut FlowState,
    action: &mut PlayerAction,
    next_screen: &mut NextState<AppScreen>,
) {
    let Some(session) = session.session.as_mut() else {
        return;
    };
    if let Some(target) = action.selected_target {
        let death = resolve_hunter_shot(session, target);
        flow.public_records
            .push(format!("猎人开枪带走了 {} 号。", death.player.0));
    }
    action.selected_target = None;
    action.hunter_shot_pending = false;

    if let Some(winner) = session.winner() {
        flow.winner = Some(match winner {
            Winner::Good => "好人胜利".to_string(),
            Winner::Werewolf => "狼人胜利".to_string(),
        });
        flow.phase = FlowPhase::Review;
        next_screen.set(AppScreen::Review);
    } else {
        flow.phase = FlowPhase::DaySpeech;
        next_screen.set(AppScreen::Game);
    }
}

fn night_action_hint(role: Option<Role>) -> &'static str {
    match role {
        Some(Role::Werewolf) => "点击座位选择今晚击杀目标。",
        Some(Role::Seer) => "点击座位选择查验目标。",
        Some(Role::Witch) => "点击座位选择毒药目标，或点击女巫救人。",
        Some(Role::Hunter) | Some(Role::Villager) | None => "你今晚没有主动行动，点击继续。",
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

fn role_goal(role: Role) -> &'static str {
    match role {
        Role::Werewolf => "阵营目标：隐藏身份，淘汰足够多的好人。",
        Role::Seer => "阵营目标：查验身份，帮助好人找出全部狼人。",
        Role::Witch => "阵营目标：谨慎使用解药和毒药，帮助好人阵营。",
        Role::Hunter => "阵营目标：在关键死亡时带走可疑目标。",
        Role::Villager => "阵营目标：通过发言和投票找出狼人。",
    }
}

fn role_color(role: Role) -> Color {
    match role {
        Role::Werewolf => DANGER,
        Role::Seer | Role::Witch | Role::Hunter => GOLD,
        Role::Villager => Color::srgb(0.86, 0.84, 0.76),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_can_advance_from_night_to_vote() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        let mut flow = FlowState::default();
        let mut action = PlayerAction::default();
        let mut pending = PendingInput::default();

        assert_eq!(flow.phase, FlowPhase::Night);

        assert!(!advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));
        assert_eq!(flow.phase, FlowPhase::DaySpeech);
        assert!(flow.public_records.iter().any(|record| record.contains("夜")));

        assert!(!advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));
        assert_eq!(flow.phase, FlowPhase::Vote);
        assert!(flow.public_records.iter().any(|record| record.contains("号：")));
    }

    #[test]
    fn player_vote_target_is_exiled() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        let mut flow = FlowState {
            phase: FlowPhase::Vote,
            ..default()
        };
        let mut action = PlayerAction {
            selected_target: Some(PlayerId(4)),
            ..default()
        };
        let mut pending = PendingInput::default();

        assert!(!advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));

        assert!(!session.player(PlayerId(4)).unwrap().alive);
        assert_eq!(action.selected_target, None);
        assert!(flow.public_records.iter().any(|record| record.contains("4 号被放逐")));
    }

    #[test]
    fn player_speech_is_recorded() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        let mut flow = FlowState {
            phase: FlowPhase::DaySpeech,
            ..default()
        };
        let mut action = PlayerAction::default();
        let mut pending = PendingInput {
            text: "我怀疑 4 号".to_string(),
        };

        assert!(!advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));

        assert!(pending.text.is_empty());
        assert!(flow.public_records.iter().any(|record| record.contains("我怀疑 4 号")));
    }

    #[test]
    fn dead_human_cannot_advance_active_play() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        session.player_mut(PlayerId(1)).unwrap().alive = false;
        let mut flow = FlowState {
            phase: FlowPhase::DaySpeech,
            ..default()
        };
        let mut action = PlayerAction {
            selected_target: Some(PlayerId(4)),
            ..default()
        };
        let mut pending = PendingInput {
            text: "死人不该继续发言".to_string(),
        };

        assert!(advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));

        assert_eq!(flow.phase, FlowPhase::Review);
        assert_eq!(flow.winner.as_deref(), Some("你已死亡"));
        assert_eq!(action.selected_target, None);
        assert!(pending.text.is_empty());
        assert!(!flow.public_records.iter().any(|record| record.contains("死人不该继续发言")));
        assert!(session.player(PlayerId(4)).unwrap().alive);
    }

    #[test]
    fn human_death_after_night_resolution_enters_review() {
        let mut session = GameSession::new_with_roles(
            vec![
                Role::Seer,
                Role::Werewolf,
                Role::Werewolf,
                Role::Werewolf,
                Role::Witch,
                Role::Hunter,
                Role::Villager,
                Role::Villager,
                Role::Villager,
            ],
            1,
        );
        let mut flow = FlowState {
            human_role: Some(Role::Seer),
            ..default()
        };
        let mut action = PlayerAction::default();
        let mut pending = PendingInput::default();

        assert!(advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));

        assert_eq!(flow.phase, FlowPhase::Review);
        assert_eq!(flow.winner.as_deref(), Some("你已死亡"));
        assert!(!session.player(PlayerId(1)).unwrap().alive);
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

    #[test]
    fn seer_check_is_private_and_not_public() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        let mut flow = FlowState {
            human_role: Some(Role::Seer),
            ..default()
        };
        let mut action = PlayerAction {
            selected_target: Some(PlayerId(4)),
            ..default()
        };
        let mut pending = PendingInput::default();

        assert!(!advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));

        assert!(flow.my_clues.iter().any(|record| record.contains("查验了 4 号")));
        assert!(!flow.public_records.iter().any(|record| record.contains("查验")));
        assert!(!flow.public_records.iter().any(|record| record.contains("私有线索")));
    }

    #[test]
    fn witch_selection_without_poison_intent_does_not_poison_target() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        let mut flow = FlowState {
            human_role: Some(Role::Witch),
            ..default()
        };
        let mut action = PlayerAction {
            selected_target: Some(PlayerId(5)),
            witch_intent: WitchIntent::None,
            ..default()
        };
        let mut pending = PendingInput::default();

        assert!(!advance_flow_state(&mut session, &mut flow, &mut action, &mut pending));

        assert!(session.player(PlayerId(5)).unwrap().alive);
        assert!(!flow
            .public_records
            .iter()
            .any(|record| record.contains("5 号死亡")));
    }

    #[test]
    fn night_actions_are_filtered_by_role() {
        let wolf_actions = visible_actions(FlowPhase::Night, Some(Role::Werewolf));
        assert!(wolf_actions.contains(&("确认击杀", ButtonAction::AdvanceFlow)));
        assert!(!wolf_actions.iter().any(|(label, _)| label.contains("女巫")));
        assert!(!wolf_actions
            .iter()
            .any(|(_, action)| matches!(action, ButtonAction::WitchSave | ButtonAction::WitchPoison)));

        let witch_actions = visible_actions(FlowPhase::Night, Some(Role::Witch));
        assert!(witch_actions.iter().any(|(_, action)| *action == ButtonAction::WitchSave));
        assert!(witch_actions.iter().any(|(_, action)| *action == ButtonAction::WitchPoison));
    }
}
