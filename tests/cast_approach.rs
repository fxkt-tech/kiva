//! Unit-targeted casts use the same public command and observation seam as input.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

const SPELL: AbilityId = AbilityId(700);
const SELF: AbilityId = AbilityId(701);

struct Arena {
    app: App,
    events: MessageCursor<SimulationEvent>,
}
impl Arena {
    fn new(obstacles: Vec<Obstacle>) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(NavigationMap::new(20.0, 1.0, obstacles));
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                SPELL,
                AbilityDefinition {
                    name: "Targeted spell".into(),
                    targeting: Targeting::Enemy,
                    range: 3.0,
                    windup: 0.2,
                    recovery: 0.1,
                    cooldown: 3.0,
                    mana_cost: 40.0,
                    execution: AbilityExecution::Impact(ImpactDefinition {
                        delivery: Delivery::Direct,
                        effects: vec![Effect::Damage(DamageSpec {
                            kind: DamageType::True,
                            base: 50.0,
                            ad_ratio: 0.0,
                            ap_ratio: 0.0,
                        })],
                    }),
                },
            )
            .unwrap();
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                SELF,
                AbilityDefinition {
                    name: "Self ability".into(),
                    targeting: Targeting::SelfCast,
                    range: 0.0,
                    windup: 0.1,
                    recovery: 0.1,
                    cooldown: 2.0,
                    mana_cost: 10.0,
                    execution: AbilityExecution::Impact(ImpactDefinition {
                        delivery: Delivery::Area {
                            radius: 1.0,
                            delay: 0.0,
                        },
                        effects: vec![Effect::Damage(DamageSpec::physical(0.0))],
                    }),
                },
            )
            .unwrap();
        Self {
            app,
            events: MessageCursor::default(),
        }
    }
    fn spawn(&mut self, team: Team, position: Vec2) -> Entity {
        self.spawn_with(
            team,
            position,
            UnitStats {
                max_health: 1000.0,
                mana_regen: 0.0,
                base_attributes: BaseAttributes {
                    move_speed: 6.0,
                    ..default()
                },
                ..default()
            },
            None,
        )
    }
    fn spawn_with(
        &mut self,
        team: Team,
        position: Vec2,
        stats: UnitStats,
        respawn_delay: Option<f32>,
    ) -> Entity {
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Unit".into(),
                team,
                position,
                stats,
                abilities: [Some(SPELL), Some(SELF), None, None],
                respawn_delay,
            },
        )
        .unwrap()
    }
    fn extra_spell(&mut self, id: AbilityId, effects: Vec<Effect>) {
        let mut definition = self
            .app
            .world()
            .resource::<AbilityCatalog>()
            .get(SPELL)
            .unwrap()
            .clone();
        definition.range = 30.0;
        definition.windup = 0.0;
        definition.mana_cost = 0.0;
        definition.execution = AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects,
        });
        self.app
            .world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(id, definition)
            .unwrap();
    }
    fn spell_caster(&mut self, team: Team, position: Vec2, id: AbilityId) -> Entity {
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Effect source".into(),
                team,
                position,
                stats: UnitStats {
                    mana_regen: 0.0,
                    ..default()
                },
                abilities: [Some(id), None, None, None],
                respawn_delay: None,
            },
        )
        .unwrap()
    }
    fn equip(&mut self, unit: Entity, slot: usize) -> EquipmentInstanceId {
        if self
            .app
            .world()
            .resource::<EquipmentCatalog>()
            .get(EquipmentId(1))
            .is_none()
        {
            self.app
                .world_mut()
                .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
                    catalog
                        .register(
                            EquipmentId(1),
                            EquipmentDefinition {
                                name: "Targeted active".into(),
                                description: "test".into(),
                                flat: default(),
                                additive_percent: default(),
                                passives: vec![],
                                active: Some(SPELL),
                            },
                            world.resource::<AbilityCatalog>(),
                        )
                        .unwrap();
                });
        }
        equip_equipment(self.app.world_mut(), unit, slot, EquipmentId(1)).unwrap()
    }
    fn command(&mut self, unit: Entity, command: UnitCommand) {
        self.app
            .world_mut()
            .resource_mut::<CommandQueue>()
            .submit(unit, command);
    }
    fn cast(&mut self, unit: Entity, target: Entity) {
        self.command(
            unit,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Unit(target),
            },
        );
    }
    fn step(&mut self) {
        self.app.world_mut().run_schedule(FixedUpdate);
    }
    fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.step();
        }
    }
    fn position(&self, unit: Entity) -> Vec2 {
        self.app.world().get::<UnitPose>(unit).unwrap().position()
    }
    fn mana(&self, unit: Entity) -> f32 {
        self.app.world().get::<UnitState>(unit).unwrap().mana()
    }
    fn cooldown(&self, unit: Entity) -> f32 {
        self.app
            .world()
            .get::<AbilitySlots>(unit)
            .unwrap()
            .cooldown(0)
            .unwrap()
    }
    fn drain(&mut self) -> Vec<SimulationEvent> {
        self.events
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect()
    }
}

#[test]
fn out_of_range_enemy_cast_walks_then_commits_once_without_early_cost() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    assert_eq!(
        preview_cast(
            a.app.world(),
            caster,
            CastSource::AbilitySlot(0),
            AbilityTarget::Unit(target)
        ),
        Err(Rejection::OutOfRange)
    );
    a.cast(caster, target);
    a.steps(10);
    assert!(
        a.position(caster).x > 0.5,
        "out-of-range cast must approach its target"
    );
    assert_eq!(a.mana(caster), 300.0);
    assert_eq!(a.cooldown(caster), 0.0);
    assert!(
        !a.drain()
            .iter()
            .any(|e| matches!(e, SimulationEvent::CommandRejected { unit, .. } if *unit == caster))
    );
    let mut committed = 0;
    for _ in 0..180 {
        a.step();
        for event in a.drain() {
            if matches!(event, SimulationEvent::AbilityCommitted { unit, .. } if unit == caster) {
                committed += 1;
                assert!(a.position(caster).distance(a.position(target)) <= 3.001);
            }
        }
    }
    assert_eq!(committed, 1);
    assert_eq!(a.mana(caster), 260.0);
    assert_eq!(
        a.app.world().get::<UnitState>(target).unwrap().health(),
        950.0
    );
}

fn committed(events: &[SimulationEvent], caster: Entity) -> usize {
    events.iter().filter(|event| matches!(event, SimulationEvent::AbilityCommitted { unit, .. } if *unit == caster)).count()
}

#[test]
fn moving_enemy_is_followed_and_cast_once_at_its_current_position() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    a.cast(caster, target);
    a.command(target, UnitCommand::MoveTo(Vec2::new(8.0, 6.0)));
    let mut commits = 0;
    for _ in 0..200 {
        a.step();
        let events = a.drain();
        if committed(&events, caster) > 0 {
            assert!(a.position(caster).distance(a.position(target)) <= 3.001);
        }
        commits += committed(&events, caster);
    }
    assert_eq!(commits, 1);
    assert!(a.position(caster).y > 2.0);
    assert_eq!(a.mana(caster), 260.0);
}

#[test]
fn approach_uses_walkable_cast_region_including_when_enemy_center_is_unreachable() {
    for (obstacle, target_position, must_detour) in [
        (
            Obstacle {
                center: Vec2::ZERO,
                half_size: Vec2::new(1.0, 3.0),
            },
            Vec2::X * 5.0,
            true,
        ),
        (
            Obstacle {
                center: Vec2::ZERO,
                half_size: Vec2::new(0.3, 20.0),
            },
            Vec2::X * 2.0,
            false,
        ),
    ] {
        let mut a = Arena::new(vec![obstacle]);
        let caster = a.spawn(Team::Ally, Vec2::X * -8.0);
        let target = a.spawn(Team::Enemy, target_position);
        if !must_detour {
            assert_eq!(
                a.app.world().resource::<NavigationMap>().path(
                    a.position(caster),
                    target_position,
                    0.45
                ),
                Err(PathError::Unreachable)
            );
        }
        a.cast(caster, target);
        let mut detoured = false;
        let mut commits = 0;
        for _ in 0..300 {
            a.step();
            let position = a.position(caster);
            assert!(
                a.app
                    .world()
                    .resource::<NavigationMap>()
                    .walkable(position, 0.45)
            );
            detoured |= position.y.abs() > 3.4;
            commits += committed(&a.drain(), caster);
        }
        assert_eq!(commits, 1, "target at {target_position:?}");
        assert!(a.position(caster).distance(target_position) <= 3.001);
        assert!(!must_detour || detoured);
    }
}

#[test]
fn unreachable_cast_region_rejects_without_erasing_prior_movement() {
    let mut a = Arena::new(vec![Obstacle {
        center: Vec2::ZERO,
        half_size: Vec2::new(0.3, 20.0),
    }]);
    let caster = a.spawn(Team::Ally, Vec2::X * -8.0);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    a.command(caster, UnitCommand::MoveTo(Vec2::new(-12.0, 0.0)));
    a.step();
    a.cast(caster, target);
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::MoveTo(_))
    ));
    assert!(a.drain().iter().any(|event| matches!(event, SimulationEvent::CommandRejected { unit, reason: Rejection::NoPath } if *unit == caster)));
    a.steps(50);
    assert!(a.position(caster).x < -11.0);
    assert_eq!(a.mana(caster), 300.0);
}

#[test]
fn legitimate_move_attack_stop_or_new_skill_replaces_the_pending_cast() {
    for replacement in 0..5 {
        let mut a = Arena::new(vec![]);
        let caster = a.spawn(Team::Ally, Vec2::ZERO);
        let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
        let other = a.spawn(Team::Enemy, Vec2::Y * -2.0);
        a.cast(caster, target);
        a.steps(5);
        let command = match replacement {
            0 => UnitCommand::MoveTo(Vec2::new(-4.0, 4.0)),
            1 => UnitCommand::AttackTarget(other),
            2 => UnitCommand::AttackMove(Vec2::new(-4.0, 4.0)),
            3 => UnitCommand::Stop,
            _ => UnitCommand::CastAbility {
                slot: 1,
                target: AbilityTarget::SelfCast,
            },
        };
        a.command(caster, command);
        a.steps(180);
        let events = a.drain();
        assert!(!events.iter().any(|event| matches!(event, SimulationEvent::AbilityCommitted { unit, ability: SPELL, .. } if *unit == caster)), "replacement {replacement}");
        assert_eq!(a.mana(caster), if replacement == 4 { 290.0 } else { 300.0 });
        assert!(!matches!(
            a.app.world().get::<UnitOrder>(caster),
            Some(UnitOrder::CastTarget { .. })
        ));
    }
}

#[test]
fn invalid_new_commands_do_not_cancel_the_valid_pending_cast() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    a.cast(caster, target);
    a.steps(5);
    for command in [
        UnitCommand::MoveTo(Vec2::splat(f32::NAN)),
        UnitCommand::AttackTarget(caster),
        UnitCommand::CastAbility {
            slot: 99,
            target: AbilityTarget::Unit(target),
        },
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::SelfCast,
        },
    ] {
        a.command(caster, command);
    }
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::CastTarget { .. })
    ));
    a.steps(160);
    assert_eq!(committed(&a.drain(), caster), 1);
    assert_eq!(a.mana(caster), 260.0);
}

#[test]
fn equipment_active_approaches_and_commits_only_its_actual_instance() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    let instance = a.equip(caster, 0);
    let other = a.equip(caster, 1);
    a.command(
        caster,
        UnitCommand::UseEquipment {
            slot: 0,
            instance,
            target: AbilityTarget::Unit(target),
        },
    );
    a.steps(10);
    assert_eq!(a.mana(caster), 300.0);
    assert_eq!(a.cooldown(caster), 0.0);
    assert_eq!(
        a.app
            .world()
            .get::<EquipmentSlots>(caster)
            .unwrap()
            .item(0)
            .unwrap()
            .cooldown(),
        0.0
    );
    unequip_equipment(a.app.world_mut(), caster, 1, other).unwrap();
    a.steps(80);
    assert_eq!(committed(&a.drain(), caster), 1);
    assert_eq!(a.mana(caster), 260.0);
    assert_eq!(a.cooldown(caster), 0.0);
    assert!(
        a.app
            .world()
            .get::<EquipmentSlots>(caster)
            .unwrap()
            .item(0)
            .unwrap()
            .cooldown()
            > 2.0
    );
}

#[test]
fn unequipping_pending_source_cancels_it_and_stale_instance_cannot_replace_move() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    let instance = a.equip(caster, 0);
    a.command(
        caster,
        UnitCommand::UseEquipment {
            slot: 0,
            instance,
            target: AbilityTarget::Unit(target),
        },
    );
    a.steps(5);
    unequip_equipment(a.app.world_mut(), caster, 0, instance).unwrap();
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::Idle)
    ));
    let replacement = a.equip(caster, 0);
    assert_ne!(instance, replacement);
    a.command(caster, UnitCommand::MoveTo(Vec2::X * -3.0));
    a.command(
        caster,
        UnitCommand::UseEquipment {
            slot: 0,
            instance,
            target: AbilityTarget::Unit(target),
        },
    );
    a.steps(160);
    assert_eq!(committed(&a.drain(), caster), 0);
    assert!(a.position(caster).x < -2.9);
    assert_eq!(a.mana(caster), 300.0);
    assert_eq!(
        a.app
            .world()
            .get::<EquipmentSlots>(caster)
            .unwrap()
            .item(0)
            .unwrap()
            .cooldown(),
        0.0
    );
}

#[test]
fn loss_of_vision_cancels_tracking_instead_of_using_hidden_enemy_coordinates() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn_with(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            mana_regen: 0.0,
            vision_radius: 8.5,
            base_attributes: BaseAttributes {
                move_speed: 6.0,
                ..default()
            },
            ..default()
        },
        None,
    );
    let target = a.spawn_with(
        Team::Enemy,
        Vec2::X * 8.0,
        UnitStats {
            mana_regen: 0.0,
            base_attributes: BaseAttributes {
                move_speed: 12.0,
                ..default()
            },
            ..default()
        },
        None,
    );
    assert!(team_can_see(a.app.world(), Team::Ally, target));
    a.cast(caster, target);
    a.command(target, UnitCommand::MoveTo(Vec2::X * 18.0));
    a.steps(50);
    assert!(!team_can_see(a.app.world(), Team::Ally, target));
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::Idle)
    ));
    let stopped = a.position(caster);
    a.steps(50);
    assert_eq!(a.position(caster), stopped);
    assert_eq!(committed(&a.drain(), caster), 0);
    assert_eq!(a.mana(caster), 300.0);
}

#[test]
fn old_target_life_and_caster_death_never_replay_a_pending_cast() {
    for kill_caster in [false, true] {
        let mut a = Arena::new(vec![]);
        let lethal = AbilityId(710);
        a.extra_spell(
            lethal,
            vec![Effect::Damage(DamageSpec {
                kind: DamageType::True,
                base: 10000.0,
                ad_ratio: 0.0,
                ap_ratio: 0.0,
            })],
        );
        let caster = a.spawn_with(
            Team::Ally,
            Vec2::ZERO,
            UnitStats {
                mana_regen: 0.0,
                ..default()
            },
            Some(0.001),
        );
        let target = a.spawn_with(
            Team::Enemy,
            Vec2::X * 8.0,
            UnitStats {
                mana_regen: 0.0,
                ..default()
            },
            Some(0.001),
        );
        let victim = if kill_caster { caster } else { target };
        let old = a
            .app
            .world()
            .get::<UnitState>(victim)
            .unwrap()
            .life_ref(victim);
        let killer = a.spell_caster(
            if kill_caster { Team::Enemy } else { Team::Ally },
            Vec2::new(-3.0, 2.0),
            lethal,
        );
        a.cast(caster, target);
        a.steps(5);
        a.cast(killer, victim);
        a.steps(4);
        assert_ne!(
            old,
            a.app
                .world()
                .get::<UnitState>(victim)
                .unwrap()
                .life_ref(victim)
        );
        assert!(matches!(
            a.app.world().get::<UnitOrder>(caster),
            Some(UnitOrder::Idle)
        ));
        a.steps(160);
        assert_eq!(committed(&a.drain(), caster), 0);
        assert_eq!(a.mana(caster), 300.0);
    }
}

#[test]
fn pause_freezes_the_pending_route_and_stop_or_reset_removes_it() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    a.cast(caster, target);
    a.steps(5);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = true;
    let position = a.position(caster);
    let tick = a.app.world().resource::<SimulationTick>().0;
    a.command(caster, UnitCommand::Stop);
    a.steps(100);
    assert_eq!(a.position(caster), position);
    assert_eq!(a.app.world().resource::<SimulationTick>().0, tick);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.steps(100);
    assert_eq!(a.position(caster), position);
    assert_eq!(committed(&a.drain(), caster), 0);
    a.cast(caster, target);
    a.steps(5);
    reset_simulation(a.app.world_mut());
    assert!(a.app.world().get_entity(caster).is_err());
    assert!(a.app.world().get_entity(target).is_err());
    let next = a.spawn(Team::Ally, Vec2::ZERO);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.steps(100);
    assert_eq!(a.position(next), Vec2::ZERO);
    assert_eq!(committed(&a.drain(), next), 0);
}

#[test]
fn target_escaping_during_windup_rejects_once_without_resuming_the_old_purpose() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn_with(
        Team::Enemy,
        Vec2::X * 8.0,
        UnitStats {
            mana_regen: 0.0,
            base_attributes: BaseAttributes {
                move_speed: 20.0,
                ..default()
            },
            ..default()
        },
        None,
    );
    a.cast(caster, target);
    for _ in 0..100 {
        a.step();
        if matches!(
            a.app.world().get::<UnitAction>(caster),
            Some(UnitAction::Casting { .. })
        ) {
            break;
        }
    }
    assert!(matches!(
        a.app.world().get::<UnitAction>(caster),
        Some(UnitAction::Casting { .. })
    ));
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::Idle)
    ));
    let origin = a.position(caster);
    a.command(target, UnitCommand::MoveTo(Vec2::X * 16.0));
    a.steps(60);
    a.command(target, UnitCommand::MoveTo(origin + Vec2::X * 2.0));
    a.steps(100);
    assert_eq!(a.position(caster), origin);
    assert_eq!(committed(&a.drain(), caster), 0);
    assert_eq!(a.mana(caster), 300.0);
    assert_eq!(a.cooldown(caster), 0.0);
}

#[test]
fn root_pauses_approach_and_rejects_new_movement_without_erasing_pending_nonmobility_cast() {
    let mut a = Arena::new(vec![]);
    let root = AbilityId(710);
    a.extra_spell(
        root,
        vec![Effect::ApplyStatus(StatusSpec::Root {
            duration: StatusDuration::from_ticks(120).unwrap(),
        })],
    );
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    let controller = a.spell_caster(Team::Enemy, Vec2::new(0.0, -2.0), root);
    a.cast(caster, target);
    a.steps(5);
    a.cast(controller, caster);
    a.step();
    let frozen = a.position(caster);
    assert!(
        !a.app
            .world()
            .get::<ControlState>(caster)
            .unwrap()
            .can_move()
    );
    a.command(caster, UnitCommand::MoveTo(Vec2::Y * 4.0));
    a.step();
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::CastTarget { .. })
    ));
    assert!(a.drain().iter().any(|event| matches!(event, SimulationEvent::CommandRejected { unit, reason: Rejection::Controlled } if *unit == caster)));
    a.command(target, UnitCommand::MoveTo(frozen + Vec2::X * 2.0));
    let mut casts = 0;
    for _ in 0..100 {
        a.step();
        let n = committed(&a.drain(), caster);
        if n > 0 {
            assert!(
                !a.app
                    .world()
                    .get::<ControlState>(caster)
                    .unwrap()
                    .can_move()
            );
        }
        casts += n;
    }
    assert_eq!(a.position(caster), frozen);
    assert_eq!(casts, 1);
    assert_eq!(a.mana(caster), 260.0);
}

#[test]
fn stun_cancels_pending_cast_and_does_not_release_after_control_ends() {
    let mut a = Arena::new(vec![]);
    let stun = AbilityId(710);
    a.extra_spell(
        stun,
        vec![Effect::ApplyStatus(StatusSpec::Stun {
            duration: StatusDuration::from_ticks(12).unwrap(),
        })],
    );
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 8.0);
    let controller = a.spell_caster(Team::Enemy, Vec2::Y * -2.0, stun);
    a.cast(caster, target);
    a.steps(5);
    a.cast(controller, caster);
    a.steps(2);
    assert!(matches!(
        a.app.world().get::<UnitOrder>(caster),
        Some(UnitOrder::Idle)
    ));
    let stopped = a.position(caster);
    a.command(target, UnitCommand::MoveTo(stopped + Vec2::X * 2.0));
    a.steps(160);
    assert_eq!(a.position(caster), stopped);
    assert_eq!(committed(&a.drain(), caster), 0);
    assert_eq!(a.mana(caster), 300.0);
}

#[test]
fn another_valid_targeted_request_replaces_the_previous_target() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let first = a.spawn(Team::Enemy, Vec2::X * 8.0);
    let second = a.spawn(Team::Enemy, Vec2::Y * 8.0);
    a.cast(caster, first);
    a.steps(5);
    a.cast(caster, second);
    a.steps(160);
    assert_eq!(committed(&a.drain(), caster), 1);
    assert_eq!(
        a.app.world().get::<UnitState>(first).unwrap().health(),
        1000.0
    );
    assert_eq!(
        a.app.world().get::<UnitState>(second).unwrap().health(),
        950.0
    );
    assert!(a.position(caster).y > 4.5);
    assert_eq!(a.mana(caster), 260.0);
}

#[test]
fn in_range_cast_keeps_existing_persistent_movement_continuation() {
    let mut a = Arena::new(vec![]);
    let caster = a.spawn(Team::Ally, Vec2::ZERO);
    let target = a.spawn(Team::Enemy, Vec2::X * 2.0);
    a.command(caster, UnitCommand::MoveTo(Vec2::new(-5.0, 4.0)));
    a.step();
    a.cast(caster, target);
    a.steps(160);
    assert_eq!(committed(&a.drain(), caster), 1);
    assert!(a.position(caster).distance(Vec2::new(-5.0, 4.0)) < 0.1);
    assert_eq!(a.mana(caster), 260.0);
}

#[test]
fn direction_and_maximum_range_point_casts_do_not_create_approach_purposes() {
    for directional in [true, false] {
        let mut a = Arena::new(vec![]);
        let id = AbilityId(710);
        a.app
            .world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                id,
                AbilityDefinition {
                    name: "Non-targeted".into(),
                    targeting: if directional {
                        Targeting::Direction
                    } else {
                        Targeting::Point
                    },
                    range: 3.0,
                    windup: 0.2,
                    recovery: 0.1,
                    cooldown: 1.0,
                    mana_cost: 40.0,
                    execution: if directional {
                        AbilityExecution::Impact(ImpactDefinition {
                            delivery: Delivery::Projectile {
                                speed: 20.0,
                                radius: 0.1,
                            },
                            effects: vec![Effect::Damage(DamageSpec::physical(10.0))],
                        })
                    } else {
                        AbilityExecution::Dash { speed: 20.0 }
                    },
                },
            )
            .unwrap();
        let caster = a.spell_caster(Team::Ally, Vec2::ZERO, id);
        a.command(
            caster,
            UnitCommand::CastAbility {
                slot: 0,
                target: if directional {
                    AbilityTarget::Direction(Vec2::X * 100.0)
                } else {
                    AbilityTarget::Point(Vec2::X * 100.0)
                },
            },
        );
        a.steps(5);
        assert_eq!(a.position(caster), Vec2::ZERO);
        assert!(matches!(
            a.app.world().get::<UnitAction>(caster),
            Some(UnitAction::Casting { .. })
        ));
        assert!(!matches!(
            a.app.world().get::<UnitOrder>(caster),
            Some(UnitOrder::CastTarget { .. })
        ));
        a.steps(40);
        assert_eq!(committed(&a.drain(), caster), 1);
        assert!((a.position(caster).x - if directional { 0.0 } else { 3.0 }).abs() < 0.01);
    }
}

#[test]
fn zero_windup_queued_cast_commits_on_boundary_entry_before_target_moves_again() {
    let mut a = Arena::new(vec![]);
    let instant = AbilityId(710);
    let mut definition = a
        .app
        .world()
        .resource::<AbilityCatalog>()
        .get(SPELL)
        .unwrap()
        .clone();
    definition.windup = 0.0;
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(instant, definition)
        .unwrap();
    let caster = a.spell_caster(Team::Ally, Vec2::ZERO, instant);
    let target = a.spawn_with(
        Team::Enemy,
        Vec2::X * 3.05,
        UnitStats {
            mana_regen: 0.0,
            base_attributes: BaseAttributes {
                move_speed: 2.5,
                ..default()
            },
            ..default()
        },
        None,
    );
    a.cast(caster, target);
    a.step();
    assert!(a.position(caster).x > 0.0);
    assert!(a.position(caster).distance(a.position(target)) <= 3.001);
    assert_eq!(a.mana(caster), 300.0);
    assert_eq!(committed(&a.drain(), caster), 0);
    // The target starts fleeing on the tick that should begin and commit the
    // zero-windup cast. An artificial extra windup tick lets it escape first.
    a.command(target, UnitCommand::MoveTo(Vec2::X * 18.0));
    a.step();
    assert_eq!(
        committed(&a.drain(), caster),
        1,
        "zero-windup purpose must commit at boundary entry without another action-clock tick"
    );
    assert_eq!(a.mana(caster), 260.0);
    a.steps(40);
    assert_eq!(
        committed(&a.drain(), caster),
        0,
        "the one-shot cast must not repeat"
    );
}

#[test]
fn faster_caster_can_close_range_on_a_continuously_fleeing_enemy() {
    let mut a = Arena::new(vec![]);
    let instant = AbilityId(710);
    let mut definition = a
        .app
        .world()
        .resource::<AbilityCatalog>()
        .get(SPELL)
        .unwrap()
        .clone();
    definition.windup = 0.0;
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(instant, definition)
        .unwrap();
    let caster = a.spell_caster(Team::Ally, Vec2::ZERO, instant);
    let target = a.spawn_with(
        Team::Enemy,
        Vec2::X * 3.3,
        UnitStats {
            mana_regen: 0.0,
            base_attributes: BaseAttributes {
                move_speed: 2.5,
                ..default()
            },
            ..default()
        },
        None,
    );
    a.cast(caster, target);
    a.command(target, UnitCommand::MoveTo(Vec2::X * 18.0));
    let mut commits = 0;
    for _ in 0..100 {
        a.step();
        commits += committed(&a.drain(), caster);
    }
    assert!(
        a.position(target).x < 10.0,
        "target never reached its destination or map boundary"
    );
    assert_eq!(
        commits, 1,
        "faster caster must enter range without waiting for the enemy to stop"
    );
    assert_eq!(a.mana(caster), 260.0);
}
