//! Damage expressions are snapshotted at commit; defenses are read at impact.
use super::{AbilityId, EffectiveAttributes, EquipmentId, EquipmentInstanceId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Magical,
    True,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageSpec {
    pub kind: DamageType,
    pub base: f32,
    pub ad_ratio: f32,
    pub ap_ratio: f32,
}
impl DamageSpec {
    pub const fn physical(base: f32) -> Self {
        Self {
            kind: DamageType::Physical,
            base,
            ad_ratio: 0.0,
            ap_ratio: 0.0,
        }
    }
    pub const fn magical(base: f32) -> Self {
        Self {
            kind: DamageType::Magical,
            base,
            ad_ratio: 0.0,
            ap_ratio: 0.0,
        }
    }
    pub(super) fn valid(self) -> bool {
        [self.base, self.ad_ratio, self.ap_ratio]
            .into_iter()
            .all(|v| v.is_finite() && v >= 0.0)
    }
    pub(super) fn snapshot(
        self,
        attributes: &EffectiveAttributes,
        origin: DamageOrigin,
    ) -> Option<DamageSnapshot> {
        let raw = f64::from(self.base)
            + f64::from(self.ad_ratio) * f64::from(attributes.attack_damage())
            + f64::from(self.ap_ratio) * f64::from(attributes.ability_power());
        if !self.valid() || !raw.is_finite() || raw > f64::from(f32::MAX) {
            return None;
        }
        Some(DamageSnapshot {
            raw: raw as f32,
            kind: self.kind,
            origin,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageOrigin {
    BasicAttack,
    Ability(AbilityId),
    EquipmentActive {
        equipment: EquipmentId,
        instance: EquipmentInstanceId,
        ability: AbilityId,
    },
    EquipmentPassive {
        equipment: EquipmentId,
        instance: EquipmentInstanceId,
        passive: usize,
    },
}
/// A committed effect's immutable damage. Construction stays inside simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageSnapshot {
    raw: f32,
    kind: DamageType,
    origin: DamageOrigin,
}
impl DamageSnapshot {
    pub fn raw(&self) -> f32 {
        self.raw
    }
    pub fn kind(&self) -> DamageType {
        self.kind
    }
    pub fn origin(&self) -> DamageOrigin {
        self.origin
    }
    pub(super) fn health_after(self, attributes: &EffectiveAttributes, health: f32) -> f32 {
        let resistance = match self.kind {
            DamageType::Physical => attributes.armor(),
            DamageType::Magical => attributes.magic_resistance(),
            DamageType::True => return (f64::from(health) - f64::from(self.raw)).max(0.0) as f32,
        };
        let resistance = f64::from(resistance);
        let multiplier = if resistance >= 0.0 {
            100.0 / (100.0 + resistance)
        } else {
            2.0 - 100.0 / (100.0 - resistance)
        };
        (f64::from(health) - f64::from(self.raw) * multiplier).max(0.0) as f32
    }
}
