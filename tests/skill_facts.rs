//! Presentation facts observe real commands and immutable combat deliveries.
//! These fixtures never write unit runtime state or install rendering plugins.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::{gameplay::prepare_world, simulation::*};
use std::collections::BTreeSet;

struct Arena {
    app: App,
    cursor: MessageCursor<SimulationEvent>,
    next_ability: u16,
}
impl Arena {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        prepare_world(app.world_mut());
        app.insert_resource(NavigationMap::new(50.0, 1.0, vec![]));
        Self {
            app,
            cursor: default(),
            next_ability: 1000,
        }
    }
    fn register(
        &mut self,
        targeting: Targeting,
        range: f32,
        execution: AbilityExecution,
    ) -> AbilityId {
        let id = AbilityId(self.next_ability);
        self.next_ability += 1;
        self.app
            .world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                id,
                AbilityDefinition {
                    name: format!("fact fixture {}", id.0),
                    targeting,
                    range,
                    windup: 0.0,
                    recovery: 0.0,
                    cooldown: 0.0,
                    mana_cost: 0.0,
                    execution,
                },
            )
            .unwrap();
        id
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
        for (slot, id) in slots.iter_mut().zip(abilities) {
            *slot = Some(*id);
        }
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Fact actor".into(),
                team,
                position,
                abilities: slots,
                stats,
                respawn_delay,
            },
        )
        .unwrap()
    }
    fn unit(&mut self, team: Team, position: Vec2, abilities: &[AbilityId]) -> Entity {
        self.spawn(team, position, abilities, stats(), None)
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
    fn ticks(&mut self, count: usize) {
        for _ in 0..count {
            self.app.world_mut().run_schedule(FixedUpdate);
        }
    }
    fn events(&mut self) -> Vec<SimulationEvent> {
        self.cursor
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect()
    }
    fn life(&self, unit: Entity) -> UnitLifeRef {
        self.app
            .world()
            .get::<UnitState>(unit)
            .unwrap()
            .life_ref(unit)
    }
    fn projectiles(&mut self) -> Vec<Entity> {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<Projectile>>()
            .iter(world)
            .collect()
    }
}
fn stats() -> UnitStats {
    UnitStats {
        kind: UnitKind::Hero,
        max_health: 10_000.0,
        max_mana: 10_000.0,
        mana_regen: 0.0,
        vision_radius: 80.0,
        radius: 0.4,
        attack_delivery: BasicAttackDelivery::Direct,
        attack_windup_fraction: 0.0,
        base_attributes: BaseAttributes {
            attack_damage: 10.0,
            attack_speed: 5.0,
            attack_range: 20.0,
            move_speed: 0.0,
            ..default()
        },
        ..default()
    }
}
fn damage(amount: f32) -> Effect {
    Effect::Damage(DamageSpec {
        kind: DamageType::True,
        base: amount,
        ad_ratio: 0.0,
        ap_ratio: 0.0,
    })
}
fn impact(delivery: Delivery, effects: Vec<Effect>) -> AbilityExecution {
    AbilityExecution::Impact(ImpactDefinition { delivery, effects })
}
fn shield(amount: f32, slow: Option<StatusSpec>) -> AbilityExecution {
    AbilityExecution::Shield(ShieldSpec {
        base: amount,
        bonus_ad_ratio: 0.0,
        max_health_per_enemy_hero: 0.0,
        radius: 3.0,
        duration: 2.0,
        slow,
    })
}
fn contact_count(events: &[SimulationEvent], origin: DamageOrigin, target: UnitLifeRef) -> usize {
    events
        .iter()
        .filter(|event| {
            matches!(event, SimulationEvent::ImpactResolved {
        origin: actual, kind: ImpactKind::Contact { target: hit }, ..
    } if *actual == origin && *hit == target)
        })
        .count()
}
fn ends(events: &[SimulationEvent]) -> Vec<(Entity, ProjectileEndReason, Vec2)> {
    events
        .iter()
        .filter_map(|event| match event {
            SimulationEvent::ImpactResolved {
                position,
                kind:
                    ImpactKind::ProjectileEnded {
                        carrier_entity,
                        reason,
                    },
                ..
            } => Some((*carrier_entity, *reason, *position)),
            _ => None,
        })
        .collect()
}

#[test]
fn committed_direction_and_clamped_point_are_the_actual_resolved_targets() {
    let mut a = Arena::new();
    let actor = a.unit(Team::Ally, Vec2::ZERO, &[AbilityId(11), AbilityId(13)]);
    let target = a.unit(Team::Enemy, Vec2::new(4.0, 0.0), &[]);
    let life = a.life(actor);
    assert_eq!(
        a.app.world().get::<UnitPose>(actor).unwrap().facing(),
        Vec2::NEG_Y
    );
    a.cast(actor, 0, AbilityTarget::Direction(Vec2::new(f32::MAX, 0.0)));
    a.ticks(30);
    let events = a.events();
    assert!(events.iter().any(|event| matches!(event, SimulationEvent::AbilityCommitted {
        unit, ability: AbilityId(11), position, source_life, cast_source: CastSource::AbilitySlot(0),
        origin, target: AbilityTarget::Direction(direction), target_life: None, recast: false,
    } if *unit == actor && *source_life == life && *origin == Vec2::ZERO && *position == Vec2::ZERO && *direction == Vec2::X)));
    assert_eq!(
        contact_count(
            &events,
            DamageOrigin::Ability(AbilityId(11)),
            a.life(target)
        ),
        1
    );
    assert!(a.app.world().get::<UnitState>(target).unwrap().health() < 10_000.0);
    a.cast(
        actor,
        1,
        AbilityTarget::Point(Vec2::new(f32::MAX, f32::MAX)),
    );
    a.ticks(2);
    let expected = Vec2::splat(8.6 / 2.0_f32.sqrt());
    assert!(a.events().iter().any(|event| matches!(event, SimulationEvent::AbilityCommitted {
        ability: AbilityId(13), position, target: AbilityTarget::Point(point), origin, ..
    } if position.distance(expected) < 1e-5 && point.distance(expected) < 1e-5 && *origin == Vec2::ZERO)));
}

#[test]
fn cancelled_and_rejected_casts_publish_no_commit_or_impact_facts() {
    let mut a = Arena::new();
    let actor = a.unit(Team::Ally, Vec2::ZERO, &[AbilityId(11)]);
    a.cast(actor, 0, AbilityTarget::Direction(Vec2::X));
    a.ticks(1);
    a.command(actor, UnitCommand::Stop);
    a.ticks(30);
    a.cast(actor, 0, AbilityTarget::Unit(actor));
    a.ticks(1);
    let events = a.events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SimulationEvent::CommandRejected { .. }))
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        SimulationEvent::AbilityCommitted { .. } | SimulationEvent::ImpactResolved { .. }
    )));
}

#[test]
fn shield_absorption_and_pure_status_contact_are_once_per_payload_at_the_recipient() {
    let mut a = Arena::new();
    let barrier = a.register(Targeting::SelfCast, 0.0, shield(1000.0, None));
    let status = a.register(
        Targeting::Enemy,
        10.0,
        impact(
            Delivery::Direct,
            vec![
                Effect::ApplyStatus(StatusSpec::Root {
                    duration: StatusDuration::from_ticks(30).unwrap(),
                }),
                Effect::ApplyStatus(StatusSpec::Slow {
                    duration: StatusDuration::from_ticks(30).unwrap(),
                    fraction: 0.25,
                }),
            ],
        ),
    );
    let defender = a.unit(Team::Enemy, Vec2::new(2.0, 0.0), &[barrier]);
    let attacker = a.unit(Team::Ally, Vec2::ZERO, &[status]);
    a.cast(defender, 0, AbilityTarget::SelfCast);
    a.command(attacker, UnitCommand::AttackTarget(defender));
    a.ticks(2);
    let views: Vec<_> = a
        .app
        .world()
        .get::<UnitMechanics>(defender)
        .unwrap()
        .shields()
        .collect();
    assert_eq!(
        views,
        vec![ShieldView {
            amount: 990.0,
            origin: DamageOrigin::Ability(barrier),
            expires_at: 121
        }]
    );
    a.command(attacker, UnitCommand::Stop);
    a.cast(attacker, 0, AbilityTarget::Unit(defender));
    a.ticks(1);
    let events = a.events();
    let target = a.life(defender);
    assert_eq!(contact_count(&events, DamageOrigin::BasicAttack, target), 1);
    assert_eq!(
        contact_count(&events, DamageOrigin::Ability(status), target),
        1
    );
    assert_eq!(
        contact_count(&events, DamageOrigin::Ability(barrier), a.life(attacker)),
        0,
        "an empty self-shield payload must not invent a contact"
    );
    assert!(!events.iter().any(
        |event| matches!(event, SimulationEvent::Damaged { target, .. } if *target == defender)
    ));
    assert!(events.iter().filter(|event| matches!(event, SimulationEvent::ImpactResolved { kind: ImpactKind::Contact { .. }, .. }))
        .all(|event| matches!(event, SimulationEvent::ImpactResolved { position, .. } if *position == Vec2::new(2.0, 0.0))));
    assert_eq!(
        a.app.world().get::<UnitStatuses>(defender).unwrap().len(),
        2
    );
}

#[test]
fn equipment_commit_and_persistent_shield_keep_the_actual_instance_origin() {
    let mut a = Arena::new();
    let spell = a.register(Targeting::SelfCast, 0.0, shield(100.0, None));
    let item = EquipmentId(60_000);
    a.app
        .world_mut()
        .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
            catalog.register(
                item,
                EquipmentDefinition {
                    name: "Fact barrier".into(),
                    description: String::new(),
                    flat: default(),
                    additive_percent: default(),
                    passives: vec![],
                    active: Some(spell),
                },
                world.resource::<AbilityCatalog>(),
            )
        })
        .unwrap();
    let actor = a.unit(Team::Ally, Vec2::new(1.0, 2.0), &[]);
    let life = a.life(actor);
    let instance = equip_equipment(a.app.world_mut(), actor, 2, item).unwrap();
    a.command(
        actor,
        UnitCommand::UseEquipment {
            slot: 2,
            instance,
            target: AbilityTarget::SelfCast,
        },
    );
    a.ticks(1);
    let events = a.events();
    assert!(events.iter().any(|event| matches!(event, SimulationEvent::AbilityCommitted {
        unit, source_life, ability, cast_source: CastSource::EquipmentSlot { slot: 2, instance: actual },
        origin, target: AbilityTarget::SelfCast, target_life: None, recast: false, ..
    } if *unit == actor && *source_life == life && *ability == spell && *actual == instance && *origin == Vec2::new(1.0, 2.0))));
    unequip_equipment(a.app.world_mut(), actor, 2, instance).unwrap();
    let view = a
        .app
        .world()
        .get::<UnitMechanics>(actor)
        .unwrap()
        .shields()
        .next()
        .unwrap();
    assert_eq!(
        view.origin,
        DamageOrigin::EquipmentActive {
            equipment: item,
            instance,
            ability: spell
        }
    );
    assert_eq!(view.amount, 100.0);
    assert_eq!(view.expires_at, 121);
    a.ticks(120);
    assert!(
        a.app
            .world()
            .get::<UnitMechanics>(actor)
            .unwrap()
            .shields()
            .next()
            .is_none()
    );
}

#[test]
fn stacked_shield_views_preserve_each_source_and_remove_only_the_absorbed_or_expired_instance() {
    let mut a = Arena::new();
    let first = a.register(Targeting::SelfCast, 0.0, shield(100.0, None));
    let second = a.register(
        Targeting::SelfCast,
        0.0,
        AbilityExecution::Shield(ShieldSpec {
            base: 200.0,
            bonus_ad_ratio: 0.0,
            max_health_per_enemy_hero: 0.0,
            radius: 3.0,
            duration: 0.1,
            slow: None,
        }),
    );
    let defender = a.unit(Team::Enemy, Vec2::X, &[first, second]);
    let attacker = a.spawn(
        Team::Ally,
        Vec2::ZERO,
        &[],
        UnitStats {
            base_attributes: BaseAttributes {
                attack_damage: 150.0,
                ..stats().base_attributes
            },
            ..stats()
        },
        None,
    );
    a.cast(defender, 0, AbilityTarget::SelfCast);
    a.ticks(1);
    a.cast(defender, 1, AbilityTarget::SelfCast);
    a.ticks(1);
    let views: Vec<_> = a
        .app
        .world()
        .get::<UnitMechanics>(defender)
        .unwrap()
        .shields()
        .collect();
    assert_eq!(
        views.iter().map(|view| view.origin).collect::<Vec<_>>(),
        vec![DamageOrigin::Ability(first), DamageOrigin::Ability(second)]
    );
    a.command(attacker, UnitCommand::AttackTarget(defender));
    a.ticks(2);
    a.command(attacker, UnitCommand::Stop);
    assert_eq!(
        a.app
            .world()
            .get::<UnitMechanics>(defender)
            .unwrap()
            .shields()
            .collect::<Vec<_>>(),
        vec![ShieldView {
            amount: 150.0,
            origin: DamageOrigin::Ability(second),
            expires_at: 8
        }]
    );
    a.ticks(4);
    assert!(
        a.app
            .world()
            .get::<UnitMechanics>(defender)
            .unwrap()
            .shields()
            .next()
            .is_none()
    );
    assert_eq!(
        a.app.world().get::<UnitState>(defender).unwrap().health(),
        10_000.0
    );
}

#[test]
fn delayed_area_publishes_once_on_real_activation_even_empty_and_after_source_despawn() {
    let mut a = Arena::new();
    let spell = a.register(
        Targeting::Point,
        20.0,
        impact(
            Delivery::Area {
                radius: 2.0,
                delay: 0.1,
            },
            vec![damage(10.0)],
        ),
    );
    let source = a.unit(Team::Enemy, Vec2::ZERO, &[spell]);
    let life = a.life(source);
    a.cast(source, 0, AbilityTarget::Point(Vec2::new(5.0, 0.0)));
    a.ticks(1);
    let (entity, group) = {
        let world = a.app.world_mut();
        let (entity, area) = world
            .query::<(Entity, &AreaEffect)>()
            .single(world)
            .unwrap();
        assert_eq!(area.source_life(), life);
        assert_eq!(area.origin(), DamageOrigin::Ability(spell));
        assert_eq!(area.empower_origin(), None);
        (entity, area.carrier())
    };
    a.app.world_mut().despawn(source);
    a.ticks(4);
    assert!(
        !a.events()
            .iter()
            .any(|event| matches!(event, SimulationEvent::ImpactResolved { .. }))
    );
    a.ticks(5);
    let events = a.events();
    assert_eq!(events.iter().filter(|event| matches!(event, SimulationEvent::ImpactResolved {
        source_life, team: Team::Enemy, origin: DamageOrigin::Ability(id), carrier,
        position, kind: ImpactKind::AreaActivated { carrier_entity: Some(actual), radius: 2.0 }, ..
    } if *source_life == life && *id == spell && *carrier == group && *actual == entity && *position == Vec2::new(5.0, 0.0))).count(), 1);
    assert!(!events.iter().any(|event| matches!(
        event,
        SimulationEvent::ImpactResolved {
            kind: ImpactKind::Contact { .. },
            ..
        }
    )));
}

#[test]
fn instant_area_and_empty_leap_activate_only_at_the_real_execution_points() {
    let mut a = Arena::new();
    let barrier = a.register(Targeting::SelfCast, 0.0, shield(100.0, None));
    let leap = a.register(
        Targeting::Enemy,
        10.0,
        AbilityExecution::Leap(LeapSpec {
            duration: 0.1,
            radius: 2.0,
            effects: vec![damage(1.0)],
            terrain: None,
            heroes_only: true,
        }),
    );
    let actor = a.unit(Team::Ally, Vec2::ZERO, &[barrier, leap]);
    let target = a.unit(Team::Enemy, Vec2::new(4.0, 0.0), &[]);
    a.cast(actor, 0, AbilityTarget::SelfCast);
    a.ticks(1);
    assert_eq!(a.events().iter().filter(|event| matches!(event, SimulationEvent::ImpactResolved {
        origin: DamageOrigin::Ability(id), kind: ImpactKind::AreaActivated { carrier_entity: None, radius: 3.0 }, ..
    } if *id == barrier)).count(), 1);
    a.cast(actor, 1, AbilityTarget::Unit(target));
    a.ticks(1);
    assert!(
        !a.events()
            .iter()
            .any(|event| matches!(event, SimulationEvent::ImpactResolved { .. }))
    );
    a.app.world_mut().despawn(target);
    a.ticks(8);
    let events = a.events();
    assert_eq!(events.iter().filter(|event| matches!(event, SimulationEvent::ImpactResolved {
        origin: DamageOrigin::Ability(id), position, kind: ImpactKind::AreaActivated { carrier_entity: None, radius: 2.0 }, ..
    } if *id == leap && position.distance(Vec2::new(4.0, 0.0)) < 1e-5)).count(), 1);
    assert!(!events.iter().any(|event| matches!(
        event,
        SimulationEvent::ImpactResolved {
            kind: ImpactKind::Contact { .. },
            ..
        }
    )));
}

#[test]
fn projectile_hit_reports_the_swept_contact_point_not_the_step_endpoint() {
    let mut a = Arena::new();
    let spell = a.register(
        Targeting::Direction,
        6.0,
        impact(
            Delivery::Projectile {
                speed: 120.0,
                radius: 0.1,
            },
            vec![damage(10.0)],
        ),
    );
    let source = a.unit(Team::Ally, Vec2::ZERO, &[spell]);
    let target = a.unit(Team::Enemy, Vec2::new(1.5, 0.0), &[]);
    a.cast(source, 0, AbilityTarget::Direction(Vec2::X));
    a.ticks(1);
    let events = a.events();
    let ended = ends(&events);
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0].1, ProjectileEndReason::Hit(a.life(target)));
    assert!(ended[0].2.distance(Vec2::X) < 1e-5);
    assert!(a.app.world().get::<Projectile>(ended[0].0).is_none());
    assert_eq!(
        contact_count(&events, DamageOrigin::Ability(spell), a.life(target)),
        1
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SimulationEvent::ImpactResolved {
        kind: ImpactKind::Contact { .. }, position, direction, ..
    } if *position == Vec2::new(1.5, 0.0) && *direction == Vec2::X))
    );
}

#[test]
fn projectiles_report_expiry_static_block_and_map_boundary_separately_from_hits() {
    for mode in 0..3 {
        let mut a = Arena::new();
        if mode == 1 {
            a.app.insert_resource(NavigationMap::new(
                50.0,
                1.0,
                vec![Obstacle {
                    center: Vec2::new(3.0, 0.0),
                    half_size: Vec2::splat(0.5),
                }],
            ));
        } else if mode == 2 {
            a.app.insert_resource(NavigationMap::new(3.0, 1.0, vec![]));
        }
        let range = if mode == 0 { 3.0 } else { 10.0 };
        let spell = a.register(
            Targeting::Direction,
            range,
            impact(
                Delivery::Projectile {
                    speed: 120.0,
                    radius: 0.1,
                },
                vec![damage(10.0)],
            ),
        );
        let source = a.unit(Team::Ally, Vec2::ZERO, &[spell]);
        a.cast(source, 0, AbilityTarget::Direction(Vec2::X));
        a.ticks(8);
        let events = a.events();
        let ended = ends(&events);
        assert_eq!(ended.len(), 1, "mode {mode}");
        if mode == 0 {
            assert_eq!(ended[0].1, ProjectileEndReason::Expired);
            assert!(ended[0].2.distance(Vec2::new(3.0, 0.0)) < 1e-5);
        } else {
            assert_eq!(ended[0].1, ProjectileEndReason::Blocked);
            let expected = if mode == 1 { 2.4 } else { 2.9 };
            assert!((ended[0].2.x - expected).abs() < 0.02);
        }
        assert!(!events.iter().any(|event| matches!(
            event,
            SimulationEvent::ImpactResolved {
                kind: ImpactKind::Contact { .. },
                ..
            }
        )));
    }
}

#[test]
fn homing_old_target_life_ends_invalid_instead_of_contacting_the_respawn() {
    let mut a = Arena::new();
    let bolt = a.register(
        Targeting::Enemy,
        20.0,
        impact(
            Delivery::Projectile {
                speed: 1.0,
                radius: 0.1,
            },
            vec![damage(10.0)],
        ),
    );
    let kill = a.register(
        Targeting::Enemy,
        20.0,
        impact(Delivery::Direct, vec![damage(20_000.0)]),
    );
    let caster = a.unit(Team::Ally, Vec2::ZERO, &[bolt, kill]);
    let target = a.spawn(
        Team::Enemy,
        Vec2::new(5.0, 0.0),
        &[],
        stats(),
        Some(TICK_SECONDS),
    );
    let old = a.life(target);
    a.cast(caster, 0, AbilityTarget::Unit(target));
    a.ticks(1);
    let projectile = a.projectiles()[0];
    a.cast(caster, 1, AbilityTarget::Unit(target));
    a.ticks(2);
    assert_ne!(a.life(target), old);
    let events = a.events();
    assert_eq!(
        ends(&events)
            .iter()
            .filter(|(entity, reason, _)| *entity == projectile
                && *reason == ProjectileEndReason::InvalidTarget)
            .count(),
        1
    );
    assert_eq!(
        contact_count(&events, DamageOrigin::Ability(bolt), a.life(target)),
        0
    );
    assert_eq!(contact_count(&events, DamageOrigin::Ability(bolt), old), 0);
}

#[test]
fn deleted_homing_target_ends_at_the_last_position_without_another_movement_step() {
    let mut a = Arena::new();
    let bolt = a.register(
        Targeting::Enemy,
        20.0,
        impact(
            Delivery::Projectile {
                speed: 1.0,
                radius: 0.1,
            },
            vec![damage(10.0)],
        ),
    );
    let source = a.unit(Team::Ally, Vec2::ZERO, &[bolt]);
    let target = a.unit(Team::Enemy, Vec2::new(5.0, 0.0), &[]);
    a.cast(source, 0, AbilityTarget::Unit(target));
    a.ticks(1);
    let entity = a.projectiles()[0];
    let position = a.app.world().get::<Projectile>(entity).unwrap().position();
    a.app.world_mut().despawn(target);
    a.ticks(3);
    let events = a.events();
    assert_eq!(
        ends(&events),
        vec![(entity, ProjectileEndReason::InvalidTarget, position)]
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        SimulationEvent::ImpactResolved {
            kind: ImpactKind::Contact { .. },
            ..
        }
    )));
}

#[test]
fn seven_volley_entities_share_one_carrier_but_end_individually_and_contact_once() {
    let mut a = Arena::new();
    let caster = a.unit(Team::Ally, Vec2::ZERO, &[AbilityId(22)]);
    let target = a.spawn(
        Team::Enemy,
        Vec2::new(3.0, 0.0),
        &[],
        UnitStats {
            radius: 1.0,
            ..stats()
        },
        None,
    );
    a.cast(caster, 0, AbilityTarget::Direction(Vec2::X));
    for _ in 0..30 {
        a.ticks(1);
        if a.projectiles().len() == 7 {
            break;
        }
    }
    let entities: BTreeSet<_> = a.projectiles().into_iter().collect();
    assert_eq!(entities.len(), 7);
    let groups: BTreeSet<_> = entities
        .iter()
        .map(|entity| {
            let p = a.app.world().get::<Projectile>(*entity).unwrap();
            assert_eq!(p.source_life(), a.life(caster));
            assert_eq!(p.origin(), DamageOrigin::Ability(AbilityId(22)));
            assert_eq!(p.empower_origin(), None);
            p.carrier()
        })
        .collect();
    assert_eq!(groups.len(), 1);
    a.ticks(50);
    let events = a.events();
    let ended = ends(&events);
    assert_eq!(ended.len(), 7);
    assert_eq!(
        ended.iter().map(|end| end.0).collect::<BTreeSet<_>>(),
        entities
    );
    assert!(
        ended
            .iter()
            .all(|(_, reason, _)| *reason == ProjectileEndReason::Hit(a.life(target)))
    );
    assert!(events.iter().filter(|event| matches!(event, SimulationEvent::ImpactResolved { .. })).all(|event| {
        matches!(event, SimulationEvent::ImpactResolved { carrier, .. } if groups.contains(carrier))
    }));
    assert_eq!(
        contact_count(
            &events,
            DamageOrigin::Ability(AbilityId(22)),
            a.life(target)
        ),
        1
    );
}

#[test]
fn committed_projectile_keeps_source_life_team_and_origin_after_despawn_or_respawn() {
    for respawn in [false, true] {
        let mut a = Arena::new();
        let bolt = a.register(
            Targeting::Enemy,
            20.0,
            impact(
                Delivery::Projectile {
                    speed: 10.0,
                    radius: 0.1,
                },
                vec![damage(10.0)],
            ),
        );
        let kill = a.register(
            Targeting::Enemy,
            20.0,
            impact(Delivery::Direct, vec![damage(20_000.0)]),
        );
        let source = a.spawn(
            Team::Enemy,
            Vec2::ZERO,
            &[bolt],
            stats(),
            Some(TICK_SECONDS),
        );
        let victim = a.unit(Team::Ally, Vec2::new(5.0, 0.0), &[kill]);
        let old_source = a.life(source);
        a.cast(source, 0, AbilityTarget::Unit(victim));
        a.ticks(1);
        if respawn {
            a.cast(victim, 0, AbilityTarget::Unit(source));
            a.ticks(2);
            assert_ne!(a.life(source), old_source);
        } else {
            a.app.world_mut().despawn(source);
        }
        a.ticks(40);
        let events = a.events();
        assert!(
            events
                .iter()
                .any(|event| matches!(event, SimulationEvent::ImpactResolved {
            source_life, team: Team::Enemy, origin: DamageOrigin::Ability(id),
            kind: ImpactKind::Contact { target }, ..
        } if *source_life == old_source && *id == bolt && *target == a.life(victim)))
        );
        assert!(
            ends(&events)
                .iter()
                .any(|(_, reason, _)| *reason == ProjectileEndReason::Hit(a.life(victim)))
        );
    }
}

#[test]
fn objects_keep_actual_skill_origins_through_scout_reveal_standard_and_terrain() {
    let mut a = Arena::new();
    let source = a.unit(
        Team::Ally,
        Vec2::ZERO,
        &[AbilityId(23), AbilityId(24), AbilityId(13), AbilityId(14)],
    );
    a.cast(source, 0, AbilityTarget::Point(Vec2::new(2.0, 0.0)));
    a.ticks(15);
    let scout = {
        let world = a.app.world_mut();
        let (entity, object) = world
            .query::<(Entity, &BattlefieldObject)>()
            .single(world)
            .unwrap();
        assert_eq!(object.kind(), ObjectKind::Scout);
        assert_eq!(object.origin(), DamageOrigin::Ability(AbilityId(23)));
        entity
    };
    a.ticks(12);
    let object = a.app.world().get::<BattlefieldObject>(scout).unwrap();
    assert_eq!(object.kind(), ObjectKind::Reveal);
    assert_eq!(object.origin(), DamageOrigin::Ability(AbilityId(23)));
    let target = a.unit(Team::Enemy, Vec2::new(0.0, 4.0), &[]);
    a.cast(source, 1, AbilityTarget::Direction(Vec2::Y));
    a.ticks(45);
    {
        let world = a.app.world_mut();
        let reveals: Vec<_> = world
            .query::<&BattlefieldObject>()
            .iter(world)
            .filter(|object| object.kind() == ObjectKind::Reveal)
            .map(BattlefieldObject::origin)
            .collect();
        assert!(reveals.contains(&DamageOrigin::Ability(AbilityId(23))));
        assert!(reveals.contains(&DamageOrigin::Ability(AbilityId(24))));
    }
    a.cast(source, 2, AbilityTarget::Point(Vec2::new(-3.0, 0.0)));
    a.ticks(1);
    {
        let tick = a.app.world().resource::<SimulationTick>().0;
        let world = a.app.world_mut();
        let flag = world
            .query::<&BattlefieldObject>()
            .iter(world)
            .find(|o| o.kind() == ObjectKind::Standard)
            .unwrap();
        assert_eq!(flag.origin(), DamageOrigin::Ability(AbilityId(13)));
        assert!(flag.activates_at() > tick);
        assert!(!flag.active(tick));
    }
    a.ticks(35);
    // The arrow's stun does not alter the legal target identity for the leap.
    a.cast(source, 3, AbilityTarget::Unit(target));
    a.ticks(85);
    let events = a.events();
    let target_life = a.life(target);
    assert!(events.iter().any(|event| matches!(event, SimulationEvent::AbilityCommitted {
        ability: AbilityId(14), target: AbilityTarget::Unit(unit), target_life: Some(life), recast: false, ..
    } if *unit == target && *life == target_life)));
    let ring = {
        let world = a.app.world_mut();
        let (entity, object) = world
            .query::<(Entity, &BattlefieldObject)>()
            .iter(world)
            .find(|(_, o)| o.kind() == ObjectKind::Terrain)
            .unwrap();
        assert_eq!(object.origin(), DamageOrigin::Ability(AbilityId(14)));
        entity
    };
    a.cast(source, 3, AbilityTarget::SelfCast);
    a.ticks(1);
    assert!(a.app.world().get::<BattlefieldObject>(ring).is_none());
    let events = a.events();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event,
                SimulationEvent::AbilityCommitted {
                    ability: AbilityId(14),
                    target: AbilityTarget::SelfCast,
                    target_life: None,
                    recast: true,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SimulationEvent::ImpactResolved { .. }))
    );
}

#[test]
fn empower_origin_is_frozen_for_inflight_basic_attacks_and_cleared_on_expiry_or_death() {
    for kill_source in [false, true] {
        let mut a = Arena::new();
        let empower = a.register(
            Targeting::SelfCast,
            0.0,
            AbilityExecution::Empower(EmpowerSpec {
                required_stacks: 1,
                stack_duration: 10.0,
                stack_decay: 1.0,
                duration: if kill_source { 5.0 } else { 0.2 },
                attack_speed: 0.0,
                attack_damage_multiplier: 1.1,
                arrows: 5,
                reset_attack: true,
            }),
        );
        let kill = a.register(
            Targeting::Enemy,
            20.0,
            impact(Delivery::Direct, vec![damage(20_000.0)]),
        );
        let source = a.spawn(
            Team::Ally,
            Vec2::ZERO,
            &[empower],
            UnitStats {
                attack_delivery: BasicAttackDelivery::Projectile {
                    speed: 1.0,
                    radius: 0.1,
                },
                ..stats()
            },
            Some(TICK_SECONDS),
        );
        let target = a.unit(Team::Enemy, Vec2::new(5.0, 0.0), &[kill]);
        a.command(source, UnitCommand::AttackTarget(target));
        a.ticks(2);
        assert_eq!(
            a.app.world().get::<UnitMechanics>(source).unwrap().focus(),
            1
        );
        a.command(source, UnitCommand::Stop);
        a.cast(source, 0, AbilityTarget::SelfCast);
        a.ticks(1);
        assert_eq!(
            a.app
                .world()
                .get::<UnitMechanics>(source)
                .unwrap()
                .empower_origin(),
            Some(DamageOrigin::Ability(empower))
        );
        a.command(source, UnitCommand::AttackTarget(target));
        a.ticks(2);
        a.command(source, UnitCommand::Stop);
        let boosted = a
            .projectiles()
            .into_iter()
            .find(|entity| a.app.world().get::<Projectile>(*entity).unwrap().arrows() == 5)
            .unwrap();
        let source_life = a.life(source);
        if kill_source {
            a.cast(target, 0, AbilityTarget::Unit(source));
        }
        a.ticks(15);
        let state = a.app.world().get::<UnitMechanics>(source).unwrap();
        assert!(!state.empowered());
        assert_eq!(state.empower_origin(), None);
        let projectile = a.app.world().get::<Projectile>(boosted).unwrap();
        assert_eq!(projectile.source_life(), source_life);
        assert_eq!(projectile.origin(), DamageOrigin::BasicAttack);
        assert_eq!(
            projectile.empower_origin(),
            Some(DamageOrigin::Ability(empower))
        );
        a.ticks(300);
        assert!(a.events().iter().any(|event| matches!(event, SimulationEvent::ImpactResolved {
            source_life: actual, origin: DamageOrigin::BasicAttack, empower_origin: Some(DamageOrigin::Ability(id)),
            kind: ImpactKind::Contact { target: hit }, ..
        } if *actual == source_life && *id == empower && *hit == a.life(target))));
    }
}

#[test]
fn structure_rejection_and_reset_cannot_publish_spurious_or_stale_contacts() {
    let mut a = Arena::new();
    let spell = a.register(
        Targeting::Point,
        20.0,
        impact(
            Delivery::Area {
                radius: 3.0,
                delay: 0.0,
            },
            vec![damage(100.0)],
        ),
    );
    let delayed = a.register(
        Targeting::Point,
        20.0,
        impact(
            Delivery::Area {
                radius: 3.0,
                delay: 1.0,
            },
            vec![damage(100.0)],
        ),
    );
    let source = a.unit(Team::Ally, Vec2::ZERO, &[spell, delayed]);
    let structure = a.spawn(
        Team::Enemy,
        Vec2::new(4.0, 0.0),
        &[],
        UnitStats {
            class: UnitClass::Structure,
            ..stats()
        },
        None,
    );
    a.cast(source, 0, AbilityTarget::Point(Vec2::new(4.0, 0.0)));
    a.ticks(1);
    let events = a.events();
    assert_eq!(
        contact_count(&events, DamageOrigin::Ability(spell), a.life(structure)),
        0
    );
    assert_eq!(
        a.app.world().get::<UnitState>(structure).unwrap().health(),
        10_000.0
    );
    assert!(events.iter().any(|event| matches!(
        event,
        SimulationEvent::ImpactResolved {
            kind: ImpactKind::AreaActivated { .. },
            ..
        }
    )));
    a.cast(source, 1, AbilityTarget::Point(Vec2::new(4.0, 0.0)));
    a.ticks(1);
    {
        let world = a.app.world_mut();
        assert_eq!(world.query::<&AreaEffect>().iter(world).count(), 1);
    }
    reset_simulation(a.app.world_mut());
    assert!(
        a.app
            .world()
            .resource::<Messages<SimulationEvent>>()
            .is_empty()
    );
    assert!(a.projectiles().is_empty());
    {
        let world = a.app.world_mut();
        assert_eq!(world.query::<&AreaEffect>().iter(world).count(), 0);
    }
    a.ticks(2);
    assert!(a.events().is_empty());
}
