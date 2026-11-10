//! Concrete first-version weapon pairs. Grip mode belongs to attachment geometry.
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
    pub(super) fn melee(self) -> (f32, f32, f32, f32) {
        match self {
            Self::HammerMusket => (24.0, 2.8, 0.12, 0.60),
            Self::ChainAxeBow => (18.0, 2.6, 0.08, 0.40),
        }
    }
}
