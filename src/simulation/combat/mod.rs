//! Fixed-step combat: admission, action timelines, hit resolution and ordinary
//! enemies. Rendering never creates damage.
//!
//! Skill numbers live in [`skills`]; the systems below only run the machinery.
mod arsenal;
mod availability;
mod combatant;
mod companion;
mod enemy;
mod energy;
mod feedback;
mod health;
mod hit;
// Generated data: its layout belongs to the baker, not to rustfmt.
#[rustfmt::skip]
#[allow(clippy::approx_constant, clippy::excessive_precision)]
mod hit_tracks;
mod prepare;
mod projectile;
mod resolve;
mod runtime;
mod skills;
mod sweep;

pub use arsenal::Arsenal;
pub(super) use availability::publish_availability;
pub use combatant::{CombatIntent, CombatStatus, Combatant};
pub(crate) use companion::ensure_companions;
pub(super) use companion::publish_companions;
pub use enemy::Enemy;
pub(super) use enemy::advance_enemies;
pub use energy::Energy;
pub(super) use energy::{drain_rage, release_rage};
pub(super) use feedback::ActionIds;
pub use feedback::{CombatEffect, CombatEvent, CombatFeedback};
pub use health::{Health, SpawnPoint};
pub(super) use hit::{CombatWorld, apply_hit, settle_defense};
pub use hit::{HitContext, HitOutcome};
pub(super) use prepare::{prepare, tick_slows};
pub(super) use projectile::advance_projectiles;
pub use projectile::{Projectile, clear_projectiles};
pub(super) use resolve::{recover_stamina, resolve_actions};
pub use sweep::{HitProbe, HitProbes, HitTrack, Hurtbox, ProbeShape};
pub(super) use sweep::{strike, tick_probes, ticks_between};

#[cfg(test)]
mod tests;
