//! Builtin ability definitions; identifiers stay in content assembly.
use crate::simulation::*;
use bevy::prelude::*;

pub(super) fn register(world: &mut World) {
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
        if catalog.get(id).is_none() {
            catalog
                .register(id, definition)
                .expect("valid builtin ability");
        }
    }
}
