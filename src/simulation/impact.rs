//! A short world-wide impact pause, measured by fixed ticks rather than rendering.
use bevy::prelude::*;

use super::SimulationActive;

#[derive(Resource, Default)]
pub struct HitStop {
    remaining: f32,
    blocked_tick: bool,
}

impl HitStop {
    pub const SNIPER_DURATION: f32 = 0.04;

    /// Rendering uses the current physical snapshots while the world is held.
    pub fn is_active(&self) -> bool {
        self.remaining > 0.0 || self.blocked_tick
    }

    pub(super) fn request_sniper(&mut self) {
        self.remaining = self.remaining.max(Self::SNIPER_DURATION);
        // Keep the current tick's decision: combat requests the pause after
        // physics, and same-tick teleport cleanup must still complete.
    }
}

pub(super) fn begin_tick(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    mut hit_stop: ResMut<HitStop>,
) {
    if !active.0 {
        return;
    }
    hit_stop.blocked_tick = hit_stop.remaining > 0.0;
    if hit_stop.blocked_tick {
        hit_stop.remaining = (hit_stop.remaining - time.delta_secs()).max(0.0);
    }
}

pub(super) fn simulation_running(active: Res<SimulationActive>, hit_stop: Res<HitStop>) -> bool {
    active.0 && !hit_stop.blocked_tick
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gameplay::{self, Player},
        simulation::{
            Body, CombatEffect, CombatFeedback, CombatIntent, Combatant, Health, PhysicsPose,
            SimulationPlugin, spawn_box,
        },
    };
    use avian3d::prelude::{LinearVelocity, Position};
    use std::time::Duration;

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn arena() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        gameplay::setup(app.world_mut());
        tick(&mut app, 12);
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        (app, player)
    }

    fn fire_sniper(app: &mut App, player: Entity) {
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_skill();
        tick(app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        for _ in 0..25 {
            tick(app, 1);
            if app.world().resource::<HitStop>().is_active() {
                return;
            }
        }
        panic!("committed sniper should produce an impact pause");
    }

    #[test]
    fn sniper_hit_stop_holds_actors_boxes_velocities_and_combat_then_resumes() {
        let (mut app, player) = arena();
        let crate_entity = spawn_box(app.world_mut(), Vec3::new(10.0, 8.0, 10.0), Vec3::ONE, true);
        fire_sniper(&mut app, player);
        let target = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        assert_eq!(
            app.world().get::<Health>(target).unwrap().current(),
            999_895.0
        );
        let actor_position = app.world().get::<Body>(target).unwrap().world_position();
        let box_position = app.world().get::<Position>(crate_entity).unwrap().0;
        let box_velocity = app.world().get::<LinearVelocity>(crate_entity).unwrap().0;
        let elapsed = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .sniper_sequence()
            .unwrap()
            .elapsed();
        for _ in 0..3 {
            tick(&mut app, 1);
            assert_eq!(
                app.world().get::<Body>(target).unwrap().world_position(),
                actor_position
            );
            assert_eq!(
                app.world().get::<Position>(crate_entity).unwrap().0,
                box_position
            );
            assert_eq!(
                app.world()
                    .get::<PhysicsPose>(crate_entity)
                    .unwrap()
                    .position(),
                box_position
            );
            assert_eq!(
                app.world().get::<LinearVelocity>(crate_entity).unwrap().0,
                box_velocity
            );
            assert_eq!(
                app.world()
                    .get::<Combatant>(player)
                    .unwrap()
                    .sniper_sequence()
                    .unwrap()
                    .elapsed(),
                elapsed
            );
        }
        tick(&mut app, 1);
        assert!(!app.world().resource::<HitStop>().is_active());
        assert_ne!(
            app.world().get::<Position>(crate_entity).unwrap().0,
            box_position
        );
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .sniper_sequence()
                .unwrap()
                .elapsed()
                > elapsed
        );
    }

    #[test]
    fn user_pause_preserves_the_hit_stop_budget_and_ordinary_shots_do_not_trigger_it() {
        let (mut app, player) = arena();
        {
            let mut intent = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
            intent.request_switch_weapon();
            intent.request_attack();
        }
        tick(&mut app, 1);
        assert!(!app.world().resource::<HitStop>().is_active());
        assert!(
            !app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| matches!(
                    event.kind,
                    CombatEffect::SniperShot | CombatEffect::SniperImpact
                ))
        );
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_switch_weapon();
        tick(&mut app, 30);
        fire_sniper(&mut app, player);
        let before = app.world().resource::<HitStop>().remaining;
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        tick(&mut app, 120);
        assert_eq!(app.world().resource::<HitStop>().remaining, before);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 3);
        assert!(app.world().resource::<HitStop>().is_active());
        tick(&mut app, 1);
        assert!(!app.world().resource::<HitStop>().is_active());
    }
}
