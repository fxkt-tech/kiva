//! Read-only match and map projections shared by HUD and diagnostics.
use crate::{gameplay::*, simulation::*};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct BaseView {
    id: String,
    health: f32,
    max_health: f32,
}

#[derive(Serialize)]
struct BasesView {
    ally: Option<BaseView>,
    enemy: Option<BaseView>,
}

#[derive(Serialize)]
pub(super) struct MatchView {
    id: Option<String>,
    phase: &'static str,
    outcome: Option<&'static str>,
    error: Option<String>,
    elapsed_seconds: f64,
    wave_number: u64,
    next_wave_seconds: Option<f64>,
    bases: BasesView,
}

#[derive(Serialize)]
pub(super) struct MapView {
    half_size: f32,
    lane_width: f32,
    ally_base: [f32; 2],
    enemy_base: [f32; 2],
}

#[derive(SystemParam)]
pub(super) struct MatchProjection<'w, 's> {
    state: Res<'w, MatchState>,
    map: Res<'w, MapDefinition>,
    units: Query<'w, 's, (&'static Unit, &'static UnitState)>,
}

impl MatchProjection<'_, '_> {
    pub fn changed(&self) -> bool {
        self.state.is_changed()
    }

    pub fn can_manage(&self) -> bool {
        self.state.can_advance()
    }

    pub fn view(&self, tick: u64) -> MatchView {
        let (phase, outcome) = match self.state.phase() {
            MatchPhase::Waiting => ("Waiting", None),
            MatchPhase::Running => ("Running", None),
            MatchPhase::Finished(result) => (
                "Finished",
                Some(match result {
                    MatchOutcome::Winner(Team::Ally) => "Victory",
                    MatchOutcome::Winner(Team::Enemy) => "Defeat",
                    MatchOutcome::Draw => "Draw",
                }),
            ),
        };
        let base = |team| {
            let id = self.state.base(team)?;
            let (unit, state) = self.units.get(id).ok()?;
            Some(BaseView {
                id: id.to_bits().to_string(),
                health: state.health(),
                max_health: unit.definition().max_health,
            })
        };
        MatchView {
            id: self.state.id().map(|id| id.to_string()),
            phase,
            outcome,
            error: self.state.wave_error().map(str::to_owned),
            elapsed_seconds: self.state.finished_tick().unwrap_or(tick) as f64 / TICK_RATE,
            wave_number: self.state.wave_number(),
            next_wave_seconds: self
                .state
                .next_wave_tick()
                .map(|due| due.saturating_sub(tick) as f64 / TICK_RATE),
            bases: BasesView {
                ally: base(Team::Ally),
                enemy: base(Team::Enemy),
            },
        }
    }

    pub fn map_view(&self) -> MapView {
        MapView {
            half_size: self.map.half_size(),
            lane_width: self.map.lane_width(),
            ally_base: self.map.base_position(Team::Ally).to_array(),
            enemy_base: self.map.base_position(Team::Enemy).to_array(),
        }
    }
}

pub(super) fn role_name(role: UnitRole) -> &'static str {
    match role {
        UnitRole::Hero => "Hero",
        UnitRole::MeleeMinion => "MeleeMinion",
        UnitRole::RangedMinion => "RangedMinion",
        UnitRole::Base => "Base",
    }
}
