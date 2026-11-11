//! Concrete first-version weapon pairs. Grip mode belongs to attachment geometry.
use super::MeleeStage;

/// Kiva's authoritative numeric attack timing; presentation only reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeProfile {
    pub damage: f32,
    pub range: f32,
    pub hit_time: f32,
    pub duration: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponPair {
    #[default]
    HammerMusket,
    ChainAxeBow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponMode {
    #[default]
    Melee,
    Ranged,
}
impl WeaponPair {
    pub fn name(self) -> &'static str {
        match self {
            Self::HammerMusket => "锤子 / 鸟铳",
            Self::ChainAxeBow => "链斧 / 弓箭",
        }
    }
    pub fn weapon_name(self, mode: WeaponMode) -> &'static str {
        match (self, mode) {
            (Self::HammerMusket, WeaponMode::Melee) => "锤子",
            (Self::HammerMusket, WeaponMode::Ranged) => "鸟铳",
            (Self::ChainAxeBow, WeaponMode::Melee) => "链斧",
            (Self::ChainAxeBow, WeaponMode::Ranged) => "弓箭",
        }
    }
    pub fn melee_profile(self, stage: MeleeStage) -> MeleeProfile {
        // The three stages deliberately retain the existing ordinary-attack
        // numbers. Stage three has no extra damage, armor or guard break.
        let profiles = match self {
            Self::HammerMusket => {
                [MeleeProfile {
                    damage: 24.0,
                    range: 2.8,
                    hit_time: 0.12,
                    duration: 0.60,
                }; 3]
            }
            Self::ChainAxeBow => {
                [MeleeProfile {
                    damage: 18.0,
                    range: 2.6,
                    hit_time: 0.08,
                    duration: 0.40,
                }; 3]
            }
        };
        profiles[match stage {
            MeleeStage::One => 0,
            MeleeStage::Two => 1,
            MeleeStage::Three => 2,
        }]
    }
}
