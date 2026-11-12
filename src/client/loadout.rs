//! How each weapon pair looks and sounds: one row per pair.
//!
//! Presentation modules read these rows instead of branching on a concrete
//! pair, so a new pair is a new row here (plus its skills in the simulation
//! and its files under `assets/`). Rows choose among presentation forms the
//! client already implements; a new form still needs code where it is drawn.
use crate::simulation::{WeaponMode, WeaponPair};

/// The body gesture of an ordinary melee swing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Swing {
    Hammer,
    Axe,
}

/// What a fired shot looks like on arrival.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Projectile {
    Bullet,
    Arrow,
}

/// How the camera frames the pair's combo sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComboCamera {
    /// Low behind the shoulder looking up at the target; shots kick it back.
    Sniper,
    /// Level with the chest while the player is pulled in.
    EyeLevel,
}

/// A native source clip and the note the inspection trace shows for it.
pub(super) struct SourceClip {
    pub name: &'static str,
    pub adaptation: &'static str,
}

pub(super) struct Loadout {
    /// The pair's number in the `window.kivaHud.renderLoadout` contract; the
    /// native build has no page to send it to.
    #[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
    pub hud_index: u32,
    pub melee_name: &'static str,
    pub melee_model: &'static str,
    pub ranged_model: &'static str,
    /// The ranged model carries a part worn on the companion's spine.
    pub spine_accessory: bool,
    /// The melee weapon is thrown on a chain and drawn while it is away.
    pub tethered: bool,
    pub swing: Swing,
    pub projectile: Projectile,
    /// The cue of the companion's shot.
    pub shot_sound: &'static str,
    pub companion_hold: SourceClip,
    pub companion_attack: SourceClip,
    pub combo_camera: ComboCamera,
}

impl Loadout {
    pub(super) fn model(&self, mode: WeaponMode) -> &'static str {
        match mode {
            WeaponMode::Melee => self.melee_model,
            WeaponMode::Ranged => self.ranged_model,
        }
    }
}

// Verified female_fk overrides share these male-named original FK clips.
// Target lower body and physical grips remain the established Kiva choreography.
const HAMMER_MUSKET: Loadout = Loadout {
    hud_index: 1,
    melee_name: "伏浪锚 · 锤子",
    melee_model: "fulangmao/weapon.bin",
    ranged_model: "yinbingluo/weapon.bin",
    spine_accessory: false,
    tethered: false,
    swing: Swing::Hammer,
    projectile: Projectile::Bullet,
    shot_sound: "naraka_gun",
    companion_hold: SourceClip {
        name: "male_fk_hold_idle_01",
        adaptation: "verified shared female_fk source on canonical Avatar; upper body + CC0 lower body / target grip IK",
    },
    companion_attack: SourceClip {
        name: "male_fk_attack_01",
        adaptation: "verified shared female_fk source on canonical Avatar; upper body + CC0 lower body / Kiva shot clock / target grip IK",
    },
    combo_camera: ComboCamera::Sniper,
};

const CHAIN_AXE_BOW: Loadout = Loadout {
    hud_index: 2,
    melee_name: "谪星·摧山 · 链斧",
    melee_model: "zhexingcuishan/weapon.bin",
    ranged_model: "yizhiying/weapon.bin",
    spine_accessory: true,
    tethered: true,
    swing: Swing::Axe,
    projectile: Projectile::Arrow,
    shot_sound: "naraka_bow",
    companion_hold: SourceClip {
        name: "male_bow_hold_idle_01",
        adaptation: "verified shared female bow source; Kiva upper-body/fingers + CC0 lower body / grip IK",
    },
    companion_attack: SourceClip {
        name: "male_bow_attack_01",
        adaptation: "verified shared female bow source; Kiva shot clock / upper body + CC0 lower body / grip IK",
    },
    combo_camera: ComboCamera::EyeLevel,
};

pub(super) fn of(pair: WeaponPair) -> &'static Loadout {
    match pair {
        WeaponPair::HammerMusket => &HAMMER_MUSKET,
        WeaponPair::ChainAxeBow => &CHAIN_AXE_BOW,
    }
}

/// Every weapon model file, for the runtime asset manifest.
pub(super) fn model_paths() -> Vec<&'static str> {
    WeaponPair::ALL
        .into_iter()
        .flat_map(|pair| [of(pair).melee_model, of(pair).ranged_model])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_distinct_where_the_page_and_the_asset_cache_need_them_to_be() {
        let rows = WeaponPair::ALL.map(of);
        let mut hud: Vec<_> = rows.iter().map(|row| row.hud_index).collect();
        hud.sort_unstable();
        hud.dedup();
        // web/hud.js accepts exactly these numbers.
        assert_eq!(hud, [1, 2]);
        let mut models = model_paths();
        assert_eq!(models.len(), WeaponPair::ALL.len() * 2);
        models.sort_unstable();
        models.dedup();
        assert_eq!(models.len(), WeaponPair::ALL.len() * 2);
        // The chain rope has one flying-weapon view.
        assert_eq!(rows.iter().filter(|row| row.tethered).count(), 1);
    }
}
