//! What the player could start right now, published for input hints and the HUD.
use super::hit::{Candidates, CombatWorld, target_valid};
use super::skills::Slot;
use super::{Combatant, Enemy, Health};
use crate::simulation::{
    ActionAvailability, Body, ComboWindow, DefenseState, MotionState, WeaponPair,
};
use bevy::prelude::*;

/// The follow-up stays open while the original target is still airborne (hammer)
/// or still stunned by the very action which opened it (chain axe).
pub(super) fn window_valid(combo: ComboWindow, enemies: &Candidates) -> bool {
    combo.remaining > 0.0
        && enemies
            .get(combo.original_target)
            .is_ok_and(|(_, _, motion, health, enemy, boss)| {
                target_valid(health, boss)
                    && match combo.pair {
                        WeaponPair::HammerMusket => !motion.is_grounded(),
                        WeaponPair::ChainAxeBow => boss.map_or_else(
                            || {
                                enemy.stun_source() == Some(combo.source_action_id)
                                    && enemy.stun_remaining() > 0.0
                            },
                            |boss| {
                                boss.stun_source() == Some(combo.source_action_id)
                                    && boss.stun_remaining() > 0.0
                            },
                        ),
                    }
            })
}
pub(super) fn availability_for(
    combat: &Combatant,
    motion: &MotionState,
    defense: &DefenseState,
    target_distance: Option<f32>,
) -> ActionAvailability {
    let ground = motion.is_grounded() && !motion.is_dodging() && !motion.is_sliding();
    let free =
        !combat.busy() && !defense.is_active() && !motion.is_dodging() && !motion.is_sliding();
    // A skill without a cast range needs no locked target.
    let castable = |slot| {
        combat
            .pair
            .action(slot)
            .spec()
            .cast_range
            .is_none_or(|range| target_distance.is_some_and(|distance| distance <= range))
    };
    let branch = combat.combo.is_some_and(|combo| {
        combat.action.as_ref().is_none_or(|action| {
            action.snapshot.action_id == combo.source_action_id
                && action.snapshot.kind.spec().opens_window()
        })
    });
    ActionAvailability {
        attack: free && combat.attack_cooldown == 0.0 && !combat.axe_away(),
        melee_skill: free
            && ground
            && combat.melee_cooldown == 0.0
            && !combat.axe_away()
            && castable(Slot::MeleeSkill),
        ranged_skill: free
            && ground
            && combat.ranged_cooldown == 0.0
            && castable(Slot::RangedSkill),
        combo: ground && !defense.is_active() && branch,
        defense: ground && !combat.busy(),
        equip: free && ground,
        movement_actions: !combat.busy(),
    }
}

type AvailabilityPlayer = (
    Entity,
    &'static Body,
    &'static MotionState,
    &'static DefenseState,
    &'static Health,
    &'static mut Combatant,
);

pub(in crate::simulation) fn publish_availability(
    world: CombatWorld,
    enemies: Candidates,
    mut players: Query<AvailabilityPlayer, Without<Enemy>>,
) {
    for (entity, body, motion, defense, health, mut combat) in &mut players {
        if !health.is_alive() {
            combat.clear_targets();
            continue;
        }
        let valid = |target| {
            enemies
                .get(target)
                .is_ok_and(|(_, _, _, h, _, boss)| target_valid(h, boss))
        };
        if combat.target.is_some_and(|t| !valid(t)) {
            combat.target = None;
        }
        if combat.combo.is_some_and(|c| !window_valid(c, &enemies)) {
            combat.combo = None;
        }
        if let Some(action) = combat.action.as_mut()
            && action.snapshot.target.is_some_and(|t| !valid(t))
        {
            action.cancel();
            combat.axe_away = false;
            combat.pull_velocity = None;
        }
        let distance = combat
            .target
            .and_then(|t| enemies.get(t).ok())
            .filter(|(t, b, _, _, _, _)| {
                world.visible(body.aim_position(), b.aim_position(), entity, *t)
            })
            .map(|(_, b, _, _, _, _)| b.world_position().distance(body.world_position()));
        combat.availability = availability_for(&combat, motion, defense, distance);
    }
}
