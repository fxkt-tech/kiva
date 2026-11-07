//! Demo content. Unit loadouts and ability numbers are authored here; the
//! simulation and controls know slots and definitions, never these ability IDs.
use super::controls::{Patrol, PlayerController};
use crate::simulation::*;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct Scenery {
    pub walls: Vec<Obstacle>,
    pub trees: Vec<(Vec2, f32)>,
    pub pillars: Vec<Vec2>,
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
    let mut catalog = world.resource_mut::<AbilityCatalog>();
    for (id, definition) in [
        (
            AbilityId(1),
            AbilityDefinition {
                name: "奥术飞矢".into(),
                targeting: Targeting::Direction,
                range: 12.0,
                windup: 0.18,
                recovery: 0.10,
                cooldown: 2.5,
                mana_cost: 20.0,
                effect: AbilityEffect::Projectile {
                    speed: 22.0,
                    radius: 0.28,
                    damage: DamageSpec {
                        kind: DamageType::Magical,
                        base: 75.0,
                        ad_ratio: 0.0,
                        ap_ratio: 0.6,
                    },
                },
            },
        ),
        (
            AbilityId(2),
            AbilityDefinition {
                name: "震荡领域".into(),
                targeting: Targeting::Point,
                range: 10.0,
                windup: 0.22,
                recovery: 0.12,
                cooldown: 6.0,
                mana_cost: 40.0,
                effect: AbilityEffect::Area {
                    radius: 2.8,
                    delay: 0.65,
                    damage: DamageSpec {
                        kind: DamageType::Magical,
                        base: 120.0,
                        ad_ratio: 0.0,
                        ap_ratio: 0.8,
                    },
                },
            },
        ),
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
                effect: AbilityEffect::Dash { speed: 24.0 },
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
                effect: AbilityEffect::Channel {
                    duration: 2.0,
                    interval: 0.5,
                    radius: 4.5,
                    damage: DamageSpec {
                        kind: DamageType::Magical,
                        base: 35.0,
                        ad_ratio: 0.0,
                        ap_ratio: 0.2,
                    },
                },
            },
        ),
    ] {
        catalog
            .register(id, definition)
            .expect("valid demo ability");
    }
    let player = spawn_unit(
        world,
        UnitSpec {
            name: "旅者".into(),
            team: Team::Ally,
            position: Vec2::new(0.0, 6.0),
            stats: UnitStats::default(),
            abilities: [
                Some(AbilityId(1)),
                Some(AbilityId(2)),
                Some(AbilityId(3)),
                Some(AbilityId(4)),
            ],
            respawn_delay: Some(3.0),
        },
    )
    .expect("valid player");
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
    world.insert_resource(PlayerController {
        player,
        selected: None,
        aim: None,
    });
}
