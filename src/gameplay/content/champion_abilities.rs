//! Rank-one kits composed from generic simulation executions. The legacy
//! abilities remain separate because equipment and standalone arenas use them.
use crate::{gameplay::units::world_units, simulation::*};
use bevy::prelude::*;

fn physical_with_bonus_ad(base: f32, ratio: f32) -> Effect {
    Effect::BonusAttackDamage {
        damage: DamageSpec {
            kind: DamageType::Physical,
            base,
            ad_ratio: 0.0,
            ap_ratio: 0.0,
        },
        ratio,
    }
}

fn magic(base: f32, ap_ratio: f32) -> Effect {
    Effect::Damage(DamageSpec {
        kind: DamageType::Magical,
        base,
        ad_ratio: 0.0,
        ap_ratio,
    })
}

pub(super) fn register(world: &mut World) {
    let duration = |millis| StatusDuration::from_millis(millis).expect("valid rank-one duration");
    let mut catalog = world.resource_mut::<AbilityCatalog>();
    for (id, definition) in [
        (
            11,
            AbilityDefinition {
                name: "巨龙撞击".into(),
                targeting: Targeting::Direction,
                range: world_units(785.0),
                windup: 0.4,
                recovery: 0.0,
                cooldown: 10.0,
                mana_cost: 45.0,
                execution: AbilityExecution::Line(LineSpec {
                    width: world_units(136.0),
                    effects: vec![
                        physical_with_bonus_ad(90.0, 1.45),
                        Effect::ArmorReduction {
                            fraction: 0.1,
                            duration: 3.0,
                        },
                    ],
                    link: Some(DashLinkSpec {
                        reach: world_units(850.0),
                        width: world_units(300.0),
                        speed: world_units(1800.0),
                        knockup: 0.75,
                        traverse_terrain: true,
                        can_cast_during_motion: true,
                    }),
                }),
            },
        ),
        (
            12,
            AbilityDefinition {
                name: "黄金圣盾".into(),
                targeting: Targeting::SelfCast,
                range: world_units(600.0),
                windup: 0.0,
                recovery: 0.0,
                cooldown: 9.0,
                mana_cost: 30.0,
                execution: AbilityExecution::Shield(ShieldSpec {
                    base: 60.0,
                    bonus_ad_ratio: 0.7,
                    max_health_per_enemy_hero: 0.013,
                    radius: world_units(600.0),
                    duration: 4.0,
                    slow: Some(StatusSpec::Slow {
                        duration: duration(2000),
                        fraction: 0.15,
                    }),
                }),
            },
        ),
        (
            13,
            AbilityDefinition {
                name: "德邦军旗".into(),
                targeting: Targeting::Point,
                range: world_units(860.0),
                windup: 0.0,
                recovery: 0.0,
                cooldown: 12.0,
                mana_cost: 55.0,
                execution: AbilityExecution::Standard(StandardSpec {
                    impact: ImpactDefinition {
                        delivery: Delivery::Area {
                            radius: world_units(200.0),
                            delay: 0.5,
                        },
                        effects: vec![magic(80.0, 0.8)],
                    },
                    duration: 8.0,
                    aura_radius: world_units(1000.0),
                    attack_speed: 0.2,
                    passive_attack_speed: 0.2,
                    vision_radius: world_units(700.0),
                }),
            },
        ),
        (
            14,
            AbilityDefinition {
                name: "天崩地裂".into(),
                targeting: Targeting::Enemy,
                range: world_units(650.0),
                windup: 0.0,
                recovery: 0.0,
                cooldown: 120.0,
                mana_cost: 100.0,
                execution: AbilityExecution::Leap(LeapSpec {
                    duration: 0.35,
                    radius: world_units(350.0),
                    effects: vec![physical_with_bonus_ad(200.0, 1.8)],
                    terrain: Some(RingTerrainSpec {
                        radius: world_units(350.0),
                        thickness: world_units(45.0),
                        duration: 3.5,
                        recast_delay: 0.75,
                    }),
                    heroes_only: true,
                }),
            },
        ),
        (
            21,
            AbilityDefinition {
                name: "射手的专注".into(),
                targeting: Targeting::SelfCast,
                range: 0.0,
                windup: 0.0,
                recovery: 0.0,
                cooldown: 0.0,
                mana_cost: 30.0,
                execution: AbilityExecution::Empower(EmpowerSpec {
                    required_stacks: 4,
                    stack_duration: 4.0,
                    stack_decay: 1.0,
                    duration: 6.0,
                    attack_speed: 0.2,
                    attack_damage_multiplier: 1.1,
                    arrows: 5,
                    reset_attack: true,
                }),
            },
        ),
        (
            22,
            AbilityDefinition {
                name: "万箭齐发".into(),
                targeting: Targeting::Direction,
                range: world_units(1200.0),
                windup: 0.25,
                recovery: 0.0,
                cooldown: 18.0,
                mana_cost: 75.0,
                execution: AbilityExecution::Volley(VolleySpec {
                    count: 7,
                    angle: 27.75_f32.to_radians(),
                    speed: world_units(2000.0),
                    radius: world_units(10.0),
                    effects: vec![physical_with_bonus_ad(60.0, 1.0)],
                    hero_slow: Some(StatusSpec::DecayingSlow {
                        duration: duration(2000),
                        fraction: 0.4,
                        to_fraction: 0.2,
                        decay: duration(1000),
                    }),
                }),
            },
        ),
        (
            23,
            AbilityDefinition {
                name: "鹰击长空".into(),
                targeting: Targeting::Point,
                range: world_units(25000.0),
                windup: 0.25,
                recovery: 0.0,
                cooldown: 5.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Scout(ScoutSpec {
                    speed: world_units(1400.0),
                    vision_radius: world_units(1000.0),
                    path_duration: 0.5,
                    destination_duration: 5.0,
                    charges: 2,
                    initial_charges: 1,
                    recharge: 90.0,
                }),
            },
        ),
        (
            24,
            AbilityDefinition {
                name: "魔法水晶箭".into(),
                targeting: Targeting::Direction,
                range: world_units(25000.0),
                windup: 0.25,
                recovery: 0.0,
                cooldown: 100.0,
                mana_cost: 100.0,
                execution: AbilityExecution::Arrow(ArrowSpec {
                    speed: world_units(1500.0),
                    max_speed: world_units(2100.0),
                    acceleration: world_units(200.0),
                    radius: world_units(130.0),
                    impact_radius: world_units(400.0),
                    effects: vec![magic(200.0, 1.2)],
                    stun_min: 1.0,
                    stun_max: 3.5,
                    stun_distance: world_units(2800.0),
                    vision_radius: world_units(350.0),
                }),
            },
        ),
    ] {
        if catalog.get(AbilityId(id)).is_none() {
            catalog
                .register(AbilityId(id), definition)
                .expect("valid rank-one champion ability");
        }
    }
}
