//! Mode tuning is configuration, independent of AI decisions and rendering.
use super::units::world_units;
use crate::simulation::*;
use bevy::prelude::*;

pub(super) const MINIONS_PER_TEAM: u8 = 6;

#[derive(Resource, Clone, Debug)]
pub struct MatchDefinition {
    default_character: CharacterId,
    opponent_character: Option<CharacterId>,
    wave_interval_ticks: u64,
    minion_spawn_interval_ticks: u64,
    hero_respawn_delay: f32,
    base_stats: UnitStats,
    melee_stats: UnitStats,
    ranged_stats: UnitStats,
    recovery: RecoverySpec,
}

impl Default for MatchDefinition {
    fn default() -> Self {
        Self {
            default_character: CharacterId(1),
            opponent_character: None,
            wave_interval_ticks: (60.0 * TICK_RATE) as u64,
            minion_spawn_interval_ticks: (0.5 * TICK_RATE) as u64,
            hero_respawn_delay: 3.0,
            base_stats: UnitStats {
                class: UnitClass::Structure,
                vision_radius: world_units(1350.0),
                attack_delivery: BasicAttackDelivery::Direct,
                max_health: 3000.0,
                max_mana: 0.0,
                mana_regen: 0.0,
                radius: 1.6,
                base_attributes: BaseAttributes {
                    attack_damage: 0.0,
                    armor: 20.0,
                    magic_resistance: 20.0,
                    move_speed: 0.0,
                    attack_speed: 0.0,
                    attack_range: 0.0,
                    ..default()
                },
                ..default()
            },
            melee_stats: UnitStats {
                kind: UnitKind::Minion,
                vision_radius: world_units(1200.0),
                attack_delivery: BasicAttackDelivery::Direct,
                max_health: 465.0,
                max_mana: 0.0,
                mana_regen: 0.0,
                radius: 0.3,
                base_attributes: BaseAttributes {
                    attack_damage: 11.0,
                    move_speed: world_units(350.0),
                    attack_speed: 1.25,
                    attack_range: world_units(110.0),
                    ..default()
                },
                ..default()
            },
            ranged_stats: UnitStats {
                kind: UnitKind::Minion,
                vision_radius: world_units(1200.0),
                attack_delivery: BasicAttackDelivery::Projectile {
                    speed: 14.0,
                    radius: 0.1,
                },
                max_health: 284.0,
                max_mana: 0.0,
                mana_regen: 0.0,
                radius: 0.28,
                base_attributes: BaseAttributes {
                    attack_damage: 21.0,
                    move_speed: world_units(350.0),
                    attack_speed: 0.667,
                    attack_range: world_units(550.0),
                    ..default()
                },
                ..default()
            },
            recovery: RecoverySpec {
                radius: 2.5,
                health_fraction_per_second: 0.20,
                mana_fraction_per_second: 0.20,
                out_of_combat_ticks: 0,
            },
        }
    }
}

impl MatchDefinition {
    pub fn default_character(&self) -> CharacterId {
        self.default_character
    }
    pub fn opponent_character(&self) -> CharacterId {
        self.opponent_for(self.default_character)
    }
    pub fn opponent_for(&self, player: CharacterId) -> CharacterId {
        self.opponent_character
            .unwrap_or_else(|| super::content::default_opponent(player))
    }
    pub fn wave_interval_ticks(&self) -> u64 {
        self.wave_interval_ticks
    }
    pub fn minion_spawn_interval_ticks(&self) -> u64 {
        self.minion_spawn_interval_ticks
    }
    pub fn hero_respawn_delay(&self) -> f32 {
        self.hero_respawn_delay
    }
    pub fn base_stats(&self) -> &UnitStats {
        &self.base_stats
    }
    pub fn melee_stats(&self) -> &UnitStats {
        &self.melee_stats
    }
    pub fn ranged_stats(&self) -> &UnitStats {
        &self.ranged_stats
    }
    pub fn recovery(&self) -> RecoverySpec {
        self.recovery
    }

    /// Configuration builders are applied before a match starts. They never
    /// mutate a running unit; start_match validates the complete definition.
    pub fn with_opponent(mut self, character: CharacterId) -> Self {
        self.opponent_character = Some(character);
        self
    }
    pub fn with_base_stats(mut self, stats: UnitStats) -> Self {
        self.base_stats = stats;
        self
    }
    pub fn with_wave_interval_ticks(mut self, ticks: u64) -> Self {
        self.wave_interval_ticks = ticks;
        self
    }
    pub fn with_minion_spawn_interval_ticks(mut self, ticks: u64) -> Self {
        self.minion_spawn_interval_ticks = ticks;
        self
    }

    pub(super) fn valid(&self) -> bool {
        self.wave_interval_ticks > 0
            && self.minion_spawn_interval_ticks > 0
            && self
                .minion_spawn_interval_ticks
                .checked_mul(u64::from(MINIONS_PER_TEAM - 1))
                .is_some_and(|last| last < self.wave_interval_ticks)
            && self.hero_respawn_delay.is_finite()
            && self.hero_respawn_delay > 0.0
            && self.recovery.valid()
            && self.base_stats.class == UnitClass::Structure
            && self.melee_stats.class == UnitClass::Actor
            && self.ranged_stats.class == UnitClass::Actor
    }
}
