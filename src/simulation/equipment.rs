//! Equipment owns six actual instances and their persistent attribute sources.
use super::{
    AbilityCatalog, AbilityId, AttributeChanges, AttributeDelta, AttributeModifier, DamageOrigin,
    Delivery, Effect, ImpactDefinition, ImpactSnapshot, ModifierSourceId, Targeting, Unit,
    UnitState, actions, attributes, effects,
};
use bevy::prelude::*;
use std::{collections::BTreeMap, fmt};

pub const EQUIPMENT_SLOTS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EquipmentId(pub u16);

/// Never reused within this simulation World, including after a trial reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EquipmentInstanceId(ModifierSourceId);
impl fmt::Display for EquipmentInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.value().fmt(formatter)
    }
}

#[derive(Clone, Debug)]
pub enum EquipmentPassive {
    OnBasicAttackHit { name: String, effects: Vec<Effect> },
}

#[derive(Clone, Debug)]
pub struct EquipmentDefinition {
    pub name: String,
    pub description: String,
    pub flat: AttributeDelta,
    pub additive_percent: AttributeDelta,
    pub passives: Vec<EquipmentPassive>,
    pub active: Option<AbilityId>,
}
impl EquipmentDefinition {
    fn modifier(&self) -> AttributeModifier {
        AttributeModifier {
            flat: self.flat,
            additive_percent: self.additive_percent,
            ..default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquipmentError {
    DuplicateId(EquipmentId),
    InvalidDefinition,
    UnknownEquipment(EquipmentId),
    UnknownActive(AbilityId),
    InvalidUnit,
    InvalidSlot,
    SlotOccupied,
    EmptySlot,
    StaleInstance,
    SourceExhausted,
}
impl fmt::Display for EquipmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(formatter, "duplicate equipment id {}", id.0),
            Self::InvalidDefinition => formatter.write_str("invalid equipment definition"),
            Self::UnknownEquipment(id) => write!(formatter, "unknown equipment id {}", id.0),
            Self::UnknownActive(id) => write!(formatter, "unknown active ability {}", id.0),
            Self::InvalidUnit => formatter.write_str("invalid equipment owner"),
            Self::InvalidSlot => formatter.write_str("invalid equipment slot"),
            Self::SlotOccupied => formatter.write_str("equipment slot is occupied"),
            Self::EmptySlot => formatter.write_str("equipment slot is empty"),
            Self::StaleInstance => formatter.write_str("equipment instance has changed"),
            Self::SourceExhausted => formatter.write_str("equipment source space exhausted"),
        }
    }
}
impl std::error::Error for EquipmentError {}

#[derive(Resource, Default)]
pub struct EquipmentCatalog(BTreeMap<EquipmentId, EquipmentDefinition>);
impl EquipmentCatalog {
    pub fn register(
        &mut self,
        id: EquipmentId,
        definition: EquipmentDefinition,
        abilities: &AbilityCatalog,
    ) -> Result<(), EquipmentError> {
        if self.0.contains_key(&id) {
            return Err(EquipmentError::DuplicateId(id));
        }
        if definition.name.trim().is_empty()
            || !definition.flat.valid()
            || !definition.additive_percent.valid()
            || definition.passives.iter().any(|passive| match passive {
                EquipmentPassive::OnBasicAttackHit { name, effects } => {
                    name.trim().is_empty()
                        || !ImpactDefinition {
                            delivery: Delivery::Direct,
                            effects: effects.clone(),
                        }
                        .valid(Targeting::Enemy, 0.0)
                }
            })
        {
            return Err(EquipmentError::InvalidDefinition);
        }
        if let Some(active) = definition.active
            && abilities.get(active).is_none()
        {
            return Err(EquipmentError::UnknownActive(active));
        }
        self.0.insert(id, definition);
        Ok(())
    }

    pub fn get(&self, id: EquipmentId) -> Option<&EquipmentDefinition> {
        self.0.get(&id)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (EquipmentId, &EquipmentDefinition)> + '_ {
        self.0.iter().map(|(&id, definition)| (id, definition))
    }
}

#[derive(Clone, Debug)]
pub struct EquippedItem {
    equipment: EquipmentId,
    instance: EquipmentInstanceId,
    cooldown: f32,
}
impl EquippedItem {
    pub fn equipment(&self) -> EquipmentId {
        self.equipment
    }
    pub fn instance(&self) -> EquipmentInstanceId {
        self.instance
    }
    pub fn cooldown(&self) -> f32 {
        self.cooldown
    }
}

#[derive(Component, Default, Debug)]
pub struct EquipmentSlots {
    items: [Option<EquippedItem>; EQUIPMENT_SLOTS],
}
impl EquipmentSlots {
    pub fn item(&self, slot: usize) -> Option<&EquippedItem> {
        self.items.get(slot).and_then(Option::as_ref)
    }
}

/// Creates a new instance only after validating its owner, slot and definition.
/// Persistent attribute changes are synchronous, including while paused/dead.
pub fn equip_equipment(
    world: &mut World,
    unit: Entity,
    slot: usize,
    equipment: EquipmentId,
) -> Result<EquipmentInstanceId, EquipmentError> {
    let slots = world
        .get::<EquipmentSlots>(unit)
        .filter(|_| world.get::<Unit>(unit).is_some() && world.get::<UnitState>(unit).is_some())
        .ok_or(EquipmentError::InvalidUnit)?;
    let current = slots.items.get(slot).ok_or(EquipmentError::InvalidSlot)?;
    if current.is_some() {
        return Err(EquipmentError::SlotOccupied);
    }
    let modifier = world
        .resource::<EquipmentCatalog>()
        .get(equipment)
        .ok_or(EquipmentError::UnknownEquipment(equipment))?
        .modifier();
    let source = world
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .ok_or(EquipmentError::SourceExhausted)?;
    attributes::set_immediate(world, unit, source, Some(modifier))
        .expect("validated persistent equipment modifier");
    let instance = EquipmentInstanceId(source);
    world
        .get_mut::<EquipmentSlots>(unit)
        .expect("validated slots")
        .items[slot] = Some(EquippedItem {
        equipment,
        instance,
        cooldown: 0.0,
    });
    Ok(instance)
}

pub fn unequip_equipment(
    world: &mut World,
    unit: Entity,
    slot: usize,
    expected: EquipmentInstanceId,
) -> Result<(), EquipmentError> {
    let slots = world
        .get::<EquipmentSlots>(unit)
        .filter(|_| world.get::<Unit>(unit).is_some() && world.get::<UnitState>(unit).is_some())
        .ok_or(EquipmentError::InvalidUnit)?;
    let item = slots
        .items
        .get(slot)
        .ok_or(EquipmentError::InvalidSlot)?
        .as_ref()
        .ok_or(EquipmentError::EmptySlot)?;
    if item.instance != expected {
        return Err(EquipmentError::StaleInstance);
    }
    attributes::set_immediate(world, unit, expected.0, None)
        .expect("validated equipment owner and source");
    actions::cancel_equipment_cast(world, unit, expected);
    world
        .get_mut::<EquipmentSlots>(unit)
        .expect("validated slots")
        .items[slot] = None;
    Ok(())
}

pub(super) fn binding(
    world: &World,
    unit: Entity,
    slot: usize,
    expected: EquipmentInstanceId,
) -> Option<(EquipmentId, AbilityId, f32)> {
    let item = world.get::<EquipmentSlots>(unit)?.item(slot)?;
    if item.instance != expected {
        return None;
    }
    let active = world
        .resource::<EquipmentCatalog>()
        .get(item.equipment)?
        .active?;
    Some((item.equipment, active, item.cooldown))
}

pub(super) fn commit_cooldown(
    world: &mut World,
    unit: Entity,
    slot: usize,
    expected: EquipmentInstanceId,
    cooldown: f32,
) {
    let mut slots = world
        .get_mut::<EquipmentSlots>(unit)
        .expect("validated slots");
    let item = slots.items[slot].as_mut().expect("validated item");
    assert_eq!(item.instance, expected, "validated casting instance");
    item.cooldown = cooldown;
}

pub(super) fn advance_cooldowns(world: &mut World) {
    for mut slots in world.query::<&mut EquipmentSlots>().iter_mut(world) {
        for item in slots.items.iter_mut().flatten() {
            item.cooldown = (item.cooldown - super::TICK_SECONDS).max(0.0);
        }
    }
}

pub(super) fn snapshot_passives(world: &mut World, unit: Entity) -> Option<Vec<ImpactSnapshot>> {
    let slots = world.get::<EquipmentSlots>(unit)?;
    let catalog = world.resource::<EquipmentCatalog>();
    let definitions: Vec<_> = slots
        .items
        .iter()
        .flatten()
        .flat_map(|item| {
            catalog
                .get(item.equipment)
                .expect("registered equipment")
                .passives
                .iter()
                .enumerate()
                .map(move |(passive, definition)| {
                    let EquipmentPassive::OnBasicAttackHit { effects, .. } = definition;
                    (
                        DamageOrigin::EquipmentPassive {
                            equipment: item.equipment,
                            instance: item.instance,
                            passive,
                        },
                        ImpactDefinition {
                            delivery: Delivery::Direct,
                            effects: effects.clone(),
                        },
                    )
                })
        })
        .collect();
    definitions
        .into_iter()
        .map(|(origin, definition)| effects::snapshot(world, unit, origin, &definition))
        .collect()
}
