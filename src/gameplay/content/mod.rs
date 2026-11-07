//! Pure builtin content assembly shared by the headless mode and client.
mod abilities;
mod champion_abilities;
mod characters;
mod equipment;

use crate::simulation::CharacterId;
use bevy::prelude::*;

/// Pair the builtin roster at mode assembly time. Autonomous decisions only
/// inspect generic executions and never branch on these content identities.
pub(super) fn default_opponent(player: CharacterId) -> CharacterId {
    if player == CharacterId(2) {
        CharacterId(1)
    } else {
        CharacterId(2)
    }
}

pub(super) fn register(world: &mut World) {
    abilities::register(world);
    champion_abilities::register(world);
    characters::register(world);
    equipment::register(world);
}
