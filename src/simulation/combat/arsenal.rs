//! Durability of each weapon pair. A worn-out pair is rotated out for the
//! next one and restored, so the pairs keep cycling.
use crate::simulation::WeaponPair;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Arsenal {
    durability: [u8; WeaponPair::ALL.len()],
}

impl Default for Arsenal {
    fn default() -> Self {
        Self {
            durability: [Self::DURABILITY; WeaponPair::ALL.len()],
        }
    }
}

impl Arsenal {
    pub const DURABILITY: u8 = 50;

    pub fn durability(&self, pair: WeaponPair) -> u8 {
        self.durability[pair.index()]
    }

    /// Starting an action wears the equipped pair; it never refuses the action.
    pub(super) fn wear(&mut self, pair: WeaponPair, amount: u8) {
        let durability = &mut self.durability[pair.index()];
        *durability = durability.saturating_sub(amount);
    }
    pub(super) fn exhausted(&self, pair: WeaponPair) -> bool {
        self.durability(pair) == 0
    }
    pub(super) fn restore(&mut self, pair: WeaponPair) {
        self.durability[pair.index()] = Self::DURABILITY;
    }
    pub(super) fn refill(&mut self) {
        *self = Self::default();
    }

    #[cfg(test)]
    pub(in crate::simulation) fn set(&mut self, pair: WeaponPair, durability: u8) {
        self.durability[pair.index()] = durability;
    }
}
