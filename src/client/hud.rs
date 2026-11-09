//! Converts simulation state into a display-only HUD; no DOM value enters the simulation.
use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{
        Boss, BossPhase, BossState, Combatant, Enemy, Health, MotionState, WeaponMode, WeaponSet,
    },
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = render)]
    fn render(
        persona: &str,
        weapon: &str,
        health_fraction: f32,
        health: f32,
        combo: f32,
        cooldown: f32,
        context: &str,
    );
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderLoadout)]
    fn render_loadout(
        set: u32,
        mode: u32,
        maximum: f32,
        attack_ready: bool,
        skill_ready: bool,
        guard_ready: bool,
        combo_ready: bool,
    );
    #[wasm_bindgen(js_namespace = ["window", "kivaHud"], js_name = renderTarget)]
    fn render_target(name: &str, current: f32, maximum: f32, status: &str);
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, PartialEq)]
struct LoadoutSnapshot {
    set: u32,
    mode: u32,
    maximum: f32,
    attack_ready: bool,
    skill_ready: bool,
    guard_ready: bool,
    combo_ready: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
fn loadout_snapshot(combat: &Combatant, health: &Health, motion: &MotionState) -> LoadoutSnapshot {
    // These are display hints from the current action state. The fixed-step
    // simulation still owns validation of targets, surfaces and incoming input.
    let can_act = health.is_alive()
        && !motion.is_dodging()
        && !motion.is_parrying()
        && !motion.is_sliding()
        && !combat.is_spinning()
        && !combat.is_sniping();
    let starter_mode = matches!(
        (combat.weapon_set(), combat.weapon_mode()),
        (WeaponSet::HammerSniper, WeaponMode::Melee) | (WeaponSet::AxeCannon, WeaponMode::Ranged)
    );
    LoadoutSnapshot {
        set: match combat.weapon_set() {
            WeaponSet::HammerSniper => 1,
            WeaponSet::AxeCannon => 2,
        },
        mode: u32::from(combat.weapon_mode() == WeaponMode::Ranged),
        maximum: health.maximum(),
        attack_ready: can_act && !motion.is_leaping() && combat.attack_cooldown() == 0.0,
        skill_ready: can_act && motion.is_grounded() && combat.cooldown() == 0.0 && starter_mode,
        guard_ready: health.is_alive()
            && motion.is_grounded()
            && !motion.is_dodging()
            && !motion.is_leaping()
            && !motion.is_sliding()
            && !combat.is_sniping()
            && !combat.is_spinning()
            && motion.parry_cooldown() == 0.0,
        combo_ready: can_act && combat.combo_remaining() > 0.0,
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
            BossState::Deflected => "卸势破绽",
            BossState::Transition => "转阶段 · 暂时无敌",
            BossState::Defeated => "已击败",
        }),
    })
}

pub(super) fn sync(
    player: Single<(&Combatant, &Health, &MotionState), With<Player>>,
    enemies: Query<(&Enemy, &Health, Option<&Boss>)>,
    mut window: Single<&mut Window>,
    #[cfg(not(target_arch = "wasm32"))] mode: Res<crate::gameplay::EnemyMode>,
) {
    let (combat, health, motion) = *player;
    let target = target_snapshot(combat, health, |entity| enemies.get(entity).ok());
    let persona = combat.persona();
    let weapon = combat.weapon_name();
    let context = if !health.is_alive() {
        "倒地 · 即将回到出生点"
    } else if motion.is_dodging() {
        "闪避中"
    } else if motion.is_sliding() {
        "滑铲中 · C 起身"
    } else if combat.is_spinning() {
        "联动 · 空中回旋"
    } else if motion.is_leaping() {
        "火炮跃进 · Tab 接长刀旋转"
    } else if motion.is_parrying() {
        if motion.is_crouched() {
            "卸势窗口 · C 起身"
        } else {
            "卸势窗口 · 迎向来袭"
        }
    } else if motion.is_crouched() {
        "蹲伏 · C 起身"
    } else if motion.is_sprinting() {
        "疾跑中 · 松开方向键停止"
    } else if !motion.is_grounded() {
        "空中"
    } else if combat.weapon_set() == WeaponSet::AxeCannon
        && combat.weapon_mode() == WeaponMode::Ranged
        && combat.status() == "准备就绪"
    {
        "鼠标上下调整落点 · E 跃进"
    } else {
        combat.status()
    };
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &mut window;
        let loadout = loadout_snapshot(combat, health, motion);
        render_loadout(
            loadout.set,
            loadout.mode,
            loadout.maximum,
            loadout.attack_ready,
            loadout.skill_ready,
            loadout.guard_ready,
            loadout.combo_ready,
        );
        if let Some(target) = target {
            render_target(target.name, target.current, target.maximum, target.status);
        } else {
            render_target("", 0.0, 0.0, "");
        }
        render(
            persona,
            weapon,
            health.current() / health.maximum(),
            health.current(),
            combat.combo_remaining(),
            combat.cooldown(),
            context,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let combo = if combat.combo_remaining() > 0.0 {
            " · Tab 联动"
        } else {
            ""
        };
        let target = target.map_or_else(String::new, |target| {
            format!(
                " · {} {:.0}/{:.0} · {}",
                target.name, target.current, target.maximum, target.status
            )
        });
        window.title = format!(
            "Kiva · {}模式 [F2] · {persona} / {weapon} · HP {:.0} · E {:.1}s · {context}{combo}{target}",
            mode.label(),
            health.current(),
            combat.cooldown(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gameplay::{self, EnemyMode},
        simulation::{ActionIntent, CombatIntent, SimulationPlugin, teleport_actor},
    };
    use std::time::Duration;

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

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn target(app: &App, player: Entity) -> Option<TargetSnapshot> {
        let world = app.world();
        target_snapshot(
            world.get::<Combatant>(player).unwrap(),
            world.get::<Health>(player).unwrap(),
            |entity| {
                Some((
                    world.get::<Enemy>(entity)?,
                    world.get::<Health>(entity)?,
                    world.get::<Boss>(entity),
                ))
            },
        )
    }

    fn loadout(app: &App, player: Entity) -> LoadoutSnapshot {
        let world = app.world();
        loadout_snapshot(
            world.get::<Combatant>(player).unwrap(),
            world.get::<Health>(player).unwrap(),
            world.get::<MotionState>(player).unwrap(),
        )
    }

    fn request(app: &mut App, player: Entity, command: impl FnOnce(&mut CombatIntent)) {
        command(&mut app.world_mut().get_mut::<CombatIntent>(player).unwrap());
    }

    #[test]
    fn sniper_sequence_disables_new_actions_until_its_recovery_ends() {
        let (mut app, player) = arena();
        request(&mut app, player, CombatIntent::request_skill);
        tick(&mut app, 1);
        request(&mut app, player, CombatIntent::request_combo);
        tick(&mut app, 1);
        assert!(app.world().get::<Combatant>(player).unwrap().is_sniping());
        let snapshot = loadout(&app, player);
        assert_eq!((snapshot.set, snapshot.mode), (1, 1));
        assert!(
            !snapshot.attack_ready
                && !snapshot.skill_ready
                && !snapshot.guard_ready
                && !snapshot.combo_ready
        );
        tick(&mut app, 20);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .sniper_sequence()
                .unwrap()
                .has_fired()
        );
        assert!(!loadout(&app, player).guard_ready);
        tick(&mut app, 28);
        assert!(!app.world().get::<Combatant>(player).unwrap().is_sniping());
        assert!(loadout(&app, player).guard_ready);
    }

    #[test]
    fn selected_target_snapshot_reads_live_damage_and_health_maximum() {
        let (mut app, player) = arena();
        let selected = app.world().get::<Combatant>(player).unwrap().target();
        assert_eq!(
            target(&app, player),
            Some(TargetSnapshot {
                name: "愁铃客 · 演武",
                current: 1_000_000.0,
                maximum: 1_000_000.0,
                status: "锁定目标",
            })
        );
        request(&mut app, player, CombatIntent::request_skill);
        tick(&mut app, 1);
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().target(),
            selected
        );
        assert_eq!(target(&app, player).unwrap().current, 999_980.0);
        assert_eq!(target(&app, player).unwrap().maximum, 1_000_000.0);
        // The HUD must read a changed component even before another combat tick.
        app.world_mut()
            .entity_mut(selected.unwrap())
            .insert(Health::new(500.0));
        assert_eq!(target(&app, player).unwrap().current, 500.0);
        assert_eq!(target(&app, player).unwrap().maximum, 500.0);
    }

    #[test]
    fn combat_target_reports_boss_phase_and_action_state_from_simulation() {
        let (mut app, player) = arena();
        gameplay::set_enemy_mode(app.world_mut(), EnemyMode::Combat);
        tick(&mut app, 1);
        let snapshot = target(&app, player).unwrap();
        assert_eq!(snapshot.name, "愁铃客 · 一阶段");
        assert_eq!(snapshot.maximum, Boss::PHASE_HEALTH);
        let selected = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        let boss = app.world().get::<Boss>(selected).unwrap();
        assert_eq!(boss.state(), BossState::Strafing);
        assert_eq!(snapshot.status, "游走中");
    }

    #[test]
    fn target_snapshot_hides_missing_replaced_and_defeated_actors() {
        for cause in ["despawn", "mode-change", "enemy-defeat", "player-defeat"] {
            let (mut app, player) = arena();
            let selected = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .target()
                .unwrap();
            assert!(target(&app, player).is_some());
            match cause {
                "despawn" => {
                    app.world_mut().despawn(selected);
                }
                "mode-change" => gameplay::set_enemy_mode(app.world_mut(), EnemyMode::Combat),
                "enemy-defeat" => {
                    app.world_mut()
                        .entity_mut(selected)
                        .insert(Health::new(1.0));
                    request(&mut app, player, |intent| {
                        intent.request_switch_weapon();
                        intent.request_attack();
                    });
                    tick(&mut app, 1);
                    assert!(!app.world().get::<Health>(selected).unwrap().is_alive());
                }
                "player-defeat" => {
                    teleport_actor(app.world_mut(), player, Vec3::new(0.0, -50.0, 0.0));
                    tick(&mut app, 1);
                    assert!(!app.world().get::<Health>(player).unwrap().is_alive());
                    assert!(app.world().get::<Health>(selected).unwrap().is_alive());
                }
                _ => unreachable!(),
            }
            assert_eq!(target(&app, player), None, "cause: {cause}");
        }
    }

    #[test]
    fn loadout_snapshot_tracks_real_weapon_switches_cooldowns_and_airborne_actions() {
        let (mut app, player) = arena();
        assert_eq!(
            loadout(&app, player),
            LoadoutSnapshot {
                set: 1,
                mode: 0,
                maximum: 100.0,
                attack_ready: true,
                skill_ready: true,
                guard_ready: true,
                combo_ready: false,
            }
        );
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        let hints = loadout(&app, player);
        assert!(
            !hints.attack_ready && !hints.skill_ready && !hints.guard_ready && !hints.combo_ready
        );
        tick(&mut app, 30);

        request(&mut app, player, CombatIntent::request_switch_weapon);
        tick(&mut app, 1);
        let hints = loadout(&app, player);
        assert_eq!((hints.set, hints.mode), (1, 1));
        assert!(hints.attack_ready && hints.guard_ready);
        assert!(!hints.skill_ready);
        request(&mut app, player, |intent| {
            intent.request_set(WeaponSet::AxeCannon)
        });
        tick(&mut app, 1);
        let hints = loadout(&app, player);
        assert_eq!((hints.set, hints.mode), (2, 0));
        assert!(!hints.skill_ready);
        request(&mut app, player, CombatIntent::request_switch_weapon);
        tick(&mut app, 1);
        assert!(loadout(&app, player).skill_ready);

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_jump();
        tick(&mut app, 1);
        let hints = loadout(&app, player);
        assert!(hints.attack_ready);
        assert!(!hints.skill_ready && !hints.guard_ready);
        tick(&mut app, 70);

        let landing = app
            .world()
            .get::<crate::simulation::Body>(player)
            .unwrap()
            .position()
            + Vec2::X * 6.0;
        request(&mut app, player, |intent| {
            intent.set_aim(landing);
            intent.request_skill();
        });
        tick(&mut app, 1);
        assert!(app.world().get::<Combatant>(player).unwrap().cooldown() > 0.0);
        let hints = loadout(&app, player);
        assert!(!hints.attack_ready && !hints.skill_ready && !hints.guard_ready);
        assert!(hints.combo_ready);
        request(&mut app, player, CombatIntent::request_combo);
        tick(&mut app, 1);
        let hints = loadout(&app, player);
        assert_eq!((hints.set, hints.mode), (2, 0));
        assert!(
            !hints.attack_ready && !hints.skill_ready && !hints.guard_ready && !hints.combo_ready
        );
    }
}
