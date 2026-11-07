//! Paused equipment intents and read-only projections; simulation owns items.
use super::{
    abilities::{self, AbilityDescription, AbilityView},
    session::ClientSession,
    web,
};
use crate::simulation::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Clone, Serialize)]
pub(super) struct EquipmentAck {
    request_id: Option<u32>,
    accepted: bool,
    error: Option<String>,
}
impl Default for EquipmentAck {
    fn default() -> Self {
        Self {
            request_id: Some(0),
            accepted: true,
            error: None,
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct EquipmentAdapter {
    owner: Option<Entity>,
    pub ack: EquipmentAck,
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum EquipmentRequest {
    Equip {
        player_id: String,
        slot: usize,
        equipment_id: u16,
        request_id: u32,
    },
    Unequip {
        player_id: String,
        slot: usize,
        instance_id: String,
        request_id: u32,
    },
}
impl EquipmentRequest {
    fn request_id(&self) -> u32 {
        match self {
            Self::Equip { request_id, .. } | Self::Unequip { request_id, .. } => *request_id,
        }
    }
    fn player_id(&self) -> &str {
        match self {
            Self::Equip { player_id, .. } | Self::Unequip { player_id, .. } => player_id,
        }
    }
}

#[derive(Clone, Copy)]
struct ManagementFacts {
    paused: bool,
    captured: bool,
}

pub(super) fn paused_unlocked(control: &SimulationControl) -> bool {
    (control.paused || web::paused() == Some(true)) && web::pointer_captured() != Some(true)
}

pub(super) fn process_requests(world: &mut World) {
    let owner = world.resource::<ClientSession>().player();
    if world.resource::<EquipmentAdapter>().owner != owner {
        *world.resource_mut::<EquipmentAdapter>() = EquipmentAdapter { owner, ..default() };
    }
    if let Some(raw) = web::take_equipment_requests() {
        let facts = ManagementFacts {
            paused: world.resource::<SimulationControl>().paused || web::paused() == Some(true),
            captured: web::pointer_captured().unwrap_or(false),
        };
        process_batch(world, &raw, facts);
    }
}

fn parse_requests(raw: &str) -> Result<Vec<EquipmentRequest>, &'static str> {
    let values: Vec<serde_json::Value> =
        serde_json::from_str(raw).map_err(|_| "无法读取装备请求，请重试。")?;
    if values.len() > 32 {
        return Err("装备请求过多，请重试。");
    }
    values
        .into_iter()
        .map(|value| serde_json::from_value(value).map_err(|_| "装备请求格式无效，请重试。"))
        .collect()
}

fn process_batch(world: &mut World, raw: &str, facts: ManagementFacts) {
    match parse_requests(raw) {
        Ok(requests) => {
            for request in requests {
                let request_id = request.request_id();
                let result = apply_request(world, request, facts);
                world.resource_mut::<EquipmentAdapter>().ack = EquipmentAck {
                    request_id: Some(request_id),
                    accepted: result.is_ok(),
                    error: result.err(),
                };
            }
        }
        Err(error) => {
            world.resource_mut::<EquipmentAdapter>().ack = EquipmentAck {
                request_id: None,
                accepted: false,
                error: Some(error.into()),
            }
        }
    }
}

fn apply_request(
    world: &mut World,
    request: EquipmentRequest,
    facts: ManagementFacts,
) -> Result<(), String> {
    let player = world
        .resource::<ClientSession>()
        .player()
        .ok_or("请先开始试炼。")?;
    if player.to_bits().to_string() != request.player_id() {
        return Err("这场试炼已经结束，请重新操作。".into());
    }
    if !facts.paused || facts.captured {
        return Err("请先暂停试炼并释放鼠标。".into());
    }
    match request {
        EquipmentRequest::Equip {
            slot, equipment_id, ..
        } => equip_equipment(world, player, slot, EquipmentId(equipment_id))
            .map(|_| ())
            .map_err(equipment_error),
        EquipmentRequest::Unequip {
            slot, instance_id, ..
        } => {
            if slot >= EQUIPMENT_SLOTS {
                return Err("装备格无效。".into());
            }
            let instance = world
                .get::<EquipmentSlots>(player)
                .and_then(|slots| slots.item(slot))
                .map(EquippedItem::instance)
                .ok_or("装备格为空。")?;
            if instance.to_string() != instance_id {
                return Err("装备已经变更，请重新选择。".into());
            }
            unequip_equipment(world, player, slot, instance).map_err(equipment_error)
        }
    }
}

fn equipment_error(error: EquipmentError) -> String {
    match error {
        EquipmentError::SlotOccupied => "该装备格已被占用。",
        EquipmentError::EmptySlot => "装备格为空。",
        EquipmentError::InvalidSlot => "装备格无效。",
        EquipmentError::StaleInstance => "装备已经变更，请重新选择。",
        EquipmentError::InvalidUnit => "当前角色不可用。",
        EquipmentError::SourceExhausted => "装备实例已耗尽，请重新加载。",
        EquipmentError::DuplicateId(_)
        | EquipmentError::InvalidDefinition
        | EquipmentError::UnknownEquipment(_)
        | EquipmentError::UnknownActive(_) => "该装备不可用。",
    }
    .into()
}

#[derive(Serialize)]
pub(super) struct EquipmentView {
    slots: Vec<EquipmentSlotView>,
    first_empty_slot: Option<usize>,
    can_manage: bool,
    ack: EquipmentAck,
}
#[derive(Serialize)]
struct EquipmentSlotView {
    slot: usize,
    item: Option<EquippedItemView>,
}
#[derive(Serialize)]
struct EquippedItemView {
    id: u16,
    instance_id: String,
    name: String,
    active: Option<AbilityView>,
}
impl EquipmentView {
    pub fn new(
        player: Option<(&EquipmentSlots, &UnitState, &ControlState)>,
        catalog: &EquipmentCatalog,
        abilities: &AbilityCatalog,
        can_manage: bool,
        ack: &EquipmentAck,
    ) -> Self {
        let slots = (0..EQUIPMENT_SLOTS)
            .map(|slot| EquipmentSlotView {
                slot,
                item: player.and_then(|(slots, state, control)| {
                    let item = slots.item(slot)?;
                    let definition = catalog.get(item.equipment())?;
                    Some(EquippedItemView {
                        id: item.equipment().0,
                        instance_id: item.instance().to_string(),
                        name: definition.name.clone(),
                        active: definition
                            .active
                            .and_then(|id| abilities.get(id))
                            .map(|ability| {
                                AbilityView::with_cooldown(
                                    slot,
                                    ability,
                                    state,
                                    control,
                                    item.cooldown(),
                                )
                            }),
                    })
                }),
            })
            .collect();
        Self {
            slots,
            first_empty_slot: player.and_then(|(slots, _, _)| {
                (0..EQUIPMENT_SLOTS).find(|slot| slots.item(*slot).is_none())
            }),
            can_manage: player.is_some() && can_manage,
            ack: ack.clone(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct EquipmentDefinitionView {
    id: u16,
    name: String,
    description: String,
    bonuses: Vec<String>,
    passives: Vec<PassiveView>,
    active: Option<AbilityDescription>,
}
#[derive(Serialize)]
struct PassiveView {
    name: String,
    description: String,
}

pub(super) fn catalog_view(
    catalog: &EquipmentCatalog,
    abilities: &AbilityCatalog,
) -> Vec<EquipmentDefinitionView> {
    catalog
        .iter()
        .map(|(id, definition)| EquipmentDefinitionView {
            id: id.0,
            name: definition.name.clone(),
            description: definition.description.clone(),
            bonuses: bonus_descriptions(definition.flat, definition.additive_percent),
            passives: definition
                .passives
                .iter()
                .map(|passive| {
                    let EquipmentPassive::OnBasicAttackHit { name, effects } = passive;
                    PassiveView {
                        name: name.clone(),
                        description: format!("普攻命中：{}", abilities::describe_effects(effects)),
                    }
                })
                .collect(),
            active: definition
                .active
                .and_then(|id| abilities.get(id))
                .map(AbilityDescription::new),
        })
        .collect()
}

fn bonus_descriptions(flat: AttributeDelta, percent: AttributeDelta) -> Vec<String> {
    let fields = |value: AttributeDelta| {
        [
            value.attack_damage,
            value.ability_power,
            value.armor,
            value.magic_resistance,
            value.move_speed,
            value.attack_speed,
            value.attack_range,
        ]
    };
    let mut descriptions = Vec::new();
    for (values, is_percent) in [(fields(flat), false), (fields(percent), true)] {
        for (label, value) in ["攻击", "法强", "护甲", "魔抗", "移速", "攻速", "攻距"]
            .into_iter()
            .zip(values)
        {
            if value == 0.0 {
                continue;
            }
            let value = f64::from(value) * if is_percent { 100.0 } else { 1.0 };
            let number = if value.abs() >= 1e6 || value.abs() < 0.01 {
                format!("{value:.1e}")
            } else {
                format!("{value:.2}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .into()
            };
            descriptions.push(format!(
                "{label} {}{number}{}",
                if value >= 0.0 { "+" } else { "" },
                if is_percent { "%" } else { "" }
            ));
        }
    }
    descriptions
}
