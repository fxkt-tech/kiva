//! Every skill number, once. Admission and resolution read this table; a new
//! skill of an existing [`Shape`] is a new row, not new rule code.
use super::CombatEffect;
use crate::simulation::{ActionKind, SequenceSnapshot, WeaponPair};

/// Selecting a target considers enemies this close and roughly ahead.
pub(super) const LOCK_RANGE: f32 = 22.0;
pub(super) const LOCK_ALIGNMENT: f32 = 0.35;
/// A cancelled volley keeps its camera this long while it returns.
pub(super) const RETURN_DURATION: f32 = SequenceSnapshot::RETURN_DURATION;
/// A parry strikes back at the attacker for this much.
pub(in crate::simulation) const PARRY_DAMAGE: f32 = 10.0;
/// After an ordinary attack retires, its follow-up stage stays available.
pub(super) const CHAIN_GRACE: f32 = 0.45;

/// The input which commits an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Slot {
    Attack,
    MeleeSkill,
    RangedSkill,
    Combo,
}

/// What a damaging hit additionally does to an enemy which survived it.
#[derive(Clone, Copy)]
pub(super) enum Control {
    None,
    /// Physically launch to about this height above the current surface.
    Launch(f32),
    Stun(f32),
    Slow {
        factor: f32,
        duration: f32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Centre {
    Actor,
    /// The locked target, which must still be within the cast range.
    Target,
}

/// Forced approach during a volley, solved by the motor and real contacts.
#[derive(Clone, Copy)]
pub(super) struct Pull {
    pub until: f32,
    pub stop_distance: f32,
    pub speed: f32,
    /// The motor's finite acceleration; braking is budgeted against it.
    pub acceleration: f32,
    pub max_height_gap: f32,
    /// Progress per step below which the pull counts as blocked…
    pub stalled_step: f32,
    /// …and how long that may last before the action is cancelled.
    pub stall_limit: f32,
}

/// The blade hits what it touches; when its first swing has landed, the
/// shockwave hits everything else around the actor or the target. Both
/// moments come from the attack's baked blade path.
#[derive(Clone, Copy)]
pub(super) struct Area {
    pub centre: Centre,
    pub radius: f32,
    pub damage: f32,
    pub control: Control,
    pub effect: CombatEffect,
    /// Opened when the original target is actually controlled.
    pub window: Option<f32>,
    /// Damage multiplier of the empowered version.
    pub empowered: f32,
}

/// The companion fires once at the target.
#[derive(Clone, Copy)]
pub(super) struct Shot {
    /// When the thrown chain axe leaves the hand, before it lands.
    pub depart: Option<f32>,
    pub impact: f32,
    pub damage: f32,
    pub stun: f32,
    /// Opened when the target is actually stunned.
    pub window: Option<f32>,
    /// Damage multiplier of the empowered version.
    pub empowered: f32,
}

/// The companion fires three times at the original target.
#[derive(Clone, Copy)]
pub(super) struct Volley {
    pub times: [f32; 3],
    pub range: f32,
    pub damage: f32,
    /// Dedicated impact presentation and a hit stop on the last shot.
    pub sniper: bool,
    pub pull: Option<Pull>,
    /// Where a cancelled sequence resumes, to return its camera.
    pub return_from: f32,
}

#[derive(Clone, Copy)]
pub(super) enum Shape {
    /// The ordinary attack: the swept blade alone, with the weapon's
    /// `MeleeProfile` numbers.
    Blade,
    Area(Area),
    Shot(Shot),
    Volley(Volley),
}

#[derive(Clone, Copy)]
pub(super) struct SkillSpec {
    pub duration: f32,
    /// Started on admission; ordinary attacks use their own duration.
    pub cooldown: f32,
    /// Needs a visible locked target this close; also the shot's range.
    pub cast_range: Option<f32>,
    /// Durability the equipped weapon pair loses when this starts.
    pub wear: u8,
    /// Rage built by releasing the empowered version.
    pub rage: u8,
    pub shape: Shape,
}

impl SkillSpec {
    /// Whether the action remembers a target when it starts.
    pub(super) fn targeted(&self) -> bool {
        match self.shape {
            Shape::Blade => false,
            Shape::Area(area) => area.centre == Centre::Target,
            Shape::Shot(_) | Shape::Volley(_) => true,
        }
    }

    /// Whether the logical companion performs it.
    pub(super) fn by_companion(&self) -> bool {
        matches!(self.shape, Shape::Shot(_) | Shape::Volley(_))
    }

    /// Whether landing it can open a combo window.
    pub(super) fn opens_window(&self) -> bool {
        match self.shape {
            Shape::Area(area) => area.window.is_some(),
            Shape::Shot(shot) => shot.window.is_some(),
            Shape::Blade | Shape::Volley(_) => false,
        }
    }

    /// Damage multiplier of the empowered version; 1 where there is none.
    pub(super) fn empowered(&self) -> f32 {
        match self.shape {
            Shape::Area(area) => area.empowered,
            Shape::Shot(shot) => shot.empowered,
            Shape::Blade | Shape::Volley(_) => 1.0,
        }
    }

    pub(super) fn pull(&self) -> Option<Pull> {
        match self.shape {
            Shape::Volley(volley) => volley.pull,
            _ => None,
        }
    }
}

const MELEE: SkillSpec = SkillSpec {
    duration: 0.0, // Per stage, from the weapon's MeleeProfile.
    cooldown: 0.0,
    cast_range: None,
    wear: 1,
    rage: 0,
    shape: Shape::Blade,
};
const HAMMER_SLAM: SkillSpec = SkillSpec {
    duration: 0.50,
    cooldown: 3.0,
    cast_range: Some(4.5),
    wear: 10,
    rage: 1,
    shape: Shape::Area(Area {
        centre: Centre::Target,
        radius: 3.0,
        damage: 20.0,
        control: Control::Launch(3.0),
        effect: CombatEffect::Hammer,
        window: Some(0.85),
        empowered: 1.5,
    }),
};
const CHAIN_SWEEP: SkillSpec = SkillSpec {
    duration: 0.50,
    cooldown: 3.0,
    cast_range: None,
    wear: 10,
    rage: 1,
    shape: Shape::Area(Area {
        centre: Centre::Actor,
        radius: 3.0,
        damage: 24.0,
        control: Control::Slow {
            factor: 0.65,
            duration: 1.5,
        },
        effect: CombatEffect::ChainSweep,
        window: None,
        empowered: 1.5,
    }),
};
const MUSKET_SHOT: SkillSpec = SkillSpec {
    duration: 0.40,
    cooldown: 4.0,
    cast_range: Some(22.0),
    wear: 10,
    rage: 2,
    shape: Shape::Shot(Shot {
        depart: None,
        impact: 0.18,
        damage: 40.0,
        stun: 0.45,
        window: None,
        empowered: 1.5,
    }),
};
const AXE_SHOT: SkillSpec = SkillSpec {
    duration: 0.75,
    cooldown: 4.0,
    cast_range: Some(12.0),
    wear: 10,
    rage: 2,
    shape: Shape::Shot(Shot {
        depart: Some(0.15),
        impact: 0.30,
        damage: 35.0,
        stun: 0.90,
        window: Some(0.90),
        empowered: 1.5,
    }),
};
const HAMMER_COMBO: SkillSpec = SkillSpec {
    duration: 0.96,
    cooldown: 0.0,
    cast_range: None,
    wear: 0,
    rage: 0,
    shape: Shape::Volley(Volley {
        times: [0.30, 0.42, 0.54],
        range: 22.0,
        damage: 28.0,
        sniper: true,
        pull: None,
        return_from: 0.68,
    }),
};
const CHAIN_COMBO: SkillSpec = SkillSpec {
    duration: 0.93,
    cooldown: 0.0,
    cast_range: None,
    wear: 0,
    rage: 0,
    shape: Shape::Volley(Volley {
        times: [0.12, 0.28, 0.44],
        range: 22.0,
        damage: 18.0,
        sniper: false,
        pull: Some(Pull {
            until: 0.65,
            stop_distance: 1.4,
            speed: 18.0,
            acceleration: 120.0,
            max_height_gap: 1.5,
            stalled_step: 0.012,
            stall_limit: 0.12,
        }),
        return_from: 0.65,
    }),
};

/// Read-only timeline facts for presentation, taken from the same rows the
/// rules run on, so a display never keeps its own copy of a skill number.
impl ActionKind {
    /// When each of the companion's shots leaves, in action time.
    pub fn shot_times(self) -> &'static [f32] {
        match &self.spec().shape {
            Shape::Volley(volley) => &volley.times,
            Shape::Shot(shot) => std::slice::from_ref(shot.depart.as_ref().unwrap_or(&shot.impact)),
            Shape::Blade | Shape::Area(_) => &[],
        }
    }

    /// `(departure, impact)` of a weapon which is thrown and lands later.
    pub fn throw_times(self) -> Option<(f32, f32)> {
        match self.spec().shape {
            Shape::Shot(shot) => shot.depart.map(|depart| (depart, shot.impact)),
            _ => None,
        }
    }

    /// `(start, duration)` of a sequence's closing return to normal play.
    pub fn return_window(self) -> Option<(f32, f32)> {
        match self.spec().shape {
            Shape::Volley(volley) => Some((volley.return_from, RETURN_DURATION)),
            _ => None,
        }
    }

    /// Whether the actor swings its melee weapon, and so has a baked blade path.
    pub fn swings_blade(self) -> bool {
        matches!(self.spec().shape, Shape::Blade | Shape::Area(_))
    }

    /// The effect this action's own strike will announce at its marker.
    pub fn strike_effect(self) -> Option<CombatEffect> {
        match self.spec().shape {
            Shape::Blade => Some(CombatEffect::Melee),
            Shape::Area(area) => Some(area.effect),
            Shape::Shot(_) | Shape::Volley(_) => None,
        }
    }
}

impl ActionKind {
    pub(super) fn spec(self) -> &'static SkillSpec {
        match self {
            Self::Melee => &MELEE,
            Self::HammerSlam => &HAMMER_SLAM,
            Self::ChainSweep => &CHAIN_SWEEP,
            Self::MusketShot => &MUSKET_SHOT,
            Self::AxeShot => &AXE_SHOT,
            Self::HammerCombo => &HAMMER_COMBO,
            Self::ChainCombo => &CHAIN_COMBO,
        }
    }
}

impl WeaponPair {
    /// The skill on this pair's melee-skill input.
    pub fn melee_skill(self) -> ActionKind {
        self.action(Slot::MeleeSkill)
    }

    /// The action each input commits with this pair equipped.
    pub(super) fn action(self, slot: Slot) -> ActionKind {
        match (slot, self) {
            (Slot::Attack, _) => ActionKind::Melee,
            (Slot::MeleeSkill, Self::HammerMusket) => ActionKind::HammerSlam,
            (Slot::MeleeSkill, Self::ChainAxeBow) => ActionKind::ChainSweep,
            (Slot::RangedSkill, Self::HammerMusket) => ActionKind::MusketShot,
            (Slot::RangedSkill, Self::ChainAxeBow) => ActionKind::AxeShot,
            (Slot::Combo, Self::HammerMusket) => ActionKind::HammerCombo,
            (Slot::Combo, Self::ChainAxeBow) => ActionKind::ChainCombo,
        }
    }
}
