//! Sun, moon and rage: the three meters fighting well fills.
//!
//! Sun grows with damage dealt and empowers the melee skill; moon grows with
//! clean defense and empowers the ranged skill; empowered skills build rage,
//! and a full rage meter can be spent on the enraged state.
use super::combatant::RAGE;
use super::skills::Slot;
use super::{CombatEffect, CombatFeedback, CombatIntent, CombatStatus, Combatant, Health};
use crate::simulation::Body;
use bevy::prelude::*;

/// Presentation radius of the burst when the enraged state starts.
const ENRAGE_RADIUS: f32 = 2.0;

#[derive(Component, Debug, Default)]
pub struct Energy {
    sun: u8,
    moon: u8,
    rage: f32,
    enraged: bool,
}

impl Energy {
    pub const SUN_MAX: u8 = 10;
    pub const MOON_MAX: u8 = 10;
    pub const RAGE_MAX: f32 = 20.0;
    /// A full meter lasts this long once the enraged state starts.
    pub const RAGE_DURATION: f32 = 20.0;
    /// Ordinary attacks deal this much more while enraged.
    const ENRAGED_ATTACK_SCALE: f32 = 2.0;

    pub fn sun(&self) -> u8 {
        self.sun
    }
    pub fn moon(&self) -> u8 {
        self.moon
    }
    /// Stored rage, or what is left of the enraged state.
    pub fn rage(&self) -> f32 {
        self.rage
    }
    pub fn is_enraged(&self) -> bool {
        self.enraged
    }
    /// The rage skill can be released.
    pub fn rage_ready(&self) -> bool {
        !self.enraged && self.rage >= Self::RAGE_MAX
    }

    /// One damaging hit, or one parry.
    pub(in crate::simulation) fn gain_sun(&mut self, hits: u8) {
        self.sun = self.sun.saturating_add(hits).min(Self::SUN_MAX);
    }
    /// One dodged or deflected attack.
    pub(in crate::simulation) fn gain_moon(&mut self) {
        self.moon = (self.moon + 1).min(Self::MOON_MAX);
    }

    /// Whether a skill started now is empowered. Outside the enraged state
    /// that spends the slot's full meter and builds `rage`.
    pub(super) fn empower(&mut self, slot: Slot, rage: u8) -> bool {
        let (meter, maximum) = match slot {
            Slot::MeleeSkill => (&mut self.sun, Self::SUN_MAX),
            Slot::RangedSkill => (&mut self.moon, Self::MOON_MAX),
            Slot::Attack | Slot::Combo => return false,
        };
        if self.enraged {
            return true;
        }
        if *meter < maximum {
            return false;
        }
        *meter = 0;
        self.rage = (self.rage + f32::from(rage)).min(Self::RAGE_MAX);
        true
    }

    pub(super) fn attack_scale(&self) -> f32 {
        if self.enraged {
            Self::ENRAGED_ATTACK_SCALE
        } else {
            1.0
        }
    }

    /// Spend a full meter on the enraged state.
    pub(super) fn enrage(&mut self) -> bool {
        let ready = self.rage_ready();
        self.enraged |= ready;
        ready
    }

    fn drain(&mut self, dt: f32) {
        if self.enraged {
            self.rage = (self.rage - Self::RAGE_MAX / Self::RAGE_DURATION * dt).max(0.0);
            self.enraged = self.rage > 0.0;
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    #[cfg(test)]
    pub(in crate::simulation) fn with(sun: u8, moon: u8, rage: f32) -> Self {
        Self {
            sun,
            moon,
            rage,
            enraged: false,
        }
    }
}

type RagePlayer = (
    Entity,
    &'static Body,
    &'static Health,
    &'static mut CombatIntent,
    &'static mut Combatant,
    &'static mut Energy,
);

/// The rage request is consumed here whether or not it can be honoured: a
/// living player with a full meter enters the enraged state.
pub(in crate::simulation) fn release_rage(
    mut feedback: ResMut<CombatFeedback>,
    mut players: Query<RagePlayer>,
) {
    for (entity, body, health, mut intent, mut combat, mut energy) in &mut players {
        let requested = intent.commands & RAGE != 0;
        intent.commands &= !RAGE;
        if requested && health.is_alive() && energy.enrage() {
            combat.status = CombatStatus::Enraged;
            let position = body.aim_position();
            feedback.push(
                entity,
                CombatEffect::Enrage,
                position,
                position,
                ENRAGE_RADIUS,
            );
        }
    }
}

/// The enraged state burns its meter down in rule time, so pauses and hit
/// stops do not shorten it.
pub(in crate::simulation) fn drain_rage(time: Res<Time<Fixed>>, mut energies: Query<&mut Energy>) {
    for mut energy in &mut energies {
        energy.drain(time.delta_secs());
    }
}
