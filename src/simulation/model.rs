use super::{
    AttributeRejection, BaseAttributes, DamageOrigin, DamageSnapshot, DamageType,
    EffectiveAttributes, ModifierSourceId, abilities::AbilityId, attributes::AttributeState,
    damage::DamageRequest,
};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    Ally,
    Enemy,
}
#[derive(Clone, Debug)]
pub struct UnitStats {
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
            && [self.max_mana, self.mana_regen]
                .into_iter()
                .all(|v| v.is_finite() && v >= 0.0)
            && self.attack_windup_fraction.is_finite()
            && (0.0..=1.0).contains(&self.attack_windup_fraction)
            && self.base_attributes.valid()
    }
}
#[derive(Clone, Debug)]
pub struct UnitSpec {
    pub name: String,
    pub team: Team,
    pub position: Vec2,
    pub stats: UnitStats,
    pub abilities: [Option<AbilityId>; 4],
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
    pub(super) mana: f32,
    pub(super) attack_cooldown: u32,
    pub(super) respawn_remaining: Option<f32>,
}
impl UnitState {
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
    CastAbility { slot: usize, target: AbilityTarget },
    Stop,
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
    pub(super) ids: [Option<AbilityId>; 4],
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
    AttackTarget(Entity),
    AttackMove {
        destination: Vec2,
        target: Option<Entity>,
    },
}
#[derive(Component, Clone, Debug, Default)]
pub enum UnitAction {
    #[default]
    Ready,
    AttackWindup {
        target: Entity,
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
        slot: usize,
        ability: AbilityId,
        target: AbilityTarget,
        remaining: f32,
    },
    Channeling {
        remaining: f32,
        next_pulse: f32,
        interval: f32,
        radius: f32,
        damage: DamageSnapshot,
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
    NoPath,
    Busy,
    InvalidDamage,
}
#[derive(Message, Clone, Debug)]
pub enum SimulationEvent {
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
    pub(super) source: Entity,
    pub(super) team: Team,
    pub(super) position: Vec2,
    pub(super) previous: Vec2,
    pub(super) direction: Vec2,
    pub(super) remaining: f32,
    pub(super) speed: f32,
    pub(super) radius: f32,
    pub(super) damage: DamageSnapshot,
    pub(super) homing: Option<Entity>,
}
impl Projectile {
    pub fn position(&self) -> Vec2 {
        self.position
    }
    pub fn team(&self) -> Team {
        self.team
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
    pub(super) source: Entity,
    pub(super) team: Team,
    pub(super) position: Vec2,
    pub(super) radius: f32,
    pub(super) remaining: f32,
    pub(super) duration: f32,
    pub(super) damage: DamageSnapshot,
}
impl AreaEffect {
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
pub(super) struct PendingDamage(pub Vec<DamageRequest>);
#[derive(Resource, Default)]
pub struct SimulationControl {
    pub paused: bool,
}
#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);
