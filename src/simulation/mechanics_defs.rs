//! Reusable data definitions for attack traits and compound abilities.
use super::{Effect, ImpactDefinition, StatusSpec};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitKind {
    #[default]
    Other,
    Hero,
    Minion,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttackPassive {
    CurrentHealth {
        ratio: f32,
        minimum: f32,
        non_hero_cap: f32,
        target_cooldown: f32,
    },
    Frost {
        slow: f32,
        duration: f32,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct LineSpec {
    pub width: f32,
    pub effects: Vec<Effect>,
    pub link: Option<DashLinkSpec>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashLinkSpec {
    pub traverse_terrain: bool,
    pub can_cast_during_motion: bool,
    pub reach: f32,
    pub width: f32,
    pub speed: f32,
    pub knockup: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShieldSpec {
    pub base: f32,
    pub bonus_ad_ratio: f32,
    pub max_health_per_enemy_hero: f32,
    pub radius: f32,
    pub duration: f32,
    pub slow: Option<StatusSpec>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StandardSpec {
    pub impact: ImpactDefinition,
    pub duration: f32,
    pub aura_radius: f32,
    pub attack_speed: f32,
    pub passive_attack_speed: f32,
    pub vision_radius: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingTerrainSpec {
    pub radius: f32,
    pub thickness: f32,
    pub duration: f32,
    pub recast_delay: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LeapSpec {
    pub duration: f32,
    pub radius: f32,
    pub effects: Vec<Effect>,
    pub terrain: Option<RingTerrainSpec>,
    pub heroes_only: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmpowerSpec {
    pub required_stacks: u8,
    pub stack_duration: f32,
    pub stack_decay: f32,
    pub duration: f32,
    pub attack_speed: f32,
    pub attack_damage_multiplier: f32,
    pub arrows: u8,
    pub reset_attack: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct VolleySpec {
    pub count: u8,
    pub angle: f32,
    pub speed: f32,
    pub radius: f32,
    pub effects: Vec<Effect>,
    pub hero_slow: Option<StatusSpec>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoutSpec {
    pub speed: f32,
    pub vision_radius: f32,
    pub path_duration: f32,
    pub destination_duration: f32,
    pub charges: u8,
    pub initial_charges: u8,
    pub recharge: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowSpec {
    pub speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub radius: f32,
    pub impact_radius: f32,
    pub effects: Vec<Effect>,
    pub stun_min: f32,
    pub stun_max: f32,
    pub stun_distance: f32,
    pub vision_radius: f32,
}
