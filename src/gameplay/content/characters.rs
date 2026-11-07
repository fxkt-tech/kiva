//! Builtin character definitions share the generic character factory.
use crate::simulation::*;
use bevy::prelude::*;

pub(super) fn register(world: &mut World) {
    world.resource_scope(|world, mut characters: Mut<CharacterCatalog>| {
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
            if characters.get(id).is_none() {
                characters
                    .register(id, definition, world.resource::<AbilityCatalog>())
                    .expect("valid builtin character");
            }
        }
    });
}
