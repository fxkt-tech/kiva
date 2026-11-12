//! One ordering of defenses and damage shared by players, enemies and the Boss,
//! plus the spatial queries and entity views the rule systems share.
use super::skills::Control;
use super::{
    CombatEffect, CombatEvent, CombatFeedback, CombatIntent, Combatant, Enemy, Health, SpawnPoint,
};
use crate::simulation::actions::GRAVITY;
use crate::simulation::{
    ActionIntent, ActionSnapshot, Body, Boss, DefenseKind, DefenseState, Facing, MeleeStage,
    MotionState, MoveIntent, SlowEffect, Stamina, WeaponPair,
};
use avian3d::prelude::{Sensor, SpatialQuery, SpatialQueryFilter};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitOutcome {
    Invalid,
    Invulnerable,
    Dodged,
    Deflected,
    Blocked,
    Parried,
    Damaged,
}
#[derive(Clone, Copy, Debug)]
pub struct HitContext {
    pub source: Entity,
    pub source_owner: Entity,
    pub pair: Option<WeaponPair>,
    pub melee_stage: Option<MeleeStage>,
    pub target: Entity,
    pub action_id: u64,
    pub hit_index: u8,
    pub damage: f32,
    pub origin: Vec3,
}
/// All candidates have already passed their attack-specific range/LoS checks.
/// Enemy and Boss use this same defense ordering and damage/event application.
#[allow(clippy::too_many_arguments)]
pub(in crate::simulation) fn apply_hit(
    ctx: HitContext,
    body: &Body,
    facing: &Facing,
    motion: &MotionState,
    health: &mut Health,
    defense: Option<&mut DefenseState>,
    mut stamina: Option<&mut Stamina>,
    invulnerable: bool,
    feedback: &mut CombatFeedback,
) -> HitOutcome {
    if !health.is_alive() {
        return HitOutcome::Invalid;
    }
    if invulnerable {
        return HitOutcome::Invulnerable;
    }
    if let Some(s) = stamina.as_deref_mut() {
        s.disturb();
    }
    let frontal = facing
        .direction()
        .dot((ctx.origin - body.aim_position()).xz().normalize_or_zero())
        > 0.25;
    let kind = defense.as_ref().map_or(DefenseKind::Idle, |d| d.kind());
    let outcome = if motion.is_invulnerable() {
        HitOutcome::Dodged
    } else if frontal && kind == DefenseKind::Deflect {
        HitOutcome::Deflected
    } else if frontal && kind == DefenseKind::Parry {
        HitOutcome::Parried
    } else if frontal
        && kind == DefenseKind::Guard
        && stamina.as_ref().is_some_and(|s| s.can_block())
    {
        HitOutcome::Blocked
    } else {
        HitOutcome::Damaged
    };
    if outcome == HitOutcome::Blocked {
        stamina.as_mut().unwrap().block();
    }
    if matches!(outcome, HitOutcome::Blocked | HitOutcome::Damaged) {
        health.damage(
            ctx.damage
                * if outcome == HitOutcome::Blocked {
                    0.5
                } else {
                    1.0
                },
        );
    }
    let kind = match outcome {
        HitOutcome::Dodged => CombatEffect::Dodge,
        HitOutcome::Deflected => CombatEffect::Deflect,
        HitOutcome::Parried => CombatEffect::Parry,
        HitOutcome::Blocked => CombatEffect::Block,
        _ => CombatEffect::Hit,
    };
    feedback.emit(CombatEvent {
        sequence: 0,
        actor: ctx.target,
        source_owner: ctx.source_owner,
        pair: ctx.pair,
        melee_stage: ctx.melee_stage,
        action_id: ctx.action_id,
        hit_index: ctx.hit_index,
        kind,
        position: body.aim_position(),
        end: ctx.origin,
        radius: 0.65,
    });
    if !health.is_alive() {
        if let Some(d) = defense {
            d.cancel();
        }
        feedback.emit(CombatEvent {
            sequence: 0,
            actor: ctx.target,
            source_owner: ctx.source_owner,
            pair: ctx.pair,
            melee_stage: ctx.melee_stage,
            action_id: ctx.action_id,
            hit_index: ctx.hit_index,
            kind: CombatEffect::Defeat,
            position: body.aim_position(),
            end: body.aim_position(),
            radius: 0.9,
        });
    }
    outcome
}
#[derive(SystemParam)]
pub(in crate::simulation) struct CombatWorld<'w, 's> {
    pub spatial: SpatialQuery<'w, 's>,
    pub sensors: Query<'w, 's, (), With<Sensor>>,
}
impl CombatWorld<'_, '_> {
    pub fn visible(&self, start: Vec3, end: Vec3, observer: Entity, target: Entity) -> bool {
        let delta = end - start;
        let Ok(direction) = Dir3::new(delta) else {
            return true;
        };
        self.spatial
            .cast_ray_predicate(
                start,
                direction,
                delta.length(),
                true,
                &SpatialQueryFilter::from_excluded_entities([observer, target]),
                &|e| !self.sensors.contains(e),
            )
            .is_none()
    }
}

/// Read-only enemy facts for admission and targeting.
pub(in crate::simulation) type CandidateData = (
    Entity,
    &'static Body,
    &'static MotionState,
    &'static Health,
    &'static Enemy,
    Option<&'static Boss>,
);
pub(in crate::simulation) type Candidates<'w, 's> = Query<'w, 's, CandidateData, With<Enemy>>;
pub(in crate::simulation) type PlayerData = (
    Entity,
    &'static mut Body,
    &'static mut MotionState,
    &'static Facing,
    &'static mut MoveIntent,
    &'static mut ActionIntent,
    &'static mut Health,
    &'static mut Combatant,
    &'static mut CombatIntent,
    &'static SpawnPoint,
    &'static mut DefenseState,
    &'static mut Stamina,
);
pub(in crate::simulation) type EnemyData = (
    Entity,
    &'static mut Body,
    &'static mut MotionState,
    &'static mut Facing,
    &'static mut MoveIntent,
    &'static mut ActionIntent,
    &'static mut Health,
    &'static mut Enemy,
    &'static SpawnPoint,
    Option<&'static mut Boss>,
    &'static mut SlowEffect,
);
pub(in crate::simulation) type Players<'w, 's> = Query<'w, 's, PlayerData, Without<Enemy>>;
pub(in crate::simulation) type Enemies<'w, 's> = Query<'w, 's, EnemyData, With<Enemy>>;

pub(super) fn target_valid(health: &Health, boss: Option<&Boss>) -> bool {
    health.is_alive() && boss.is_none_or(|b| !b.is_invulnerable())
}

/// Falling out of the arena defeats an actor; the respawn flow recovers it.
pub(super) fn defeat_if_fallen(
    entity: Entity,
    body: &Body,
    health: &mut Health,
    feedback: &mut CombatFeedback,
) {
    if body.world_position().y < -12.0 && health.is_alive() {
        let maximum = health.maximum();
        health.damage(maximum);
        feedback.push(
            entity,
            CombatEffect::Defeat,
            body.aim_position(),
            body.aim_position(),
            0.9,
        );
    }
}

/// One player hit on one enemy.
#[derive(Clone, Copy)]
pub(super) struct Blow {
    pub target: Entity,
    /// The performer: the player, or the logical companion for shots.
    pub source: Entity,
    pub index: u8,
    pub damage: f32,
    pub control: Control,
    pub origin: Vec3,
}

/// Returns true when the target took the damage and survived, which is also
/// the only case in which the control effect is applied.
pub(super) fn hit_enemy(
    blow: Blow,
    owner: Entity,
    action: &ActionSnapshot,
    enemies: &mut Enemies,
    feedback: &mut CombatFeedback,
) -> bool {
    let Ok((
        _,
        body,
        mut motion,
        facing,
        mut movement,
        _,
        mut health,
        mut enemy,
        _,
        mut boss,
        mut slow,
    )) = enemies.get_mut(blow.target)
    else {
        return false;
    };
    let outcome = apply_hit(
        HitContext {
            source: blow.source,
            source_owner: owner,
            pair: Some(action.pair),
            melee_stage: action.melee_stage,
            target: blow.target,
            action_id: action.action_id,
            hit_index: blow.index,
            damage: blow.damage,
            origin: blow.origin,
        },
        &body,
        &facing,
        &motion,
        &mut health,
        None,
        None,
        boss.as_ref().is_some_and(|b| b.is_invulnerable()),
        feedback,
    );
    if outcome != HitOutcome::Damaged || !health.is_alive() {
        return false;
    }
    match blow.control {
        Control::None => {}
        Control::Launch(height) => {
            motion.launch_up((2.0 * GRAVITY * height).sqrt());
            movement.stop();
        }
        Control::Stun(duration) => {
            if let Some(boss) = boss.as_deref_mut() {
                boss.stun(action.action_id, duration);
            } else {
                enemy.stun(action.action_id, duration);
            }
            movement.stop();
        }
        Control::Slow { factor, duration } => slow.apply(factor, duration),
    }
    true
}

/// The target's aim point when a shot from `source` can still reach it.
pub(super) fn valid_shot(
    world: &CombatWorld,
    source: Vec3,
    owner: (Entity, &Body),
    target: Entity,
    range: f32,
    enemies: &mut Enemies,
) -> Option<Vec3> {
    let Ok((_, body, _, _, _, _, health, _, _, boss, _)) = enemies.get_mut(target) else {
        return None;
    };
    (target_valid(&health, boss.as_deref())
        && owner.1.world_position().distance(body.world_position()) <= range
        && world.visible(source, body.aim_position(), owner.0, target))
    .then_some(body.aim_position())
}

/// Whether `target` no longer exists or can no longer be attacked.
pub(super) fn target_lost(target: Entity, enemies: &mut Enemies) -> bool {
    enemies
        .get_mut(target)
        .map_or(true, |(_, _, _, _, _, _, health, _, _, boss, _)| {
            !target_valid(&health, boss.as_deref())
        })
}
