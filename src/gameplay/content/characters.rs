//! Builtin character definitions share the generic character factory.
use crate::gameplay::units::{regen_per_second, world_units};
use crate::simulation::*;
use bevy::prelude::*;

pub(super) fn register(world: &mut World) {
    world.resource_scope(|world, mut characters: Mut<CharacterCatalog>| {
        for (id, definition) in [
            (
                CharacterId(1),
                CharacterDefinition {
                    name: "德玛西亚皇子".into(),
                    description: "长枪与军旗协同突进，以护盾和环形地形控制战场。".into(),
                    stats: UnitStats {
                        max_health: 640.0,
                        max_mana: 300.0,
                        kind: UnitKind::Hero,
                        health_regen: regen_per_second(8.0),
                        vision_radius: world_units(1350.0),
                        attack_passives: vec![AttackPassive::CurrentHealth {
                            ratio: 0.08,
                            minimum: 20.0,
                            non_hero_cap: 400.0,
                            target_cooldown: 6.0,
                        }],
                        mana_regen: regen_per_second(6.5),
                        attack_delivery: BasicAttackDelivery::Direct,
                        attack_windup_fraction: 0.17544,
                        base_attributes: BaseAttributes {
                            attack_damage: 64.0,
                            ability_power: 0.0,
                            armor: 36.0,
                            magic_resistance: 32.0,
                            move_speed: world_units(340.0),
                            attack_speed: 0.658,
                            attack_range: world_units(175.0),
                        },
                        ..default()
                    },
                    default_loadout: [
                        Some(AbilityId(11)),
                        Some(AbilityId(12)),
                        Some(AbilityId(13)),
                        Some(AbilityId(14)),
                    ],
                },
            ),
            (
                CharacterId(2),
                CharacterDefinition {
                    name: "寒冰射手".into(),
                    description: "以冰霜箭追击敌人，积蓄专注强化普攻，并用鹰灵侦察。".into(),
                    stats: UnitStats {
                        max_health: 610.0,
                        max_mana: 280.0,
                        kind: UnitKind::Hero,
                        health_regen: regen_per_second(3.5),
                        vision_radius: world_units(1350.0),
                        attack_passives: vec![AttackPassive::Frost {
                            slow: 0.2,
                            duration: 2.0,
                        }],
                        mana_regen: regen_per_second(7.0),
                        attack_delivery: BasicAttackDelivery::Projectile {
                            speed: world_units(2500.0),
                            radius: world_units(12.0),
                        },
                        attack_windup_fraction: 0.2193,
                        base_attributes: BaseAttributes {
                            attack_damage: 59.0,
                            ability_power: 0.0,
                            armor: 26.0,
                            magic_resistance: 33.0,
                            move_speed: world_units(325.0),
                            attack_speed: 0.658,
                            attack_range: world_units(600.0),
                        },
                        ..default()
                    },
                    default_loadout: [
                        Some(AbilityId(21)),
                        Some(AbilityId(22)),
                        Some(AbilityId(23)),
                        Some(AbilityId(24)),
                    ],
                },
            ),
        ] {
            if characters.get(id).is_none() {
                characters
                    .register(id, definition, world.resource::<AbilityCatalog>())
                    .expect("valid builtin character");
            }
        }
    });
}
