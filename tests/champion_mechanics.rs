//! Generic mechanisms are exercised through construction, intents and observations.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

fn arena() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(40.0, 0.5, vec![]));
    app
}
fn step(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
fn register(app: &mut App, id: u16, targeting: Targeting, execution: AbilityExecution) {
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            AbilityId(id),
            AbilityDefinition {
                name: format!("mechanism-{id}"),
                targeting,
                range: 30.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 20.0,
                execution,
            },
        )
        .unwrap();
}
fn unit(
    app: &mut App,
    team: Team,
    position: Vec2,
    stats: UnitStats,
    abilities: AbilityLoadout,
) -> Entity {
    spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "test actor".into(),
            team,
            position,
            stats,
            abilities,
            respawn_delay: None,
        },
    )
    .unwrap()
}
fn stats() -> UnitStats {
    UnitStats {
        max_health: 100.0,
        max_mana: 300.0,
        mana_regen: 0.0,
        attack_delivery: BasicAttackDelivery::Direct,
        attack_windup_fraction: 0.0,
        base_attributes: BaseAttributes {
            attack_damage: 40.0,
            attack_speed: 2.0,
            attack_range: 10.0,
            ..default()
        },
        ..default()
    }
}
fn cast(app: &mut App, unit: Entity, slot: usize, target: AbilityTarget) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(unit, UnitCommand::CastAbility { slot, target });
}
fn shield(base: f32, duration: f32) -> AbilityExecution {
    AbilityExecution::Shield(ShieldSpec {
        base,
        bonus_ad_ratio: 0.0,
        max_health_per_enemy_hero: 0.0,
        radius: 3.0,
        duration,
        slow: None,
    })
}
fn empower(stacks: u8) -> AbilityExecution {
    AbilityExecution::Empower(EmpowerSpec {
        required_stacks: stacks,
        stack_duration: 4.0,
        stack_decay: 1.0,
        duration: 2.0,
        attack_speed: 0.2,
        attack_damage_multiplier: 1.1,
        arrows: 5,
        reset_attack: true,
    })
}
#[test]
fn shield_absorbs_full_overkill_and_still_triggers_one_equipment_hit() {
    let mut app = arena();
    register(&mut app, 1, Targeting::SelfCast, shield(1000.0, 2.0));
    let defender = unit(
        &mut app,
        Team::Enemy,
        Vec2::X,
        stats(),
        [Some(AbilityId(1)), None, None, None],
    );
    let attacker = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            base_attributes: BaseAttributes {
                attack_damage: 500.0,
                ..stats().base_attributes
            },
            ..stats()
        },
        [None; 4],
    );
    app.world_mut()
        .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
            catalog.register(
                EquipmentId(1),
                EquipmentDefinition {
                    name: "one on-hit".into(),
                    description: String::new(),
                    flat: default(),
                    additive_percent: default(),
                    active: None,
                    passives: vec![EquipmentPassive::OnBasicAttackHit {
                        name: "slow".into(),
                        effects: vec![
                            Effect::Damage(DamageSpec {
                                kind: DamageType::True,
                                base: 5.0,
                                ad_ratio: 0.0,
                                ap_ratio: 0.0,
                            }),
                            Effect::ApplyStatus(StatusSpec::Slow {
                                duration: StatusDuration::from_ticks(60).unwrap(),
                                fraction: 0.25,
                            }),
                        ],
                    }],
                },
                world.resource::<AbilityCatalog>(),
            )
        })
        .unwrap();
    equip_equipment(app.world_mut(), attacker, 0, EquipmentId(1)).unwrap();
    cast(&mut app, defender, 0, AbilityTarget::SelfCast);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(attacker, UnitCommand::AttackTarget(defender));
    step(&mut app, 2);
    assert_eq!(
        app.world().get::<UnitState>(defender).unwrap().health(),
        100.0
    );
    assert_eq!(
        app.world().get::<UnitMechanics>(defender).unwrap().shield(),
        495.0
    );
    assert_eq!(
        app.world()
            .get::<UnitStatuses>(defender)
            .unwrap()
            .iter()
            .filter(|s| s.kind() == StatusKind::Slow)
            .count(),
        1
    );
    let mut cursor = MessageCursor::<SimulationEvent>::default();
    assert!(
        !cursor
            .read(app.world().resource::<Messages<SimulationEvent>>())
            .any(|event| matches!(event,SimulationEvent::Damaged {target,..} if *target==defender))
    );
}
#[test]
fn invalid_shield_snapshot_does_not_spend_resources_or_replace_a_move() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::SelfCast,
        AbilityExecution::Shield(ShieldSpec {
            base: f32::MAX,
            bonus_ad_ratio: 1.0,
            max_health_per_enemy_hero: 0.0,
            radius: 3.0,
            duration: 1.0,
            slow: None,
        }),
    );
    let actor = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [Some(AbilityId(1)), None, None, None],
    );
    let source = app
        .world_mut()
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .unwrap();
    app.world_mut().resource_mut::<AttributeChanges>().upsert(
        actor,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: f32::MAX,
                ..default()
            },
            ..default()
        },
    );
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(actor, UnitCommand::MoveTo(Vec2::new(5.0, 0.0)));
    step(&mut app, 1);
    let mana = app.world().get::<UnitState>(actor).unwrap().mana();
    assert_eq!(
        preview_cast(
            app.world(),
            actor,
            CastSource::AbilitySlot(0),
            AbilityTarget::SelfCast
        ),
        Err(Rejection::InvalidDamage)
    );
    cast(&mut app, actor, 0, AbilityTarget::SelfCast);
    step(&mut app, 1);
    assert_eq!(app.world().get::<UnitState>(actor).unwrap().mana(), mana);
    assert!(matches!(
        app.world().get::<UnitOrder>(actor),
        Some(UnitOrder::MoveTo(_))
    ));
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().shield(),
        0.0
    );
}
#[test]
fn scout_charges_recover_on_fixed_ticks_and_freeze_while_paused() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::Point,
        AbilityExecution::Scout(ScoutSpec {
            speed: 10.0,
            vision_radius: 2.0,
            path_duration: 0.1,
            destination_duration: 0.2,
            charges: 2,
            initial_charges: 1,
            recharge: 0.1,
        }),
    );
    let actor = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [Some(AbilityId(1)), None, None, None],
    );
    cast(&mut app, actor, 0, AbilityTarget::Point(Vec2::X));
    step(&mut app, 1);
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().charges(0),
        Some(0)
    );
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    step(&mut app, 100);
    assert_eq!(app.world().resource::<SimulationTick>().0, 1);
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().charges(0),
        Some(0)
    );
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 4);
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().charges(0),
        Some(0)
    );
    step(&mut app, 1);
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().charges(0),
        Some(1)
    );
    step(&mut app, 6);
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().charges(0),
        Some(2)
    );
}
#[test]
fn incompatible_actor_stack_resources_and_equipment_sources_are_rejected() {
    let mut app = arena();
    register(&mut app, 1, Targeting::SelfCast, empower(2));
    register(&mut app, 2, Targeting::SelfCast, empower(4));
    let spec = UnitSpec {
        name: "invalid actor".into(),
        team: Team::Ally,
        position: Vec2::ZERO,
        stats: stats(),
        abilities: [Some(AbilityId(1)), Some(AbilityId(2)), None, None],
        respawn_delay: None,
    };
    assert!(validate_unit_spec(app.world(), &spec).is_err());
    assert!(spawn_unit(app.world_mut(), spec).is_err());
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<Unit>>()
            .iter(app.world())
            .count(),
        0
    );
    let result = app
        .world_mut()
        .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
            catalog.register(
                EquipmentId(1),
                EquipmentDefinition {
                    name: "unsupported stack active".into(),
                    description: String::new(),
                    flat: default(),
                    additive_percent: default(),
                    active: Some(AbilityId(1)),
                    passives: vec![],
                },
                world.resource::<AbilityCatalog>(),
            )
        });
    assert_eq!(result, Err(EquipmentError::InvalidDefinition));
    let actor = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [Some(AbilityId(1)), Some(AbilityId(1)), None, None],
    );
    assert_eq!(app.world().get::<UnitMechanics>(actor).unwrap().focus(), 0);
}
#[test]
fn decaying_slow_changes_effective_speed_each_tick_then_restores_base() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::Enemy,
        AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects: vec![Effect::ApplyStatus(StatusSpec::DecayingSlow {
                duration: StatusDuration::from_ticks(120).unwrap(),
                fraction: 0.4,
                to_fraction: 0.2,
                decay: StatusDuration::from_ticks(60).unwrap(),
            })],
        }),
    );
    let source = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [Some(AbilityId(1)), None, None, None],
    );
    let target = unit(&mut app, Team::Enemy, Vec2::X, stats(), [None; 4]);
    let base = app
        .world()
        .get::<Unit>(target)
        .unwrap()
        .attributes()
        .move_speed();
    cast(&mut app, source, 0, AbilityTarget::Unit(target));
    step(&mut app, 1);
    assert!(
        (app.world()
            .get::<Unit>(target)
            .unwrap()
            .attributes()
            .move_speed()
            - base * 0.6)
            .abs()
            < 1e-5
    );
    step(&mut app, 30);
    assert!(
        (app.world()
            .get::<Unit>(target)
            .unwrap()
            .attributes()
            .move_speed()
            - base * 0.7)
            .abs()
            < 1e-5
    );
    step(&mut app, 30);
    assert!(
        (app.world()
            .get::<Unit>(target)
            .unwrap()
            .attributes()
            .move_speed()
            - base * 0.8)
            .abs()
            < 1e-5
    );
    step(&mut app, 60);
    assert_eq!(
        app.world()
            .get::<Unit>(target)
            .unwrap()
            .attributes()
            .move_speed(),
        base
    );
}
#[test]
fn temporary_terrain_blocks_walking_but_not_projectiles_and_reset_clears_it() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::Enemy,
        AbilityExecution::Leap(LeapSpec {
            duration: 0.1,
            radius: 1.0,
            effects: vec![Effect::Damage(DamageSpec::physical(0.0))],
            terrain: Some(RingTerrainSpec {
                radius: 2.0,
                thickness: 0.25,
                duration: 3.0,
                recast_delay: 0.1,
            }),
            heroes_only: true,
        }),
    );
    register(
        &mut app,
        2,
        Targeting::Direction,
        AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Projectile {
                speed: 20.0,
                radius: 0.1,
            },
            effects: vec![Effect::Damage(DamageSpec::magical(25.0))],
        }),
    );
    let source = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            kind: UnitKind::Hero,
            ..stats()
        },
        [Some(AbilityId(1)), None, None, None],
    );
    let target = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(4.0, 0.0),
        UnitStats {
            kind: UnitKind::Hero,
            ..stats()
        },
        [None; 4],
    );
    let shooter = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(8.0, 0.0),
        stats(),
        [Some(AbilityId(2)), None, None, None],
    );
    cast(&mut app, source, 0, AbilityTarget::Unit(target));
    step(&mut app, 8);
    let inside = app.world().get::<UnitPose>(source).unwrap().position();
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .path(inside, Vec2::new(8.0, 0.0), 0.45)
            .is_err()
    );
    cast(&mut app, shooter, 0, AbilityTarget::Direction(Vec2::NEG_X));
    step(&mut app, 15);
    assert_eq!(app.world().get::<UnitState>(source).unwrap().health(), 75.0);
    reset_simulation(app.world_mut());
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .line_clear(inside, Vec2::new(8.0, 0.0), 0.45)
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<BattlefieldObject>>()
            .iter(app.world())
            .count(),
        0
    );
    assert!(!app.world().resource::<TeamVision>().enabled());
}

#[test]
fn committed_arrow_keeps_frost_and_max_distance_stun_after_source_is_destroyed() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::Direction,
        AbilityExecution::Arrow(ArrowSpec {
            speed: 30.0,
            max_speed: 30.0,
            acceleration: 0.0,
            radius: 0.1,
            impact_radius: 1.0,
            effects: vec![Effect::Damage(DamageSpec::magical(20.0))],
            stun_min: 1.0,
            stun_max: 3.5,
            stun_distance: 28.0,
            vision_radius: 1.0,
        }),
    );
    let source = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        UnitStats {
            attack_passives: vec![AttackPassive::Frost {
                slow: 0.2,
                duration: 2.0,
            }],
            ..stats()
        },
        [Some(AbilityId(1)), None, None, None],
    );
    let target = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(30.0, 0.0),
        UnitStats {
            kind: UnitKind::Hero,
            ..stats()
        },
        [None; 4],
    );
    cast(&mut app, source, 0, AbilityTarget::Direction(Vec2::X));
    step(&mut app, 1);
    app.world_mut().despawn(source);
    for _ in 0..80 {
        step(&mut app, 1);
        if app
            .world()
            .get::<UnitStatuses>(target)
            .unwrap()
            .iter()
            .any(|status| status.kind() == StatusKind::Stun)
        {
            break;
        }
    }
    let statuses = app.world().get::<UnitStatuses>(target).unwrap();
    assert_eq!(
        statuses
            .iter()
            .find(|status| status.kind() == StatusKind::Stun)
            .unwrap()
            .duration_ticks(),
        210
    );
    assert!(
        statuses
            .iter()
            .any(|status| status.kind() == StatusKind::Slow && status.fraction() == Some(0.2))
    );
    assert_eq!(app.world().get::<UnitState>(target).unwrap().health(), 80.0);
}

#[test]
fn lethal_damage_during_a_leap_cancels_the_pending_landing_and_wall() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::Enemy,
        AbilityExecution::Leap(LeapSpec {
            duration: 0.5,
            radius: 2.0,
            effects: vec![Effect::Damage(DamageSpec::physical(30.0))],
            terrain: Some(RingTerrainSpec {
                radius: 2.0,
                thickness: 0.25,
                duration: 3.0,
                recast_delay: 0.1,
            }),
            heroes_only: true,
        }),
    );
    register(
        &mut app,
        2,
        Targeting::Enemy,
        AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects: vec![Effect::Damage(DamageSpec::magical(1000.0))],
        }),
    );
    let source = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [Some(AbilityId(1)), None, None, None],
    );
    let victim = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(5.0, 0.0),
        UnitStats {
            kind: UnitKind::Hero,
            ..stats()
        },
        [Some(AbilityId(2)), None, None, None],
    );
    cast(&mut app, source, 0, AbilityTarget::Unit(victim));
    step(&mut app, 3);
    assert!(is_dashing(app.world(), source));
    cast(&mut app, victim, 0, AbilityTarget::Unit(source));
    step(&mut app, 1);
    assert!(!app.world().get::<UnitState>(source).unwrap().alive());
    step(&mut app, 40);
    assert_eq!(
        app.world().get::<UnitState>(victim).unwrap().health(),
        100.0
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<BattlefieldObject>>()
            .iter(app.world())
            .count(),
        0
    );
    assert!(!is_dashing(app.world(), source));
}

#[test]
fn configured_link_crosses_terrain_accepts_shield_while_rooted_and_freezes_on_pause() {
    let mut app = arena();
    register(
        &mut app,
        1,
        Targeting::Point,
        AbilityExecution::Standard(StandardSpec {
            impact: ImpactDefinition {
                delivery: Delivery::Area {
                    radius: 1.0,
                    delay: 0.0,
                },
                effects: vec![Effect::Damage(DamageSpec::magical(0.0))],
            },
            duration: 5.0,
            aura_radius: 2.0,
            attack_speed: 0.0,
            passive_attack_speed: 0.0,
            vision_radius: 2.0,
        }),
    );
    register(
        &mut app,
        2,
        Targeting::Direction,
        AbilityExecution::Line(LineSpec {
            width: 1.0,
            effects: vec![Effect::Damage(DamageSpec::physical(0.0))],
            link: Some(DashLinkSpec {
                traverse_terrain: true,
                can_cast_during_motion: true,
                reach: 10.0,
                width: 3.0,
                speed: 4.0,
                knockup: 0.75,
            }),
        }),
    );
    register(&mut app, 3, Targeting::SelfCast, shield(50.0, 2.0));
    register(
        &mut app,
        4,
        Targeting::Enemy,
        AbilityExecution::Leap(LeapSpec {
            duration: 0.1,
            radius: 1.0,
            effects: vec![Effect::Damage(DamageSpec::physical(0.0))],
            terrain: Some(RingTerrainSpec {
                radius: 2.0,
                thickness: 0.25,
                duration: 3.0,
                recast_delay: 0.1,
            }),
            heroes_only: true,
        }),
    );
    register(
        &mut app,
        5,
        Targeting::Enemy,
        AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects: vec![Effect::ApplyStatus(StatusSpec::Root {
                duration: StatusDuration::from_ticks(120).unwrap(),
            })],
        }),
    );
    let actor = unit(
        &mut app,
        Team::Ally,
        Vec2::ZERO,
        stats(),
        [
            Some(AbilityId(1)),
            Some(AbilityId(2)),
            Some(AbilityId(3)),
            Some(AbilityId(4)),
        ],
    );
    let enemy = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(4.0, 0.0),
        UnitStats {
            kind: UnitKind::Hero,
            ..stats()
        },
        [Some(AbilityId(5)), None, None, None],
    );
    let rooter = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(4.0, 3.0),
        stats(),
        [Some(AbilityId(5)), None, None, None],
    );
    cast(&mut app, actor, 3, AbilityTarget::Unit(enemy));
    step(&mut app, 8);
    cast(
        &mut app,
        actor,
        0,
        AbilityTarget::Point(Vec2::new(8.0, 0.0)),
    );
    step(&mut app, 1);
    cast(&mut app, actor, 1, AbilityTarget::Direction(Vec2::X));
    step(&mut app, 1);
    assert!(is_dashing(app.world(), actor));
    assert!(motion_allows_cast(app.world(), actor));
    cast(&mut app, rooter, 0, AbilityTarget::Unit(actor));
    cast(&mut app, actor, 2, AbilityTarget::SelfCast);
    step(&mut app, 1);
    assert_eq!(
        app.world().get::<UnitMechanics>(actor).unwrap().shield(),
        50.0
    );
    assert!(!app.world().get::<ControlState>(actor).unwrap().can_move());
    let position = app.world().get::<UnitPose>(actor).unwrap().position();
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    step(&mut app, 30);
    assert_eq!(
        app.world().get::<UnitPose>(actor).unwrap().position(),
        position
    );
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 65);
    assert!(!is_dashing(app.world(), actor));
    assert!(
        app.world()
            .get::<UnitPose>(actor)
            .unwrap()
            .position()
            .distance(Vec2::new(8.0, 0.0))
            < 0.01
    );
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .path(Vec2::new(4.0, 0.0), Vec2::new(8.0, 0.0), 0.45)
            .is_err()
    );
    // A control interrupt in the middle of traversed terrain must resolve to a
    // walkable endpoint rather than leave the actor embedded in the wall.
    register(
        &mut app,
        6,
        Targeting::Enemy,
        AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects: vec![Effect::ApplyStatus(StatusSpec::Stun {
                duration: StatusDuration::from_ticks(60).unwrap(),
            })],
        }),
    );
    let stunner = unit(
        &mut app,
        Team::Enemy,
        Vec2::new(4.0, 3.0),
        stats(),
        [Some(AbilityId(6)), None, None, None],
    );
    cast(
        &mut app,
        actor,
        0,
        AbilityTarget::Point(Vec2::new(4.0, 0.0)),
    );
    step(&mut app, 1);
    cast(&mut app, actor, 1, AbilityTarget::Direction(Vec2::NEG_X));
    step(&mut app, 1);
    for _ in 0..50 {
        let position = app.world().get::<UnitPose>(actor).unwrap().position();
        if !app
            .world()
            .resource::<NavigationMap>()
            .walkable(position, 0.45)
        {
            break;
        }
        step(&mut app, 1);
    }
    let position = app.world().get::<UnitPose>(actor).unwrap().position();
    assert!(
        !app.world()
            .resource::<NavigationMap>()
            .walkable(position, 0.45)
    );
    cast(&mut app, stunner, 0, AbilityTarget::Unit(actor));
    step(&mut app, 1);
    assert!(!is_dashing(app.world(), actor));
    let position = app.world().get::<UnitPose>(actor).unwrap().position();
    assert!(
        app.world()
            .resource::<NavigationMap>()
            .walkable(position, 0.45)
    );
}
