//! One interpolation sample drives every visible part of a simulation frame.
use crate::{
    gameplay::{MatchId, MatchPhase, MatchState},
    simulation::SimulationControl,
};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct PresentationFrame {
    fraction: f32,
    should_sync: bool,
    finished: Option<MatchId>,
}

impl Default for PresentationFrame {
    fn default() -> Self {
        Self {
            fraction: 0.0,
            should_sync: true,
            finished: None,
        }
    }
}

impl PresentationFrame {
    pub fn fraction(&self) -> f32 {
        self.fraction
    }

    pub fn should_sync(&self) -> bool {
        self.should_sync
    }
}

pub(super) fn prepare(
    fixed: Res<Time<Fixed>>,
    control: Res<SimulationControl>,
    game: Option<Res<MatchState>>,
    mut frame: ResMut<PresentationFrame>,
) {
    let finished = game.as_ref().and_then(|game| {
        matches!(game.phase(), MatchPhase::Finished(_))
            .then(|| game.id())
            .flatten()
    });
    if let Some(id) = finished {
        // Complete the last authoritative step once. Re-reading changing
        // overstep fractions after the step clock stops would rewind the scene.
        frame.should_sync = frame.finished != Some(id);
        frame.fraction = 1.0;
        frame.finished = Some(id);
    } else if control.paused {
        frame.should_sync = false;
    } else {
        frame.fraction = fixed.overstep_fraction();
        frame.should_sync = true;
        frame.finished = None;
    }
}
