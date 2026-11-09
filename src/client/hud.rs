//! Converts simulation state into a display-only HUD; no DOM value enters the simulation.
use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{Combatant, Health, MotionState, WeaponMode, WeaponSet},
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
}

pub(super) fn sync(
    player: Single<(&Combatant, &Health, &MotionState), With<Player>>,
    mut window: Single<&mut Window>,
) {
    let (combat, health, motion) = *player;
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
        "火炮跃进 · Tab 接战斧旋转"
    } else if motion.is_blocking() {
        if motion.is_crouched() {
            "蹲伏格挡 · C 起身"
        } else {
            "格挡 · 防御前方攻击"
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
        window.title = format!(
            "Kiva · {persona} / {weapon} · HP {:.0} · E {:.1}s · {context}{combo}",
            health.current(),
            combat.cooldown(),
        );
    }
}
