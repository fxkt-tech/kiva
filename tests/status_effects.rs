//! Status/effect contracts through the same public commands used by clients.
//! The fixture installs no renderer and never writes simulation runtime fields.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

struct Arena {
    app: App,
    cursor: MessageCursor<SimulationEvent>,
    next_ability: u16,
}

impl Arena {
    fn new(obstacles: Vec<Obstacle>) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(NavigationMap::new(20.0, 1.0, obstacles));
        Self {
            app,
            cursor: MessageCursor::default(),
            next_ability: 1,
        }
    }

    fn register(&mut self, definition: AbilityDefinition) -> AbilityId {
        let id = AbilityId(self.next_ability);
        self.next_ability += 1;
        self.app
            .world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(id, definition)
            .unwrap();
        id
    }

    fn direct(&mut self, effects: Vec<Effect>) -> AbilityId {
        self.register(impact(Targeting::Enemy, Delivery::Direct, effects))
    }

    fn unit(&mut self, team: Team, position: Vec2, abilities: &[AbilityId]) -> Entity {
        self.spawn(team, position, abilities, stats(), None)
    }

    fn spawn(
        &mut self,
        team: Team,
        position: Vec2,
        abilities: &[AbilityId],
        stats: UnitStats,
        respawn_delay: Option<f32>,
    ) -> Entity {
        let mut slots = [None; 4];
        for (slot, ability) in slots.iter_mut().zip(abilities) {
            *slot = Some(*ability);
        }
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Status test unit".into(),
                team,
                position,
                stats,
                abilities: slots,
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

    fn ticks(&mut self, count: usize) {
        for _ in 0..count {
            self.step();
        }
    }

    fn position(&self, unit: Entity) -> Vec2 {
        self.app.world().get::<UnitPose>(unit).unwrap().position()
    }

    fn state(&self, unit: Entity) -> &UnitState {
        self.app.world().get::<UnitState>(unit).unwrap()
    }

    fn control(&self, unit: Entity) -> &ControlState {
        self.app.world().get::<ControlState>(unit).unwrap()
    }

    fn statuses(&self, unit: Entity) -> &UnitStatuses {
        self.app.world().get::<UnitStatuses>(unit).unwrap()
    }

    fn speed(&self, unit: Entity) -> f32 {
        self.app
            .world()
            .get::<Unit>(unit)
            .unwrap()
            .attributes()
            .move_speed()
    }

    fn cooldown(&self, unit: Entity, slot: usize) -> f32 {
        self.app
            .world()
            .get::<AbilitySlots>(unit)
            .unwrap()
            .cooldown(slot)
            .unwrap()
    }

    fn modifier(&mut self, unit: Entity, modifier: AttributeModifier) -> ModifierSourceId {
        let mut changes = self.app.world_mut().resource_mut::<AttributeChanges>();
        let source = changes.allocate_source().unwrap();
        changes.upsert(unit, source, modifier);
        source
    }

    fn events(&mut self) -> Vec<SimulationEvent> {
        self.cursor
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect()
    }
}

fn stats() -> UnitStats {
    UnitStats {
        mana_regen: 0.0,
        ..default()
    }
}

fn duration(ticks: u32) -> StatusDuration {
    StatusDuration::from_ticks(ticks).unwrap()
}

fn stun(ticks: u32) -> Effect {
    Effect::ApplyStatus(StatusSpec::Stun {
        duration: duration(ticks),
    })
}

fn root(ticks: u32) -> Effect {
    Effect::ApplyStatus(StatusSpec::Root {
        duration: duration(ticks),
    })
}

fn slow(ticks: u32, fraction: f32) -> Effect {
    Effect::ApplyStatus(StatusSpec::Slow {
        duration: duration(ticks),
        fraction,
    })
}

fn push(distance: f32, speed: f32, direction: DisplacementDirection) -> Effect {
    Effect::Displace(DisplacementSpec {
        distance,
        speed,
        direction,
    })
}

fn impact(targeting: Targeting, delivery: Delivery, effects: Vec<Effect>) -> AbilityDefinition {
    AbilityDefinition {
        name: "Composable test ability".into(),
        targeting,
        range: 20.0,
        windup: 0.0,
        recovery: 0.0,
        cooldown: 0.0,
        mana_cost: 0.0,
        execution: AbilityExecution::Impact(ImpactDefinition { delivery, effects }),
    }
}

fn dash(speed: f32) -> AbilityDefinition {
    AbilityDefinition {
        execution: AbilityExecution::Dash { speed },
        mana_cost: 40.0,
        cooldown: 3.0,
        ..impact(Targeting::Point, Delivery::Direct, vec![])
    }
}

fn channel() -> AbilityDefinition {
    AbilityDefinition {
        execution: AbilityExecution::Channel {
            duration: 1.0,
            interval: 0.1,
            pulse: ImpactDefinition {
                delivery: Delivery::Area {
                    radius: 6.0,
                    delay: 0.0,
                },
                effects: vec![Effect::Damage(DamageSpec::magical(10.0))],
            },
        },
        mana_cost: 40.0,
        cooldown: 3.0,
        ..impact(Targeting::SelfCast, Delivery::Direct, vec![])
    }
}

fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn one_tick_and_one_hundred_milliseconds_block_exactly_the_promised_steps() {
    assert_eq!(StatusDuration::from_millis(100).unwrap().ticks(), 6);
    assert!(StatusDuration::from_ticks(0).is_none());
    assert!(StatusDuration::from_millis(0).is_none());
    assert!(StatusDuration::from_millis(u64::MAX).is_none());

    for blocked_steps in [1, 6] {
        let mut a = Arena::new(vec![]);
        let ability = a.direct(vec![stun(blocked_steps)]);
        let caster = a.unit(Team::Enemy, Vec2::new(-2.0, 0.0), &[ability]);
        let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
        a.command(victim, UnitCommand::MoveTo(Vec2::new(8.0, 0.0)));
        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        let hit_position = a.position(victim);
        assert!(
            hit_position.x > 0.0,
            "the hit happens after this step's walk"
        );
        assert_eq!(
            a.statuses(victim).iter().next().unwrap().remaining_ticks(),
            blocked_steps
        );
        for _ in 0..blocked_steps {
            assert!(!a.control(victim).can_move());
            a.step();
            assert_eq!(a.position(victim), hit_position);
        }
        assert!(a.statuses(victim).is_empty());
        assert!(a.control(victim).can_move());
        a.step();
        assert!(a.position(victim).x > hit_position.x);
    }
}

#[test]
fn direct_control_does_not_prevent_the_opponents_same_tick_commit() {
    for reverse_spawn in [false, true] {
        let mut a = Arena::new(vec![]);
        let mut definition = impact(
            Targeting::Enemy,
            Delivery::Direct,
            vec![Effect::Damage(DamageSpec::magical(10.0)), stun(2)],
        );
        definition.mana_cost = 40.0;
        definition.cooldown = 3.0;
        let ability = a.register(definition);
        let first_team = if reverse_spawn {
            Team::Enemy
        } else {
            Team::Ally
        };
        let second_team = if reverse_spawn {
            Team::Ally
        } else {
            Team::Enemy
        };
        let first = a.unit(first_team, Vec2::ZERO, &[ability]);
        let second = a.unit(second_team, Vec2::X * 2.0, &[ability]);
        a.cast(first, 0, AbilityTarget::Unit(second));
        a.cast(second, 0, AbilityTarget::Unit(first));
        a.step();
        for unit in [first, second] {
            close(a.state(unit).health(), 590.0);
            close(a.state(unit).mana(), 260.0);
            assert!(a.cooldown(unit, 0) > 2.9);
            assert!(!a.control(unit).can_cast());
        }
        assert_eq!(
            a.events()
                .iter()
                .filter(|e| matches!(e, SimulationEvent::AbilityCommitted { .. }))
                .count(),
            2
        );
    }
}

#[test]
fn targeted_control_rejects_friendly_or_non_unit_selection_without_replacing_a_move() {
    let mut a = Arena::new(vec![]);
    let mut definition = impact(Targeting::Enemy, Delivery::Direct, vec![stun(3)]);
    definition.mana_cost = 40.0;
    definition.cooldown = 3.0;
    let ability = a.register(definition);
    let caster = a.unit(Team::Ally, Vec2::ZERO, &[ability]);
    let friend = a.unit(Team::Ally, Vec2::X * 2.0, &[]);
    a.command(caster, UnitCommand::MoveTo(Vec2::X * 8.0));
    a.cast(caster, 0, AbilityTarget::Unit(friend));
    a.cast(caster, 0, AbilityTarget::Point(Vec2::X * 2.0));
    a.step();
    close(a.state(caster).mana(), 300.0);
    close(a.cooldown(caster, 0), 0.0);
    assert!(a.position(caster).x > 0.0);
    assert!(a.statuses(friend).is_empty());
    assert_eq!(a.events().iter().filter(|e| matches!(e, SimulationEvent::CommandRejected { unit, reason: Rejection::InvalidTarget } if *unit == caster)).count(), 2);
}

#[test]
fn controlled_new_commands_are_rejected_without_replacing_the_prior_move() {
    let mut a = Arena::new(vec![]);
    let control = a.direct(vec![stun(4)]);
    let damage = a.direct(vec![Effect::Damage(DamageSpec::magical(25.0))]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[control]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[damage]);
    a.command(victim, UnitCommand::MoveTo(Vec2::X * 8.0));
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    a.events();
    let hit_position = a.position(victim);
    a.command(victim, UnitCommand::MoveTo(Vec2::X * -8.0));
    a.command(victim, UnitCommand::AttackTarget(caster));
    a.cast(victim, 0, AbilityTarget::Unit(caster));
    a.step();
    assert_eq!(a.position(victim), hit_position);
    let rejected = a.events().iter().filter(|e| matches!(e, SimulationEvent::CommandRejected { unit, reason: Rejection::Controlled } if *unit == victim)).count();
    assert_eq!(rejected, 3);
    a.ticks(4);
    assert!(a.position(victim).x > hit_position.x);
    close(a.state(caster).health(), 600.0);
    close(a.state(victim).mana(), 300.0);
}

#[test]
fn stop_during_control_clears_the_order_instead_of_resuming_it() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![stun(3)]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.command(victim, UnitCommand::MoveTo(Vec2::X * 8.0));
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    let position = a.position(victim);
    a.command(victim, UnitCommand::Stop);
    a.ticks(12);
    assert_eq!(a.position(victim), position);
    assert!(matches!(
        a.app.world().get::<UnitOrder>(victim),
        Some(UnitOrder::Idle)
    ));
}

#[test]
fn root_allows_attacks_and_non_movement_casts_but_rejects_walk_and_dash() {
    let mut a = Arena::new(vec![]);
    let root_ability = a.direct(vec![root(60)]);
    let damage = a.direct(vec![Effect::Damage(DamageSpec::magical(25.0))]);
    let dash_ability = a.register(dash(6.0));
    let caster = a.unit(Team::Enemy, Vec2::X * 2.0, &[root_ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[damage, dash_ability]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    assert!(!a.control(victim).can_move());
    assert!(!a.control(victim).can_dash());
    assert!(a.control(victim).can_attack());
    assert!(a.control(victim).can_cast());
    a.command(victim, UnitCommand::MoveTo(Vec2::X * 8.0));
    a.command(victim, UnitCommand::AttackMove(Vec2::X * 8.0));
    a.cast(victim, 1, AbilityTarget::Point(Vec2::X * 8.0));
    a.cast(victim, 0, AbilityTarget::Unit(caster));
    a.step();
    close(a.state(caster).health(), 575.0);
    close(a.cooldown(victim, 1), 0.0);
    close(a.state(victim).mana(), 300.0);
    a.command(victim, UnitCommand::AttackTarget(caster));
    a.ticks(20);
    assert_eq!(a.position(victim), Vec2::ZERO);
    assert!(
        a.events().iter().any(
            |e| matches!(e, SimulationEvent::AttackReleased { source, .. } if *source == victim)
        )
    );
}

#[test]
fn rooted_attack_target_is_kept_without_chasing_then_resumes_after_expiry() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![root(4)]);
    let caster = a.unit(Team::Enemy, Vec2::X * 12.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    a.command(victim, UnitCommand::AttackTarget(caster));
    a.ticks(4);
    assert_eq!(a.position(victim), Vec2::ZERO);
    a.step();
    assert!(a.position(victim).x > 0.0);
}

#[test]
fn root_preserves_an_attack_move_and_its_allowed_in_range_attack() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![root(30)]);
    let caster = a.unit(Team::Enemy, Vec2::X * 2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.command(victim, UnitCommand::AttackMove(Vec2::X * 8.0));
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.ticks(20);
    assert_eq!(a.position(victim), Vec2::ZERO);
    assert!(matches!(
        a.app.world().get::<UnitOrder>(victim),
        Some(UnitOrder::AttackMove { .. })
    ));
    assert!(
        a.events().iter().any(
            |e| matches!(e, SimulationEvent::AttackReleased { source, .. } if *source == victim)
        )
    );
}

#[test]
fn interrupted_cast_spends_nothing_and_does_not_restart_after_control() {
    let mut a = Arena::new(vec![]);
    let control = a.direct(vec![stun(3)]);
    let mut definition = impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![Effect::Damage(DamageSpec::magical(100.0))],
    );
    definition.windup = 0.5;
    definition.mana_cost = 40.0;
    definition.cooldown = 3.0;
    let damage = a.register(definition);
    let caster = a.unit(Team::Enemy, Vec2::X * 2.0, &[control]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[damage]);
    a.cast(victim, 0, AbilityTarget::Unit(caster));
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitAction>(victim),
        Some(UnitAction::Ready)
    ));
    a.ticks(45);
    close(a.state(victim).mana(), 300.0);
    close(a.cooldown(victim, 0), 0.0);
    close(a.state(caster).health(), 600.0);
    assert!(
        !a.events().iter().any(
            |e| matches!(e, SimulationEvent::AbilityCommitted { unit, .. } if *unit == victim)
        )
    );
}

#[test]
fn stun_cancels_attack_windup_but_keeps_already_released_attack_and_interval() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![stun(3)]);
    let attacker = a.unit(Team::Ally, Vec2::ZERO, &[]);
    let target = a.unit(Team::Enemy, Vec2::X * 4.0, &[ability]);
    a.command(attacker, UnitCommand::AttackTarget(target));
    a.cast(target, 0, AbilityTarget::Unit(attacker));
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitAction>(attacker),
        Some(UnitAction::Ready)
    ));
    close(a.state(attacker).attack_cooldown(), 0.0);
    assert!(
        !a.events()
            .iter()
            .any(|e| matches!(e, SimulationEvent::AttackReleased { .. }))
    );

    let mut released = false;
    for _ in 0..40 {
        a.step();
        if a.events().iter().any(
            |e| matches!(e, SimulationEvent::AttackReleased { source, .. } if *source == attacker),
        ) {
            released = true;
            break;
        }
    }
    assert!(released);
    let cooldown = a.state(attacker).attack_cooldown();
    a.cast(target, 0, AbilityTarget::Unit(attacker));
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitAction>(attacker),
        Some(UnitAction::Ready)
    ));
    close(a.state(attacker).attack_cooldown(), cooldown - TICK_SECONDS);
    a.ticks(12);
    close(a.state(target).health(), 560.0);
    assert!(!a.events().iter().any(
        |e| matches!(e, SimulationEvent::AttackReleased { source, .. } if *source == attacker)
    ));
}

#[test]
fn stun_stops_future_channel_pulses_without_refunding_committed_costs() {
    let mut a = Arena::new(vec![]);
    let control = a.direct(vec![stun(3)]);
    let channel_ability = a.register(channel());
    let caster = a.unit(Team::Enemy, Vec2::X * 2.0, &[control]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[channel_ability]);
    a.cast(victim, 0, AbilityTarget::SelfCast);
    a.ticks(8);
    assert!(a.state(caster).health() < 600.0);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    let health = a.state(caster).health();
    close(a.state(victim).mana(), 260.0);
    let cooldown = a.cooldown(victim, 0);
    a.ticks(65);
    close(a.state(caster).health(), health);
    close(a.state(victim).mana(), 260.0);
    assert!(a.cooldown(victim, 0) < cooldown);
    assert!(a.cooldown(victim, 0) > 0.0);
}

#[test]
fn root_keeps_non_movement_channel_running_with_exact_pulse_count() {
    let mut a = Arena::new(vec![]);
    let control = a.direct(vec![root(80)]);
    let channel_ability = a.register(channel());
    let caster = a.unit(Team::Enemy, Vec2::X * 2.0, &[control]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[channel_ability]);
    a.cast(victim, 0, AbilityTarget::SelfCast);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.ticks(65);
    close(a.state(caster).health(), 500.0);
    close(a.state(victim).mana(), 260.0);
    assert_eq!(a.position(victim), Vec2::ZERO);
}

#[test]
fn root_and_stun_interrupt_committed_dash_without_refund_or_later_resume() {
    for effect in [root(3), stun(3)] {
        let mut a = Arena::new(vec![]);
        let ability = a.direct(vec![effect]);
        let dash_ability = a.register(dash(6.0));
        let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
        let victim = a.unit(Team::Ally, Vec2::ZERO, &[dash_ability]);
        a.cast(victim, 0, AbilityTarget::Point(Vec2::X * 8.0));
        a.step();
        assert!(a.position(victim).x > 0.0);
        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        let position = a.position(victim);
        a.ticks(15);
        assert_eq!(a.position(victim), position);
        close(a.state(victim).mana(), 260.0);
        assert!(a.cooldown(victim, 0) > 2.0);
    }
}

#[test]
fn complete_slow_stops_walking_but_not_dash_and_expires_without_residuals() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![slow(5, 1.0)]);
    let dash_ability = a.register(dash(6.0));
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[dash_ability]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    close(a.speed(victim), 0.0);
    assert!(a.control(victim).can_move());
    assert!(a.control(victim).can_dash());
    a.command(victim, UnitCommand::MoveTo(Vec2::X * 8.0));
    a.step();
    assert_eq!(a.position(victim), Vec2::ZERO);
    a.cast(victim, 0, AbilityTarget::Point(Vec2::X * 3.0));
    a.ticks(4);
    close(a.position(victim).x, 0.4);
    close(a.speed(victim), 5.5);
    a.ticks(6);
    close(a.position(victim).x, 1.0);
}

#[test]
fn strongest_slow_multiplies_haste_and_restores_the_weaker_live_instance() {
    let mut a = Arena::new(vec![]);
    let weak = a.direct(vec![slow(10, 0.3)]);
    let strong = a.direct(vec![slow(3, 0.5)]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[weak, strong]);
    let victim = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        &[],
        UnitStats {
            base_attributes: BaseAttributes {
                move_speed: 10.0,
                ..default()
            },
            ..stats()
        },
        None,
    );
    a.modifier(
        victim,
        AttributeModifier {
            additive_percent: AttributeDelta {
                move_speed: 0.2,
                ..default()
            },
            ..default()
        },
    );
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    close(a.speed(victim), 8.4);
    a.cast(caster, 1, AbilityTarget::Unit(victim));
    a.step();
    close(a.speed(victim), 6.0);
    assert_eq!(a.statuses(victim).len(), 2);
    a.ticks(3);
    close(a.speed(victim), 8.4);
    assert_eq!(a.statuses(victim).len(), 1);
    a.ticks(6);
    close(a.speed(victim), 12.0);
    assert!(a.statuses(victim).is_empty());
}

#[test]
fn stronger_short_slow_does_not_borrow_the_lifetime_of_a_weaker_long_slow() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![slow(2, 0.7), slow(7, 0.2)]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        &[],
        UnitStats {
            base_attributes: BaseAttributes {
                move_speed: 10.0,
                ..default()
            },
            ..stats()
        },
        None,
    );
    for _ in 0..3 {
        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        close(a.speed(victim), 3.0);
        a.ticks(2);
        close(a.speed(victim), 8.0);
        a.ticks(5);
        close(a.speed(victim), 10.0);
    }
}

#[test]
fn repeated_same_source_slow_hits_keep_independent_expiration_times() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![slow(3, 0.5)]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    assert_eq!(a.statuses(victim).len(), 2);
    a.ticks(2);
    assert_eq!(a.statuses(victim).len(), 1);
    close(a.speed(victim), 2.75);
    a.step();
    assert!(a.statuses(victim).is_empty());
    close(a.speed(victim), 5.5);
}

#[test]
fn repeated_airborne_keeps_the_old_height_curve_and_remains_a_planar_target() {
    let mut a = Arena::new(vec![]);
    let airborne = a.direct(vec![Effect::ApplyStatus(StatusSpec::Airborne {
        duration: duration(12),
    })]);
    let damage = a.direct(vec![Effect::Damage(DamageSpec::magical(10.0))]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[airborne, damage]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.ticks(4);
    let old_instance = a.statuses(victim).iter().next().unwrap().id();
    let old_height = a.statuses(victim).airborne_height(1.0);
    assert!(old_height > 0.5);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    assert_eq!(a.statuses(victim).len(), 2);
    assert!(a.statuses(victim).iter().any(|s| s.id() == old_instance));
    assert!(a.statuses(victim).airborne_height(1.0) >= old_height);
    a.cast(caster, 1, AbilityTarget::Unit(victim));
    a.step();
    close(a.state(victim).health(), 590.0);
    assert_eq!(a.position(victim), Vec2::ZERO);
    a.ticks(11);
    assert!(a.statuses(victim).is_empty());
    assert!(a.control(victim).can_move());
    close(a.statuses(victim).airborne_height(1.0), 0.0);
}

#[test]
fn overlapping_control_sources_restore_each_permission_only_when_appropriate() {
    let mut a = Arena::new(vec![]);
    let root_ability = a.direct(vec![root(5)]);
    let stun_ability = a.direct(vec![stun(2)]);
    let first = a.unit(Team::Enemy, Vec2::X * -2.0, &[root_ability]);
    let second = a.unit(Team::Enemy, Vec2::Y * -2.0, &[stun_ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(first, 0, AbilityTarget::Unit(victim));
    a.cast(second, 0, AbilityTarget::Unit(victim));
    a.step();
    assert_eq!(a.statuses(victim).len(), 2);
    a.ticks(2);
    assert!(!a.control(victim).can_move());
    assert!(a.control(victim).can_attack());
    assert!(a.control(victim).can_cast());
    a.ticks(3);
    assert!(a.control(victim).can_move());
    assert!(a.control(victim).can_dash());
}

#[test]
fn same_source_hard_control_refreshes_while_different_sources_remain_independent() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![stun(5)]);
    let first = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let second = a.unit(Team::Enemy, Vec2::Y * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(first, 0, AbilityTarget::Unit(victim));
    a.step();
    let first_instance = a.statuses(victim).iter().next().unwrap().id();
    a.ticks(2);
    a.cast(first, 0, AbilityTarget::Unit(victim));
    a.step();
    assert_eq!(a.statuses(victim).len(), 1);
    assert_eq!(
        a.statuses(victim).iter().next().unwrap().id(),
        first_instance
    );
    assert_eq!(
        a.statuses(victim).iter().next().unwrap().remaining_ticks(),
        5
    );
    a.cast(second, 0, AbilityTarget::Unit(victim));
    a.step();
    assert_eq!(a.statuses(victim).len(), 2);
    a.ticks(4);
    assert_eq!(a.statuses(victim).len(), 1);
    assert!(!a.control(victim).can_cast());
    a.step();
    assert!(a.statuses(victim).is_empty());
    assert!(a.control(victim).can_cast());
}

#[test]
fn move_speed_factor_defaults_to_identity_validates_atomically_and_precedes_saturation() {
    let mut a = Arena::new(vec![]);
    let unit = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        &[],
        UnitStats {
            base_attributes: BaseAttributes {
                move_speed: f32::MAX,
                ..default()
            },
            ..stats()
        },
        None,
    );
    let source = a.modifier(
        unit,
        AttributeModifier {
            flat: AttributeDelta {
                move_speed: f32::MAX,
                ..default()
            },
            additive_percent: AttributeDelta {
                move_speed: 1.0,
                ..default()
            },
            move_speed_multiplier: 0.125,
            ..default()
        },
    );
    a.step();
    let expected = f32::MAX * 0.5;
    assert!((a.speed(unit) / expected - 1.0).abs() < 0.0001);
    for invalid in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        a.app.world_mut().resource_mut::<AttributeChanges>().upsert(
            unit,
            source,
            AttributeModifier {
                move_speed_multiplier: invalid,
                ..default()
            },
        );
        a.step();
        assert_eq!(a.speed(unit), expected);
        assert!(a.events().iter().any(|e| matches!(
            e,
            SimulationEvent::AttributeChangeRejected {
                reason: AttributeRejection::InvalidModifier,
                ..
            }
        )));
    }
    a.app
        .world_mut()
        .resource_mut::<AttributeChanges>()
        .remove(unit, source);
    a.step();
    assert_eq!(a.speed(unit), f32::MAX);

    let regular = a.unit(Team::Ally, Vec2::X * 2.0, &[]);
    a.modifier(
        regular,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 10.0,
                ..default()
            },
            ..default()
        },
    );
    a.step();
    close(a.speed(regular), 5.5);
}

#[test]
fn ordered_effects_skip_after_lethal_damage_and_zero_damage_still_applies_control() {
    for (effects, applied) in [
        (
            vec![Effect::Damage(DamageSpec::magical(1000.0)), stun(5)],
            false,
        ),
        (
            vec![stun(5), Effect::Damage(DamageSpec::magical(1000.0))],
            true,
        ),
    ] {
        let mut a = Arena::new(vec![]);
        let ability = a.direct(effects);
        let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
        let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        assert!(!a.state(victim).alive());
        assert!(a.statuses(victim).is_empty());
        assert_eq!(
            a.events().iter().any(
                |e| matches!(e, SimulationEvent::StatusApplied { unit, .. } if *unit == victim)
            ),
            applied
        );
    }
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![Effect::Damage(DamageSpec::magical(0.0)), stun(1)]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    close(a.state(victim).health(), 600.0);
    assert!(!a.control(victim).can_attack());
    assert!(
        !a.events()
            .iter()
            .any(|e| matches!(e, SimulationEvent::Damaged { .. }))
    );
}

#[test]
fn every_effect_snapshot_is_validated_before_any_cost_or_payload_is_committed() {
    let mut a = Arena::new(vec![]);
    let mut definition = impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![
            stun(5),
            Effect::Damage(DamageSpec::magical(10.0)),
            Effect::Damage(DamageSpec {
                kind: DamageType::Magical,
                base: f32::MAX,
                ad_ratio: 2.0,
                ap_ratio: 0.0,
            }),
        ],
    );
    definition.mana_cost = 40.0;
    definition.cooldown = 3.0;
    let ability = a.register(definition);
    let caster = a.spawn(
        Team::Enemy,
        Vec2::X * -2.0,
        &[ability],
        UnitStats {
            base_attributes: BaseAttributes {
                attack_damage: f32::MAX,
                ..default()
            },
            ..stats()
        },
        None,
    );
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    close(a.state(caster).mana(), 300.0);
    close(a.cooldown(caster, 0), 0.0);
    close(a.state(victim).health(), 600.0);
    assert!(a.statuses(victim).is_empty());
    assert!(a.events().iter().any(|e| matches!(
        e,
        SimulationEvent::CommandRejected {
            reason: Rejection::InvalidDamage,
            ..
        }
    )));
}

#[test]
fn stop_and_hard_control_cannot_cancel_forced_motion_or_erase_other_locks() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![
        push(3.0, 60.0, DisplacementDirection::AwayFromCenter),
        stun(8),
        Effect::ApplyStatus(StatusSpec::Airborne {
            duration: duration(10),
        }),
    ]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.command(victim, UnitCommand::MoveTo(Vec2::X * 8.0));
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    let initial = a.position(victim);
    assert!(a.control(victim).forced_motion());
    a.command(victim, UnitCommand::Stop);
    a.ticks(3);
    close(a.position(victim).x, initial.x + 3.0);
    assert!(!a.control(victim).forced_motion());
    assert!(!a.control(victim).can_attack());
    a.ticks(5);
    assert!(
        !a.control(victim).can_move(),
        "airborne outlives stun and motion"
    );
    a.ticks(15);
    close(a.position(victim).x, initial.x + 3.0);
    assert!(a.control(victim).can_move());
}

#[test]
fn forced_displacement_is_swept_against_walls_and_map_bounds() {
    for (obstacles, start, caster_position, maximum_x) in [
        (
            vec![Obstacle {
                center: Vec2::X * 2.0,
                half_size: Vec2::new(0.5, 3.0),
            }],
            Vec2::ZERO,
            Vec2::X * -2.0,
            1.05,
        ),
        (vec![], Vec2::X * 18.0, Vec2::X * 16.0, 19.55),
    ] {
        let mut a = Arena::new(obstacles);
        let ability = a.direct(vec![push(
            8.0,
            600.0,
            DisplacementDirection::AwayFromCenter,
        )]);
        let caster = a.unit(Team::Enemy, caster_position, &[ability]);
        let victim = a.unit(Team::Ally, start, &[]);
        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        assert_eq!(
            a.position(victim),
            start,
            "new motion waits for the next movement phase"
        );
        a.step();
        let position = a.position(victim);
        assert!(position.x > start.x);
        assert!(position.x < maximum_x);
        assert!(
            a.app
                .world()
                .resource::<NavigationMap>()
                .walkable(position, 0.45)
        );
        assert!(!a.control(victim).forced_motion());
        a.ticks(3);
        assert_eq!(a.position(victim), position);

        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        a.step();
        assert!(
            !a.control(victim).forced_motion(),
            "a push blocked at its starting point must release its lock"
        );
        assert!(a.position(victim).distance(position) < 0.0011);
    }
}

#[test]
fn very_slow_forced_motion_finishes_when_blocked_but_moves_normally_without_a_wall() {
    for blocked in [true, false] {
        let mut a = Arena::new(vec![]);
        let start = if blocked {
            a.app
                .world()
                .resource::<NavigationMap>()
                .sweep(Vec2::X * 18.0, Vec2::X * 20.0, 0.45)
        } else {
            Vec2::ZERO
        };
        let ability = a.direct(vec![push(
            1.0,
            0.001,
            DisplacementDirection::AwayFromCenter,
        )]);
        let caster = a.unit(Team::Enemy, start - Vec2::X * 2.0, &[ability]);
        let victim = a.unit(Team::Ally, start, &[]);
        a.cast(caster, 0, AbilityTarget::Unit(victim));
        a.step();
        assert!(a.control(victim).forced_motion());
        a.step();
        if blocked {
            assert_eq!(a.position(victim), start);
            assert!(!a.control(victim).forced_motion());
        } else {
            assert!(a.position(victim).x > start.x);
            assert!(a.control(victim).forced_motion());
        }
    }
}

#[test]
fn a_forced_speed_below_one_representable_step_releases_its_lock() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![push(
        1.0,
        f32::from_bits(1),
        DisplacementDirection::AwayFromCenter,
    )]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    assert!(a.control(victim).forced_motion());
    a.step();
    assert_eq!(a.position(victim), Vec2::ZERO);
    assert!(!a.control(victim).forced_motion());
    assert!(a.control(victim).can_move());
}

#[test]
fn last_displacement_replaces_prior_motion_and_pull_never_overshoots_its_center() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![
        push(6.0, 60.0, DisplacementDirection::AwayFromCenter),
        push(8.0, 60.0, DisplacementDirection::TowardCenter),
    ]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    a.ticks(2);
    close(a.position(victim).x, -2.0);
    assert!(!a.control(victim).forced_motion());
    a.ticks(5);
    close(a.position(victim).x, -2.0);
}

#[test]
fn a_new_forced_request_replaces_an_already_running_motion() {
    let mut a = Arena::new(vec![]);
    let first = a.direct(vec![push(6.0, 60.0, DisplacementDirection::AwayFromCenter)]);
    let replacement = a.direct(vec![push(1.0, 60.0, DisplacementDirection::TowardCenter)]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[first, replacement]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    a.cast(caster, 1, AbilityTarget::Unit(victim));
    a.step();
    close(a.position(victim).x, 1.0);
    a.step();
    close(a.position(victim).x, 0.0);
    assert!(!a.control(victim).forced_motion());
    a.ticks(8);
    close(a.position(victim).x, 0.0);
}

#[test]
fn pull_at_its_center_is_finite_and_releases_its_motion_lock() {
    let mut a = Arena::new(vec![]);
    let ability = a.register(impact(
        Targeting::Point,
        Delivery::Area {
            radius: 1.0,
            delay: 0.0,
        },
        vec![push(5.0, 60.0, DisplacementDirection::TowardCenter)],
    ));
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Point(Vec2::ZERO));
    a.ticks(2);
    assert_eq!(a.position(victim), Vec2::ZERO);
    assert!(a.position(victim).is_finite());
    assert!(!a.control(victim).forced_motion());
    assert!(a.control(victim).can_move());
}

#[test]
fn committed_projectile_effects_keep_source_snapshot_and_direction_after_source_removal() {
    let mut a = Arena::new(vec![]);
    let ability = a.register(impact(
        Targeting::Direction,
        Delivery::Projectile {
            speed: 6.0,
            radius: 0.1,
        },
        vec![
            Effect::Damage(DamageSpec {
                kind: DamageType::Magical,
                base: 10.0,
                ad_ratio: 0.0,
                ap_ratio: 1.0,
            }),
            slow(3, 0.5),
            push(2.0, 60.0, DisplacementDirection::AlongImpact),
        ],
    ));
    let caster = a.spawn(
        Team::Enemy,
        Vec2::ZERO,
        &[ability],
        UnitStats {
            base_attributes: BaseAttributes {
                ability_power: 20.0,
                ..default()
            },
            ..stats()
        },
        None,
    );
    let victim = a.unit(Team::Ally, Vec2::X * 3.0, &[]);
    a.cast(caster, 0, AbilityTarget::Direction(Vec2::X));
    a.step();
    a.app.world_mut().despawn(caster);
    let mut hit = false;
    for _ in 0..40 {
        a.step();
        if a.state(victim).health() < 600.0 {
            hit = true;
            close(a.speed(victim), 2.75);
            break;
        }
    }
    assert!(hit);
    close(a.state(victim).health(), 570.0);
    a.ticks(5);
    close(a.position(victim).x, 5.0);
    close(a.speed(victim), 5.5);
    assert!(
        a.events()
            .iter()
            .any(|e| matches!(e, SimulationEvent::Damaged { source, .. } if *source == caster))
    );
}

#[test]
fn death_clears_status_motion_and_temporary_speed_while_respawn_keeps_explicit_sources() {
    let mut a = Arena::new(vec![]);
    let control = a.direct(vec![
        slow(30, 0.5),
        stun(30),
        Effect::ApplyStatus(StatusSpec::Airborne {
            duration: duration(30),
        }),
        push(8.0, 60.0, DisplacementDirection::AwayFromCenter),
    ]);
    let kill = a.direct(vec![Effect::Damage(DamageSpec::magical(1000.0))]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[control, kill]);
    let victim = a.spawn(Team::Ally, Vec2::ZERO, &[], stats(), Some(0.05));
    let old_life = a.state(victim).life_ref(victim);
    a.modifier(
        victim,
        AttributeModifier {
            additive_percent: AttributeDelta {
                move_speed: 0.2,
                ..default()
            },
            ..default()
        },
    );
    a.modifier(
        victim,
        AttributeModifier {
            flat: AttributeDelta {
                move_speed: 2.0,
                ..default()
            },
            removal: ModifierRemoval::OnDeath,
            ..default()
        },
    );
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    close(a.speed(victim), 4.5);
    a.cast(caster, 1, AbilityTarget::Unit(victim));
    a.step();
    assert!(!a.state(victim).alive());
    assert!(a.statuses(victim).is_empty());
    assert!(!a.control(victim).forced_motion());
    close(a.speed(victim), 6.6);
    a.ticks(12);
    assert!(a.state(victim).alive());
    assert_ne!(a.state(victim).life_ref(victim), old_life);
    assert_eq!(a.position(victim), Vec2::ZERO);
    assert!(a.control(victim).can_move());
    close(a.speed(victim), 6.6);
}

#[test]
fn old_targeted_cast_and_homing_projectile_do_not_retarget_a_respawned_life() {
    let mut a = Arena::new(vec![]);
    let homing = a.register(impact(
        Targeting::Enemy,
        Delivery::Projectile {
            speed: 3.0,
            radius: 0.1,
        },
        vec![Effect::Damage(DamageSpec::magical(50.0)), stun(20)],
    ));
    let mut delayed_definition = impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![Effect::Damage(DamageSpec::magical(80.0))],
    );
    delayed_definition.windup = 0.3;
    delayed_definition.mana_cost = 40.0;
    delayed_definition.cooldown = 3.0;
    let delayed = a.register(delayed_definition);
    let area = a.register(impact(
        Targeting::Point,
        Delivery::Area {
            radius: 1.0,
            delay: 0.2,
        },
        vec![Effect::Damage(DamageSpec::magical(25.0))],
    ));
    let kill = a.direct(vec![Effect::Damage(DamageSpec::magical(1000.0))]);
    let projectile_source = a.unit(Team::Ally, Vec2::ZERO, &[homing, area]);
    let casting_source = a.unit(Team::Ally, Vec2::Y, &[delayed]);
    let killer = a.unit(Team::Ally, Vec2::new(6.0, 0.0), &[kill]);
    let victim = a.spawn(Team::Enemy, Vec2::X * 8.0, &[], stats(), Some(0.001));
    a.cast(projectile_source, 0, AbilityTarget::Unit(victim));
    a.cast(casting_source, 0, AbilityTarget::Unit(victim));
    a.step();
    a.cast(projectile_source, 1, AbilityTarget::Point(Vec2::X * 8.0));
    a.cast(killer, 0, AbilityTarget::Unit(victim));
    a.step();
    assert!(!a.state(victim).alive());
    a.ticks(200);
    assert!(a.state(victim).alive());
    close(a.state(victim).health(), 575.0);
    close(a.state(casting_source).mana(), 300.0);
    close(a.cooldown(casting_source, 0), 0.0);
    assert!(a.statuses(victim).is_empty());
    let events = a.events();
    assert!(!events.iter().any(|e| matches!(e, SimulationEvent::Damaged { origin: DamageOrigin::Ability(id), .. } if *id == homing || *id == delayed)));
    assert!(events.iter().any(|e| matches!(e, SimulationEvent::Damaged { origin: DamageOrigin::Ability(id), .. } if *id == area)));
}

#[test]
fn pause_freezes_status_and_forced_motion_without_duplicate_application() {
    let mut a = Arena::new(vec![]);
    let ability = a.direct(vec![
        stun(4),
        push(3.0, 60.0, DisplacementDirection::AwayFromCenter),
    ]);
    let caster = a.unit(Team::Enemy, Vec2::X * -2.0, &[ability]);
    let victim = a.unit(Team::Ally, Vec2::ZERO, &[]);
    a.cast(caster, 0, AbilityTarget::Unit(victim));
    a.step();
    a.events();
    let tick = a.app.world().resource::<SimulationTick>().0;
    a.app.world_mut().resource_mut::<SimulationControl>().paused = true;
    a.ticks(20);
    assert_eq!(a.app.world().resource::<SimulationTick>().0, tick);
    assert_eq!(a.position(victim), Vec2::ZERO);
    assert_eq!(
        a.statuses(victim).iter().next().unwrap().remaining_ticks(),
        4
    );
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.ticks(4);
    close(a.position(victim).x, 3.0);
    assert!(a.statuses(victim).is_empty());
    assert!(
        !a.events()
            .iter()
            .any(|e| matches!(e, SimulationEvent::StatusApplied { .. }))
    );
}

#[test]
fn shared_damage_and_status_payload_works_for_direct_projectile_and_area() {
    for (targeting, delivery, target) in [
        (Targeting::Enemy, Delivery::Direct, None),
        (
            Targeting::Direction,
            Delivery::Projectile {
                speed: 60.0,
                radius: 0.1,
            },
            Some(AbilityTarget::Direction(Vec2::X)),
        ),
        (
            Targeting::Point,
            Delivery::Area {
                radius: 1.0,
                delay: 0.0,
            },
            Some(AbilityTarget::Point(Vec2::X * 3.0)),
        ),
    ] {
        let mut a = Arena::new(vec![]);
        let payload = vec![Effect::Damage(DamageSpec::magical(10.0)), stun(3)];
        let ability = a.register(impact(targeting, delivery, payload));
        let caster = a.unit(Team::Enemy, Vec2::ZERO, &[ability]);
        let victim = a.unit(Team::Ally, Vec2::X * 3.0, &[]);
        a.cast(caster, 0, target.unwrap_or(AbilityTarget::Unit(victim)));
        let mut hit = false;
        for _ in 0..5 {
            a.step();
            if a.state(victim).health() < 600.0 {
                hit = true;
                break;
            }
        }
        assert!(hit);
        close(a.state(victim).health(), 590.0);
        assert!(!a.control(victim).can_cast());
        assert_eq!(
            a.statuses(victim).iter().next().unwrap().remaining_ticks(),
            3
        );
    }
}

#[test]
fn clocks_that_cannot_decrease_by_one_fixed_step_are_rejected() {
    let frozen = 1_000_000.0_f32;
    assert_eq!(frozen - TICK_SECONDS, frozen);
    let mut catalog = AbilityCatalog::default();
    let base = impact(Targeting::Enemy, Delivery::Direct, vec![stun(1)]);
    let mut invalid = vec![
        AbilityDefinition {
            windup: frozen,
            ..base.clone()
        },
        AbilityDefinition {
            recovery: frozen,
            ..base.clone()
        },
        AbilityDefinition {
            cooldown: frozen,
            ..base
        },
        impact(
            Targeting::Point,
            Delivery::Area {
                radius: 1.0,
                delay: frozen,
            },
            vec![stun(1)],
        ),
    ];
    for interval in [0.5, frozen] {
        let mut definition = channel();
        if let AbilityExecution::Channel {
            duration,
            interval: pulse_interval,
            ..
        } = &mut definition.execution
        {
            *duration = frozen;
            *pulse_interval = interval;
        }
        invalid.push(definition);
    }
    for definition in invalid {
        assert!(catalog.register(AbilityId(1), definition).is_err());
        assert!(catalog.get(AbilityId(1)).is_none());
    }
}

#[test]
fn invalid_effects_and_target_delivery_combinations_are_rejected_at_registration() {
    let mut catalog = AbilityCatalog::default();
    let invalid = [
        impact(Targeting::Enemy, Delivery::Direct, vec![]),
        impact(Targeting::Point, Delivery::Direct, vec![stun(1)]),
        impact(Targeting::SelfCast, Delivery::Direct, vec![stun(1)]),
        impact(
            Targeting::Point,
            Delivery::Projectile {
                speed: 6.0,
                radius: 0.1,
            },
            vec![stun(1)],
        ),
        impact(
            Targeting::Direction,
            Delivery::Area {
                radius: 1.0,
                delay: 0.0,
            },
            vec![stun(1)],
        ),
        impact(Targeting::Enemy, Delivery::Direct, vec![slow(1, 0.0)]),
        impact(Targeting::Enemy, Delivery::Direct, vec![slow(1, 1.1)]),
        impact(Targeting::Enemy, Delivery::Direct, vec![slow(1, f32::NAN)]),
        impact(
            Targeting::Enemy,
            Delivery::Direct,
            vec![push(0.0, 5.0, DisplacementDirection::AlongImpact)],
        ),
        impact(
            Targeting::Enemy,
            Delivery::Direct,
            vec![push(5.0, 0.0, DisplacementDirection::AlongImpact)],
        ),
        impact(
            Targeting::Enemy,
            Delivery::Direct,
            vec![push(f32::INFINITY, 5.0, DisplacementDirection::AlongImpact)],
        ),
        AbilityDefinition {
            windup: f32::MAX,
            ..impact(Targeting::Enemy, Delivery::Direct, vec![stun(1)])
        },
        impact(
            Targeting::Enemy,
            Delivery::Area {
                radius: 1.0,
                delay: f32::MAX,
            },
            vec![stun(1)],
        ),
        AbilityDefinition {
            execution: AbilityExecution::Channel {
                duration: 1.0,
                interval: 0.1,
                pulse: ImpactDefinition {
                    delivery: Delivery::Direct,
                    effects: vec![stun(1)],
                },
            },
            ..impact(Targeting::SelfCast, Delivery::Direct, vec![])
        },
    ];
    for definition in invalid {
        assert!(catalog.register(AbilityId(1), definition).is_err());
        assert!(catalog.get(AbilityId(1)).is_none());
    }
    assert!(
        catalog
            .register(
                AbilityId(1),
                impact(Targeting::Enemy, Delivery::Direct, vec![stun(1)])
            )
            .is_ok()
    );
}
