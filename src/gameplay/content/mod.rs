//! Pure builtin content assembly shared by the headless mode and client.
mod abilities;
mod characters;
mod equipment;

use bevy::prelude::*;

pub(super) fn register(world: &mut World) {
    abilities::register(world);
    characters::register(world);
    equipment::register(world);
}
