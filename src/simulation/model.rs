use super::abilities::AbilityId;
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
    pub move_speed: f32,
    pub radius: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_period: f32,
    pub attack_windup: f32,
}
impl Default for UnitStats {
    fn default() -> Self {
        Self {
            max_health: 600.0,
            max_mana: 300.0,
            mana_regen: 8.0,
            move_speed: 5.5,
            radius: 0.45,
            attack_range: 5.5,
            attack_damage: 40.0,
            attack_period: 0.8,
            attack_windup: 0.22,
        }
    }
}
impl UnitStats {
    pub(super) fn valid(&self) -> bool {
        [
            self.max_health,
            self.move_speed,
            self.radius,
            self.attack_range,
            self.attack_period,
        ]
        .into_iter()
        .all(|v| v.is_finite() && v > 0.0)
            && [
                self.max_mana,
                self.mana_regen,
                self.attack_damage,
                self.attack_windup,
            ]
            .into_iter()
            .all(|v| v.is_finite() && v >= 0.0)
            && self.attack_windup <= self.attack_period
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
    pub fn stats(&self) -> &UnitStats {
        &self.stats
    }
}
#[derive(Component, Debug)]
pub struct UnitState {
    pub(super) health: f32,
    pub(super) mana: f32,
    pub(super) attack_cooldown: f32,
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
        self.attack_cooldown
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
        remaining: f32,
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
        damage: f32,
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
}
#[derive(Message, Clone, Debug)]
pub enum SimulationEvent {
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
    pub(super) damage: f32,
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
    pub(super) damage: f32,
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
pub(super) struct PendingDamage(pub Vec<(Entity, Entity, f32)>);
#[derive(Resource, Default)]
pub struct SimulationControl {
    pub paused: bool,
}
#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);
