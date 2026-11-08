//! Read-only skill presentation shared by HUD and input affordances.
use crate::simulation::*;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct AbilityDescription {
    id: u16,
    name: String,
    total_cooldown: f32,
    cost: f32,
    range: f32,
    display_range: f32,
    targeting: &'static str,
    target_hint: &'static str,
    icon: &'static str,
    icon_asset: Option<&'static str>,
    description: String,
}
impl AbilityDescription {
    pub fn new(id: AbilityId, definition: &AbilityDefinition) -> Self {
        let (targeting, target_hint) = match definition.targeting {
            Targeting::Enemy if matches!(&definition.execution, AbilityExecution::Leap(spec) if spec.heroes_only) => {
                ("Enemy", "点选可见敌方英雄，超距自动走近")
            }
            Targeting::Enemy => ("Enemy", "点选可见敌方单位，超距自动走近"),
            Targeting::Direction => ("Direction", "选择释放方向"),
            Targeting::Point => ("Point", "选择目标地点"),
            Targeting::SelfCast => ("SelfCast", "以自身为中心立即释放"),
        };
        let (icon, description) = describe_execution(definition.targeting, &definition.execution);
        Self {
            id: id.0,
            name: definition.name.clone(),
            total_cooldown: definition.cooldown,
            cost: definition.mana_cost,
            range: definition.range,
            display_range: crate::gameplay::units::league_units(definition.range),
            targeting,
            target_hint,
            icon,
            icon_asset: super::imported_models::ability_icon(id),
            description,
        }
    }
}

#[derive(Serialize)]
pub(super) struct AbilityMetadata {
    slot: usize,
    #[serde(flatten)]
    description: AbilityDescription,
}
impl AbilityMetadata {
    pub fn new(slot: usize, id: AbilityId, definition: &AbilityDefinition) -> Self {
        Self {
            slot,
            description: AbilityDescription::new(id, definition),
        }
    }
}

#[derive(Serialize)]
pub(super) struct AbilityView {
    #[serde(flatten)]
    metadata: AbilityMetadata,
    cooldown: f32,
    available: bool,
    unavailable_reason: Option<&'static str>,
    mechanics: Option<AbilityRuntimeView>,
}

#[derive(Serialize)]
struct AbilityRuntimeView {
    focus: Option<u8>,
    required_focus: Option<u8>,
    empowered: bool,
    charges: Option<u8>,
    max_charges: Option<u8>,
    recharge: Option<f32>,
    recast: bool,
    recast_ready: bool,
}
impl AbilityView {
    pub fn new(
        slot: usize,
        id: AbilityId,
        definition: &AbilityDefinition,
        state: &UnitState,
        control: &ControlState,
        slots: &AbilitySlots,
    ) -> Self {
        Self::with_cooldown(
            slot,
            id,
            definition,
            state,
            control,
            slots.cooldown(slot).unwrap_or(0.0),
        )
    }
    pub fn with_cooldown(
        slot: usize,
        id: AbilityId,
        definition: &AbilityDefinition,
        state: &UnitState,
        control: &ControlState,
        cooldown: f32,
    ) -> Self {
        let unavailable_reason = unavailable_reason(state, control, cooldown, definition);
        Self {
            metadata: AbilityMetadata::new(slot, id, definition),
            cooldown,
            available: unavailable_reason.is_none(),
            unavailable_reason,
            mechanics: None,
        }
    }
    pub fn with_mechanics(
        mut self,
        definition: &AbilityDefinition,
        state: &UnitState,
        control: &ControlState,
        mechanics: Option<&UnitMechanics>,
        slot: usize,
        tick: u64,
    ) -> Self {
        let Some(mechanics) = mechanics else {
            return self;
        };
        let focus = match definition.execution {
            AbilityExecution::Empower(spec) => Some(spec.required_stacks),
            _ => None,
        };
        let charges = match definition.execution {
            AbilityExecution::Scout(spec) => Some(spec.charges),
            _ => None,
        };
        let recast = mechanics.recast(slot);
        self.unavailable_reason = runtime_unavailable_reason(
            state,
            control,
            self.cooldown,
            definition,
            Some(mechanics),
            slot,
            tick,
        );
        self.available = self.unavailable_reason.is_none();
        self.mechanics = Some(AbilityRuntimeView {
            focus: focus.map(|_| mechanics.focus()),
            required_focus: focus,
            empowered: mechanics.empowered(),
            charges: mechanics.charges(slot),
            max_charges: charges,
            recharge: mechanics.recharge(slot, tick),
            recast,
            recast_ready: mechanics.recast_ready(slot, tick),
        });
        if recast {
            self.cooldown = 0.0;
            self.metadata.description.cost = 0.0;
            self.metadata.description.target_hint = "再次释放，立即移除当前地形";
        }
        self
    }
}

pub(super) struct CastInfo<'a> {
    pub ability: AbilityId,
    pub definition: &'a AbilityDefinition,
    pub cooldown: f32,
}

/// Resolve presentation from the actual source without owning any cast rules.
pub(super) fn resolve<'a>(
    source: CastSource,
    slots: &AbilitySlots,
    equipment: &EquipmentSlots,
    abilities: &'a AbilityCatalog,
    catalog: &EquipmentCatalog,
) -> Option<CastInfo<'a>> {
    let (ability, cooldown) = match source {
        CastSource::AbilitySlot(slot) => (slots.ability(slot)?, slots.cooldown(slot)?),
        CastSource::EquipmentSlot { slot, instance } => {
            let item = equipment.item(slot)?;
            if item.instance() != instance {
                return None;
            }
            (catalog.get(item.equipment())?.active?, item.cooldown())
        }
    };
    Some(CastInfo {
        ability,
        definition: abilities.get(ability)?,
        cooldown,
    })
}

/// An affordance only: the simulation validates the actual cast again.
pub(super) fn unavailable_reason(
    state: &UnitState,
    control: &ControlState,
    cooldown: f32,
    definition: &AbilityDefinition,
) -> Option<&'static str> {
    if !state.alive() {
        Some("等待重生")
    } else if !control.permits(definition) {
        Some(if control.forced_motion() {
            "强制位移中，无法施法"
        } else if matches!(definition.execution, AbilityExecution::Dash { .. }) {
            "受控制限制，无法使用位移技能"
        } else {
            "受控制限制，无法施法"
        })
    } else if cooldown > 1e-5 {
        Some("技能冷却中")
    } else if state.mana() < definition.mana_cost {
        Some("能量不足")
    } else {
        None
    }
}

pub(super) fn runtime_unavailable_reason(
    state: &UnitState,
    control: &ControlState,
    cooldown: f32,
    definition: &AbilityDefinition,
    mechanics: Option<&UnitMechanics>,
    slot: usize,
    tick: u64,
) -> Option<&'static str> {
    if let Some(mechanics) = mechanics {
        if mechanics.recast(slot) {
            return if !state.alive() {
                Some("等待重生")
            } else if !control.can_cast() {
                Some("受控制限制，无法施法")
            } else if !mechanics.recast_ready(slot, tick) {
                Some("稍后可再次释放")
            } else {
                None
            };
        }
        if matches!(definition.execution, AbilityExecution::Empower(_)) && mechanics.empowered() {
            return Some("强化已生效");
        }
        if matches!(definition.execution,AbilityExecution::Empower(spec) if mechanics.focus()<spec.required_stacks)
        {
            return Some("专注层数不足，请先普攻积攒");
        }
        if mechanics.charges(slot) == Some(0) {
            return Some("侦察充能不足");
        }
    }
    unavailable_reason(state, control, cooldown, definition)
}

pub(super) fn describe_passives(stats: &UnitStats) -> Vec<String> {
    stats
        .attack_passives
        .iter()
        .map(|passive| match passive {
            AttackPassive::CurrentHealth {
                ratio,
                minimum,
                non_hero_cap,
                target_cooldown,
            } => format!(
                "普攻附加目标当前生命{}%物理伤害，至少{}，非英雄最多{}；每目标{}秒冷却",
                number(ratio * 100.0),
                number(*minimum),
                number(*non_hero_cap),
                number(*target_cooldown)
            ),
            AttackPassive::Frost { slow, duration } => format!(
                "普攻及技能伤害使目标减速{}%，持续{}秒",
                number(slow * 100.0),
                number(*duration)
            ),
        })
        .collect()
}

fn number(value: f32) -> String {
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').into()
}

fn describe_execution(
    targeting: Targeting,
    execution: &AbilityExecution,
) -> (&'static str, String) {
    match execution {
        AbilityExecution::Line(spec) => (
            "direction",
            format!(
                "直线贯穿：{}{}",
                describe_effects(&spec.effects),
                if spec.link.is_some() {
                    "；命中自己的军旗时突进并沿途击飞"
                } else {
                    ""
                }
            ),
        ),
        AbilityExecution::Shield(spec) => (
            "shield",
            format!(
                "获得{}（+{}%额外攻击力）护盾，持续{}秒；每个附近敌方英雄增加{}%最大生命护盾{}",
                number(spec.base),
                number(spec.bonus_ad_ratio * 100.0),
                number(spec.duration),
                number(spec.max_health_per_enemy_hero * 100.0),
                spec.slow.map_or(String::new(), |slow| format!(
                    "；{}",
                    describe_effects(&[Effect::ApplyStatus(slow)])
                ))
            ),
        ),
        AbilityExecution::Standard(spec) => (
            "standard",
            format!(
                "投掷军旗：{}；旗帜持续{}秒，附近友方攻速+{}%；已学被动攻速+{}%；超距按最远地点释放",
                describe_effects(&spec.impact.effects),
                number(spec.duration),
                number(spec.attack_speed * 100.0),
                number(spec.passive_attack_speed * 100.0)
            ),
        ),
        AbilityExecution::Leap(spec) => (
            "leap",
            format!(
                "跃向选定英雄：{}{}",
                describe_effects(&spec.effects),
                spec.terrain.map_or(String::new(), |ring| format!(
                    "；生成{}秒阻挡地形，{}秒后可再次释放拆除",
                    number(ring.duration),
                    number(ring.recast_delay)
                ))
            ),
        ),
        AbilityExecution::Empower(spec) => (
            "focus",
            format!(
                "普攻积攒{}层专注；主动持续{}秒，攻速+{}%，每次普攻发射{}箭合计{}%攻击力，装备命中效果只触发一次",
                spec.required_stacks,
                number(spec.duration),
                number(spec.attack_speed * 100.0),
                spec.arrows,
                number(spec.attack_damage_multiplier * 100.0)
            ),
        ),
        AbilityExecution::Volley(spec) => (
            "volley",
            format!(
                "射出{}支扇形箭，同一目标只命中一次：{}{}",
                spec.count,
                describe_effects(&spec.effects),
                spec.hero_slow.map_or(String::new(), |slow| format!(
                    "；对英雄{}",
                    describe_effects(&[Effect::ApplyStatus(slow)])
                ))
            ),
        ),
        AbilityExecution::Scout(spec) => (
            "scout",
            format!(
                "鹰灵沿途侦察并在终点提供{}秒团队视野，无伤害；最多{}次充能，每{}秒恢复一次；未知敌人需侦察或接近才能看见",
                number(spec.destination_duration),
                spec.charges,
                number(spec.recharge)
            ),
        ),
        AbilityExecution::Arrow(spec) => (
            "arrow",
            format!(
                "穿过小兵，仅首个敌方英雄拦截：{}；周围范围伤害，眩晕随飞行距离从{}秒增至{}秒",
                describe_effects(&spec.effects),
                number(spec.stun_min),
                number(spec.stun_max)
            ),
        ),
        AbilityExecution::Dash { .. } => (
            "dash",
            "冲刺至目标地点；超出射程时沿该方向冲刺至最远距离，障碍会截断位移".into(),
        ),
        AbilityExecution::Impact(impact) => {
            let (icon, delivery) = match impact.delivery {
                Delivery::Direct => ("enemy", "直接命中".into()),
                Delivery::Projectile { .. } if targeting == Targeting::Enemy => {
                    ("enemy", "弹丸追踪选定目标".into())
                }
                Delivery::Projectile { .. } => ("direction", "弹丸命中首个敌人".into()),
                Delivery::Area { delay, .. } => ("area", format!("{}秒后范围生效", number(delay))),
            };
            (
                icon,
                format!("{delivery}：{}", describe_effects(&impact.effects)),
            )
        }
        AbilityExecution::Channel {
            duration,
            interval,
            pulse,
        } => (
            "channel",
            format!(
                "引导{}秒，每{}秒{}；移动可中断",
                number(*duration),
                number(*interval),
                describe_effects(&pulse.effects)
            ),
        ),
    }
}

pub(super) fn describe_effects(effects: &[Effect]) -> String {
    effects
        .iter()
        .map(|effect| match effect {
            Effect::Damage(damage) => {
                let kind = match damage.kind {
                    DamageType::Physical => "物理",
                    DamageType::Magical => "魔法",
                    DamageType::True => "真实",
                };
                let mut scaling = Vec::new();
                if damage.ad_ratio != 0.0 {
                    scaling.push(format!("{}%攻击力", number(damage.ad_ratio * 100.0)));
                }
                if damage.ap_ratio != 0.0 {
                    scaling.push(format!("{}%法强", number(damage.ap_ratio * 100.0)));
                }
                let bonus = if scaling.is_empty() {
                    String::new()
                } else {
                    format!("（+{}）", scaling.join("、"))
                };
                format!("{}{kind}伤害{bonus}", number(damage.base))
            }
            Effect::ApplyStatus(status) => match status {
                StatusSpec::Stun { duration } => format!("眩晕{}秒", number(duration.seconds())),
                StatusSpec::Root { duration } => format!("禁锢{}秒", number(duration.seconds())),
                StatusSpec::DecayingSlow {
                    duration,
                    fraction,
                    to_fraction,
                    ..
                } => format!(
                    "减速{}%逐渐回落至{}%，持续{}秒",
                    number(fraction * 100.0),
                    number(to_fraction * 100.0),
                    number(duration.seconds())
                ),
                StatusSpec::Slow { duration, fraction } => format!(
                    "减速{}%，持续{}秒",
                    number(fraction * 100.0),
                    number(duration.seconds())
                ),
                StatusSpec::Airborne { duration } => {
                    format!("击飞{}秒", number(duration.seconds()))
                }
            },
            Effect::BonusAttackDamage { damage, ratio } => format!(
                "{}{}伤害（+{}%额外攻击力）",
                number(damage.base),
                match damage.kind {
                    DamageType::Physical => "物理",
                    DamageType::Magical => "魔法",
                    DamageType::True => "真实",
                },
                number(ratio * 100.0)
            ),
            Effect::ArmorReduction { fraction, duration } => format!(
                "护甲降低{}%，持续{}秒",
                number(fraction * 100.0),
                number(*duration)
            ),
            Effect::Displace(displacement) => {
                let label = match displacement.direction {
                    DisplacementDirection::AlongImpact | DisplacementDirection::AwayFromCenter => {
                        "击退"
                    }
                    DisplacementDirection::TowardCenter => "拉拽",
                };
                format!(
                    "{label}{}",
                    number(crate::gameplay::units::league_units(displacement.distance))
                )
            }
        })
        .collect::<Vec<_>>()
        .join("；")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn homing_description_follows_targeting_instead_of_the_projectile_shape() {
        let execution = AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Projectile {
                speed: 20.0,
                radius: 0.2,
            },
            effects: vec![Effect::Damage(DamageSpec::magical(10.0))],
        });
        let (icon, homing) = describe_execution(Targeting::Enemy, &execution);
        assert_eq!(icon, "enemy");
        assert!(homing.contains("追踪选定目标"));
        assert!(!homing.contains("首个敌人"));
        let (icon, direction) = describe_execution(Targeting::Direction, &execution);
        assert_eq!(icon, "direction");
        assert!(direction.contains("首个敌人"));
    }

    #[test]
    fn combined_effect_description_keeps_authored_order_and_durations() {
        let text = describe_effects(&[
            Effect::Damage(DamageSpec::magical(75.0)),
            Effect::ApplyStatus(StatusSpec::Stun {
                duration: StatusDuration::from_millis(750).unwrap(),
            }),
        ]);
        assert_eq!(text, "75魔法伤害；眩晕0.75秒");
    }
    #[test]
    fn actual_mana_consumption_and_regeneration_update_the_read_only_skill_view() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(NavigationMap::new(10.0, 1.0, vec![]));
        let id = AbilityId(90);
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                id,
                AbilityDefinition {
                    name: "Mana test".into(),
                    targeting: Targeting::Point,
                    range: 5.0,
                    windup: 0.0,
                    recovery: 0.0,
                    cooldown: 0.0,
                    mana_cost: 60.0,
                    execution: AbilityExecution::Dash { speed: 20.0 },
                },
            )
            .unwrap();
        let player = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Caster".into(),
                team: Team::Ally,
                position: Vec2::ZERO,
                stats: UnitStats {
                    max_mana: 60.0,
                    mana_regen: 30.0,
                    ..default()
                },
                abilities: [Some(id), None, None, None],
                respawn_delay: None,
            },
        )
        .unwrap();
        let view = |world: &World| {
            serde_json::to_value(AbilityView::new(
                0,
                id,
                world.resource::<AbilityCatalog>().get(id).unwrap(),
                world.get::<UnitState>(player).unwrap(),
                world.get::<ControlState>(player).unwrap(),
                world.get::<AbilitySlots>(player).unwrap(),
            ))
            .unwrap()
        };
        assert_eq!(
            view(app.world())["available"],
            true,
            "exactly enough mana is usable"
        );
        app.world_mut().resource_mut::<CommandQueue>().submit(
            player,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Point(Vec2::X),
            },
        );
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(app.world().get::<UnitState>(player).unwrap().mana(), 0.0);
        assert_eq!(view(app.world())["available"], false);
        assert_eq!(view(app.world())["unavailable_reason"], "能量不足");
        for _ in 0..120 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        assert_eq!(view(app.world())["available"], true);
        assert!(view(app.world())["unavailable_reason"].is_null());
    }
}
