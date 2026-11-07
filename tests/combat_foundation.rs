//! Mode-independent battle extensions through public construction and commands.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

struct Arena {
    app: App,
    events: MessageCursor<SimulationEvent>,
    next_ability: u16,
}
impl Arena {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(NavigationMap::new(20.0, 1.0, vec![]));
        Self {
            app,
            events: MessageCursor::default(),
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
    fn unit(
        &mut self,
        team: Team,
        point: Vec2,
        stats: UnitStats,
        abilities: AbilityLoadout,
    ) -> Entity {
        spawn_unit(self.app.world_mut(), spec(team, point, stats, abilities)).unwrap()
    }
    fn structure(&mut self, team: Team, point: Vec2) -> Entity {
        self.unit(
            team,
            point,
            UnitStats {
                class: UnitClass::Structure,
                ..stats()
            },
            [None; 4],
        )
    }
    fn state(&self, unit: Entity) -> &UnitState {
        self.app.world().get::<UnitState>(unit).unwrap()
    }
    fn position(&self, unit: Entity) -> Vec2 {
        self.app.world().get::<UnitPose>(unit).unwrap().position()
    }
    fn life(&self, unit: Entity) -> UnitLifeRef {
        self.state(unit).life_ref(unit)
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
    fn rejections(&mut self) -> Vec<Rejection> {
        self.events
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .filter_map(|event| match event {
                SimulationEvent::CommandRejected { reason, .. } => Some(reason.clone()),
                _ => None,
            })
            .collect()
    }
}

fn stats() -> UnitStats {
    UnitStats {
        max_mana: 100.0,
        mana_regen: 0.0,
        base_attributes: BaseAttributes {
            attack_speed: MAX_ATTACK_SPEED,
            ..default()
        },
        ..default()
    }
}
fn spec(team: Team, position: Vec2, stats: UnitStats, abilities: AbilityLoadout) -> UnitSpec {
    UnitSpec {
        name: "Foundation unit".into(),
        team,
        position,
        stats,
        abilities,
        respawn_delay: None,
    }
}
fn impact(targeting: Targeting, delivery: Delivery, effects: Vec<Effect>) -> AbilityDefinition {
    AbilityDefinition {
        name: "Foundation spell".into(),
        targeting,
        range: 15.0,
        windup: 0.0,
        recovery: 0.0,
        cooldown: 0.0,
        mana_cost: 0.0,
        execution: AbilityExecution::Impact(ImpactDefinition { delivery, effects }),
    }
}
fn damage(value: f32) -> Effect {
    Effect::Damage(DamageSpec {
        kind: DamageType::True,
        base: value,
        ad_ratio: 0.0,
        ap_ratio: 0.0,
    })
}
fn item(active: Option<AbilityId>, passives: Vec<EquipmentPassive>) -> EquipmentDefinition {
    EquipmentDefinition {
        name: "Foundation equipment".into(),
        description: String::new(),
        flat: AttributeDelta {
            attack_damage: 20.0,
            ..default()
        },
        additive_percent: default(),
        passives,
        active,
    }
}
fn equip(a: &mut Arena, unit: Entity, definition: EquipmentDefinition) -> EquipmentInstanceId {
    a.app
        .world_mut()
        .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
            catalog
                .register(
                    EquipmentId(1),
                    definition,
                    world.resource::<AbilityCatalog>(),
                )
                .unwrap();
        });
    equip_equipment(a.app.world_mut(), unit, 0, EquipmentId(1)).unwrap()
}
fn recovery() -> RecoverySpec {
    RecoverySpec {
        radius: 2.5,
        health_fraction_per_second: 0.08,
        mana_fraction_per_second: 0.12,
        out_of_combat_ticks: 120,
    }
}

#[test]
fn construction_preflight_is_pure_and_checks_delivery_and_structure_constraints() {
    let mut a = Arena::new();
    assert_eq!(UnitStats::default().class, UnitClass::Actor);
    assert_eq!(
        UnitStats::default().attack_delivery,
        BasicAttackDelivery::Projectile {
            speed: 22.0,
            radius: 0.12
        }
    );
    let valid = spec(Team::Ally, Vec2::ZERO, stats(), [None; 4]);
    for _ in 0..3 {
        assert_eq!(validate_unit_spec(a.app.world(), &valid), Ok(()));
    }
    assert_eq!(
        a.app
            .world_mut()
            .query::<&Unit>()
            .iter(a.app.world())
            .count(),
        0
    );
    for delivery in [
        BasicAttackDelivery::Projectile {
            speed: 0.0,
            radius: 0.1,
        },
        BasicAttackDelivery::Projectile {
            speed: f32::NAN,
            radius: 0.1,
        },
        BasicAttackDelivery::Projectile {
            speed: 1.0,
            radius: f32::INFINITY,
        },
        BasicAttackDelivery::Projectile {
            speed: 1.0,
            radius: -0.1,
        },
    ] {
        let invalid = UnitSpec {
            stats: UnitStats {
                attack_delivery: delivery,
                ..stats()
            },
            ..valid.clone()
        };
        assert!(validate_unit_spec(a.app.world(), &invalid).is_err());
        assert!(spawn_unit(a.app.world_mut(), invalid).is_err());
    }
    for (abilities, respawn_delay) in [
        ([Some(AbilityId(42)), None, None, None], None),
        ([None; 4], Some(1.0)),
    ] {
        assert!(
            spawn_unit(
                a.app.world_mut(),
                UnitSpec {
                    stats: UnitStats {
                        class: UnitClass::Structure,
                        ..stats()
                    },
                    abilities,
                    respawn_delay,
                    ..valid.clone()
                }
            )
            .is_err()
        );
    }
    assert_eq!(
        a.app
            .world_mut()
            .query::<&Unit>()
            .iter(a.app.world())
            .count(),
        0
    );
    spawn_unit(a.app.world_mut(), valid).unwrap();
    assert_eq!(
        a.app
            .world_mut()
            .query::<&Unit>()
            .iter(a.app.world())
            .count(),
        1
    );
}

#[test]
fn direct_and_projectile_attacks_share_release_and_recovery_but_not_flight_delay() {
    let mut a = Arena::new();
    let direct = a.unit(
        Team::Ally,
        Vec2::new(-5.0, 0.0),
        UnitStats {
            attack_delivery: BasicAttackDelivery::Direct,
            ..stats()
        },
        [None; 4],
    );
    let ranged = a.unit(
        Team::Ally,
        Vec2::new(5.0, 0.0),
        UnitStats {
            attack_delivery: BasicAttackDelivery::Projectile {
                speed: 1.0,
                radius: 0.12,
            },
            ..stats()
        },
        [None; 4],
    );
    let near = a.unit(Team::Enemy, Vec2::new(-3.0, 0.0), stats(), [None; 4]);
    let far = a.unit(Team::Enemy, Vec2::new(7.0, 0.0), stats(), [None; 4]);
    a.command(direct, UnitCommand::AttackTarget(near));
    a.command(ranged, UnitCommand::AttackTarget(far));
    a.ticks(1);
    assert!(matches!(
        a.app.world().get::<UnitAction>(direct),
        Some(UnitAction::AttackWindup { .. })
    ));
    assert_eq!(a.state(near).health(), 600.0);
    a.ticks(1);
    assert_eq!(a.state(near).health(), 560.0);
    assert_eq!(a.state(far).health(), 600.0);
    for unit in [direct, ranged] {
        assert!(matches!(
            a.app.world().get::<UnitAction>(unit),
            Some(UnitAction::AttackBackswing {
                remaining: 1,
                duration: 1
            })
        ));
        a.command(unit, UnitCommand::Stop);
    }
    assert_eq!(
        a.app
            .world_mut()
            .query::<&Projectile>()
            .iter(a.app.world())
            .count(),
        1
    );
    a.ticks(100);
    assert_eq!(a.state(near).health(), 560.0);
    assert_eq!(
        a.state(far).health(),
        560.0,
        "one committed shot survives stop and hits later"
    );
    a.command(direct, UnitCommand::AttackTarget(near));
    a.ticks(1);
    a.command(direct, UnitCommand::Stop);
    a.ticks(3);
    assert_eq!(
        a.state(near).health(),
        560.0,
        "cancelled melee preparation never creates a hit"
    );
}

#[test]
fn structures_take_basic_attack_stats_but_not_on_hit_effects_or_enemy_targeted_spells() {
    let mut a = Arena::new();
    let spell = a.register(impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![damage(200.0)],
    ));
    let hero = a.unit(
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            attack_delivery: BasicAttackDelivery::Direct,
            ..stats()
        },
        [Some(spell), None, None, None],
    );
    let base = a.unit(
        Team::Enemy,
        Vec2::X * 2.0,
        UnitStats {
            class: UnitClass::Structure,
            base_attributes: BaseAttributes {
                armor: 20.0,
                ..stats().base_attributes
            },
            ..stats()
        },
        [None; 4],
    );
    let actor = a.unit(Team::Enemy, Vec2::new(2.0, 2.0), stats(), [None; 4]);
    let duration = StatusDuration::from_ticks(30).unwrap();
    equip(
        &mut a,
        hero,
        item(
            None,
            vec![EquipmentPassive::OnBasicAttackHit {
                name: "Extra damage and control".into(),
                effects: vec![
                    damage(100.0),
                    Effect::ApplyStatus(StatusSpec::Airborne { duration }),
                    Effect::Displace(DisplacementSpec {
                        distance: 2.0,
                        speed: 6.0,
                        direction: DisplacementDirection::AlongImpact,
                    }),
                ],
            }],
        ),
    );
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::AbilitySlot(0),
            AbilityTarget::Unit(base)
        ),
        Err(Rejection::InvalidTarget)
    );
    a.cast(hero, 0, AbilityTarget::Unit(base));
    a.ticks(1);
    assert!(a.rejections().contains(&Rejection::InvalidTarget));
    assert_eq!(a.state(hero).mana(), 100.0);
    a.command(hero, UnitCommand::AttackTarget(base));
    a.ticks(2);
    a.command(hero, UnitCommand::Stop);
    a.ticks(2);
    assert_eq!(
        a.state(base).health(),
        550.0,
        "60 AD after equipment meets 20 armor; proc stays excluded"
    );
    assert!(a.app.world().get::<UnitStatuses>(base).unwrap().is_empty());
    assert_eq!(a.position(base), Vec2::X * 2.0);
    a.command(hero, UnitCommand::AttackTarget(actor));
    a.ticks(2);
    assert_eq!(
        a.state(actor).health(),
        440.0,
        "the same proc still works on ordinary actors"
    );
    assert!(!a.app.world().get::<UnitStatuses>(actor).unwrap().is_empty());
}

#[test]
fn ability_and_equipment_area_projectiles_filter_structures_at_the_shared_hit_boundary() {
    for equipment in [false, true] {
        for area in [false, true] {
            let mut a = Arena::new();
            let duration = StatusDuration::from_ticks(30).unwrap();
            let ability = a.register(impact(
                if area {
                    Targeting::Point
                } else {
                    Targeting::Direction
                },
                if area {
                    Delivery::Area {
                        radius: 3.0,
                        delay: 0.0,
                    }
                } else {
                    Delivery::Projectile {
                        speed: 100.0,
                        radius: 0.12,
                    }
                },
                vec![
                    damage(100.0),
                    Effect::ApplyStatus(StatusSpec::Stun { duration }),
                    Effect::Displace(DisplacementSpec {
                        distance: 2.0,
                        speed: 6.0,
                        direction: DisplacementDirection::AlongImpact,
                    }),
                ],
            ));
            let hero = a.unit(
                Team::Ally,
                Vec2::ZERO,
                stats(),
                [Some(ability), None, None, None],
            );
            let base = a.structure(Team::Enemy, Vec2::X * 2.0);
            let actor = a.unit(
                Team::Enemy,
                if area {
                    Vec2::new(2.0, 1.0)
                } else {
                    Vec2::X * 4.0
                },
                stats(),
                [None; 4],
            );
            let target = if area {
                AbilityTarget::Point(Vec2::X * 2.0)
            } else {
                AbilityTarget::Direction(Vec2::X)
            };
            if equipment {
                let instance = equip(&mut a, hero, item(Some(ability), vec![]));
                a.command(
                    hero,
                    UnitCommand::UseEquipment {
                        slot: 0,
                        instance,
                        target,
                    },
                );
            } else {
                a.cast(hero, 0, target);
            }
            a.ticks(5);
            assert_eq!(a.state(base).health(), 600.0);
            assert_eq!(a.position(base), Vec2::X * 2.0);
            assert!(a.app.world().get::<UnitStatuses>(base).unwrap().is_empty());
            assert_eq!(
                a.state(actor).health(),
                500.0,
                "immune structures must not intercept an actor's skill shot"
            );
            assert!(!a.app.world().get::<UnitStatuses>(actor).unwrap().is_empty());
        }
    }
}

#[test]
fn structure_control_permissions_survive_equipment_and_reject_movement_and_casts() {
    let mut a = Arena::new();
    let ability = a.register(impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![damage(100.0)],
    ));
    let base = a.structure(Team::Ally, Vec2::ZERO);
    let actor = a.unit(Team::Enemy, Vec2::X * 2.0, stats(), [None; 4]);
    let instance = equip(&mut a, base, item(Some(ability), vec![]));
    let control = a.app.world().get::<ControlState>(base).unwrap();
    assert!(!control.can_move() && !control.can_cast() && !control.can_dash());
    assert_eq!(
        preview_cast(
            a.app.world(),
            base,
            CastSource::EquipmentSlot { slot: 0, instance },
            AbilityTarget::Unit(actor)
        ),
        Err(Rejection::Controlled)
    );
    a.command(base, UnitCommand::MoveTo(Vec2::Y * 5.0));
    a.command(base, UnitCommand::AttackMove(Vec2::Y * 5.0));
    a.command(
        base,
        UnitCommand::UseEquipment {
            slot: 0,
            instance,
            target: AbilityTarget::Unit(actor),
        },
    );
    a.ticks(10);
    assert_eq!(a.position(base), Vec2::ZERO);
    assert_eq!(a.state(actor).health(), 600.0);
    assert_eq!(a.rejections(), vec![Rejection::Controlled; 3]);
    assert!(matches!(
        a.app.world().get::<UnitOrder>(base),
        Some(UnitOrder::Idle)
    ));
}

#[test]
fn cast_preflight_is_read_only_and_tracks_real_dash_busy_resource_and_instance_rules() {
    let mut a = Arena::new();
    let dash = a.register(AbilityDefinition {
        name: "Preflight dash".into(),
        targeting: Targeting::Point,
        range: 5.0,
        windup: 0.0,
        recovery: 0.0,
        cooldown: 2.0,
        mana_cost: 20.0,
        execution: AbilityExecution::Dash { speed: 10.0 },
    });
    let expensive = a.register(AbilityDefinition {
        mana_cost: 200.0,
        ..impact(Targeting::Enemy, Delivery::Direct, vec![damage(10.0)])
    });
    let delayed = a.register(AbilityDefinition {
        windup: 0.2,
        ..impact(Targeting::Enemy, Delivery::Direct, vec![damage(10.0)])
    });
    let hero = a.unit(
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [Some(dash), Some(expensive), Some(delayed), None],
    );
    let enemy = a.unit(Team::Enemy, Vec2::Y * 2.0, stats(), [None; 4]);
    for _ in 0..4 {
        assert_eq!(
            preview_cast(
                a.app.world(),
                hero,
                CastSource::AbilitySlot(0),
                AbilityTarget::Point(Vec2::X * 100.0)
            ),
            Ok(AbilityTarget::Point(Vec2::X * 5.0))
        );
    }
    assert_eq!(a.state(hero).mana(), 100.0);
    assert_eq!(
        a.app.world().get::<AbilitySlots>(hero).unwrap().cooldown(0),
        Some(0.0)
    );
    assert!(matches!(
        a.app.world().get::<UnitAction>(hero),
        Some(UnitAction::Ready)
    ));
    assert!(
        a.events
            .read(a.app.world().resource::<Messages<SimulationEvent>>())
            .next()
            .is_none()
    );
    a.cast(hero, 0, AbilityTarget::Point(Vec2::X * 100.0));
    a.ticks(1);
    assert!(is_dashing(a.app.world(), hero));
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::AbilitySlot(2),
            AbilityTarget::Unit(enemy)
        ),
        Err(Rejection::Busy)
    );
    a.ticks(40);
    assert!(!is_dashing(a.app.world(), hero));
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::AbilitySlot(0),
            AbilityTarget::Point(Vec2::ZERO)
        ),
        Err(Rejection::OnCooldown)
    );
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::AbilitySlot(1),
            AbilityTarget::Unit(enemy)
        ),
        Err(Rejection::InsufficientMana)
    );
    a.cast(hero, 2, AbilityTarget::Unit(enemy));
    a.ticks(1);
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::AbilitySlot(2),
            AbilityTarget::Unit(enemy)
        ),
        Err(Rejection::Busy)
    );
    let instance = equip(&mut a, hero, item(Some(delayed), vec![]));
    unequip_equipment(a.app.world_mut(), hero, 0, instance).unwrap();
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::EquipmentSlot { slot: 0, instance },
            AbilityTarget::Unit(enemy)
        ),
        Err(Rejection::NoAbility)
    );
}

#[test]
fn zone_recovery_checks_lives_team_range_combat_pause_and_tick_idempotency() {
    let mut a = Arena::new();
    let hurt = a.register(impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![damage(150.0)],
    ));
    let spend = a.register(AbilityDefinition {
        mana_cost: 40.0,
        ..impact(
            Targeting::Point,
            Delivery::Area {
                radius: 1.0,
                delay: 0.0,
            },
            vec![damage(1.0)],
        )
    });
    let base = a.structure(Team::Ally, Vec2::ZERO);
    let hero = a.unit(
        Team::Ally,
        Vec2::X,
        stats(),
        [Some(spend), None, None, None],
    );
    let enemy = a.unit(
        Team::Enemy,
        Vec2::new(1.0, 2.0),
        stats(),
        [Some(hurt), None, None, None],
    );
    let distant = a.unit(Team::Ally, Vec2::X * 10.0, stats(), [None; 4]);
    let (source, target, hostile, far) =
        (a.life(base), a.life(hero), a.life(enemy), a.life(distant));
    a.cast(hero, 0, AbilityTarget::Point(Vec2::Y * 10.0));
    a.cast(enemy, 0, AbilityTarget::Unit(hero));
    a.ticks(1);
    assert_eq!(a.state(hero).health(), 450.0);
    assert_eq!(a.state(hero).mana(), 60.0);
    assert_eq!(a.state(hero).last_damaged_tick(), Some(1));
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, target, recovery()),
        Err(RecoveryError::InCombat)
    );
    assert_eq!(
        restore_in_zone(a.app.world_mut(), target, target, recovery()),
        Err(RecoveryError::InvalidSource)
    );
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, source, recovery()),
        Err(RecoveryError::InvalidTarget)
    );
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, hostile, recovery()),
        Err(RecoveryError::WrongTeam)
    );
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, far, recovery()),
        Err(RecoveryError::OutOfRange)
    );
    assert_eq!(
        restore_in_zone(
            a.app.world_mut(),
            source,
            target,
            RecoverySpec {
                radius: f32::NAN,
                ..recovery()
            }
        ),
        Err(RecoveryError::InvalidDefinition)
    );
    a.app.world_mut().resource_mut::<SimulationControl>().paused = true;
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, target, recovery()),
        Err(RecoveryError::Paused)
    );
    a.ticks(100);
    assert_eq!(a.app.world().resource::<SimulationTick>().0, 1);
    a.app.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.ticks(119);
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, target, recovery()),
        Err(RecoveryError::InCombat)
    );
    a.ticks(1);
    let amount = restore_in_zone(a.app.world_mut(), source, target, recovery()).unwrap();
    assert!((amount.health - 0.8).abs() < 0.0001);
    assert!((amount.mana - 0.2).abs() < 0.0001);
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, target, recovery()),
        Err(RecoveryError::AlreadyRestored)
    );
    for _ in 0..220 {
        a.ticks(1);
        restore_in_zone(a.app.world_mut(), source, target, recovery()).unwrap();
    }
    assert_eq!(a.state(hero).health(), 600.0);
    assert_eq!(a.state(hero).mana(), 100.0);
}

#[test]
fn recovery_cannot_revive_a_dead_or_old_life_and_cannot_use_a_dead_source() {
    let mut a = Arena::new();
    let kill = a.register(impact(
        Targeting::Enemy,
        Delivery::Direct,
        vec![damage(1000.0)],
    ));
    let base = a.structure(Team::Ally, Vec2::ZERO);
    let hero = spawn_unit(
        a.app.world_mut(),
        UnitSpec {
            respawn_delay: Some(TICK_SECONDS),
            ..spec(Team::Ally, Vec2::X, stats(), [None; 4])
        },
    )
    .unwrap();
    let enemy = a.unit(
        Team::Enemy,
        Vec2::Y,
        UnitStats {
            attack_delivery: BasicAttackDelivery::Direct,
            base_attributes: BaseAttributes {
                attack_damage: 1000.0,
                ..stats().base_attributes
            },
            ..stats()
        },
        [Some(kill), None, None, None],
    );
    let (source, old) = (a.life(base), a.life(hero));
    a.cast(enemy, 0, AbilityTarget::Unit(hero));
    a.ticks(1);
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, old, recovery()),
        Err(RecoveryError::InvalidTarget)
    );
    assert_eq!(
        preview_cast(
            a.app.world(),
            hero,
            CastSource::AbilitySlot(0),
            AbilityTarget::Unit(enemy)
        ),
        Err(Rejection::Dead)
    );
    a.ticks(1);
    let new = a.life(hero);
    assert_ne!(old, new);
    assert_eq!(a.state(hero).last_damaged_tick(), None);
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, old, recovery()),
        Err(RecoveryError::InvalidTarget)
    );
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, new, recovery()).unwrap(),
        RecoveryAmount::default()
    );
    a.command(enemy, UnitCommand::AttackTarget(base));
    a.ticks(2);
    assert!(!a.state(base).alive());
    assert_eq!(
        restore_in_zone(a.app.world_mut(), source, new, recovery()),
        Err(RecoveryError::InvalidSource)
    );
}

#[test]
fn damage_estimation_matches_actual_damage_and_rejects_invalid_inputs() {
    for (kind, armor, resistance) in [
        (DamageType::Physical, 100.0, 0.0),
        (DamageType::Magical, 0.0, 100.0),
        (DamageType::Physical, -50.0, 0.0),
        (DamageType::True, 1000.0, 1000.0),
    ] {
        let mut a = Arena::new();
        let expression = DamageSpec {
            kind,
            base: 90.0,
            ad_ratio: 0.5,
            ap_ratio: 0.6,
        };
        let ability = a.register(impact(
            Targeting::Enemy,
            Delivery::Direct,
            vec![Effect::Damage(expression)],
        ));
        let source = a.unit(
            Team::Ally,
            Vec2::ZERO,
            stats(),
            [Some(ability), None, None, None],
        );
        let target = a.unit(
            Team::Enemy,
            Vec2::X,
            UnitStats {
                base_attributes: BaseAttributes {
                    armor,
                    magic_resistance: resistance,
                    ..stats().base_attributes
                },
                ..stats()
            },
            [None; 4],
        );
        let source_attributes = *a.app.world().get::<Unit>(source).unwrap().attributes();
        let target_attributes = *a.app.world().get::<Unit>(target).unwrap().attributes();
        let estimate = expression
            .estimate_damage(&source_attributes, &target_attributes, 600.0)
            .unwrap();
        assert_eq!(a.state(target).health(), 600.0);
        a.cast(source, 0, AbilityTarget::Unit(target));
        a.ticks(1);
        assert_eq!(estimate, 600.0 - a.state(target).health());
        assert_eq!(
            expression.estimate_damage(&source_attributes, &target_attributes, -1.0),
            None
        );
        assert_eq!(
            expression.estimate_damage(&source_attributes, &target_attributes, f32::NAN),
            None
        );
        assert_eq!(
            DamageSpec {
                base: f32::NAN,
                ..expression
            }
            .estimate_damage(&source_attributes, &target_attributes, 600.0),
            None
        );
        assert_eq!(
            DamageSpec {
                base: f32::MAX,
                ad_ratio: f32::MAX,
                ..expression
            }
            .estimate_damage(&source_attributes, &target_attributes, 600.0),
            None
        );
    }
}

#[test]
fn public_step_set_supports_headless_mode_ordering_and_does_not_run_partial_ticks() {
    #[derive(Resource, Default)]
    struct Trace(Vec<(&'static str, u64)>);
    #[derive(Resource)]
    struct Running(bool);
    let mut a = Arena::new();
    a.app
        .init_resource::<Trace>()
        .insert_resource(Running(true))
        .configure_sets(
            FixedUpdate,
            SimulationSet::Step.run_if(|running: Res<Running>| running.0),
        )
        .add_systems(
            FixedUpdate,
            (
                (|tick: Res<SimulationTick>, mut trace: ResMut<Trace>| {
                    trace.0.push(("before", tick.0))
                })
                .before(SimulationSet::Step),
                (|tick: Res<SimulationTick>, mut trace: ResMut<Trace>| {
                    trace.0.push(("after", tick.0))
                })
                .after(SimulationSet::Step),
            ),
        );
    a.ticks(1);
    assert_eq!(
        a.app.world().resource::<Trace>().0,
        [("before", 0), ("after", 1)]
    );
    a.app.world_mut().resource_mut::<Running>().0 = false;
    a.ticks(2);
    assert_eq!(a.app.world().resource::<SimulationTick>().0, 1);
    assert_eq!(
        a.app.world().resource::<Trace>().0[2..],
        [("before", 1), ("after", 1), ("before", 1), ("after", 1)]
    );
}
