//! Fixed-step combat: admission, action timelines, hit resolution and ordinary
//! enemies. Rendering never creates damage.
//!
//! Skill numbers live in [`skills`]; the systems below only run the machinery.
mod availability;
mod combatant;
mod companion;
mod enemy;
mod feedback;
mod health;
mod hit;
mod prepare;
mod resolve;
mod runtime;
mod skills;

pub(super) use availability::publish_availability;
pub use combatant::{CombatIntent, CombatStatus, Combatant};
pub(crate) use companion::ensure_companions;
pub(super) use companion::publish_companions;
pub use enemy::Enemy;
pub(super) use enemy::advance_enemies;
pub(super) use feedback::ActionIds;
pub use feedback::{CombatEffect, CombatEvent, CombatFeedback};
pub use health::{Health, SpawnPoint};
pub(super) use hit::apply_hit;
pub use hit::{HitContext, HitOutcome};
pub(super) use prepare::{prepare, tick_slows};
pub(super) use resolve::{recover_stamina, resolve_actions};

#[cfg(test)]
mod tests;
