use super::{
    AttributeRejection, BaseAttributes, DamageOrigin, DamageType, EffectiveAttributes,
    EquipmentInstanceId, ModifierSourceId, StatusKind,
    abilities::{AbilityId, AbilityLoadout},
    attributes::AttributeState,
    effects::ImpactSnapshot,
};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    Ally,
    Enemy,
}
/// Combat properties, independent of a mode's hero/minion/base roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitClass {
    #[default]
    Actor,
    Structure,
}
impl UnitClass {
    pub(super) fn accepts(self, origin: DamageOrigin) -> bool {
        self == Self::Actor || origin == DamageOrigin::BasicAttack
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BasicAttackDelivery {
    Direct,
    Projectile { speed: f32, radius: f32 },
}
impl Default for BasicAttackDelivery {
    fn default() -> Self {
        Self::Projectile {
            speed: 22.0,
            radius: 0.12,
        }
    }
}
impl BasicAttackDelivery {
    pub(super) fn valid(self) -> bool {
        match self {
            Self::Direct => true,
            Self::Projectile { speed, radius } => {
                speed.is_finite() && speed > 0.0 && radius.is_finite() && radius > 0.0
            }
        }
    }
    pub(super) fn delivery(self) -> super::Delivery {
        match self {
            Self::Direct => super::Delivery::Direct,
            Self::Projectile { speed, radius } => super::Delivery::Projectile { speed, radius },
        }
    }
}
#[derive(Clone, Debug)]
pub struct UnitStats {
    pub class: UnitClass,
    pub kind: super::UnitKind,
    pub health_regen: f32,
    pub vision_radius: f32,
    pub attack_passives: Vec<super::AttackPassive>,
    pub attack_delivery: BasicAttackDelivery,
    pub max_health: f32,
    pub max_mana: f32,
    pub mana_regen: f32,
    pub radius: f32,
    pub base_attributes: BaseAttributes,
    pub attack_windup_fraction: f32,
}
impl Default for UnitStats {
    fn default() -> Self {
        Self {
            class: UnitClass::Actor,
            kind: super::UnitKind::Other,
            health_regen: 0.0,
            vision_radius: 0.0,
            attack_passives: Vec::new(),
            attack_delivery: BasicAttackDelivery::default(),
            max_health: 600.0,
            max_mana: 300.0,
            mana_regen: 8.0,
            radius: 0.45,
            base_attributes: BaseAttributes::default(),
            attack_windup_fraction: 0.275,
        }
    }
}
impl UnitStats {
    pub(super) fn valid(&self) -> bool {
        [self.max_health, self.radius]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
            && [
                self.max_mana,
                self.mana_regen,
                self.health_regen,
                self.vision_radius,
            ]
            .into_iter()
            .all(|v| v.is_finite() && v >= 0.0)
            && self.attack_windup_fraction.is_finite()
            && (0.0..=1.0).contains(&self.attack_windup_fraction)
            && self.base_attributes.valid()
            && self.attack_delivery.valid()
            && self.attack_passives.iter().all(|passive| match passive {
                super::AttackPassive::CurrentHealth {
                    ratio,
                    minimum,
                    non_hero_cap,
                    target_cooldown,
                } => {
                    ratio.is_finite()
                        && (0.0..=1.0).contains(ratio)
                        && minimum.is_finite()
                        && *minimum >= 0.0
                        && non_hero_cap.is_finite()
                        && *non_hero_cap >= *minimum
                        && super::abilities::positive_time(*target_cooldown)
                }
                super::AttackPassive::Frost { slow, duration } => {
                    slow.is_finite()
                        && *slow > 0.0
                        && *slow <= 1.0
                        && super::abilities::positive_time(*duration)
                }
            })
    }
}
#[derive(Clone, Debug)]
pub struct UnitSpec {
    pub name: String,
    pub team: Team,
    pub position: Vec2,
    pub stats: UnitStats,
    pub abilities: AbilityLoadout,
    pub respawn_delay: Option<f32>,
}
#[derive(Component, Debug)]
pub struct Unit {
    pub(super) name: String,
    pub(super) team: Team,
    pub(super) stats: UnitStats,
    pub(super) attributes: AttributeState,
    pub(super) home: Vec2,
    pub(super) respawn_delay: Option<f32>,
}
impl Unit {
    pub fn class(&self) -> UnitClass {
        self.stats.class
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn team(&self) -> Team {
        self.team
    }
    pub fn definition(&self) -> &UnitStats {
        &self.stats
    }
    pub fn attributes(&self) -> &EffectiveAttributes {
        self.attributes.effective()
    }
}
#[derive(Component, Debug)]
pub struct UnitState {
    pub(super) health: f32,
    pub(super) life_epoch: u64,
    pub(super) mana: f32,
    pub(super) attack_cooldown: u32,
    pub(super) respawn_remaining: Option<f32>,
    pub(super) last_damaged_tick: Option<u64>,
    pub(super) last_recovery_tick: Option<u64>,
}
/// Identity of one life; Entity alone does not distinguish respawns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitLifeRef {
    entity: Entity,
    epoch: u64,
}
impl UnitLifeRef {
    pub fn entity(self) -> Entity {
        self.entity
    }
    pub fn epoch(self) -> u64 {
        self.epoch
    }
    pub(super) fn is_current(self, world: &World) -> bool {
        world
            .get::<UnitState>(self.entity)
            .is_some_and(|s| s.alive() && s.life_epoch == self.epoch)
    }
}
impl UnitState {
    pub fn life_ref(&self, entity: Entity) -> UnitLifeRef {
        UnitLifeRef {
            entity,
            epoch: self.life_epoch,
        }
    }

    pub fn health(&self) -> f32 {
        self.health
    }
    pub fn mana(&self) -> f32 {
        self.mana
    }
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }
    pub fn attack_cooldown(&self) -> f32 {
        self.attack_cooldown as f32 * super::TICK_SECONDS
    }
    pub fn last_damaged_tick(&self) -> Option<u64> {
        self.last_damaged_tick
    }
}
#[derive(Component, Debug)]
pub struct UnitPose {
    pub(super) previous: Vec2,
    pub(super) position: Vec2,
    pub(super) facing: Vec2,
}
impl UnitPose {
    pub fn position(&self) -> Vec2 {
        self.position
    }
    pub fn facing(&self) -> Vec2 {
        self.facing
    }
    pub fn interpolated(&self, fraction: f32) -> Vec2 {
        self.previous.lerp(self.position, fraction.clamp(0.0, 1.0))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AbilityTarget {
    Direction(Vec2),
    Point(Vec2),
    Unit(Entity),
    SelfCast,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnitCommand {
    MoveTo(Vec2),
    AttackTarget(Entity),
    AttackMove(Vec2),
    CastAbility {
        slot: usize,
        target: AbilityTarget,
    },
    UseEquipment {
        slot: usize,
        instance: EquipmentInstanceId,
        target: AbilityTarget,
    },
    Stop,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastSource {
    AbilitySlot(usize),
    EquipmentSlot {
        slot: usize,
        instance: EquipmentInstanceId,
    },
}
#[derive(Resource, Default)]
pub struct CommandQueue(VecDeque<(Entity, UnitCommand)>);
impl CommandQueue {
    pub fn submit(&mut self, unit: Entity, command: UnitCommand) {
        self.0.push_back((unit, command));
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub(super) fn drain(&mut self) -> impl Iterator<Item = (Entity, UnitCommand)> + '_ {
        self.0.drain(..)
    }
}
#[derive(Component, Debug)]
pub struct AbilitySlots {
    pub(super) ids: AbilityLoadout,
    pub(super) cooldowns: [f32; 4],
}
impl AbilitySlots {
    pub fn ability(&self, slot: usize) -> Option<AbilityId> {
        self.ids.get(slot).copied().flatten()
    }
    pub fn cooldown(&self, slot: usize) -> Option<f32> {
        self.cooldowns.get(slot).copied()
    }
}
#[derive(Component, Clone, Debug, Default)]
pub enum UnitOrder {
    #[default]
    Idle,
    MoveTo(Vec2),
    AttackTarget(UnitLifeRef),
    /// One targeted cast awaiting a reachable position inside its range.
    CastTarget {
        source: CastSource,
        ability: AbilityId,
        target: UnitLifeRef,
    },
    AttackMove {
        destination: Vec2,
        target: Option<UnitLifeRef>,
    },
}
#[derive(Component, Clone, Debug, Default)]
pub enum UnitAction {
    #[default]
    Ready,
    AttackWindup {
        target: UnitLifeRef,
        remaining: u32,
        interval: u32,
    },
    /// Cancellable recovery pose after a committed basic attack. The separate
    /// attack cooldown remains authoritative even if this action is cancelled.
    AttackBackswing {
        remaining: u32,
        duration: u32,
    },
    Casting {
        source: CastSource,
        ability: AbilityId,
        target: AbilityTarget,
        target_life: Option<UnitLifeRef>,
        remaining: f32,
    },
    Channeling {
        remaining: f32,
        next_pulse: f32,
        interval: f32,
        pulse: ImpactSnapshot,
        recovery: f32,
    },
    Recovery {
        remaining: f32,
    },
}
impl UnitAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::AttackWindup { .. } => "Attacking",
            Self::AttackBackswing { .. } => "AttackBackswing",
            Self::Casting { .. } => "Casting",
            Self::Channeling { .. } => "Channeling",
            Self::Recovery { .. } => "Recovery",
        }
    }
}
#[derive(Component, Default)]
pub(super) struct Movement {
    pub route: VecDeque<Vec2>,
    pub goal: Option<Vec2>,
    pub repath_remaining: f32,
    pub dash: Option<(Vec2, f32)>,
    pub forced: Option<ForcedMotion>,
    pub motion_sequence: u64,
}
pub(super) struct ForcedMotion {
    pub id: u64,
    pub target: UnitLifeRef,
    pub destination: Vec2,
    pub speed: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Rejection {
    InvalidUnit,
    Dead,
    InvalidTarget,
    OutOfRange,
    NoAbility,
    OnCooldown,
    InsufficientMana,
    InsufficientStacks,
    NoCharges,
    NoPath,
    Busy,
    InvalidDamage,
    Controlled,
}
#[derive(Message, Clone, Debug)]
pub enum SimulationEvent {
    StatusApplied {
        unit: Entity,
        instance: u64,
        kind: StatusKind,
        source: UnitLifeRef,
    },
    StatusExpired {
        unit: Entity,
        instance: u64,
        kind: StatusKind,
        source: UnitLifeRef,
    },
    AttributeChangeRejected {
        unit: Entity,
        source: ModifierSourceId,
        reason: AttributeRejection,
    },
    CommandRejected {
        unit: Entity,
        reason: Rejection,
    },
    MoveOrdered {
        unit: Entity,
        destination: Vec2,
    },
    AttackReleased {
        source: Entity,
        target: Entity,
    },
    AbilityCommitted {
        unit: Entity,
        ability: AbilityId,
        position: Vec2,
    },
    /// The landing remains committed, but nearby actors cannot safely leave
    /// the proposed terrain. No wall or partial relocation was applied.
    TerrainPlacementFailed {
        unit: Entity,
        position: Vec2,
    },
    Damaged {
        source: Entity,
        target: Entity,
        amount: f32,
        kind: DamageType,
        origin: DamageOrigin,
    },
    Died {
        unit: Entity,
    },
    Respawned {
        unit: Entity,
    },
}
#[derive(Component, Debug)]
pub struct Projectile {
    pub(super) payload: ImpactSnapshot,
    pub(super) carrier: u64,
    pub(super) position: Vec2,
    pub(super) previous: Vec2,
    pub(super) direction: Vec2,
    pub(super) remaining: f32,
    pub(super) speed: f32,
    pub(super) radius: f32,
    pub(super) homing: Option<UnitLifeRef>,
    pub(super) behavior: super::mechanics::ProjectileBehavior,
    pub(super) arrows: u8,
}
impl Projectile {
    pub fn arrows(&self) -> u8 {
        self.arrows
    }
    pub fn is_crystal_arrow(&self) -> bool {
        matches!(
            self.behavior,
            super::mechanics::ProjectileBehavior::Arrow { .. }
        )
    }
    pub fn position(&self) -> Vec2 {
        self.position
    }
    pub fn direction(&self) -> Vec2 {
        self.direction
    }
    pub fn team(&self) -> Team {
        self.payload.team
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn interpolated(&self, fraction: f32) -> Vec2 {
        self.previous.lerp(self.position, fraction.clamp(0.0, 1.0))
    }
}
#[derive(Component, Debug)]
pub struct AreaEffect {
    pub(super) payload: ImpactSnapshot,
    pub(super) carrier: u64,
    pub(super) position: Vec2,
    pub(super) radius: f32,
    pub(super) remaining: f32,
    pub(super) duration: f32,
}
impl AreaEffect {
    pub fn team(&self) -> Team {
        self.payload.team
    }
    pub fn position(&self) -> Vec2 {
        self.position
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn progress(&self) -> f32 {
        (1.0 - self.remaining / self.duration.max(0.001)).clamp(0.0, 1.0)
    }
}
#[derive(Resource, Default)]
pub struct SimulationControl {
    pub paused: bool,
}
#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);
