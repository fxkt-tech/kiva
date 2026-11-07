//! Source-owned attribute modifiers. Only this module computes effective values.
use super::{SimulationEvent, TICK_RATE, Unit, UnitState};
use bevy::prelude::*;
use std::collections::{BTreeMap, VecDeque};

const ARMOR: usize = 2;
const MAGIC_RESISTANCE: usize = 3;
const ATTACK_SPEED: usize = 6;
const MOVE_SPEED: usize = 4;
/// Technical limits imposed by integer attack clocks, independent of balance.
pub const MIN_ATTACK_SPEED: f32 = (TICK_RATE / u32::MAX as f64) as f32;
pub const MAX_ATTACK_SPEED: f32 = (TICK_RATE / 2.0) as f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaseAttributes {
    pub attack_damage: f32,
    pub ability_power: f32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub move_speed: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
}
impl Default for BaseAttributes {
    fn default() -> Self {
        Self {
            attack_damage: 40.0,
            ability_power: 0.0,
            armor: 0.0,
            magic_resistance: 0.0,
            move_speed: 5.5,
            attack_range: 5.5,
            attack_speed: 1.25,
        }
    }
}
impl BaseAttributes {
    fn values(self) -> [f32; 7] {
        [
            self.attack_damage,
            self.ability_power,
            self.armor,
            self.magic_resistance,
            self.move_speed,
            self.attack_range,
            self.attack_speed,
        ]
    }
    pub(super) fn valid(self) -> bool {
        self.values().into_iter().enumerate().all(|(index, value)| {
            value.is_finite() && (matches!(index, ARMOR | MAGIC_RESISTANCE) || value >= 0.0)
        })
    }
}

/// Signed deltas; percentages use fractions (0.2 means +20%).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttributeDelta {
    pub attack_damage: f32,
    pub ability_power: f32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub move_speed: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
}
impl AttributeDelta {
    fn values(self) -> [f32; 7] {
        [
            self.attack_damage,
            self.ability_power,
            self.armor,
            self.magic_resistance,
            self.move_speed,
            self.attack_range,
            self.attack_speed,
        ]
    }
    fn valid(self) -> bool {
        self.values().into_iter().all(f32::is_finite)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectiveAttributes([f32; 7]);
impl EffectiveAttributes {
    pub fn attack_damage(&self) -> f32 {
        self.0[0]
    }
    pub fn ability_power(&self) -> f32 {
        self.0[1]
    }
    pub fn armor(&self) -> f32 {
        self.0[ARMOR]
    }
    pub fn magic_resistance(&self) -> f32 {
        self.0[MAGIC_RESISTANCE]
    }
    pub fn move_speed(&self) -> f32 {
        self.0[MOVE_SPEED]
    }
    pub fn attack_range(&self) -> f32 {
        self.0[5]
    }
    pub fn attack_speed(&self) -> f32 {
        self.0[ATTACK_SPEED]
    }

    pub(super) fn attack_timing(&self, windup_fraction: f32) -> Option<(u32, u32)> {
        let speed = self.attack_speed();
        if speed == 0.0 {
            return None;
        }
        let period = (TICK_RATE / f64::from(speed))
            .ceil()
            .clamp(2.0, f64::from(u32::MAX)) as u32;
        let windup = (f64::from(period) * f64::from(windup_fraction))
            .ceil()
            .clamp(1.0, f64::from(period - 1)) as u32;
        Some((windup, period - windup))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModifierRemoval {
    #[default]
    Explicit,
    OnDeath,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttributeModifier {
    pub flat: AttributeDelta,
    pub additive_percent: AttributeDelta,
    pub removal: ModifierRemoval,
    /// Final movement-speed factor. Statuses aggregate their slows before using it.
    pub move_speed_multiplier: f32,
}
impl Default for AttributeModifier {
    fn default() -> Self {
        Self {
            flat: AttributeDelta::default(),
            additive_percent: AttributeDelta::default(),
            removal: ModifierRemoval::Explicit,
            move_speed_multiplier: 1.0,
        }
    }
}
impl AttributeModifier {
    fn valid(self) -> bool {
        self.flat.valid()
            && self.additive_percent.valid()
            && self.move_speed_multiplier.is_finite()
            && (0.0..=1.0).contains(&self.move_speed_multiplier)
    }
}

/// Allocated by AttributeChanges; never recycled within one simulation World.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifierSourceId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeRejection {
    InvalidUnit,
    InvalidSource,
    InvalidModifier,
    Dead,
}
#[derive(Clone, Copy)]
enum Change {
    Upsert(AttributeModifier),
    Remove,
}
#[derive(Resource, Default)]
pub struct AttributeChanges {
    last_source: u64,
    pending: VecDeque<(Entity, ModifierSourceId, Change)>,
}
impl AttributeChanges {
    /// Discard requests from the finished trial without recycling source IDs.
    pub(super) fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// None means the ID space is exhausted; no existing source can be reused.
    pub fn allocate_source(&mut self) -> Option<ModifierSourceId> {
        self.last_source = self.last_source.checked_add(1)?;
        Some(ModifierSourceId(self.last_source))
    }
    pub fn upsert(&mut self, unit: Entity, source: ModifierSourceId, modifier: AttributeModifier) {
        self.pending
            .push_back((unit, source, Change::Upsert(modifier)));
    }
    pub fn remove(&mut self, unit: Entity, source: ModifierSourceId) {
        self.pending.push_back((unit, source, Change::Remove));
    }
}

#[derive(Debug)]
pub(super) struct AttributeState {
    modifiers: BTreeMap<ModifierSourceId, AttributeModifier>,
    effective: EffectiveAttributes,
}
impl AttributeState {
    pub fn new(base: BaseAttributes) -> Self {
        Self {
            modifiers: BTreeMap::new(),
            effective: initial_attributes(base),
        }
    }
    pub fn effective(&self) -> &EffectiveAttributes {
        &self.effective
    }
}

/// Pure initial projection, shared by runtime construction and content previews.
pub(super) fn initial_attributes(base: BaseAttributes) -> EffectiveAttributes {
    compute(base, &BTreeMap::new())
}

fn compute(
    base: BaseAttributes,
    modifiers: &BTreeMap<ModifierSourceId, AttributeModifier>,
) -> EffectiveAttributes {
    let mut flat = [0.0_f64; 7];
    let mut percent = [0.0_f64; 7];
    let mut move_multiplier = 1.0_f64;
    for modifier in modifiers.values() {
        move_multiplier *= f64::from(modifier.move_speed_multiplier);
        for (total, value) in flat.iter_mut().zip(modifier.flat.values()) {
            *total += f64::from(value);
        }
        for (total, value) in percent.iter_mut().zip(modifier.additive_percent.values()) {
            *total += f64::from(value);
        }
    }
    let mut result = [0.0; 7];
    for (index, value) in base.values().into_iter().enumerate() {
        // All inputs are finite f32. Even usize::MAX such terms and their
        // product fit f64, so removal/death cleanup cannot overflow aggregation.
        let mut value = (f64::from(value) + flat[index]) * (1.0 + percent[index]).max(0.0);
        if index == MOVE_SPEED {
            value *= move_multiplier;
        }
        if index == ATTACK_SPEED && value > 0.0 {
            value = value.clamp(f64::from(MIN_ATTACK_SPEED), f64::from(MAX_ATTACK_SPEED));
        }
        let minimum = if matches!(index, ARMOR | MAGIC_RESISTANCE) {
            -f64::from(f32::MAX)
        } else {
            0.0
        };
        result[index] = value.clamp(minimum, f64::from(f32::MAX)) as f32;
    }
    EffectiveAttributes(result)
}

fn apply(
    world: &mut World,
    entity: Entity,
    source: ModifierSourceId,
    change: Change,
) -> Result<(), AttributeRejection> {
    let alive = world
        .get::<UnitState>(entity)
        .ok_or(AttributeRejection::InvalidUnit)?
        .alive();
    let unit = world
        .get::<Unit>(entity)
        .ok_or(AttributeRejection::InvalidUnit)?;
    let mut candidate = unit.attributes.modifiers.clone();
    match change {
        Change::Upsert(modifier) => {
            if !modifier.valid() {
                return Err(AttributeRejection::InvalidModifier);
            }
            if !alive && modifier.removal == ModifierRemoval::OnDeath {
                return Err(AttributeRejection::Dead);
            }
            candidate.insert(source, modifier);
        }
        Change::Remove => {
            candidate.remove(&source);
        }
    }
    if candidate == unit.attributes.modifiers {
        return Ok(());
    }
    let effective = compute(unit.stats.base_attributes, &candidate);
    let mut unit = world.get_mut::<Unit>(entity).expect("validated unit");
    unit.attributes.modifiers = candidate;
    if unit.attributes.effective != effective {
        unit.attributes.effective = effective;
    }
    Ok(())
}

pub(super) fn set_immediate(
    world: &mut World,
    unit: Entity,
    source: ModifierSourceId,
    modifier: Option<AttributeModifier>,
) -> Result<(), AttributeRejection> {
    apply(
        world,
        unit,
        source,
        modifier.map_or(Change::Remove, Change::Upsert),
    )
}

pub(super) fn consume_changes(world: &mut World) {
    let (pending, last_source) = {
        let mut changes = world.resource_mut::<AttributeChanges>();
        (std::mem::take(&mut changes.pending), changes.last_source)
    };
    for (unit, source, change) in pending {
        let result = if source.0 == 0 || source.0 > last_source {
            Err(AttributeRejection::InvalidSource)
        } else {
            apply(world, unit, source, change)
        };
        if let Err(reason) = result {
            world.write_message(SimulationEvent::AttributeChangeRejected {
                unit,
                source,
                reason,
            });
        }
    }
}

pub(super) fn clear_on_death(unit: &mut Unit) {
    let old_len = unit.attributes.modifiers.len();
    unit.attributes
        .modifiers
        .retain(|_, modifier| modifier.removal != ModifierRemoval::OnDeath);
    if old_len != unit.attributes.modifiers.len() {
        unit.attributes.effective = compute(unit.stats.base_attributes, &unit.attributes.modifiers);
    }
}
