//! Client selection defaults; all combat content and map data belong to gameplay.
use crate::{gameplay, simulation::CharacterId};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct SelectionSettings {
    pub default_character: CharacterId,
}

pub(super) fn setup(world: &mut World) {
    gameplay::prepare_world(world);
    let default_character = world
        .resource::<gameplay::MatchDefinition>()
        .default_character();
    world.insert_resource(SelectionSettings { default_character });
}
