//! Public definition validation rejects unusable clocks before allocating state.
use bevy::prelude::*;
use kiva::simulation::*;

fn arena() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(40.0, 0.5, vec![]));
    app
}
fn spec(stats: UnitStats) -> UnitSpec {
    UnitSpec {
        name: "validation actor".into(),
        team: Team::Ally,
        position: Vec2::ZERO,
        stats,
        abilities: [None; 4],
        respawn_delay: None,
    }
}
fn definition(execution: AbilityExecution) -> AbilityDefinition {
    AbilityDefinition {
        name: "validation ability".into(),
        targeting: Targeting::Direction,
        range: 20.0,
        windup: 0.0,
        recovery: 0.0,
        cooldown: 0.0,
        mana_cost: 0.0,
        execution,
    }
}

#[test]
fn unrepresentable_passive_times_reject_the_entire_unit_before_spawn() {
    let mut app = arena();
    let entities_before = app.world().entities().len();
    for duration in [0.0, f32::MAX, f32::INFINITY, f32::NAN] {
        for passive in [
            AttackPassive::CurrentHealth {
                ratio: 0.08,
                minimum: 20.0,
                non_hero_cap: 400.0,
                target_cooldown: duration,
            },
            AttackPassive::Frost {
                slow: 0.2,
                duration,
            },
        ] {
            let stats = UnitStats {
                attack_passives: vec![passive],
                ..default()
            };
            assert!(spawn_unit(app.world_mut(), spec(stats)).is_err());
            assert_eq!(app.world().entities().len(), entities_before);
        }
    }
    let valid = UnitStats {
        attack_passives: vec![
            AttackPassive::CurrentHealth {
                ratio: 0.08,
                minimum: 20.0,
                non_hero_cap: 400.0,
                target_cooldown: 6.0,
            },
            AttackPassive::Frost {
                slow: 0.2,
                duration: 2.0,
            },
        ],
        ..default()
    };
    assert!(spawn_unit(app.world_mut(), spec(valid)).is_ok());
}

#[test]
fn arrow_maximum_speed_must_be_finite_and_failed_registration_keeps_the_id_free() {
    let mut app = arena();
    let arrow = ArrowSpec {
        speed: 15.0,
        max_speed: 21.0,
        acceleration: 2.0,
        radius: 1.3,
        impact_radius: 4.0,
        effects: vec![Effect::Damage(DamageSpec::magical(200.0))],
        stun_min: 1.0,
        stun_max: 3.5,
        stun_distance: 28.0,
        vision_radius: 3.5,
    };
    let mut catalog = app.world_mut().resource_mut::<AbilityCatalog>();
    for max_speed in [f32::INFINITY, f32::NAN, 0.0, 14.0] {
        let mut invalid = arrow.clone();
        invalid.max_speed = max_speed;
        assert!(
            catalog
                .register(AbilityId(1), definition(AbilityExecution::Arrow(invalid)))
                .is_err()
        );
        assert!(catalog.get(AbilityId(1)).is_none());
    }
    catalog
        .register(AbilityId(1), definition(AbilityExecution::Arrow(arrow)))
        .unwrap();
}

#[test]
fn multiple_finite_shields_project_a_finite_total_and_expire_independently() {
    let mut app = arena();
    for (id, duration) in [(1, 0.1), (2, 1.0)] {
        let mut shield = definition(AbilityExecution::Shield(ShieldSpec {
            base: f32::MAX,
            bonus_ad_ratio: 0.0,
            max_health_per_enemy_hero: 0.0,
            radius: 3.0,
            duration,
            slow: None,
        }));
        shield.targeting = Targeting::SelfCast;
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(AbilityId(id), shield)
            .unwrap();
    }
    let mut unit_spec = spec(UnitStats::default());
    unit_spec.abilities = [Some(AbilityId(1)), Some(AbilityId(2)), None, None];
    let unit = spawn_unit(app.world_mut(), unit_spec).unwrap();
    for slot in [0, 1] {
        app.world_mut().resource_mut::<CommandQueue>().submit(
            unit,
            UnitCommand::CastAbility {
                slot,
                target: AbilityTarget::SelfCast,
            },
        );
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().get::<UnitMechanics>(unit).unwrap().shield(),
        f32::MAX
    );
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().get::<UnitMechanics>(unit).unwrap().shield(),
        f32::MAX
    );
    for _ in 0..60 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().get::<UnitMechanics>(unit).unwrap().shield(),
        0.0
    );
}

#[test]
fn finite_range_and_speed_cannot_register_an_unrepresentable_flight_clock() {
    let mut app = arena();
    for execution in [
        AbilityExecution::Volley(VolleySpec {
            count: 7,
            angle: 0.5,
            speed: f32::MIN_POSITIVE,
            radius: 0.1,
            effects: vec![Effect::Damage(DamageSpec::physical(1.0))],
            hero_slow: None,
        }),
        AbilityExecution::Scout(ScoutSpec {
            speed: f32::MIN_POSITIVE,
            vision_radius: 10.0,
            path_duration: 0.5,
            destination_duration: 5.0,
            charges: 2,
            initial_charges: 1,
            recharge: 90.0,
        }),
    ] {
        let mut invalid = definition(execution);
        if matches!(invalid.execution, AbilityExecution::Scout(_)) {
            invalid.targeting = Targeting::Point;
        }
        let mut catalog = app.world_mut().resource_mut::<AbilityCatalog>();
        assert!(catalog.register(AbilityId(1), invalid).is_err());
        assert!(catalog.get(AbilityId(1)).is_none());
    }
}

#[test]
fn learned_passive_extremes_use_the_same_finite_attribute_calculation_in_preview_and_spawn() {
    let mut app = arena();
    for id in [1, 2] {
        let mut standard = definition(AbilityExecution::Standard(StandardSpec {
            impact: ImpactDefinition {
                delivery: Delivery::Area {
                    radius: 1.0,
                    delay: 0.1,
                },
                effects: vec![Effect::Damage(DamageSpec::magical(1.0))],
            },
            duration: 1.0,
            aura_radius: 1.0,
            attack_speed: 0.2,
            passive_attack_speed: f32::MAX,
            vision_radius: 1.0,
        }));
        standard.targeting = Targeting::Point;
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(AbilityId(id), standard)
            .unwrap();
    }
    let loadout = [Some(AbilityId(1)), Some(AbilityId(2)), None, None];
    for speed in [0.0, 1.0, f32::MIN_POSITIVE, f32::MAX] {
        let character = CharacterDefinition {
            name: "passive preview".into(),
            description: String::new(),
            stats: UnitStats {
                base_attributes: BaseAttributes {
                    attack_speed: speed,
                    ..default()
                },
                ..default()
            },
            default_loadout: loadout,
        };
        let preview = character
            .initial_attributes_with_loadout(app.world().resource::<AbilityCatalog>(), &loadout)
            .unwrap();
        let mut unit_spec = spec(character.stats);
        unit_spec.abilities = loadout;
        let unit = spawn_unit(app.world_mut(), unit_spec).unwrap();
        let actual = app.world().get::<Unit>(unit).unwrap().attributes();
        assert!(preview.attack_speed().is_finite());
        assert_eq!(preview.attack_speed(), actual.attack_speed());
        if speed == 0.0 {
            assert_eq!(actual.attack_speed(), 0.0);
        } else {
            assert!(actual.attack_speed() > 0.0);
        }
        if speed == 1.0 {
            assert!(actual.attack_speed() > 1.0);
        }
    }
}
