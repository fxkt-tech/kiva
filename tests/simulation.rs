//! Behaviour tests cross the same command/observation interface as the client.
//! No render, input, window, asset or browser plugins are installed.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

const BOLT: AbilityId = AbilityId(1);
const NOVA: AbilityId = AbilityId(2);
const DASH: AbilityId = AbilityId(3);
const CHANNEL: AbilityId = AbilityId(4);

struct Arena {
    app: App,
    cursor: MessageCursor<SimulationEvent>,
}
impl Arena {
    fn new(obstacles: Vec<Obstacle>) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(NavigationMap::new(20.0, 1.0, obstacles));
        let mut catalog = app.world_mut().resource_mut::<AbilityCatalog>();
        for (id, name, targeting, range, effect) in [
            (
                BOLT,
                "Bolt",
                Targeting::Direction,
                12.0,
                AbilityEffect::Projectile {
                    speed: 30.0,
                    radius: 0.2,
                    damage: 80.0,
                },
            ),
            (
                NOVA,
                "Nova",
                Targeting::Point,
                10.0,
                AbilityEffect::Area {
                    radius: 2.0,
                    delay: 0.5,
                    damage: 100.0,
                },
            ),
            (
                DASH,
                "Dash",
                Targeting::Point,
                8.0,
                AbilityEffect::Dash { speed: 40.0 },
            ),
            (
                CHANNEL,
                "Channel",
                Targeting::SelfCast,
                0.0,
                AbilityEffect::Channel {
                    duration: 2.0,
                    interval: 0.5,
                    radius: 4.0,
                    damage: 20.0,
                },
            ),
        ] {
            catalog
                .register(
                    id,
                    AbilityDefinition {
                        name: name.into(),
                        targeting,
                        range,
                        windup: 0.3,
                        recovery: 0.1,
                        cooldown: 3.0,
                        mana_cost: 40.0,
                        effect,
                    },
                )
                .unwrap();
        }
        Self {
            app,
            cursor: MessageCursor::default(),
        }
    }
    fn unit(&mut self, team: Team, position: Vec2) -> Entity {
        self.spawn(team, position, UnitStats::default(), None)
    }
    fn spawn(
        &mut self,
        team: Team,
        position: Vec2,
        stats: UnitStats,
        respawn_delay: Option<f32>,
    ) -> Entity {
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Test unit".into(),
                team,
                position,
                stats,
                abilities: [Some(BOLT), Some(NOVA), Some(DASH), Some(CHANNEL)],
                respawn_delay,
            },
        )
        .unwrap()
    }
    fn command(&mut self, unit: Entity, command: UnitCommand) {
        self.app
            .world_mut()
            .resource_mut::<CommandQueue>()
            .submit(unit, command);
    }
    fn cast(&mut self, unit: Entity, slot: usize, target: AbilityTarget) {
        self.command(unit, UnitCommand::CastAbility { slot, target });
    }
    fn step(&mut self) {
        self.app.world_mut().run_schedule(FixedUpdate);
    }
    fn advance(&mut self, seconds: f32) {
        for _ in 0..(seconds / TICK_SECONDS).round() as usize {
            self.step();
        }
    }
    fn position(&self, unit: Entity) -> Vec2 {
        self.app.world().get::<UnitPose>(unit).unwrap().position()
    }
    fn health(&self, unit: Entity) -> f32 {
        self.app.world().get::<UnitState>(unit).unwrap().health()
    }
    fn mana(&self, unit: Entity) -> f32 {
        self.app.world().get::<UnitState>(unit).unwrap().mana()
    }
    fn cooldown(&self, unit: Entity, slot: usize) -> f32 {
        self.app
            .world()
            .get::<AbilitySlots>(unit)
            .unwrap()
            .cooldown(slot)
            .unwrap()
    }
    fn events(&mut self) -> Vec<SimulationEvent> {
        self.cursor
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect()
    }
}

#[test]
fn moves_and_stops_without_rendering_or_input() {
    let mut a = Arena::new(vec![]);
    let player = a.unit(Team::Ally, Vec2::ZERO);
    a.command(player, UnitCommand::MoveTo(Vec2::new(8.0, 0.0)));
    a.advance(0.5);
    assert!(a.position(player).x > 2.0);
    a.command(player, UnitCommand::Stop);
    a.step();
    let stopped = a.position(player);
    a.advance(2.0);
    assert_eq!(a.position(player), stopped);
}

#[test]
fn routes_around_obstacles_without_clipping_corners() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::ZERO,
        half_size: Vec2::new(1.0, 3.0),
    }]);
    let player = a.unit(Team::Ally, Vec2::new(-6.0, 0.0));
    a.command(player, UnitCommand::MoveTo(Vec2::new(6.0, 0.0)));
    for _ in 0..240 {
        a.step();
        assert!(
            a.app
                .world()
                .resource::<NavigationMap>()
                .walkable(a.position(player), 0.45)
        );
    }
    assert!(a.position(player).distance(Vec2::new(6.0, 0.0)) < 0.01);
}

#[test]
fn blocked_click_projects_to_a_walkable_destination() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::ZERO,
        half_size: Vec2::ONE,
    }]);
    let player = a.unit(Team::Ally, Vec2::new(-5.0, 0.0));
    a.command(player, UnitCommand::MoveTo(Vec2::ZERO));
    a.advance(2.0);
    assert!(a.position(player).length() < 3.0);
    assert!(
        a.app
            .world()
            .resource::<NavigationMap>()
            .walkable(a.position(player), 0.45)
    );
}

#[test]
fn blocked_click_chooses_a_reachable_projection_across_a_sealed_wall() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::ZERO,
        half_size: Vec2::new(1.0, 20.0),
    }]);
    let player = a.unit(Team::Ally, Vec2::new(-5.0, 0.0));
    a.command(player, UnitCommand::MoveTo(Vec2::new(0.2, 0.0)));
    a.advance(2.0);
    assert!(a.position(player).x < -1.45 && a.position(player).x > -3.0);
    assert!(
        !a.events()
            .iter()
            .any(|event| matches!(event, SimulationEvent::CommandRejected { .. }))
    );
}

#[test]
fn unreachable_or_nonfinite_command_preserves_the_previous_order() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::ZERO,
        half_size: Vec2::new(1.0, 20.0),
    }]);
    let player = a.unit(Team::Ally, Vec2::new(-5.0, 0.0));
    a.command(player, UnitCommand::MoveTo(Vec2::new(-5.0, 5.0)));
    a.step();
    a.command(player, UnitCommand::MoveTo(Vec2::new(5.0, 0.0)));
    a.command(player, UnitCommand::MoveTo(Vec2::splat(f32::NAN)));
    a.advance(2.0);
    assert!(a.position(player).distance(Vec2::new(-5.0, 5.0)) < 0.01);
    assert_eq!(
        a.events()
            .iter()
            .filter(|e| matches!(
                e,
                SimulationEvent::CommandRejected {
                    reason: Rejection::NoPath,
                    ..
                }
            ))
            .count(),
        2
    );
}

#[test]
fn latest_move_replaces_the_route() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.command(p, UnitCommand::MoveTo(Vec2::new(10.0, 0.0)));
    a.advance(0.2);
    a.command(p, UnitCommand::MoveTo(Vec2::new(0.0, 5.0)));
    a.advance(2.0);
    assert!(a.position(p).distance(Vec2::new(0.0, 5.0)) < 0.01);
}

#[test]
fn cancelled_windup_spends_no_mana_or_cooldown() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.1);
    a.command(p, UnitCommand::Stop);
    a.advance(1.0);
    assert_eq!(a.mana(p), 300.0);
    assert_eq!(a.cooldown(p, 0), 0.0);
    assert!(
        !a.events()
            .iter()
            .any(|e| matches!(e, SimulationEvent::AbilityCommitted { .. }))
    );
}

#[test]
fn duplicate_casts_commit_once_and_cooldown_survives_movement() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    for _ in 0..30 {
        a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
        a.step();
    }
    assert!(a.mana(p) >= 260.0 && a.mana(p) < 265.0);
    let before = a.cooldown(p, 0);
    a.command(p, UnitCommand::MoveTo(Vec2::new(4.0, 0.0)));
    a.advance(0.5);
    assert!((a.cooldown(p, 0) - (before - 0.5)).abs() < 0.001);
    assert_eq!(
        a.events()
            .iter()
            .filter(|e| matches!(e, SimulationEvent::AbilityCommitted { .. }))
            .count(),
        1
    );
}

#[test]
fn projectile_and_area_share_damage_rules_and_ignore_allies() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let ally = a.unit(Team::Ally, Vec2::new(2.0, 0.0));
    let enemy = a.unit(Team::Enemy, Vec2::new(4.0, 0.0));
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.8);
    assert_eq!(a.health(enemy), 520.0);
    assert_eq!(a.health(ally), 600.0);
    a.cast(p, 1, AbilityTarget::Point(Vec2::new(3.0, 0.0)));
    a.advance(1.0);
    assert_eq!(a.health(enemy), 420.0);
    assert_eq!(a.health(ally), 600.0);
}

#[test]
fn swept_projectile_cannot_tunnel_through_targets_or_walls() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::new(5.0, 0.0),
        half_size: Vec2::new(0.1, 3.0),
    }]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.unit(Team::Enemy, Vec2::new(7.0, 0.0));
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(1.0);
    assert_eq!(a.health(enemy), 600.0);
}

#[test]
fn dash_uses_the_same_swept_collision_as_walking() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::new(3.0, 0.0),
        half_size: Vec2::new(0.1, 3.0),
    }]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.cast(p, 2, AbilityTarget::Point(Vec2::new(7.0, 0.0)));
    a.advance(1.0);
    assert!(a.position(p).x > 2.0 && a.position(p).x < 2.45);
    assert!(a.cooldown(p, 2) > 0.0);
}

#[test]
fn channel_pulses_stop_on_a_new_move_without_refunding_costs() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.unit(Team::Enemy, Vec2::new(2.0, 0.0));
    a.cast(p, 3, AbilityTarget::SelfCast);
    a.advance(0.9);
    assert_eq!(a.health(enemy), 580.0);
    a.command(p, UnitCommand::MoveTo(Vec2::new(-5.0, 0.0)));
    a.advance(2.0);
    assert_eq!(a.health(enemy), 580.0);
    assert!(a.cooldown(p, 3) > 0.0);
}

#[test]
fn channel_produces_exactly_the_defined_number_of_pulses() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.unit(Team::Enemy, Vec2::new(2.0, 0.0));
    a.cast(p, 3, AbilityTarget::SelfCast);
    a.advance(3.0);
    assert_eq!(a.health(enemy), 520.0);
}

#[test]
fn attack_move_engages_then_resumes_its_destination() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.spawn(
        Team::Enemy,
        Vec2::new(7.0, 0.0),
        UnitStats {
            max_health: 40.0,
            ..default()
        },
        None,
    );
    a.command(p, UnitCommand::AttackMove(Vec2::new(12.0, 0.0)));
    a.advance(4.0);
    assert_eq!(a.health(enemy), 0.0);
    assert!(a.position(p).distance(Vec2::new(12.0, 0.0)) < 0.01);
}

#[test]
fn attack_cancel_and_kiting_cannot_reset_attack_cooldown() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    a.command(p, UnitCommand::AttackTarget(enemy));
    a.advance(0.1);
    a.command(p, UnitCommand::Stop);
    a.advance(0.5);
    assert_eq!(a.health(enemy), 600.0);
    a.command(p, UnitCommand::AttackTarget(enemy));
    a.advance(0.3);
    a.command(p, UnitCommand::MoveTo(Vec2::new(0.0, 1.0)));
    a.step();
    a.command(p, UnitCommand::AttackTarget(enemy));
    a.advance(0.3);
    assert_eq!(a.health(enemy), 560.0);
}

#[test]
fn stop_never_automatically_acquires_an_enemy() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.unit(Team::Enemy, Vec2::new(2.0, 0.0));
    a.command(p, UnitCommand::Stop);
    a.advance(3.0);
    assert_eq!(a.health(enemy), 600.0);
}

#[test]
fn death_cancels_actions_and_respawns_with_a_clean_state() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.spawn(
        Team::Enemy,
        Vec2::new(3.0, 0.0),
        UnitStats {
            max_health: 40.0,
            ..default()
        },
        Some(1.0),
    );
    a.command(enemy, UnitCommand::MoveTo(Vec2::new(3.0, 1.0)));
    a.command(p, UnitCommand::AttackTarget(enemy));
    a.advance(0.6);
    assert_eq!(a.health(enemy), 0.0);
    a.command(enemy, UnitCommand::MoveTo(Vec2::new(8.0, 0.0)));
    a.advance(1.1);
    assert_eq!(a.health(enemy), 40.0);
    assert_eq!(a.position(enemy), Vec2::new(3.0, 0.0));
    assert!(matches!(
        a.app.world().get::<UnitOrder>(enemy),
        Some(UnitOrder::Idle)
    ));
}

#[test]
fn pause_freezes_timers_positions_and_queued_commands() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.command(p, UnitCommand::MoveTo(Vec2::new(5.0, 0.0)));
    a.app.world_mut().resource_mut::<SimulationControl>().paused = true;
    a.advance(2.0);
    assert_eq!(a.position(p), Vec2::ZERO);
    assert_eq!(a.app.world().resource::<SimulationTick>().0, 0);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.advance(1.0);
    assert!(a.position(p).x > 4.0);
}

#[test]
fn invalid_targets_and_ability_slots_do_not_spend_resources() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.cast(p, 0, AbilityTarget::Direction(Vec2::ZERO));
    a.cast(p, 1, AbilityTarget::Point(Vec2::new(18.0, 0.0)));
    a.cast(p, 99, AbilityTarget::SelfCast);
    a.advance(1.0);
    assert_eq!(a.mana(p), 300.0);
    assert_eq!(a.cooldown(p, 0), 0.0);
    assert_eq!(a.events().len(), 3);
}

#[test]
fn attack_tracks_a_moving_target_and_replans_after_small_movements() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.spawn(
        Team::Enemy,
        Vec2::new(8.0, 0.0),
        UnitStats {
            move_speed: 2.0,
            ..default()
        },
        None,
    );
    a.command(enemy, UnitCommand::MoveTo(Vec2::new(16.0, 0.0)));
    a.command(p, UnitCommand::AttackTarget(enemy));
    a.advance(5.0);
    assert!(a.health(enemy) < 600.0);
    assert!(a.position(p).x > 9.0);
}

#[test]
fn repeated_attack_commands_do_not_restart_the_attack_windup() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let enemy = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    for _ in 0..60 {
        a.command(p, UnitCommand::AttackTarget(enemy));
        a.step();
    }
    assert_eq!(a.health(enemy), 560.0);
}

#[test]
fn invalid_move_does_not_cancel_a_valid_cast() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.1);
    a.command(p, UnitCommand::MoveTo(Vec2::splat(f32::NAN)));
    a.advance(0.4);
    assert!(a.cooldown(p, 0) > 0.0);
    assert_eq!(
        a.events()
            .iter()
            .filter(|event| matches!(event, SimulationEvent::AbilityCommitted { .. }))
            .count(),
        1
    );
}

#[test]
fn insufficient_mana_rejects_cast_without_changing_the_current_order() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            max_mana: 10.0,
            mana_regen: 0.0,
            ..default()
        },
        None,
    );
    a.command(p, UnitCommand::MoveTo(Vec2::new(5.0, 0.0)));
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(1.0);
    assert!(a.position(p).distance(Vec2::new(5.0, 0.0)) < 0.01);
    assert_eq!(a.mana(p), 10.0);
    assert_eq!(a.cooldown(p, 0), 0.0);
}

#[test]
fn target_dying_during_a_targeted_cast_prevents_commit() {
    let mut a = Arena::new(vec![]);
    let mut definition = a
        .app
        .world()
        .resource::<AbilityCatalog>()
        .get(NOVA)
        .unwrap()
        .clone();
    definition.targeting = Targeting::Enemy;
    definition.windup = 0.7;
    let targeted = AbilityId(5);
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(targeted, definition)
        .unwrap();
    let p = spawn_unit(
        a.app.world_mut(),
        UnitSpec {
            name: "Targeted caster".into(),
            team: Team::Ally,
            position: Vec2::ZERO,
            stats: UnitStats::default(),
            abilities: [Some(targeted), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    let other = a.unit(Team::Ally, Vec2::new(1.0, 0.0));
    let enemy = a.spawn(
        Team::Enemy,
        Vec2::new(4.0, 0.0),
        UnitStats {
            max_health: 40.0,
            ..default()
        },
        None,
    );
    a.cast(p, 0, AbilityTarget::Unit(enemy));
    a.cast(other, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(1.0);
    assert_eq!(a.health(enemy), 0.0);
    assert_eq!(a.cooldown(p, 0), 0.0);
    assert_eq!(a.mana(p), 300.0);
}

#[test]
fn committed_area_effect_survives_the_casters_death() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            max_health: 40.0,
            ..default()
        },
        None,
    );
    let killer = a.unit(Team::Enemy, Vec2::new(-2.0, 0.0));
    let target = a.unit(Team::Enemy, Vec2::new(4.0, 0.0));
    a.cast(p, 1, AbilityTarget::Point(Vec2::new(4.0, 0.0)));
    a.cast(killer, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(1.0);
    assert_eq!(a.health(p), 0.0);
    assert_eq!(a.health(target), 500.0);
}

#[test]
fn high_speed_projectile_sweeps_the_entire_tick() {
    let mut a = Arena::new(vec![]);
    let mut definition = a
        .app
        .world()
        .resource::<AbilityCatalog>()
        .get(BOLT)
        .unwrap()
        .clone();
    definition.effect = AbilityEffect::Projectile {
        speed: 2000.0,
        radius: 0.1,
        damage: 90.0,
    };
    let fast = AbilityId(6);
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(fast, definition)
        .unwrap();
    let p = spawn_unit(
        a.app.world_mut(),
        UnitSpec {
            name: "Fast caster".into(),
            team: Team::Ally,
            position: Vec2::ZERO,
            stats: UnitStats::default(),
            abilities: [Some(fast), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    let enemy = a.unit(Team::Enemy, Vec2::new(10.0, 0.0));
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.5);
    assert_eq!(a.health(enemy), 510.0);
}

#[test]
fn fractional_channel_duration_does_not_add_a_final_pulse() {
    let mut a = Arena::new(vec![]);
    let mut definition = a
        .app
        .world()
        .resource::<AbilityCatalog>()
        .get(CHANNEL)
        .unwrap()
        .clone();
    definition.effect = AbilityEffect::Channel {
        duration: 0.55,
        interval: 0.17,
        radius: 4.0,
        damage: 20.0,
    };
    let fractional = AbilityId(7);
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(fractional, definition)
        .unwrap();
    let p = spawn_unit(
        a.app.world_mut(),
        UnitSpec {
            name: "Channel caster".into(),
            team: Team::Ally,
            position: Vec2::ZERO,
            stats: UnitStats::default(),
            abilities: [Some(fractional), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    let enemy = a.unit(Team::Enemy, Vec2::new(2.0, 0.0));
    a.cast(p, 0, AbilityTarget::SelfCast);
    a.advance(1.5);
    assert_eq!(a.health(enemy), 540.0);
}

#[test]
fn invalid_definitions_and_spawns_are_rejected_at_construction() {
    let mut a = Arena::new(vec![]);
    let mut definition = a
        .app
        .world()
        .resource::<AbilityCatalog>()
        .get(BOLT)
        .unwrap()
        .clone();
    definition.cooldown = f32::NAN;
    assert!(
        a.app
            .world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(AbilityId(8), definition)
            .is_err()
    );
    let bad_spawn = UnitSpec {
        name: "Invalid unit".into(),
        team: Team::Ally,
        position: Vec2::new(100.0, 0.0),
        stats: UnitStats::default(),
        abilities: [None; 4],
        respawn_delay: None,
    };
    assert!(spawn_unit(a.app.world_mut(), bad_spawn).is_err());
}

#[test]
fn queued_input_survives_frames_without_fixed_steps_and_commits_once() {
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    a.app
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    a.app.update();
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.app.update();
    assert_eq!(a.app.world().resource::<SimulationTick>().0, 0);
    a.app
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 144.0,
        )));
    let mut commits = 0;
    for _ in 0..144 {
        a.app.update();
        commits += a
            .events()
            .iter()
            .filter(|event| matches!(event, SimulationEvent::AbilityCommitted { .. }))
            .count();
    }
    assert_eq!(commits, 1);
    assert!(a.cooldown(p, 0) > 0.0);
}
