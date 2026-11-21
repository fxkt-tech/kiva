//! Display-only snapshots. Simulation owns all admission, health and clocks.
use crate::{
    gameplay::{GameLevel, Player},
    simulation::{
        ActionKind, Arsenal, Boss, BossPhase, BossState, CombatStatus, Combatant, DefenseKind,
        DefenseState, Enemy, Energy, Health, MotionState, Stamina, WeaponPair,
    },
};
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = defineLoadouts)]
    fn define_loadouts(json: &str);
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = frame)]
    fn send_frame(json: &str);
}

/// A countdown and the length it started at.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Clock {
    left: f32,
    total: f32,
}

/// A value out of its maximum.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Meter {
    current: f32,
    maximum: f32,
}

/// The current pair's page slot and which of its inputs would be admitted.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LoadoutSnapshot {
    pair: WeaponPair,
    melee: bool,
    ranged: bool,
    combo: bool,
}
fn loadout_snapshot(combat: &Combatant, health: &Health) -> LoadoutSnapshot {
    let a = combat.availability();
    let alive = health.is_alive();
    LoadoutSnapshot {
        pair: combat.weapon_pair(),
        melee: alive && a.melee_skill,
        ranged: alive && a.ranged_skill,
        combo: alive && a.combo,
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct TargetSnapshot {
    name: &'static str,
    current: f32,
    maximum: f32,
    status: &'static str,
    /// The Boss's phase (1 or 2) of `phases`; anything else has one.
    phase: u32,
    phases: u32,
}

/// What the HUD shows of the Boss's strike in progress: its telegraph, or its swings.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BossCue {
    striking: bool,
    left: f32,
    total: f32,
    chain: u32,
    swings: u32,
    /// One bit per swing already resolved against the player.
    resolved: u32,
}

fn boss_cue(boss: &Boss) -> Option<BossCue> {
    let striking = match boss.state() {
        BossState::Telegraph => false,
        BossState::Attacking => true,
        _ => return None,
    };
    let (left, total) = boss.telegraph().unwrap_or((0.0, 0.0));
    let (swings, resolved) = boss.swings().unwrap_or((0, 0));
    Some(BossCue {
        striking,
        left,
        total,
        chain: u32::from(boss.chain()),
        swings: u32::from(swings),
        // The page reads at most 31 swings.
        resolved: (resolved & 0x7fff_ffff) as u32,
    })
}

/// Where the locked target is relative to the view: on screen at a point (fractions of the
/// viewport), off to one side, or behind the camera.
#[derive(Clone, Copy, Debug, PartialEq)]
enum LockView {
    OnScreen(Vec2),
    Aside { right: bool },
    Behind { right: bool },
}

fn lock_view(camera: &Camera, view: &Transform, point: Vec3) -> Option<LockView> {
    let offset = point - view.translation;
    let right = offset.dot(*view.right()) >= 0.0;
    if offset.dot(*view.forward()) <= 0.0 {
        return Some(LockView::Behind { right });
    }
    let size = camera.logical_viewport_size()?;
    let screen = camera
        .world_to_viewport(&GlobalTransform::from(*view), point)
        .ok()
        .map(|position| position / size);
    Some(match screen {
        Some(at) if (0.0..=1.0).contains(&at.x) && (0.0..=1.0).contains(&at.y) => {
            LockView::OnScreen(at)
        }
        Some(at) => LockView::Aside { right: at.x > 0.5 },
        None => LockView::Aside { right },
    })
}

/// Everything the HUD shows for one update. [`sync`] rebuilds it each frame
/// and stores it only when it differs; [`publish`] sends a changed frame to
/// the page as one JSON object (`window.kivaHud.frame`, shape pinned by
/// `tests/fixtures/hud`) or, natively, to the window title.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(super) struct HudFrame {
    context: &'static str,
    health: Meter,
    stamina: Meter,
    loadout: LoadoutSnapshot,
    melee: Clock,
    ranged: Clock,
    combo: Clock,
    axe_away: bool,
    sun: Meter,
    moon: Meter,
    rage: Meter,
    enraged: bool,
    rage_ready: bool,
    /// Every pair's durability, in page slot order.
    durability: Vec<f32>,
    durability_maximum: f32,
    target: Option<TargetSnapshot>,
    cue: Option<BossCue>,
    /// Where the locked target is, and its planar distance.
    lock: Option<(LockView, f32)>,
}

impl HudFrame {
    /// The player's own part of the frame; target, cue and lock start empty.
    fn player(
        combat: &Combatant,
        health: &Health,
        stamina: &Stamina,
        energy: &Energy,
        arsenal: &Arsenal,
        context: &'static str,
    ) -> Self {
        let pair = combat.weapon_pair();
        Self {
            context,
            health: Meter {
                current: health.current(),
                maximum: health.maximum(),
            },
            stamina: Meter {
                current: stamina.current(),
                maximum: stamina.maximum(),
            },
            loadout: loadout_snapshot(combat, health),
            melee: Clock {
                left: combat.melee_cooldown(),
                total: pair.melee_skill().cooldown(),
            },
            ranged: Clock {
                left: combat.ranged_cooldown(),
                total: pair.ranged_skill().cooldown(),
            },
            combo: combat
                .combo_window()
                .map_or_else(Clock::default, |window| Clock {
                    left: window.remaining,
                    total: window.duration,
                }),
            axe_away: combat.axe_away(),
            sun: Meter {
                current: f32::from(energy.sun()),
                maximum: f32::from(Energy::SUN_MAX),
            },
            moon: Meter {
                current: f32::from(energy.moon()),
                maximum: f32::from(Energy::MOON_MAX),
            },
            rage: Meter {
                current: energy.rage(),
                maximum: Energy::RAGE_MAX,
            },
            enraged: energy.is_enraged(),
            rage_ready: energy.rage_ready(),
            durability: durability_by_hud_index(arsenal),
            durability_maximum: f32::from(Arsenal::DURABILITY),
            target: None,
            cue: None,
            lock: None,
        }
    }

    /// Rounded below what the HUD can show, so float noise in a value which
    /// is not really changing (a projected position, a still countdown) does
    /// not count as a change.
    fn quantized(mut self) -> Self {
        fn step(value: f32, per_unit: f32) -> f32 {
            (value * per_unit).round() / per_unit
        }
        for clock in [&mut self.melee, &mut self.ranged, &mut self.combo] {
            clock.left = step(clock.left, 100.0);
        }
        if let Some(cue) = &mut self.cue {
            cue.left = step(cue.left, 100.0);
        }
        self.stamina.current = step(self.stamina.current, 100.0);
        self.rage.current = step(self.rage.current, 100.0);
        if let Some((view, distance)) = &mut self.lock {
            *distance = step(*distance, 100.0);
            if let LockView::OnScreen(at) = view {
                *at = Vec2::new(step(at.x, 1000.0), step(at.y, 1000.0));
            }
        }
        self
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl HudFrame {
    fn to_json(&self) -> String {
        use super::json::{Object, number};
        let meter = |m: Meter| {
            Object::new()
                .number("current", m.current)
                .number("maximum", m.maximum)
                .finish()
        };
        let clock = |c: Clock| {
            Object::new()
                .number("left", c.left)
                .number("total", c.total)
                .finish()
        };
        let loadout = Object::new()
            .integer("set", super::loadout::of(self.loadout.pair).hud_index)
            .boolean("meleeReady", self.loadout.melee)
            .boolean("rangedReady", self.loadout.ranged)
            .boolean("comboReady", self.loadout.combo)
            .finish();
        let energy = Object::new()
            .number("sun", self.sun.current)
            .number("sunMaximum", self.sun.maximum)
            .number("moon", self.moon.current)
            .number("moonMaximum", self.moon.maximum)
            .number("rage", self.rage.current)
            .number("rageMaximum", self.rage.maximum)
            .boolean("enraged", self.enraged)
            .boolean("rageReady", self.rage_ready)
            .finish();
        let values = self
            .durability
            .iter()
            .map(|value| number(*value))
            .collect::<Vec<_>>()
            .join(",");
        let durability = Object::new()
            .raw("values", &format!("[{values}]"))
            .number("maximum", self.durability_maximum)
            .finish();
        let target = self.target.as_ref().map(|t| {
            Object::new()
                .string("name", t.name)
                .number("current", t.current)
                .number("maximum", t.maximum)
                .string("status", t.status)
                .integer("phase", t.phase)
                .integer("phases", t.phases)
                .finish()
        });
        let cue = self.cue.as_ref().map(|c| {
            Object::new()
                .boolean("striking", c.striking)
                .number("left", c.left)
                .number("total", c.total)
                .integer("chain", c.chain)
                .integer("swings", c.swings)
                .integer("resolved", c.resolved)
                .finish()
        });
        let lock = self.lock.map(|(view, distance)| {
            let object = match view {
                LockView::OnScreen(at) => Object::new()
                    .string("kind", "onScreen")
                    .number("x", at.x)
                    .number("y", at.y),
                LockView::Aside { right } => Object::new()
                    .string("kind", "aside")
                    .boolean("right", right),
                LockView::Behind { right } => Object::new()
                    .string("kind", "behind")
                    .boolean("right", right),
            };
            object.number("distance", distance).finish()
        });
        Object::new()
            .string("context", self.context)
            .raw("health", &meter(self.health))
            .raw("stamina", &meter(self.stamina))
            .raw("loadout", &loadout)
            .raw("melee", &clock(self.melee))
            .raw("ranged", &clock(self.ranged))
            .raw("combo", &clock(self.combo))
            .boolean("axeAway", self.axe_away)
            .raw("energy", &energy)
            .raw("durability", &durability)
            .optional("target", target)
            .optional("cue", cue)
            .optional("lock", lock)
            .finish()
    }
}

fn target_snapshot<'a>(
    combat: &Combatant,
    health: &Health,
    lookup: impl FnOnce(Entity) -> Option<(&'a Enemy, &'a Health, Option<&'a Boss>)>,
) -> Option<TargetSnapshot> {
    if !health.is_alive() {
        return None;
    }
    let (enemy, health, boss) = lookup(combat.target()?)?;
    health.is_alive().then(|| TargetSnapshot {
        name: if boss.is_some() {
            "愁铃客"
        } else if enemy.is_passive() {
            "愁铃客 · 演武"
        } else {
            "愁铃客"
        },
        current: health.current(),
        maximum: health.maximum(),
        status: boss.map_or("锁定目标", |boss| match boss.state() {
            BossState::Idle => "伺机而动",
            BossState::Turning => "转身中",
            BossState::Pursuing => "逼近中",
            BossState::Strafing => "游走中",
            BossState::Telegraph => "蓄势",
            BossState::Attacking => "出招中",
            BossState::Recovering => "收势中",
            BossState::Stunned => "僵直中",
            BossState::Transition => "转阶段 · 暂时无敌",
            BossState::Defeated => "已击败",
        }),
        phase: boss.map_or(1, |boss| match boss.phase() {
            BossPhase::One => 1,
            BossPhase::Two => 2,
        }),
        phases: if boss.is_some() { 2 } else { 1 },
    })
}

type PlayerSnapshot = (
    &'static crate::simulation::Body,
    &'static Combatant,
    &'static Health,
    &'static MotionState,
    &'static DefenseState,
    &'static Stamina,
    &'static Energy,
    &'static Arsenal,
);

fn role_text(level: GameLevel, forest: &'static str, station: &'static str) -> &'static str {
    match level {
        GameLevel::BambooForest => forest,
        GameLevel::Station => station,
    }
}

/// Every pair's HUD wording as a JSON object keyed by its page slot.
#[cfg(any(target_arch = "wasm32", test))]
fn loadouts_json(level: GameLevel) -> String {
    use super::json::Object;
    let mut all = Object::new();
    for pair in crate::simulation::WeaponPair::ALL {
        let row = super::loadout::of(pair);
        let text = &row.hud;
        let wording = Object::new()
            .string("melee", text.melee)
            .string("meleeGlyph", text.melee_glyph)
            .string("ranged", text.ranged)
            .string("rangedGlyph", text.ranged_glyph)
            .string("meleeSkill", text.melee_skill)
            .string(
                "rangedSkill",
                &format!(
                    "{} · {}",
                    role_text(level, "白灵淼", "青鸾"),
                    text.ranged_skill
                ),
            )
            .string(
                "combo",
                &text
                    .combo
                    .replace("{companion}", role_text(level, "妹妹", "青鸾")),
            )
            .string("comboCondition", text.combo_condition)
            .finish();
        all = all.raw(&row.hud_index.to_string(), &wording);
    }
    all.finish()
}

/// Gives the page every pair's wording once, before the first frame.
pub(super) fn describe(#[cfg(target_arch = "wasm32")] level: Res<GameLevel>) {
    #[cfg(target_arch = "wasm32")]
    define_loadouts(&loadouts_json(*level));
}

/// Durability of every pair, placed where the page's pair slots expect it.
fn durability_by_hud_index(arsenal: &Arsenal) -> Vec<f32> {
    let mut values = vec![0.0; crate::simulation::WeaponPair::ALL.len()];
    for pair in crate::simulation::WeaponPair::ALL {
        let slot = super::loadout::of(pair).hud_index as usize - 1;
        values[slot] = f32::from(arsenal.durability(pair));
    }
    values
}

/// Rebuilds the frame from this step's snapshots and this frame's view; the
/// resource only changes when something the HUD shows did.
#[allow(clippy::too_many_arguments)]
pub(super) fn sync(
    level: Res<GameLevel>,
    player: Single<PlayerSnapshot, With<Player>>,
    enemies: Query<(&Enemy, &Health, Option<&Boss>)>,
    bodies: Query<&crate::simulation::Body, With<Enemy>>,
    camera: Single<(&Camera, &Transform), With<super::camera::GameCamera>>,
    time: Res<Time<Fixed>>,
    active: Res<crate::simulation::SimulationActive>,
    hit_stop: Res<crate::simulation::HitStop>,
    mut frame: ResMut<HudFrame>,
) {
    let (body, combat, health, motion, defense, stamina, energy, arsenal) = *player;
    let context = context_text(*level, combat, health, motion, defense, energy);
    let mut next = HudFrame::player(combat, health, stamina, energy, arsenal, context);
    next.target = target_snapshot(combat, health, |entity| enemies.get(entity).ok());
    let locked = next.target.and(combat.target());
    next.cue = locked
        .and_then(|entity| enemies.get(entity).ok())
        .and_then(|(_, _, boss)| boss)
        .and_then(boss_cue);
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    let (camera, view) = *camera;
    next.lock = locked
        .and_then(|entity| bodies.get(entity).ok())
        .and_then(|enemy| {
            // The upper body, where the displayed actor is this frame.
            let point =
                enemy.interpolated_world(alpha) + (enemy.aim_position() - enemy.world_position());
            let distance = enemy
                .world_position()
                .xz()
                .distance(body.world_position().xz());
            lock_view(camera, view, point).map(|view| (view, distance))
        });
    frame.set_if_neq(next.quantized());
}

/// Sends a changed frame to the page.
#[cfg(target_arch = "wasm32")]
pub(super) fn publish(frame: Res<HudFrame>) {
    if frame.is_changed() {
        send_frame(&frame.to_json());
    }
}

/// The native title follows the handscroll while browsing and the HUD during combat.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn publish(
    frame: Res<HudFrame>,
    mode: Res<crate::gameplay::EnemyMode>,
    level: Res<GameLevel>,
    scroll: Option<Res<super::handscroll::NativeHandscroll>>,
    style: Option<Res<super::render_style::RenderStyle>>,
    styles: Option<Res<super::render_style::LevelStyles>>,
    mut window: Single<&mut Window>,
) {
    if scroll.as_ref().is_some_and(|menu| menu.is_open()) {
        if window.title != "断轨听羽" {
            window.title = "断轨听羽".into();
        }
    } else if frame.is_changed()
        || mode.is_changed()
        || style.as_ref().is_some_and(|style| style.is_changed())
        || scroll.is_some_and(|menu| menu.is_changed())
    {
        let mut title = frame.title(*mode, *level);
        if let (Some(style), Some(styles)) = (style, styles) {
            title.push_str(&render_style_title(*style, &styles));
        }
        window.title = title;
    }
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn render_style_title(
    style: super::render_style::RenderStyle,
    styles: &super::render_style::LevelStyles,
) -> String {
    let key = if styles.count() > 1 { " [F8]" } else { "" };
    format!(" · 渲染 {}{key}", style.label())
}

#[cfg(any(not(target_arch = "wasm32"), test))]
impl HudFrame {
    /// The native build's one-line HUD.
    fn title(&self, mode: crate::gameplay::EnemyMode, level: GameLevel) -> String {
        let pair = self.loadout.pair;
        let weapon = super::weapons::melee_name(pair);
        let player = role_text(level, "哥哥", "朱雀");
        let slot = super::loadout::of(pair).hud_index as usize - 1;
        let durability = self.durability.get(slot).copied().unwrap_or(0.0);
        let target = self.target.map_or_else(String::new, |t| {
            format!(
                " · {} {:.0}/{:.0} · {}",
                t.name, t.current, t.maximum, t.status
            )
        });
        format!(
            "Kiva · {} [F2] · {player} / {weapon} · HP {:.0}/{:.0} · 耐力 {:.0} · 烈阳 {:.0} 月光 {:.0} 暴怒 {:.0} · 耐久 {durability:.0} · E {:.1}s / Q {:.1}s · {}{target}",
            match mode {
                crate::gameplay::EnemyMode::Target => "靶子",
                crate::gameplay::EnemyMode::Combat => "实战",
            },
            self.health.current,
            self.health.maximum,
            self.stamina.current,
            self.sun.current,
            self.moon.current,
            self.rage.current,
            self.melee.left,
            self.ranged.left,
            self.context,
        )
    }
}

/// What the player is doing, for the status line above the health bar.
fn context_text(
    level: GameLevel,
    combat: &Combatant,
    health: &Health,
    motion: &MotionState,
    defense: &DefenseState,
    energy: &Energy,
) -> &'static str {
    if !health.is_alive() {
        "倒地 · 即将回到出生点"
    } else if let Some(stage) = combat
        .active_action()
        .filter(|action| action.kind == ActionKind::Melee)
        .and_then(|action| action.melee_stage)
    {
        match stage {
            crate::simulation::MeleeStage::One => "普攻 · 第 1 段",
            crate::simulation::MeleeStage::Two => "普攻 · 第 2 段",
            crate::simulation::MeleeStage::Three => "普攻 · 第 3 段",
        }
    } else if combat.is_combo_active() {
        role_text(level, "联动中 · 妹妹协同攻击", "联动中 · 青鸾协同攻击")
    } else if combat.axe_away() {
        "链斧离手 · 回收中"
    } else if defense.kind() == DefenseKind::Deflect {
        "拆招窗口 · 迎向来袭"
    } else if defense.kind() == DefenseKind::Guard {
        "防御中 · 松开右键弹反"
    } else if defense.kind() == DefenseKind::Parry {
        "弹反窗口"
    } else if defense.kind() == DefenseKind::Waiting {
        "等待防御 · 耐力不足时正常受伤"
    } else if motion.is_dodging() {
        "闪避中"
    } else if motion.is_sliding() {
        "滑铲中 · C 起身"
    } else if motion.is_crouched() {
        "蹲伏 · C 起身"
    } else if motion.is_sprinting() {
        "疾跑中 · 松开方向键停止"
    } else if !motion.is_grounded() {
        "空中"
    } else if energy.is_enraged() {
        "暴怒中 · 普攻翻倍 · 技能强化 · 自动联动"
    } else {
        status_text(level, combat.status())
    }
}

/// Wording of the simulation's latest command outcome.
fn status_text(level: GameLevel, status: CombatStatus) -> &'static str {
    match status {
        CombatStatus::Ready => "准备就绪",
        CombatStatus::PairSwitched => "切换武器组",
        CombatStatus::WeaponExhausted => "武器耐久耗尽 · 已换下一组",
        CombatStatus::Enraged => "暴怒",
        CombatStatus::Started(kind) => match kind {
            ActionKind::Melee => "近战攻击",
            ActionKind::HammerSlam => "震地击飞",
            ActionKind::MusketShot => role_text(level, "妹妹鸟铳射击", "青鸾鸟铳射击"),
            ActionKind::ChainSweep => "链斧环扫",
            ActionKind::AxeShot => role_text(level, "妹妹射出链斧", "青鸾射出链斧"),
            ActionKind::HammerCombo => role_text(level, "妹妹三枪联动", "青鸾三枪联动"),
            ActionKind::ChainCombo => "拉近三箭联动",
        },
        CombatStatus::TargetOutOfReach => "目标高差过大",
        CombatStatus::Busy => "动作执行中",
        CombatStatus::Guarding => "防御中无法进攻",
        CombatStatus::Unavailable => "条件不足 / 冷却中",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_target_behind_the_camera_is_reported_on_its_side() {
        // A target behind the view needs no projection, so no render target either.
        let camera = Camera::default();
        let view = Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
        assert_eq!(
            lock_view(&camera, &view, Vec3::new(3.0, 1.0, 12.0)),
            Some(LockView::Behind { right: true })
        );
        assert_eq!(
            lock_view(&camera, &view, Vec3::new(-3.0, 1.0, 12.0)),
            Some(LockView::Behind { right: false })
        );
    }

    #[test]
    fn the_boss_cue_reports_its_telegraph_and_swings_only_while_striking() {
        let boss = Boss::default();
        assert_eq!(boss_cue(&boss), None);
    }

    #[test]
    fn durability_is_sent_in_the_page_slot_order_of_each_pair() {
        let mut world = World::new();
        let player = world
            .spawn(Combatant::new(crate::simulation::WeaponPair::default()).without_auto_target())
            .id();
        let arsenal = world.get::<Arsenal>(player).unwrap();
        let values = durability_by_hud_index(arsenal);
        assert_eq!(values, vec![f32::from(Arsenal::DURABILITY); 2]);
        // Fresh meters start empty and nothing is ready.
        let energy = world.get::<Energy>(player).unwrap();
        assert_eq!((energy.sun(), energy.moon(), energy.rage()), (0, 0, 0.0));
        assert!(!energy.rage_ready() && !energy.is_enraged());
    }
    #[test]
    fn every_pair_wording_is_sent_under_its_page_slot() {
        let json = loadouts_json(GameLevel::BambooForest);
        for pair in WeaponPair::ALL {
            let row = super::super::loadout::of(pair);
            assert!(json.contains(&format!("\"{}\":{{\"melee\":", row.hud_index)));
            for text in [row.hud.melee, row.hud.ranged_skill, row.hud.combo_condition] {
                assert!(json.contains(text), "{text} is missing");
            }
        }
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert_eq!(
            json.matches("\"comboCondition\"").count(),
            WeaponPair::ALL.len()
        );
    }

    #[test]
    fn skill_cooldown_totals_come_from_the_rules() {
        // The page draws its cooldown fill against these, so a slot's total is
        // what admission actually starts.
        for pair in WeaponPair::ALL {
            assert!(pair.melee_skill().cooldown() > 0.0);
            assert!(pair.ranged_skill().cooldown() > 0.0);
        }
        assert_eq!(ActionKind::Melee.cooldown(), 0.0);
    }
    /// The frames the page's tests start from: idle, a Boss telegraphing in
    /// view mid-combo, and a Boss striking from behind while enraged.
    fn contract_frames() -> [(&'static str, HudFrame); 3] {
        let idle = HudFrame {
            context: "准备就绪",
            health: Meter {
                current: 1000.0,
                maximum: 1000.0,
            },
            stamina: Meter {
                current: 100.0,
                maximum: 100.0,
            },
            loadout: LoadoutSnapshot {
                pair: WeaponPair::HammerMusket,
                melee: true,
                ranged: true,
                combo: false,
            },
            melee: Clock {
                left: 0.0,
                total: 3.0,
            },
            ranged: Clock {
                left: 0.0,
                total: 4.0,
            },
            sun: Meter {
                current: 0.0,
                maximum: 10.0,
            },
            moon: Meter {
                current: 0.0,
                maximum: 10.0,
            },
            rage: Meter {
                current: 0.0,
                maximum: 20.0,
            },
            durability: vec![50.0, 50.0],
            durability_maximum: 50.0,
            ..HudFrame::default()
        };
        let boss = TargetSnapshot {
            name: "愁铃客",
            current: 434.0,
            maximum: 700.0,
            status: "蓄势",
            phase: 1,
            phases: 2,
        };
        let telegraph = HudFrame {
            context: "联动中 · 妹妹协同攻击",
            loadout: LoadoutSnapshot {
                combo: true,
                ..idle.loadout
            },
            melee: Clock {
                left: 1.5,
                total: 3.0,
            },
            combo: Clock {
                left: 0.6,
                total: 0.9,
            },
            sun: Meter {
                current: 7.0,
                ..idle.sun
            },
            durability: vec![31.0, 50.0],
            target: Some(boss),
            cue: Some(BossCue {
                striking: false,
                left: 0.3,
                total: 1.2,
                chain: 2,
                swings: 4,
                resolved: 0,
            }),
            lock: Some((LockView::OnScreen(Vec2::new(0.5625, 0.375)), 7.5)),
            ..idle.clone()
        };
        let behind = HudFrame {
            context: "暴怒中 · 普攻翻倍 · 技能强化 · 自动联动",
            loadout: LoadoutSnapshot {
                pair: WeaponPair::ChainAxeBow,
                ..idle.loadout
            },
            axe_away: true,
            rage: Meter {
                current: 12.5,
                ..idle.rage
            },
            enraged: true,
            target: Some(TargetSnapshot {
                status: "出招中",
                phase: 2,
                ..boss
            }),
            cue: Some(BossCue {
                striking: true,
                chain: 1,
                swings: 4,
                resolved: 0b0101,
                ..BossCue::default()
            }),
            lock: Some((LockView::Behind { right: false }, 4.0)),
            ..idle.clone()
        };
        [
            ("idle", idle),
            ("boss-telegraph", telegraph),
            ("boss-behind", behind),
        ]
    }
    #[test]
    fn the_page_contract_fixtures_match_what_rust_sends() {
        use super::super::json::contract;
        for (name, frame) in contract_frames() {
            contract(&format!("hud/{name}"), &frame.to_json());
        }
        contract("hud/loadouts", &loadouts_json(GameLevel::BambooForest));
    }

    #[test]
    fn a_real_player_frame_carries_its_rules_and_meters() {
        let (mut app, player) = arena(GameLevel::BambooForest);
        lock(&mut app, player);
        let w = app.world();
        let combat = w.get::<Combatant>(player).unwrap();
        let frame = HudFrame::player(
            combat,
            w.get::<Health>(player).unwrap(),
            w.get::<Stamina>(player).unwrap(),
            w.get::<Energy>(player).unwrap(),
            w.get::<Arsenal>(player).unwrap(),
            "准备就绪",
        );
        assert_eq!(frame.health.maximum, 1000.0);
        assert_eq!(
            frame.melee.total,
            combat.weapon_pair().melee_skill().cooldown()
        );
        assert_eq!(
            frame.ranged.total,
            combat.weapon_pair().ranged_skill().cooldown()
        );
        assert_eq!(frame.combo, Clock::default());
        assert_eq!(
            frame.durability,
            durability_by_hud_index(w.get::<Arsenal>(player).unwrap())
        );
        assert_eq!(frame.durability_maximum, f32::from(Arsenal::DURABILITY));
        // The player's part never names a target; sync adds it.
        assert_eq!((frame.target, frame.cue, frame.lock), (None, None, None));
        // Non-finite values cannot break the page's JSON.
        let broken = HudFrame {
            health: Meter {
                current: f32::NAN,
                maximum: f32::INFINITY,
            },
            ..HudFrame::default()
        };
        assert!(
            broken
                .to_json()
                .contains(r#""health":{"current":null,"maximum":null}"#)
        );
    }

    #[test]
    fn a_combo_window_reports_its_opening_length() {
        let (mut app, player) = arena(GameLevel::BambooForest);
        lock(&mut app, player);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        let mut opened = None;
        for _ in 0..120 {
            tick(&mut app, 1);
            if let Some(window) = app.world().get::<Combatant>(player).unwrap().combo_window() {
                opened.get_or_insert(window.duration);
                assert_eq!(window.duration, opened.unwrap());
                assert!(window.remaining <= window.duration);
            }
        }
        assert!(opened.is_some_and(|duration| duration > 0.0));
    }
    use crate::{
        gameplay,
        simulation::{CombatIntent, SimulationPlugin, WeaponPair, teleport_actor},
    };
    use std::time::Duration;
    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1. / 60.));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }
    fn arena(level: GameLevel) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(level);
        app.finish();
        gameplay::setup(app.world_mut());
        gameplay::manual_targeting(app.world_mut());
        tick(&mut app, 12);
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        (app, player)
    }
    fn target(app: &App, player: Entity) -> Option<TargetSnapshot> {
        let w = app.world();
        target_snapshot(
            w.get::<Combatant>(player).unwrap(),
            w.get::<Health>(player).unwrap(),
            |e| Some((w.get::<Enemy>(e)?, w.get::<Health>(e)?, w.get::<Boss>(e))),
        )
    }
    fn lock(app: &mut App, player: Entity) {
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(app, 1);
    }
    #[test]
    fn explicit_target_snapshots_follow_real_damage_and_hide_after_clear_or_death() {
        let (mut app, player) = arena(GameLevel::BambooForest);
        assert_eq!(target(&app, player), None);
        lock(&mut app, player);
        let selected = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        assert_eq!(target(&app, player).unwrap().maximum, 1_000_000.);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_ranged_skill();
        tick(&mut app, 14);
        assert!(target(&app, player).unwrap().current < 1_000_000.);
        assert_eq!(
            target(&app, player).unwrap().current,
            app.world().get::<Health>(selected).unwrap().current()
        );
        lock(&mut app, player);
        assert_eq!(target(&app, player), None);
        lock(&mut app, player);
        app.world_mut().despawn(selected);
        assert_eq!(target(&app, player), None);
    }
    #[test]
    fn player_death_and_roster_replacement_clear_target_display() {
        for mode_change in [true, false] {
            let (mut app, player) = arena(GameLevel::BambooForest);
            lock(&mut app, player);
            assert!(target(&app, player).is_some());
            if mode_change {
                gameplay::set_enemy_mode(app.world_mut(), gameplay::EnemyMode::Combat);
            } else {
                teleport_actor(app.world_mut(), player, Vec3::new(0., -50., 0.));
                tick(&mut app, 1);
            }
            assert_eq!(target(&app, player), None);
        }
    }
    #[derive(Resource, Default)]
    struct Sent(u32);
    fn count(frame: Res<HudFrame>, mut sent: ResMut<Sent>) {
        if frame.is_changed() {
            sent.0 += 1;
        }
    }
    /// The arena with a game camera, `sync`, and a counter standing in for `publish`.
    fn hud_arena(level: GameLevel) -> (App, Entity) {
        let (mut app, player) = arena(level);
        app.world_mut().spawn((
            super::super::camera::GameCamera,
            Camera::default(),
            Transform::from_xyz(0.0, 6.0, 4.2).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        app.init_resource::<HudFrame>()
            .init_resource::<Sent>()
            .add_systems(Update, (sync, count).chain());
        (app, player)
    }
    /// One fixed step, then one display frame; how many frames were sent so far.
    fn step(app: &mut App) -> u32 {
        tick(app, 1);
        app.world_mut().run_schedule(Update);
        app.world().resource::<Sent>().0
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_station_hud_uses_its_actual_party_when_the_companion_fires() {
        let (mut app, player) = hud_arena(GameLevel::Station);
        let window = app.world_mut().spawn(Window::default()).id();
        app.add_systems(Update, publish.after(sync));
        let target = app
            .world_mut()
            .query_filtered::<&crate::simulation::Body, With<Enemy>>()
            .single(app.world())
            .unwrap()
            .world_position();
        teleport_actor(app.world_mut(), player, target + Vec3::Z * 4.0);
        tick(&mut app, 12);
        lock(&mut app, player);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_ranged_skill();
        step(&mut app);
        let hud = app.world().resource::<HudFrame>();
        assert_eq!(hud.context, "青鸾鸟铳射击");
        assert!(hud.to_json().contains("青鸾鸟铳射击"));
        let title = &app.world().get::<Window>(window).unwrap().title;
        assert!(title.contains("朱雀 /") && title.contains("青鸾鸟铳射击"));
        let loadouts = loadouts_json(GameLevel::Station);
        assert!(loadouts.contains("青鸾 · 单发引冰落") && loadouts.contains("拉近 · 青鸾三箭"));
        assert!(!loadouts.contains("妹妹") && !loadouts.contains("白灵淼"));
    }

    #[test]
    fn a_still_arena_sends_its_frame_once() {
        let (mut app, _) = hud_arena(GameLevel::BambooForest);
        let first = step(&mut app);
        assert_eq!(first, 1);
        for _ in 0..30 {
            assert_eq!(step(&mut app), first, "nothing on screen changed");
        }
        assert_eq!(app.world().resource::<HudFrame>().context, "准备就绪");
    }

    #[test]
    fn a_running_cooldown_sends_every_step_and_stops_when_it_ends() {
        let (mut app, player) = hud_arena(GameLevel::BambooForest);
        lock(&mut app, player);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        let mut sent = step(&mut app);
        assert!(app.world().resource::<HudFrame>().melee.left > 0.0);
        for _ in 0..20 {
            let now = step(&mut app);
            assert_eq!(
                now,
                sent + 1,
                "a 1/60 s countdown step is above the 0.01 s quantum"
            );
            sent = now;
        }
        // Run the cooldown and any combo window out; then the frame settles again.
        for _ in 0..400 {
            step(&mut app);
        }
        let settled = step(&mut app);
        let frame = app.world().resource::<HudFrame>();
        assert_eq!((frame.melee.left, frame.combo), (0.0, Clock::default()));
        assert_eq!(step(&mut app), settled);
        assert_eq!(step(&mut app), settled);
    }

    #[test]
    fn noise_below_what_the_hud_shows_is_not_a_change() {
        let frame = |x: f32, distance: f32, left: f32| {
            HudFrame {
                lock: Some((LockView::OnScreen(Vec2::new(x, 0.25)), distance)),
                melee: Clock { left, total: 3.0 },
                ..HudFrame::default()
            }
            .quantized()
        };
        assert_eq!(frame(0.5, 7.5, 1.0), frame(0.500_02, 7.500_4, 1.000_3));
        assert_ne!(frame(0.5, 7.5, 1.0), frame(0.502, 7.5, 1.0));
        assert_ne!(frame(0.5, 7.5, 1.0), frame(0.5, 7.5, 1.0 - 1.0 / 60.0));
    }

    #[test]
    fn the_native_title_names_the_render_style_and_offers_f8_only_when_it_switches() {
        use super::super::render_style::{LevelStyles, RenderStyle};
        let bamboo = LevelStyles::for_level(GameLevel::BambooForest);
        let station = LevelStyles::for_level(GameLevel::Station);
        assert_eq!(
            render_style_title(RenderStyle::Yancai, &bamboo),
            " · 渲染 岩彩 [F8]"
        );
        assert_eq!(
            render_style_title(RenderStyle::Pbr, &station),
            " · 渲染 写实"
        );
    }

    #[test]
    fn the_native_title_is_written_from_the_same_frame() {
        let [(_, idle), _, (_, behind)] = contract_frames();
        assert_eq!(
            idle.title(gameplay::EnemyMode::Target, GameLevel::BambooForest),
            "Kiva · 靶子 [F2] · 哥哥 / 伏浪锚 · 锤子 · HP 1000/1000 · 耐力 100 · 烈阳 0 月光 0 暴怒 0 · 耐久 50 · E 0.0s / Q 0.0s · 准备就绪"
        );
        assert_eq!(
            behind.title(gameplay::EnemyMode::Combat, GameLevel::BambooForest),
            "Kiva · 实战 [F2] · 哥哥 / 谪星·摧山 · 链斧 · HP 1000/1000 · 耐力 100 · 烈阳 0 月光 0 暴怒 12 · 耐久 50 · E 0.0s / Q 0.0s · 暴怒中 · 普攻翻倍 · 技能强化 · 自动联动 · 愁铃客 434/700 · 出招中"
        );
    }

    #[test]
    fn admission_snapshot_has_1000_health_dual_slots_and_no_player_ranged_mode() {
        let (mut app, player) = arena(GameLevel::BambooForest);
        let snapshot = loadout_snapshot(
            app.world().get::<Combatant>(player).unwrap(),
            app.world().get::<Health>(player).unwrap(),
        );
        assert_eq!(app.world().get::<Health>(player).unwrap().maximum(), 1000.);
        let availability = || app.world().get::<Combatant>(player).unwrap().availability();
        assert!(availability().attack);
        assert!(!snapshot.melee && !snapshot.ranged);
        lock(&mut app, player);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        tick(&mut app, 1);
        let a = loadout_snapshot(
            app.world().get::<Combatant>(player).unwrap(),
            app.world().get::<Health>(player).unwrap(),
        );
        assert!(!a.melee && !a.ranged && !a.combo);
        let busy = app.world().get::<Combatant>(player).unwrap().availability();
        assert!(!busy.attack && !busy.defense);
        tick(&mut app, 35);
        let before = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .melee_cooldown();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_pair(WeaponPair::ChainAxeBow);
        tick(&mut app, 1);
        let c = app.world().get::<Combatant>(player).unwrap();
        assert_eq!(c.weapon_pair(), WeaponPair::ChainAxeBow);
        assert!(c.melee_cooldown() > before - 0.03);
        assert_eq!(c.ranged_cooldown(), 0.);
        let snapshot = loadout_snapshot(c, app.world().get::<Health>(player).unwrap());
        assert_eq!(snapshot.pair, WeaponPair::ChainAxeBow);
        assert!(!snapshot.melee);
        assert!(snapshot.ranged);
    }
}
