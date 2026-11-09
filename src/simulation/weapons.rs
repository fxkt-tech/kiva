//! Concrete weapon content; combat owns execution and cooldown state.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponSet {
    #[default]
    HammerSniper,
    AxeCannon,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponMode {
    #[default]
    Melee,
    Ranged,
}

impl WeaponMode {
    pub(super) fn other(self) -> Self {
        match self {
            Self::Melee => Self::Ranged,
            Self::Ranged => Self::Melee,
        }
    }

    pub fn persona(self) -> &'static str {
        match self {
            Self::Melee => "哥哥",
            Self::Ranged => "妹妹",
        }
    }
}

impl WeaponSet {
    pub fn name(self) -> &'static str {
        match self {
            Self::HammerSniper => "锤子 / 狙击枪",
            Self::AxeCannon => "斧子 / 火炮",
        }
    }

    pub fn weapon_name(self, mode: WeaponMode) -> &'static str {
        match (self, mode) {
            (Self::HammerSniper, WeaponMode::Melee) => "锤子",
            (Self::HammerSniper, WeaponMode::Ranged) => "狙击枪",
            (Self::AxeCannon, WeaponMode::Melee) => "斧子",
            (Self::AxeCannon, WeaponMode::Ranged) => "火炮",
        }
    }

    pub(super) fn attack(self, mode: WeaponMode) -> Attack {
        match (self, mode) {
            (Self::HammerSniper, WeaponMode::Melee) => Attack {
                damage: 24.0,
                range: 2.8,
                cooldown: 0.6,
                splash: 0.0,
            },
            (Self::HammerSniper, WeaponMode::Ranged) => Attack {
                damage: 40.0,
                range: 22.0,
                cooldown: 0.9,
                splash: 0.0,
            },
            (Self::AxeCannon, WeaponMode::Melee) => Attack {
                damage: 18.0,
                range: 2.6,
                cooldown: 0.4,
                splash: 0.0,
            },
            (Self::AxeCannon, WeaponMode::Ranged) => Attack {
                damage: 26.0,
                range: 18.0,
                cooldown: 0.8,
                splash: 1.6,
            },
        }
    }
}

pub(super) struct Attack {
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
    pub splash: f32,
}
