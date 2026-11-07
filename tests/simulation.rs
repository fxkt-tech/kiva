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
                    damage: DamageSpec::magical(80.0),
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
                    damage: DamageSpec::magical(100.0),
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
                    damage: DamageSpec::magical(20.0),
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
    fn source(&mut self) -> ModifierSourceId {
        self.app
            .world_mut()
            .resource_mut::<AttributeChanges>()
            .allocate_source()
            .unwrap()
    }
    fn upsert(&mut self, unit: Entity, source: ModifierSourceId, modifier: AttributeModifier) {
        self.app
            .world_mut()
            .resource_mut::<AttributeChanges>()
            .upsert(unit, source, modifier);
    }
    fn remove(&mut self, unit: Entity, source: ModifierSourceId) {
        self.app
            .world_mut()
            .resource_mut::<AttributeChanges>()
            .remove(unit, source);
    }
    fn attributes(&self, unit: Entity) -> EffectiveAttributes {
        *self.app.world().get::<Unit>(unit).unwrap().attributes()
    }
    fn caster(
        &mut self,
        attributes: BaseAttributes,
        targeting: Targeting,
        effect: AbilityEffect,
    ) -> Entity {
        let id = AbilityId(100);
        self.app
            .world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                id,
                AbilityDefinition {
                    name: "Attribute test ability".into(),
                    targeting,
                    range: 12.0,
                    windup: 0.1,
                    recovery: 0.1,
                    cooldown: 3.0,
                    mana_cost: 40.0,
                    effect,
                },
            )
            .unwrap();
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Attribute caster".into(),
                team: Team::Ally,
                position: Vec2::ZERO,
                stats: UnitStats {
                    base_attributes: attributes,
                    ..default()
                },
                abilities: [Some(id), None, None, None],
                respawn_delay: None,
            },
        )
        .unwrap()
    }
    fn attack_ticks(&mut self, unit: Entity, ticks: usize) -> Vec<u64> {
        let mut released = Vec::new();
        for _ in 0..ticks {
            self.step();
            let tick = self.app.world().resource::<SimulationTick>().0;
            for event in self.events() {
                if matches!(event, SimulationEvent::AttackReleased { source, .. } if source == unit)
                {
                    released.push(tick);
                }
            }
        }
        released
    }
    fn enter_backswing(&mut self, unit: Entity, target: Entity) -> (u64, u32) {
        self.command(unit, UnitCommand::AttackTarget(target));
        for _ in 0..120 {
            self.step();
            if self.events().iter().any(
                |event| matches!(event, SimulationEvent::AttackReleased { source, .. } if *source == unit),
            ) {
                let Some(UnitAction::AttackBackswing {
                    remaining,
                    duration,
                }) = self.app.world().get::<UnitAction>(unit)
                else {
                    panic!("successful attack must enter its backswing");
                };
                assert_eq!(remaining, duration);
                return (self.app.world().resource::<SimulationTick>().0, *duration);
            }
        }
        panic!("test attack did not release");
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
            base_attributes: BaseAttributes {
                move_speed: 2.0,
                ..default()
            },
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
        damage: DamageSpec::magical(90.0),
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
        damage: DamageSpec::magical(20.0),
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

#[test]
fn default_attributes_are_observed_separately_from_static_definitions() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let unit = a.app.world().get::<Unit>(p).unwrap();
    let stats = unit.attributes();
    assert_eq!(stats.attack_damage(), 40.0);
    assert_eq!(stats.ability_power(), 0.0);
    assert_eq!(stats.armor(), 0.0);
    assert_eq!(stats.magic_resistance(), 0.0);
    assert_eq!(stats.move_speed(), 5.5);
    assert_eq!(stats.attack_range(), 5.5);
    assert_eq!(stats.attack_speed(), 1.25);
    assert_eq!(unit.definition().max_health, 600.0);
    assert_eq!(unit.definition().base_attributes, BaseAttributes::default());
}

#[test]
fn modifiers_replace_by_source_and_recompute_without_residuals() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let flat = a.source();
    let first = a.source();
    let second = a.source();
    let flat_modifier = AttributeModifier {
        flat: AttributeDelta {
            attack_damage: 10.0,
            armor: -50.0,
            ..default()
        },
        ..default()
    };
    a.upsert(p, flat, flat_modifier);
    a.upsert(p, flat, flat_modifier);
    a.upsert(
        p,
        first,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_damage: 0.2,
                ..default()
            },
            ..default()
        },
    );
    a.upsert(
        p,
        second,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_damage: 0.3,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), 75.0);
    assert_eq!(a.attributes(p).armor(), -50.0);
    a.remove(p, first);
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), 65.0);
    a.upsert(
        p,
        flat,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 20.0,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), 78.0);
    for _ in 0..8 {
        a.remove(p, flat);
        a.remove(p, second);
        a.step();
        assert_eq!(a.attributes(p).attack_damage(), 40.0);
        assert_eq!(a.attributes(p).armor(), 0.0);
        a.upsert(p, flat, flat_modifier);
        a.step();
        assert_eq!(a.attributes(p).attack_damage(), 50.0);
    }
}

#[test]
fn modifier_updates_are_fifo_atomic_and_wait_while_paused() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 10.0,
                ..default()
            },
            ..default()
        },
    );
    a.remove(p, source);
    a.upsert(
        p,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 20.0,
                ..default()
            },
            ..default()
        },
    );
    a.upsert(
        p,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 100.0,
                ability_power: f32::NAN,
                ..default()
            },
            ..default()
        },
    );
    a.command(p, UnitCommand::MoveTo(Vec2::new(8.0, 0.0)));
    a.app.world_mut().resource_mut::<SimulationControl>().paused = true;
    a.advance(1.0);
    assert_eq!(a.attributes(p).attack_damage(), 40.0);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), 60.0);
    assert_eq!(a.attributes(p).ability_power(), 0.0);
    assert!(a.position(p).x > 0.0);
    let rejections = a.events().into_iter().filter(|e| matches!(e, SimulationEvent::AttributeChangeRejected { unit, reason: AttributeRejection::InvalidModifier, .. } if *unit == p)).count();
    assert_eq!(rejections, 1);
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), 60.0);
    assert!(
        a.events()
            .iter()
            .all(|e| !matches!(e, SimulationEvent::AttributeChangeRejected { .. }))
    );
}

#[test]
fn invalid_modifiers_preserve_casts_and_missing_targets_are_rejected() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let source = a.source();
    a.cast(p, 0, AbilityTarget::Direction(Vec2::X));
    a.step();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                move_speed: f32::INFINITY,
                ..default()
            },
            ..default()
        },
    );
    let removed = a.unit(Team::Ally, Vec2::new(-4.0, 0.0));
    a.app.world_mut().despawn(removed);
    a.upsert(removed, source, AttributeModifier::default());
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::Casting { .. })
    ));
    a.advance(0.5);
    assert!(a.cooldown(p, 0) > 0.0);
    let events = a.events();
    assert!(events.iter().any(|e| matches!(e, SimulationEvent::AttributeChangeRejected { unit, reason: AttributeRejection::InvalidModifier, .. } if *unit == p)));
    assert!(events.iter().any(|e| matches!(e, SimulationEvent::AttributeChangeRejected { unit, reason: AttributeRejection::InvalidUnit, .. } if *unit == removed)));
}

#[test]
fn finite_extremes_saturate_and_source_removal_still_restores_values() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            base_attributes: BaseAttributes {
                attack_speed: f32::from_bits(1),
                ..default()
            },
            ..default()
        },
        None,
    );
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: f32::MAX,
                armor: -f32::MAX,
                ..default()
            },
            additive_percent: AttributeDelta {
                attack_damage: f32::MAX,
                armor: f32::MAX,
                attack_speed: -0.999_999_94,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), f32::MAX);
    assert_eq!(a.attributes(p).armor(), -f32::MAX);
    assert_eq!(a.attributes(p).attack_speed(), MIN_ATTACK_SPEED);
    a.remove(p, source);
    a.step();
    assert_eq!(a.attributes(p).attack_damage(), 40.0);
    assert_eq!(a.attributes(p).armor(), 0.0);
    assert_eq!(a.attributes(p).attack_speed(), MIN_ATTACK_SPEED);
}

#[test]
fn invalid_base_attributes_and_damage_definitions_are_rejected() {
    let mut a = Arena::new(vec![]);
    for value in [f32::NAN, f32::INFINITY, -1.0] {
        let spec = UnitSpec {
            name: "Invalid".into(),
            team: Team::Ally,
            position: Vec2::ZERO,
            stats: UnitStats {
                base_attributes: BaseAttributes {
                    ability_power: value,
                    ..default()
                },
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        };
        assert!(spawn_unit(a.app.world_mut(), spec).is_err());
        let mut definition = a
            .app
            .world()
            .resource::<AbilityCatalog>()
            .get(BOLT)
            .unwrap()
            .clone();
        definition.effect = AbilityEffect::Projectile {
            speed: 30.0,
            radius: 0.2,
            damage: DamageSpec {
                kind: DamageType::Magical,
                base: 75.0,
                ad_ratio: 0.0,
                ap_ratio: value,
            },
        };
        assert!(
            a.app
                .world_mut()
                .resource_mut::<AbilityCatalog>()
                .register(AbilityId(101), definition)
                .is_err()
        );
    }
}

#[test]
fn physical_magical_true_and_negative_resistance_use_one_damage_contract() {
    for (kind, armor, magic_resistance, expected) in [
        (DamageType::Physical, 100.0, 0.0, 50.0),
        (DamageType::Physical, 0.0, 100.0, 100.0),
        (DamageType::Magical, 0.0, 100.0, 50.0),
        (DamageType::Magical, 100.0, 0.0, 100.0),
        (DamageType::True, 100.0, 100.0, 100.0),
        (DamageType::Physical, -50.0, 0.0, 133.333_33),
        (DamageType::Magical, 0.0, -50.0, 133.333_33),
    ] {
        let mut a = Arena::new(vec![]);
        let caster = a.caster(
            BaseAttributes::default(),
            Targeting::Point,
            AbilityEffect::Area {
                radius: 2.0,
                delay: 0.0,
                damage: DamageSpec {
                    kind,
                    base: 100.0,
                    ad_ratio: 0.0,
                    ap_ratio: 0.0,
                },
            },
        );
        let target = a.spawn(
            Team::Enemy,
            Vec2::new(3.0, 0.0),
            UnitStats {
                base_attributes: BaseAttributes {
                    armor,
                    magic_resistance,
                    ..default()
                },
                ..default()
            },
            None,
        );
        a.cast(caster, 0, AbilityTarget::Point(Vec2::new(3.0, 0.0)));
        a.advance(0.3);
        assert!(
            (600.0 - a.health(target) - expected).abs() < 0.001,
            "{kind:?}: health {}",
            a.health(target)
        );
        assert!(a.events().iter().any(|e| matches!(e, SimulationEvent::Damaged { kind: actual, origin: DamageOrigin::Ability(AbilityId(100)), .. } if *actual == kind)));
    }
}

#[test]
fn spell_power_and_attack_damage_coefficients_scale_at_commit() {
    let mut a = Arena::new(vec![]);
    let caster = a.caster(
        BaseAttributes {
            ability_power: 50.0,
            ..default()
        },
        Targeting::Direction,
        AbilityEffect::Projectile {
            speed: 30.0,
            radius: 0.2,
            damage: DamageSpec {
                kind: DamageType::Magical,
                base: 75.0,
                ad_ratio: 0.0,
                ap_ratio: 0.6,
            },
        },
    );
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    a.cast(caster, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.5);
    assert_eq!(a.health(target), 495.0);
    let source = a.source();
    a.upsert(
        caster,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 60.0,
                ..default()
            },
            ..default()
        },
    );
    a.command(caster, UnitCommand::AttackTarget(target));
    a.advance(0.5);
    assert_eq!(a.health(target), 395.0);
    assert!(a.events().iter().any(|e| matches!(e, SimulationEvent::Damaged { kind: DamageType::Physical, origin: DamageOrigin::BasicAttack, amount, .. } if *amount == 100.0)));
}

#[test]
fn committed_area_snapshot_survives_source_removal_and_reads_impact_defense() {
    let mut a = Arena::new(vec![]);
    let caster = a.caster(
        BaseAttributes {
            ability_power: 50.0,
            ..default()
        },
        Targeting::Point,
        AbilityEffect::Area {
            radius: 2.0,
            delay: 0.5,
            damage: DamageSpec {
                kind: DamageType::Magical,
                base: 75.0,
                ad_ratio: 0.0,
                ap_ratio: 0.6,
            },
        },
    );
    let target = a.unit(Team::Enemy, Vec2::new(4.0, 0.0));
    a.cast(caster, 0, AbilityTarget::Point(Vec2::new(4.0, 0.0)));
    a.advance(0.2);
    assert_eq!(a.health(target), 600.0);
    let source = a.source();
    a.upsert(
        target,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                magic_resistance: 100.0,
                ..default()
            },
            ..default()
        },
    );
    a.app.world_mut().despawn(caster);
    a.advance(0.6);
    assert_eq!(a.health(target), 547.5);
}

#[test]
fn projectile_keeps_committed_damage_after_source_attributes_change_and_despawn() {
    let mut a = Arena::new(vec![]);
    let caster = a.caster(
        BaseAttributes {
            ability_power: 50.0,
            ..default()
        },
        Targeting::Direction,
        AbilityEffect::Projectile {
            speed: 4.0,
            radius: 0.2,
            damage: DamageSpec {
                kind: DamageType::Magical,
                base: 75.0,
                ad_ratio: 0.0,
                ap_ratio: 0.6,
            },
        },
    );
    let target = a.unit(Team::Enemy, Vec2::new(5.0, 0.0));
    a.cast(caster, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.2);
    let source = a.source();
    a.upsert(
        caster,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                ability_power: 1000.0,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    a.app.world_mut().despawn(caster);
    a.advance(1.5);
    assert_eq!(a.health(target), 495.0);
}

#[test]
fn channel_reuses_committed_damage_and_reads_defenses_for_each_pulse() {
    let mut a = Arena::new(vec![]);
    let caster = a.caster(
        BaseAttributes {
            ability_power: 50.0,
            ..default()
        },
        Targeting::SelfCast,
        AbilityEffect::Channel {
            duration: 2.0,
            interval: 0.5,
            radius: 4.0,
            damage: DamageSpec {
                kind: DamageType::Magical,
                base: 20.0,
                ad_ratio: 0.0,
                ap_ratio: 0.2,
            },
        },
    );
    let target = a.unit(Team::Enemy, Vec2::new(2.0, 0.0));
    a.cast(caster, 0, AbilityTarget::SelfCast);
    a.advance(0.65);
    assert_eq!(a.health(target), 570.0);
    let buff = a.source();
    let defense = a.source();
    a.upsert(
        caster,
        buff,
        AttributeModifier {
            flat: AttributeDelta {
                ability_power: 1000.0,
                ..default()
            },
            ..default()
        },
    );
    a.upsert(
        target,
        defense,
        AttributeModifier {
            flat: AttributeDelta {
                magic_resistance: 100.0,
                ..default()
            },
            ..default()
        },
    );
    a.advance(2.0);
    assert_eq!(a.health(target), 525.0);
}

#[test]
fn overflowing_damage_is_rejected_before_committing_resources() {
    let mut a = Arena::new(vec![]);
    let caster = a.caster(
        BaseAttributes {
            ability_power: f32::MAX,
            ..default()
        },
        Targeting::Point,
        AbilityEffect::Area {
            radius: 2.0,
            delay: 0.0,
            damage: DamageSpec {
                kind: DamageType::Magical,
                base: 75.0,
                ad_ratio: 0.0,
                ap_ratio: f32::MAX,
            },
        },
    );
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    a.cast(caster, 0, AbilityTarget::Point(Vec2::new(3.0, 0.0)));
    a.advance(0.5);
    assert_eq!(a.mana(caster), 300.0);
    assert_eq!(a.cooldown(caster, 0), 0.0);
    assert_eq!(a.health(target), 600.0);
    assert!(a.events().iter().any(|e| matches!(
        e,
        SimulationEvent::CommandRejected {
            reason: Rejection::InvalidDamage,
            ..
        }
    )));
}

#[test]
fn zero_and_overkill_damage_report_actual_health_loss() {
    for (damage, expected, injuries, deaths) in [(0.0, 40.0, 0, 0), (f32::MAX, 0.0, 1, 1)] {
        let mut a = Arena::new(vec![]);
        let caster = a.caster(
            BaseAttributes::default(),
            Targeting::Point,
            AbilityEffect::Area {
                radius: 2.0,
                delay: 0.0,
                damage: DamageSpec::physical(damage),
            },
        );
        let target = a.spawn(
            Team::Enemy,
            Vec2::new(3.0, 0.0),
            UnitStats {
                max_health: 40.0,
                base_attributes: BaseAttributes {
                    armor: -f32::MAX,
                    ..default()
                },
                ..default()
            },
            None,
        );
        a.cast(caster, 0, AbilityTarget::Point(Vec2::new(3.0, 0.0)));
        a.advance(0.3);
        assert_eq!(a.health(target), expected);
        let events = a.events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, SimulationEvent::Damaged { .. }))
                .count(),
            injuries
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, SimulationEvent::Died { .. }))
                .count(),
            deaths
        );
        if injuries != 0 {
            assert!(
                events.iter().any(
                    |e| matches!(e, SimulationEvent::Damaged { amount, .. } if *amount == 40.0)
                )
            );
        }
    }
}

#[test]
fn death_removes_temporary_modifiers_and_respawn_keeps_persistent_modifiers_once() {
    let mut a = Arena::new(vec![]);
    let killer = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.spawn(
        Team::Enemy,
        Vec2::new(3.0, 0.0),
        UnitStats {
            max_health: 40.0,
            ..default()
        },
        Some(1.0),
    );
    let persistent = a.source();
    let temporary = a.source();
    a.upsert(
        target,
        persistent,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 10.0,
                ..default()
            },
            ..default()
        },
    );
    a.upsert(
        target,
        temporary,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 20.0,
                ..default()
            },
            removal: ModifierRemoval::OnDeath,
            ..default()
        },
    );
    a.step();
    assert_eq!(a.attributes(target).attack_damage(), 70.0);
    a.cast(killer, 0, AbilityTarget::Direction(Vec2::X));
    a.advance(0.5);
    assert_eq!(a.health(target), 0.0);
    assert_eq!(a.attributes(target).attack_damage(), 50.0);
    a.upsert(
        target,
        persistent,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 15.0,
                ..default()
            },
            ..default()
        },
    );
    a.upsert(
        target,
        temporary,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 100.0,
                ..default()
            },
            removal: ModifierRemoval::OnDeath,
            ..default()
        },
    );
    a.step();
    assert_eq!(a.attributes(target).attack_damage(), 55.0);
    assert!(a.events().iter().any(|e| matches!(
        e,
        SimulationEvent::AttributeChangeRejected {
            reason: AttributeRejection::Dead,
            ..
        }
    )));
    a.advance(1.1);
    assert_eq!(a.health(target), 40.0);
    assert_eq!(a.attributes(target).attack_damage(), 55.0);
    a.remove(target, persistent);
    a.step();
    assert_eq!(a.attributes(target).attack_damage(), 40.0);
}

#[test]
fn effective_movement_speed_changes_immediately_and_zero_preserves_intent() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            base_attributes: BaseAttributes {
                move_speed: 2.0,
                ..default()
            },
            ..default()
        },
        None,
    );
    a.command(p, UnitCommand::MoveTo(Vec2::new(15.0, 0.0)));
    a.advance(0.5);
    assert!((a.position(p).x - 1.0).abs() < 0.001);
    let faster = a.source();
    let slow = a.source();
    a.upsert(
        p,
        faster,
        AttributeModifier {
            flat: AttributeDelta {
                move_speed: 2.0,
                ..default()
            },
            ..default()
        },
    );
    a.advance(0.5);
    assert!((a.position(p).x - 3.0).abs() < 0.001);
    a.upsert(
        p,
        slow,
        AttributeModifier {
            additive_percent: AttributeDelta {
                move_speed: -1.0,
                ..default()
            },
            ..default()
        },
    );
    let before = a.position(p);
    a.advance(0.5);
    assert_eq!(a.position(p), before);
    assert!(matches!(
        a.app.world().get::<UnitOrder>(p),
        Some(UnitOrder::MoveTo(_))
    ));
    a.remove(p, slow);
    a.advance(0.5);
    assert!((a.position(p).x - 5.0).abs() < 0.001);
}

#[test]
fn zero_walking_speed_does_not_change_committed_dash_speed() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                move_speed: -1.0,
                ..default()
            },
            ..default()
        },
    );
    a.cast(p, 2, AbilityTarget::Point(Vec2::new(6.0, 0.0)));
    a.advance(0.6);
    assert!(a.position(p).distance(Vec2::new(6.0, 0.0)) < 0.01);
    assert_eq!(a.attributes(p).move_speed(), 0.0);
}

#[test]
fn changed_attack_range_is_used_for_release_and_chasing() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.unit(Team::Enemy, Vec2::new(5.0, 0.0));
    a.command(p, UnitCommand::AttackTarget(target));
    a.step();
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_range: -1.0,
                ..default()
            },
            ..default()
        },
    );
    assert!(a.attack_ticks(p, 20).is_empty());
    assert!(
        a.position(p).x > 0.0,
        "failed release should resume chasing"
    );
    a.advance(1.5);
    assert!(a.health(target) < 600.0);
    assert!(a.position(p).distance(a.position(target)) <= 0.901);
}

#[test]
fn attack_speed_produces_exact_fixed_step_cadence() {
    for (speed, period) in [(1.0, 60), (2.0, 30), (f32::MAX, 2)] {
        let mut a = Arena::new(vec![]);
        let p = a.spawn(
            Team::Ally,
            Vec2::ZERO,
            UnitStats {
                base_attributes: BaseAttributes {
                    attack_speed: speed,
                    attack_damage: 0.0,
                    ..default()
                },
                ..default()
            },
            None,
        );
        let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
        a.command(p, UnitCommand::AttackTarget(target));
        let ticks = a.attack_ticks(p, 200);
        assert!(ticks.len() >= 3);
        assert!(
            ticks.windows(2).all(|pair| pair[1] - pair[0] == period),
            "{speed}: {ticks:?}"
        );
    }
}

#[test]
fn changing_attack_speed_during_windup_keeps_the_current_attack_snapshot() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            base_attributes: BaseAttributes {
                attack_speed: 1.0,
                attack_damage: 0.0,
                ..default()
            },
            ..default()
        },
        None,
    );
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    a.command(p, UnitCommand::AttackTarget(target));
    assert!(a.attack_ticks(p, 5).is_empty());
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_speed: 1.0,
                ..default()
            },
            ..default()
        },
    );
    assert_eq!(a.attack_ticks(p, 100), vec![18, 70, 100]);
}

#[test]
fn zero_attack_speed_finishes_existing_windup_and_prevents_new_attacks() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_speed: -1.0,
                ..default()
            },
            ..default()
        },
    );
    a.command(p, UnitCommand::AttackTarget(target));
    assert!(a.attack_ticks(p, 60).is_empty());
    a.remove(p, source);
    a.step();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_speed: -1.0,
                ..default()
            },
            ..default()
        },
    );
    assert_eq!(a.attack_ticks(p, 100), vec![75]);
    assert_eq!(a.health(target), 560.0);
}

#[test]
fn successful_attack_has_a_backswing_that_finishes_without_extra_cadence_ticks() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    let (released, duration) = a.enter_backswing(p, target);
    assert_eq!(duration, 34);
    assert_eq!(
        a.app.world().get::<UnitAction>(p).unwrap().label(),
        "AttackBackswing"
    );
    assert_eq!(
        a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
        duration as f32 * TICK_SECONDS
    );
    for _ in 0..duration - 1 {
        a.step();
    }
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::AttackBackswing {
            remaining: 1,
            duration: 34
        })
    ));
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::AttackWindup { .. })
    ));
    let next = a.attack_ticks(p, 14);
    assert_eq!(next, vec![released + 48]);
    assert_eq!(
        a.health(target),
        560.0,
        "the first projectile hits during backswing"
    );
}

#[test]
fn movement_stop_retarget_and_spell_cancel_backswing_without_resetting_cooldown() {
    for cancellation in ["move", "stop", "retarget", "cast"] {
        let mut a = Arena::new(vec![]);
        let p = a.unit(Team::Ally, Vec2::ZERO);
        let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
        let other = a.unit(Team::Enemy, Vec2::new(0.0, 3.0));
        let (_, duration) = a.enter_backswing(p, target);
        let command = match cancellation {
            "move" => UnitCommand::MoveTo(Vec2::new(-3.0, 0.0)),
            "stop" => UnitCommand::Stop,
            "retarget" => UnitCommand::AttackTarget(other),
            "cast" => UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Direction(Vec2::Y),
            },
            _ => unreachable!(),
        };
        a.command(p, command);
        a.step();
        let action = a.app.world().get::<UnitAction>(p).unwrap();
        if cancellation == "cast" {
            assert!(matches!(action, UnitAction::Casting { .. }));
        } else {
            assert!(matches!(action, UnitAction::Ready));
        }
        assert_eq!(
            a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
            (duration - 1) as f32 * TICK_SECONDS,
            "{cancellation}"
        );
        match cancellation {
            "move" => assert!(a.position(p).x < 0.0),
            "stop" => assert!(matches!(
                a.app.world().get::<UnitOrder>(p),
                Some(UnitOrder::Idle)
            )),
            "retarget" => assert!(
                matches!(a.app.world().get::<UnitOrder>(p), Some(UnitOrder::AttackTarget(id)) if *id == other)
            ),
            _ => {}
        }
        assert!(
            a.attack_ticks(p, 8).is_empty(),
            "{cancellation} must not grant an early attack"
        );
        assert_eq!(
            a.health(target),
            560.0,
            "{cancellation} must not retract the committed projectile"
        );
        assert_eq!(
            a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
            (duration - 9) as f32 * TICK_SECONDS
        );
    }
}

#[test]
fn repeated_target_and_invalid_requests_preserve_the_advancing_backswing() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    let (_, duration) = a.enter_backswing(p, target);
    for (index, command) in [
        UnitCommand::AttackTarget(target),
        UnitCommand::MoveTo(Vec2::splat(f32::NAN)),
        UnitCommand::AttackTarget(p),
        UnitCommand::CastAbility {
            slot: 99,
            target: AbilityTarget::SelfCast,
        },
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::Point(Vec2::new(100.0, 0.0)),
        },
    ]
    .into_iter()
    .enumerate()
    {
        a.command(p, command);
        a.step();
        assert!(
            matches!(a.app.world().get::<UnitAction>(p), Some(UnitAction::AttackBackswing { remaining, duration: original }) if *remaining == duration - index as u32 - 1 && *original == duration)
        );
        assert!(
            matches!(a.app.world().get::<UnitOrder>(p), Some(UnitOrder::AttackTarget(id)) if *id == target)
        );
    }
    let rejections = a
        .events()
        .iter()
        .filter(|e| matches!(e, SimulationEvent::CommandRejected { .. }))
        .count();
    assert_eq!(rejections, 4);
}

#[test]
fn pause_freezes_backswing_and_the_independent_attack_cooldown() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    let (released, duration) = a.enter_backswing(p, target);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = true;
    a.advance(2.0);
    assert_eq!(a.app.world().resource::<SimulationTick>().0, released);
    assert!(
        matches!(a.app.world().get::<UnitAction>(p), Some(UnitAction::AttackBackswing { remaining, duration: original }) if *remaining == duration && *original == duration)
    );
    assert_eq!(
        a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
        duration as f32 * TICK_SECONDS
    );
    assert_eq!(a.health(target), 600.0);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.step();
    assert!(
        matches!(a.app.world().get::<UnitAction>(p), Some(UnitAction::AttackBackswing { remaining, .. }) if *remaining == duration - 1)
    );
    assert_eq!(
        a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
        (duration - 1) as f32 * TICK_SECONDS
    );
}

#[test]
fn death_clears_backswing_while_the_released_attack_still_hits() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            max_health: 40.0,
            ..default()
        },
        Some(1.0),
    );
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    a.enter_backswing(p, target);
    a.cast(target, 0, AbilityTarget::Direction(Vec2::NEG_X));
    a.advance(0.5);
    assert_eq!(a.health(p), 0.0);
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::Ready)
    ));
    assert_eq!(a.health(target), 560.0);
    a.advance(1.1);
    assert_eq!(a.health(p), 40.0);
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::Ready)
    ));
    assert_eq!(
        a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
        0.0
    );
}

#[test]
fn attack_speed_changes_during_backswing_only_affect_the_next_attack() {
    let mut a = Arena::new(vec![]);
    let p = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            base_attributes: BaseAttributes {
                attack_speed: 1.0,
                attack_damage: 0.0,
                ..default()
            },
            ..default()
        },
        None,
    );
    let target = a.unit(Team::Enemy, Vec2::new(3.0, 0.0));
    let (released, duration) = a.enter_backswing(p, target);
    assert_eq!(released, 18);
    assert_eq!(duration, 43);
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_speed: 1.0,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    assert_eq!(a.attributes(p).attack_speed(), 2.0);
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::AttackBackswing {
            remaining: 42,
            duration: 43
        })
    ));
    assert_eq!(
        a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
        42.0 * TICK_SECONDS
    );
    assert_eq!(a.attack_ticks(p, 60), vec![70]);
}

#[test]
fn a_failed_attack_release_never_enters_backswing_or_commits_a_cooldown() {
    let mut a = Arena::new(vec![]);
    let p = a.unit(Team::Ally, Vec2::ZERO);
    let target = a.unit(Team::Enemy, Vec2::new(5.0, 0.0));
    a.command(p, UnitCommand::AttackTarget(target));
    a.advance(14.0 * TICK_SECONDS);
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::AttackWindup { remaining: 1, .. })
    ));
    let source = a.source();
    a.upsert(
        p,
        source,
        AttributeModifier {
            additive_percent: AttributeDelta {
                attack_range: -1.0,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitAction>(p),
        Some(UnitAction::Ready)
    ));
    assert_eq!(
        a.app.world().get::<UnitState>(p).unwrap().attack_cooldown(),
        0.0
    );
    assert!(
        a.events()
            .iter()
            .all(|event| !matches!(event, SimulationEvent::AttackReleased { .. }))
    );
    assert_eq!(a.health(target), 600.0);
}
