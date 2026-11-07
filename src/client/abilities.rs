//! Read-only skill presentation shared by HUD and input affordances.
use crate::simulation::*;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct AbilityDescription {
    name: String,
    total_cooldown: f32,
    cost: f32,
    range: f32,
    targeting: &'static str,
    target_hint: &'static str,
    icon: &'static str,
    description: String,
}
impl AbilityDescription {
    pub fn new(definition: &AbilityDefinition) -> Self {
        let (targeting, target_hint) = match definition.targeting {
            Targeting::Enemy => ("Enemy", "点选敌方单位"),
            Targeting::Direction => ("Direction", "选择释放方向"),
            Targeting::Point => ("Point", "选择目标地点"),
            Targeting::SelfCast => ("SelfCast", "以自身为中心立即释放"),
        };
        let (icon, description) = describe_execution(definition.targeting, &definition.execution);
        Self {
            name: definition.name.clone(),
            total_cooldown: definition.cooldown,
            cost: definition.mana_cost,
            range: definition.range,
            targeting,
            target_hint,
            icon,
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
    pub fn new(slot: usize, definition: &AbilityDefinition) -> Self {
        Self {
            slot,
            description: AbilityDescription::new(definition),
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
}
impl AbilityView {
    pub fn new(
        slot: usize,
        definition: &AbilityDefinition,
        state: &UnitState,
        control: &ControlState,
        slots: &AbilitySlots,
    ) -> Self {
        Self::with_cooldown(
            slot,
            definition,
            state,
            control,
            slots.cooldown(slot).unwrap_or(0.0),
        )
    }
    pub fn with_cooldown(
        slot: usize,
        definition: &AbilityDefinition,
        state: &UnitState,
        control: &ControlState,
        cooldown: f32,
    ) -> Self {
        let unavailable_reason = unavailable_reason(state, control, cooldown, definition);
        Self {
            metadata: AbilityMetadata::new(slot, definition),
            cooldown,
            available: unavailable_reason.is_none(),
            unavailable_reason,
        }
    }
}

pub(super) struct CastInfo<'a> {
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

fn number(value: f32) -> String {
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').into()
}

fn describe_execution(
    targeting: Targeting,
    execution: &AbilityExecution,
) -> (&'static str, String) {
    match execution {
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
                StatusSpec::Slow { duration, fraction } => format!(
                    "减速{}%，持续{}秒",
                    number(fraction * 100.0),
                    number(duration.seconds())
                ),
                StatusSpec::Airborne { duration } => {
                    format!("击飞{}秒", number(duration.seconds()))
                }
            },
            Effect::Displace(displacement) => {
                let label = match displacement.direction {
                    DisplacementDirection::AlongImpact | DisplacementDirection::AwayFromCenter => {
                        "击退"
                    }
                    DisplacementDirection::TowardCenter => "拉拽",
                };
                format!("{label}{}单位", number(displacement.distance))
            }
        })
        .collect::<Vec<_>>()
        .join("；")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
