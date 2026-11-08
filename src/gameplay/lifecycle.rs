//! Fixed-step post-processing: settle the outcome before spawning any new wave.
use super::{
    MatchError, MatchFinished, MatchOutcome, MatchState, UnitRole,
    definition::MINIONS_PER_TEAM,
    match_state::PendingWave,
    spawn::{spawn_wave_pair, start_wave},
};
use crate::simulation::*;
use bevy::prelude::*;

pub(super) fn running(state: Res<MatchState>, control: Res<SimulationControl>) -> bool {
    state.can_advance() && !control.paused
}

pub(super) fn outcome(world: &mut World) {
    if !world.resource::<MatchState>().can_advance() || world.resource::<SimulationControl>().paused
    {
        return;
    }
    let [ally, enemy] = world
        .resource::<MatchState>()
        .current
        .as_ref()
        .expect("running match")
        .bases;
    let alive = |entity| world.get::<UnitState>(entity).is_some_and(UnitState::alive);
    let result = match (alive(ally), alive(enemy)) {
        (true, true) => return,
        (true, false) => MatchOutcome::Winner(Team::Ally),
        (false, true) => MatchOutcome::Winner(Team::Enemy),
        (false, false) => MatchOutcome::Draw,
    };
    let tick = world.resource::<SimulationTick>().0;
    let mut state = world.resource_mut::<MatchState>();
    let current = state.current.as_mut().expect("running match");
    current.finished = Some((result, tick));
    current.pending_wave = None;
    let id = current.id;
    world.resource_mut::<SimulationControl>().paused = true;
    world.resource_mut::<CommandQueue>().clear();
    world.write_message(MatchFinished {
        id,
        outcome: result,
        tick,
    });
}

pub(super) fn waves(world: &mut World) {
    // This check is deliberately inside the function. An earlier run_if may
    // already have evaluated true before outcome ended this same fixed tick.
    if !world.resource::<MatchState>().can_advance() || world.resource::<SimulationControl>().paused
    {
        return;
    }
    let tick = world.resource::<SimulationTick>().0;
    let current = world
        .resource::<MatchState>()
        .current
        .as_ref()
        .expect("running match");
    if let Some(pending) = current.pending_wave {
        if tick < pending.next_tick {
            return;
        }
        let next = if pending.next_slot == MINIONS_PER_TEAM - 1 {
            Ok(None)
        } else {
            pending
                .next_tick
                .checked_add(current.definition.minion_spawn_interval_ticks())
                .map(|next_tick| {
                    Some(PendingWave {
                        wave: pending.wave,
                        next_slot: pending.next_slot + 1,
                        next_tick,
                    })
                })
                .ok_or(MatchError::IdentityExhausted)
        };
        match next
            .and_then(|next| spawn_wave_pair(world, pending.wave, pending.next_slot).map(|()| next))
        {
            Ok(next) => {
                world
                    .resource_mut::<MatchState>()
                    .current
                    .as_mut()
                    .expect("running match")
                    .pending_wave = next;
            }
            Err(error) => fail_wave(world, error),
        }
        return;
    }
    if tick < current.next_wave_tick {
        return;
    }
    let next_wave = current.wave_number.checked_add(1);
    let next_tick = current
        .next_wave_tick
        .checked_add(current.definition.wave_interval_ticks());
    let start_tick = current.next_wave_tick;
    let result = match (next_wave, next_tick) {
        (Some(wave), Some(next_tick)) => {
            start_wave(world, wave, start_tick).map(|pending| (wave, next_tick, pending))
        }
        _ => Err(super::MatchError::IdentityExhausted),
    };
    match result {
        Ok((wave, next_tick, pending)) => {
            let mut state = world.resource_mut::<MatchState>();
            let current = state.current.as_mut().expect("running match");
            current.wave_number = wave;
            current.next_wave_tick = next_tick;
            current.pending_wave = Some(pending);
        }
        Err(error) => fail_wave(world, error),
    }
}

fn fail_wave(world: &mut World, error: MatchError) {
    let mut state = world.resource_mut::<MatchState>();
    let current = state.current.as_mut().expect("running match");
    current.wave_error = Some(error.to_string());
    current.pending_wave = None;
    world.resource_mut::<SimulationControl>().paused = true;
}

pub(super) fn cleanup(world: &mut World) {
    let state = world.resource::<MatchState>();
    let finished = matches!(state.phase(), super::MatchPhase::Finished(_));
    if state.id().is_none() || (world.resource::<SimulationControl>().paused && !finished) {
        return;
    }
    let dead: Vec<_> = world
        .query::<(Entity, &UnitRole, &UnitState)>()
        .iter(world)
        .filter(|(_, role, state)| {
            matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion) && !state.alive()
        })
        .map(|(entity, _, _)| entity)
        .collect();
    for entity in dead {
        world.despawn(entity);
    }
}

pub(super) fn recover_heroes(world: &mut World) {
    if !world.resource::<MatchState>().can_advance() || world.resource::<SimulationControl>().paused
    {
        return;
    }
    let current = world
        .resource::<MatchState>()
        .current
        .as_ref()
        .expect("running match");
    let pairs = [
        (current.bases[0], current.heroes[0]),
        (current.bases[1], current.heroes[1]),
    ];
    let spec = current.definition.recovery();
    for (source, target) in pairs {
        let Some(source_life) = world
            .get::<UnitState>(source)
            .map(|state| state.life_ref(source))
        else {
            continue;
        };
        let Some(target_life) = world
            .get::<UnitState>(target)
            .map(|state| state.life_ref(target))
        else {
            continue;
        };
        // Out of range and dead are normal eligibility outcomes;
        // the authoritative boundary alone decides and applies any recovery.
        let _ = restore_in_zone(world, source_life, target_life, spec);
    }
}
