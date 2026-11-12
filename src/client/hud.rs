//! Display-only snapshots. Simulation owns all admission, health and clocks.
use crate::{
    gameplay::Player,
    simulation::{
        ActionKind, Arsenal, Boss, BossPhase, BossState, CombatStatus, Combatant, DefenseKind,
        DefenseState, Enemy, Energy, Health, MotionState, Stamina,
    },
};
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = render)]
    fn render(
        persona: &str,
        weapon: &str,
        fraction: f32,
        health: f32,
        combo: f32,
        cooldown: f32,
        context: &str,
    );
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderLoadout)]
    fn render_loadout(
        pair: u32,
        maximum: f32,
        attack: bool,
        melee: bool,
        ranged: bool,
        guard: bool,
        combo: bool,
    );
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderResources)]
    fn render_resources(
        stamina: f32,
        maximum: f32,
        melee: f32,
        ranged: f32,
        axe_away: bool,
        defense: u32,
    );
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderEnergy)]
    fn render_energy(
        sun: f32,
        sun_maximum: f32,
        moon: f32,
        moon_maximum: f32,
        rage: f32,
        rage_maximum: f32,
        enraged: bool,
        rage_ready: bool,
    );
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderDurability)]
    fn render_durability(durability: Vec<f32>, maximum: f32);
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderTarget)]
    fn render_target(name: &str, current: f32, maximum: f32, status: &str);
}
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, PartialEq)]
struct LoadoutSnapshot {
    pair: u32,
    maximum: f32,
    attack: bool,
    melee: bool,
    ranged: bool,
    guard: bool,
    combo: bool,
}
#[cfg(any(target_arch = "wasm32", test))]
fn loadout_snapshot(combat: &Combatant, health: &Health) -> LoadoutSnapshot {
    let a = combat.availability();
    let alive = health.is_alive();
    LoadoutSnapshot {
        pair: super::loadout::of(combat.weapon_pair()).hud_index,
        maximum: health.maximum(),
        attack: alive && a.attack,
        melee: alive && a.melee_skill,
        ranged: alive && a.ranged_skill,
        guard: alive && a.defense,
        combo: alive && a.combo,
    }
}
#[derive(Debug, PartialEq)]
struct TargetSnapshot {
    name: &'static str,
    current: f32,
    maximum: f32,
    status: &'static str,
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
        name: if let Some(boss) = boss {
            match boss.phase() {
                BossPhase::One => "愁铃客 · 一阶段",
                BossPhase::Two => "愁铃客 · 二阶段",
            }
        } else if enemy.is_passive() {
            "愁铃客 · 演武"
        } else {
            "愁铃客"
        },
        current: health.current(),
        maximum: health.maximum(),
        status: boss.map_or("锁定目标", |boss| match boss.state() {
            BossState::Idle => "伺机而动",
            BossState::Pursuing => "逼近中",
            BossState::Strafing => "游走中",
            BossState::Telegraph => "蓄势 · 留意来袭",
            BossState::Attacking => "出招中",
            BossState::Recovering => "收势中",
            BossState::Stunned => "僵直中",
            BossState::Transition => "转阶段 · 暂时无敌",
            BossState::Defeated => "已击败",
        }),
    })
}

type PlayerSnapshot = (
    &'static Combatant,
    &'static Health,
    &'static MotionState,
    &'static DefenseState,
    &'static Stamina,
    &'static Energy,
    &'static Arsenal,
);

/// Durability of every pair, placed where the page's pair slots expect it.
#[cfg(any(target_arch = "wasm32", test))]
fn durability_by_hud_index(arsenal: &Arsenal) -> Vec<f32> {
    let mut values = vec![0.0; crate::simulation::WeaponPair::ALL.len()];
    for pair in crate::simulation::WeaponPair::ALL {
        let slot = super::loadout::of(pair).hud_index as usize - 1;
        values[slot] = f32::from(arsenal.durability(pair));
    }
    values
}

pub(super) fn sync(
    player: Single<PlayerSnapshot, With<Player>>,
    enemies: Query<(&Enemy, &Health, Option<&Boss>)>,
    mut window: Single<&mut Window>,
    #[cfg(not(target_arch = "wasm32"))] mode: Res<crate::gameplay::EnemyMode>,
) {
    let (combat, health, motion, defense, stamina, energy, arsenal) = *player;
    let target = target_snapshot(combat, health, |entity| enemies.get(entity).ok());
    let weapon = super::weapons::melee_name(combat.weapon_pair());
    let context = if !health.is_alive() {
        "倒地 · 即将回到出生点"
    } else if let Some(stage) = combat
        .active_action()
        .filter(|action| action.kind == crate::simulation::ActionKind::Melee)
        .and_then(|action| action.melee_stage)
    {
        match stage {
            crate::simulation::MeleeStage::One => "普攻 · 第 1 段",
            crate::simulation::MeleeStage::Two => "普攻 · 第 2 段",
            crate::simulation::MeleeStage::Three => "普攻 · 第 3 段",
        }
    } else if combat.is_combo_active() {
        "联动中 · 妹妹协同攻击"
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
        status_text(combat.status())
    };
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &mut window;
        let a = loadout_snapshot(combat, health);
        render_loadout(
            a.pair, a.maximum, a.attack, a.melee, a.ranged, a.guard, a.combo,
        );
        let kind = match defense.kind() {
            DefenseKind::Idle => 0,
            DefenseKind::Deflect => 1,
            DefenseKind::Waiting => 2,
            DefenseKind::Guard => 3,
            DefenseKind::Parry => 4,
        };
        render_resources(
            stamina.current(),
            stamina.maximum(),
            combat.melee_cooldown(),
            combat.ranged_cooldown(),
            combat.axe_away(),
            kind,
        );
        render_energy(
            f32::from(energy.sun()),
            f32::from(Energy::SUN_MAX),
            f32::from(energy.moon()),
            f32::from(Energy::MOON_MAX),
            energy.rage(),
            Energy::RAGE_MAX,
            energy.is_enraged(),
            energy.rage_ready(),
        );
        render_durability(
            durability_by_hud_index(arsenal),
            f32::from(Arsenal::DURABILITY),
        );
        if let Some(t) = target {
            render_target(t.name, t.current, t.maximum, t.status);
        } else {
            render_target("", 0., 0., "");
        }
        render(
            "哥哥",
            weapon,
            health.current() / health.maximum(),
            health.current(),
            combat.combo_remaining(),
            combat.melee_cooldown(),
            context,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let target = target.map_or_else(String::new, |t| {
            format!(
                " · {} {:.0}/{:.0} · {}",
                t.name, t.current, t.maximum, t.status
            )
        });
        window.title = format!(
            "Kiva · {} [F2] · 哥哥 / {weapon} · HP {:.0}/{:.0} · 耐力 {:.0} · 烈阳 {} 月光 {} 暴怒 {:.0} · 耐久 {} · E {:.1}s / Q {:.1}s · {context}{target}",
            match *mode {
                crate::gameplay::EnemyMode::Target => "靶子",
                crate::gameplay::EnemyMode::Combat => "实战",
            },
            health.current(),
            health.maximum(),
            stamina.current(),
            energy.sun(),
            energy.moon(),
            energy.rage(),
            arsenal.durability(combat.weapon_pair()),
            combat.melee_cooldown(),
            combat.ranged_cooldown()
        );
    }
}
/// Wording of the simulation's latest command outcome.
fn status_text(status: CombatStatus) -> &'static str {
    match status {
        CombatStatus::Ready => "准备就绪",
        CombatStatus::PairSwitched => "切换武器组",
        CombatStatus::WeaponExhausted => "武器耐久耗尽 · 已换下一组",
        CombatStatus::Enraged => "暴怒",
        CombatStatus::Started(kind) => match kind {
            ActionKind::Melee => "近战攻击",
            ActionKind::HammerSlam => "震地击飞",
            ActionKind::MusketShot => "妹妹鸟铳射击",
            ActionKind::ChainSweep => "链斧环扫",
            ActionKind::AxeShot => "妹妹射出链斧",
            ActionKind::HammerCombo => "妹妹三枪联动",
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
    fn durability_is_sent_in_the_page_slot_order_of_each_pair() {
        let mut world = World::new();
        let player = world
            .spawn(Combatant::new(crate::simulation::WeaponPair::default()))
            .id();
        let arsenal = world.get::<Arsenal>(player).unwrap();
        let values = durability_by_hud_index(arsenal);
        assert_eq!(values, vec![f32::from(Arsenal::DURABILITY); 2]);
        // Fresh meters start empty and nothing is ready.
        let energy = world.get::<Energy>(player).unwrap();
        assert_eq!((energy.sun(), energy.moon(), energy.rage()), (0, 0, 0.0));
        assert!(!energy.rage_ready() && !energy.is_enraged());
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
    fn arena() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        gameplay::setup(app.world_mut());
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
        let (mut app, player) = arena();
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
            let (mut app, player) = arena();
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
    #[test]
    fn admission_snapshot_has_1000_health_dual_slots_and_no_player_ranged_mode() {
        let (mut app, player) = arena();
        let snapshot = loadout_snapshot(
            app.world().get::<Combatant>(player).unwrap(),
            app.world().get::<Health>(player).unwrap(),
        );
        assert_eq!(snapshot.maximum, 1000.);
        assert!(snapshot.attack);
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
        assert!(!a.attack && !a.melee && !a.ranged && !a.guard);
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
        assert_eq!(snapshot.pair, 2);
        assert!(!snapshot.melee);
        assert!(snapshot.ranged);
    }
}
