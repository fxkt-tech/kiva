//! Autonomous controllers submit exactly the same public commands as a human.
mod heroes;
mod minions;
mod observations;
mod pursuit;
mod spells;

use super::GameplaySet;
use bevy::prelude::*;

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (minions::decide, heroes::decide)
                .chain()
                .in_set(GameplaySet::Decisions),
        );
    }
}

const DECISION_TICKS: u64 = 12;
