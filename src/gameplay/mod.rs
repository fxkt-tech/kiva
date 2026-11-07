//! Headless single-lane mode. This layer composes public simulation commands;
//! it does not own a second copy of combat health, clocks or unit positions.
mod ai;
mod content;
mod definition;
mod lifecycle;
mod map;
mod match_state;
mod spawn;

pub use ai::AiPlugin;
pub use definition::MatchDefinition;
pub use map::{MapDefinition, opponent};
pub use match_state::{
    Controller, MatchFinished, MatchHandle, MatchId, MatchOutcome, MatchPhase, MatchState,
    UnitRole, WaveMember,
};
pub use spawn::{MatchError, reset_match, start_match};

use crate::simulation::*;
use bevy::prelude::*;

#[derive(Resource)]
struct Prepared;

/// Explicit ordering seams for mode AI. Decisions submit commands before the
/// complete simulation set; lifecycle always settles outcome before waves.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Decisions,
    Outcome,
    Waves,
    Cleanup,
    Recovery,
}

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MatchState>()
            .init_resource::<match_state::MatchSequence>()
            .init_resource::<MapDefinition>()
            .init_resource::<MatchDefinition>()
            .add_message::<MatchFinished>()
            .configure_sets(
                FixedUpdate,
                (
                    GameplaySet::Decisions,
                    SimulationSet::Step,
                    GameplaySet::Outcome,
                    GameplaySet::Waves,
                    GameplaySet::Cleanup,
                    GameplaySet::Recovery,
                )
                    .chain(),
            )
            .configure_sets(
                FixedUpdate,
                (
                    GameplaySet::Decisions.run_if(lifecycle::running),
                    SimulationSet::Step.run_if(lifecycle::running),
                ),
            )
            .add_systems(FixedUpdate, lifecycle::outcome.in_set(GameplaySet::Outcome))
            .add_systems(FixedUpdate, lifecycle::waves.in_set(GameplaySet::Waves))
            .add_systems(FixedUpdate, lifecycle::cleanup.in_set(GameplaySet::Cleanup))
            .add_systems(
                FixedUpdate,
                lifecycle::recover_heroes.in_set(GameplaySet::Recovery),
            );
    }
}

/// Initializes trusted builtin definitions once. Repeated calls neither reset
/// an active match nor replace its catalogs/navigation/identity allocators.
pub fn prepare_world(world: &mut World) {
    if world.contains_resource::<Prepared>() {
        return;
    }
    world.init_resource::<MatchState>();
    world.init_resource::<match_state::MatchSequence>();
    world.init_resource::<MapDefinition>();
    world.init_resource::<MatchDefinition>();
    world.init_resource::<Messages<MatchFinished>>();
    world.init_resource::<AbilityCatalog>();
    world.init_resource::<CharacterCatalog>();
    world.init_resource::<EquipmentCatalog>();
    content::register(world);
    let navigation = world.resource::<MapDefinition>().navigation();
    world.insert_resource(navigation);
    world.insert_resource(Prepared);
}
