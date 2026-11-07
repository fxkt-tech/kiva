//! Builtin equipment definitions; no shop or runtime inventory lives here.
use crate::simulation::*;
use bevy::prelude::*;

pub(super) fn register(world: &mut World) {
    world.resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
        for (id, name, description, flat, additive_percent, passives, active) in [
            (
                1,
                "锋刃",
                "强化普通攻击与攻击力加成技能。",
                AttributeDelta {
                    attack_damage: 20.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                None,
            ),
            (
                2,
                "灵巧短弓",
                "加快下一次普攻节奏，并延长攻击距离。",
                AttributeDelta {
                    attack_range: 0.5,
                    ..default()
                },
                AttributeDelta {
                    attack_speed: 0.25,
                    ..default()
                },
                vec![],
                None,
            ),
            (
                3,
                "法术典籍",
                "强化法术强度加成效果。",
                AttributeDelta {
                    ability_power: 30.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                None,
            ),
            (
                4,
                "守护披甲",
                "抵御物理与魔法伤害。",
                AttributeDelta {
                    armor: 25.0,
                    magic_resistance: 20.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                None,
            ),
            (
                5,
                "疾行之靴",
                "提高普通移动速度。",
                AttributeDelta {
                    move_speed: 0.75,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                None,
            ),
            (
                6,
                "霜纹护符",
                "普通攻击命中时附加霜纹效果。",
                AttributeDelta {
                    ability_power: 10.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![EquipmentPassive::OnBasicAttackHit {
                    name: "霜纹".into(),
                    effects: vec![
                        Effect::Damage(DamageSpec {
                            kind: DamageType::Magical,
                            base: 12.0,
                            ad_ratio: 0.0,
                            ap_ratio: 0.1,
                        }),
                        Effect::ApplyStatus(StatusSpec::Slow {
                            duration: StatusDuration::from_millis(1000)
                                .expect("valid frost duration"),
                            fraction: 0.2,
                        }),
                    ],
                }],
                None,
            ),
            (
                7,
                "束缚法珠",
                "主动点选敌人，造成伤害并眩晕。",
                AttributeDelta {
                    ability_power: 15.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                Some(AbilityId(5)),
            ),
            (
                8,
                "流风之刃",
                "主动向地点冲刺，遇到障碍时停止。",
                AttributeDelta {
                    attack_damage: 10.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                Some(AbilityId(3)),
            ),
            (
                9,
                "共鸣棱镜",
                "主动引导自身范围的共鸣脉冲。",
                AttributeDelta {
                    ability_power: 20.0,
                    ..default()
                },
                AttributeDelta::default(),
                vec![],
                Some(AbilityId(4)),
            ),
        ] {
            if catalog.get(EquipmentId(id)).is_none() {
                catalog
                    .register(
                        EquipmentId(id),
                        EquipmentDefinition {
                            name: name.into(),
                            description: description.into(),
                            flat,
                            additive_percent,
                            passives,
                            active,
                        },
                        world.resource::<AbilityCatalog>(),
                    )
                    .expect("valid builtin equipment");
            }
        }
    });
}
