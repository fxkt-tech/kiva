//! Demo content. Unit loadouts and ability numbers are authored here; the
//! simulation and controls know slots and definitions, never these ability IDs.
use super::controls::Patrol;
use crate::simulation::*;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct Scenery {
    pub walls: Vec<Obstacle>,
    pub trees: Vec<(Vec2, f32)>,
    pub pillars: Vec<Vec2>,
}

#[derive(Resource)]
pub(super) struct DemoSettings {
    pub default_character: CharacterId,
    pub player_team: Team,
    pub player_spawn: Vec2,
    pub respawn_delay: Option<f32>,
}

pub(super) fn setup(world: &mut World) {
    let walls = vec![
        Obstacle {
            center: Vec2::new(-10.0, 0.0),
            half_size: Vec2::new(1.2, 4.0),
        },
        Obstacle {
            center: Vec2::new(10.0, 2.0),
            half_size: Vec2::new(1.2, 4.0),
        },
        Obstacle {
            center: Vec2::new(0.0, -12.0),
            half_size: Vec2::new(4.0, 1.2),
        },
        Obstacle {
            center: Vec2::new(-13.0, -12.0),
            half_size: Vec2::new(2.5, 2.5),
        },
        Obstacle {
            center: Vec2::new(14.0, -13.0),
            half_size: Vec2::new(3.0, 2.0),
        },
        Obstacle {
            center: Vec2::new(-7.0, 15.0),
            half_size: Vec2::new(3.0, 1.0),
        },
    ];
    let trees: Vec<_> = (0..72)
        .map(|i| {
            let angle = i as f32 * 2.39996;
            let distance = 21.0 + (i % 5) as f32 * 1.5;
            (
                Vec2::new(angle.cos(), angle.sin()) * distance,
                1.8 + (i % 4) as f32 * 0.4,
            )
        })
        .collect();
    let pillars: Vec<_> = [-7.0, 7.0]
        .into_iter()
        .flat_map(|x| [-9.0, 11.0].map(|z| Vec2::new(x, z)))
        .collect();
    let mut obstacles = walls.clone();
    obstacles.extend(trees.iter().map(|(center, _)| Obstacle {
        center: *center,
        half_size: Vec2::splat(0.25),
    }));
    obstacles.extend(pillars.iter().map(|center| Obstacle {
        center: *center,
        half_size: Vec2::splat(0.5),
    }));
    world.insert_resource(NavigationMap::new(32.0, 1.0, obstacles));
    world.insert_resource(Scenery {
        walls,
        trees,
        pillars,
    });
    let magic = |base, ap_ratio| DamageSpec {
        kind: DamageType::Magical,
        base,
        ad_ratio: 0.0,
        ap_ratio,
    };
    let duration = |millis| StatusDuration::from_millis(millis).expect("valid demo duration");
    let bolt = AbilityDefinition {
        name: "奥术飞矢".into(),
        targeting: Targeting::Direction,
        range: 12.0,
        windup: 0.18,
        recovery: 0.10,
        cooldown: 2.5,
        mana_cost: 20.0,
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Projectile {
                speed: 22.0,
                radius: 0.28,
            },
            effects: vec![Effect::Damage(magic(75.0, 0.6))],
        }),
    };
    let area = AbilityDefinition {
        name: "震荡领域".into(),
        targeting: Targeting::Point,
        range: 10.0,
        windup: 0.22,
        recovery: 0.12,
        cooldown: 6.0,
        mana_cost: 40.0,
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Area {
                radius: 2.8,
                delay: 0.65,
            },
            effects: vec![Effect::Damage(magic(120.0, 0.8))],
        }),
    };
    let point_stun = AbilityDefinition {
        name: "奥术禁制".into(),
        targeting: Targeting::Enemy,
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Direct,
            effects: vec![
                Effect::Damage(magic(75.0, 0.6)),
                Effect::ApplyStatus(StatusSpec::Stun {
                    duration: duration(750),
                }),
            ],
        }),
        ..bolt.clone()
    };
    let control_area = AbilityDefinition {
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Area {
                radius: 2.8,
                delay: 0.65,
            },
            effects: vec![
                Effect::Damage(magic(120.0, 0.8)),
                Effect::ApplyStatus(StatusSpec::Root {
                    duration: duration(500),
                }),
                Effect::ApplyStatus(StatusSpec::Slow {
                    duration: duration(1500),
                    fraction: 0.3,
                }),
            ],
        }),
        ..area.clone()
    };
    let mut catalog = world.resource_mut::<AbilityCatalog>();
    for (id, definition) in [
        (AbilityId(1), bolt),
        (AbilityId(2), area),
        (
            AbilityId(3),
            AbilityDefinition {
                name: "流风步".into(),
                targeting: Targeting::Point,
                range: 6.0,
                windup: 0.08,
                recovery: 0.12,
                cooldown: 5.0,
                mana_cost: 20.0,
                execution: AbilityExecution::Dash { speed: 24.0 },
            },
        ),
        (
            AbilityId(4),
            AbilityDefinition {
                name: "共鸣".into(),
                targeting: Targeting::SelfCast,
                range: 4.5,
                windup: 0.25,
                recovery: 0.0,
                cooldown: 10.0,
                mana_cost: 60.0,
                execution: AbilityExecution::Channel {
                    duration: 2.0,
                    interval: 0.5,
                    pulse: ImpactDefinition {
                        delivery: Delivery::Area {
                            radius: 4.5,
                            delay: 0.0,
                        },
                        effects: vec![Effect::Damage(magic(35.0, 0.2))],
                    },
                },
            },
        ),
        (AbilityId(5), point_stun),
        (AbilityId(6), control_area),
        (
            AbilityId(7),
            AbilityDefinition {
                name: "破空冲击".into(),
                targeting: Targeting::Direction,
                range: 12.0,
                windup: 0.25,
                recovery: 0.12,
                cooldown: 10.0,
                mana_cost: 60.0,
                execution: AbilityExecution::Impact(ImpactDefinition {
                    delivery: Delivery::Projectile {
                        speed: 22.0,
                        radius: 0.36,
                    },
                    effects: vec![
                        Effect::Displace(DisplacementSpec {
                            distance: 4.0,
                            speed: 12.0,
                            direction: DisplacementDirection::AlongImpact,
                        }),
                        Effect::ApplyStatus(StatusSpec::Airborne {
                            duration: duration(800),
                        }),
                    ],
                }),
            },
        ),
    ] {
        catalog
            .register(id, definition)
            .expect("valid demo ability");
    }
    let mut characters = CharacterCatalog::default();
    for (id, definition) in [
        (
            CharacterId(1),
            CharacterDefinition {
                name: "旅者".into(),
                description: "稳健的近中距离斗士，以控制与机动掌握战场。".into(),
                stats: UnitStats::default(),
                default_loadout: [
                    Some(AbilityId(5)),
                    Some(AbilityId(6)),
                    Some(AbilityId(3)),
                    Some(AbilityId(7)),
                ],
            },
        ),
        (
            CharacterId(2),
            CharacterDefinition {
                name: "秘术师".into(),
                description: "灵活的远程施法者，以飞矢与共鸣持续压制敌人。".into(),
                stats: UnitStats {
                    max_health: 480.0,
                    max_mana: 360.0,
                    mana_regen: 10.0,
                    base_attributes: BaseAttributes {
                        attack_damage: 30.0,
                        ability_power: 40.0,
                        move_speed: 5.2,
                        attack_speed: 1.0,
                        attack_range: 6.0,
                        ..default()
                    },
                    ..default()
                },
                default_loadout: [
                    Some(AbilityId(1)),
                    Some(AbilityId(2)),
                    Some(AbilityId(3)),
                    Some(AbilityId(4)),
                ],
            },
        ),
    ] {
        characters
            .register(id, definition, world.resource::<AbilityCatalog>())
            .expect("valid demo character");
    }
    world.insert_resource(characters);
    world.insert_resource(DemoSettings {
        default_character: CharacterId(1),
        player_team: Team::Ally,
        player_spawn: Vec2::new(0.0, 6.0),
        respawn_delay: Some(3.0),
    });
    spawn_training_units(world);
}

/// Runtime demo opponents are rebuilt together when starting another trial.
/// Catalogs, navigation and static scenery are prepared only by setup.
pub(super) fn spawn_training_units(world: &mut World) {
    spawn_unit(
        world,
        UnitSpec {
            name: "训练哨兵".into(),
            team: Team::Enemy,
            position: Vec2::new(3.0, -2.0),
            stats: UnitStats {
                max_health: 400.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: Some(3.0),
        },
    )
    .expect("valid sentinel");
    let runner = spawn_unit(
        world,
        UnitSpec {
            name: "巡游哨兵".into(),
            team: Team::Enemy,
            position: Vec2::new(-5.0, -6.0),
            stats: UnitStats {
                max_health: 400.0,
                base_attributes: BaseAttributes {
                    move_speed: 2.5,
                    ..default()
                },
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: Some(3.0),
        },
    )
    .expect("valid moving sentinel");
    world.entity_mut(runner).insert(Patrol {
        points: [Vec2::new(-5.0, -6.0), Vec2::new(5.0, -6.0)],
        next: 1,
    });
}
