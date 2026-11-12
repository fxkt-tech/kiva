//! The ordinary enemy: chase, wind up, strike, recover. The Boss owns its own
//! component and is skipped here.
use super::hit::{CombatWorld, Enemies, Players, apply_hit, defeat_if_fallen};
use super::{ActionIds, CombatEffect, CombatFeedback, HitContext, HitOutcome};
use crate::simulation::defense::decay;
use crate::simulation::{Body, SlowEffect};
use bevy::prelude::*;

/// Kiva's ordinary enemy tuning.
const FIRST_ATTACK_DELAY: f32 = 0.8;
const ATTACK_COOLDOWN: f32 = 1.25;
const WINDUP: f32 = 0.42;
const DAMAGE: f32 = 12.0;
const REACH: f32 = 2.0;
const ENGAGE_DISTANCE: f32 = 1.7;
const SIGHT: f32 = 12.0;
const PARRIED_STUN: f32 = 0.80;

#[derive(Component)]
#[require(SlowEffect)]
pub struct Enemy {
    pub(super) passive: bool,
    pub(super) cooldown: f32,
    pub(super) windup: f32,
    pub(super) recovery: f32,
    pub(super) stun_source: Option<u64>,
    pub(super) attack_target: Option<Entity>,
    pub(super) attack_id: u64,
}
impl Default for Enemy {
    fn default() -> Self {
        Self {
            passive: false,
            cooldown: FIRST_ATTACK_DELAY,
            windup: 0.0,
            recovery: 0.0,
            stun_source: None,
            attack_target: None,
            attack_id: 0,
        }
    }
}
impl Enemy {
    pub fn target_dummy() -> Self {
        Self {
            passive: true,
            ..default()
        }
    }
    pub fn is_passive(&self) -> bool {
        self.passive
    }
    pub fn is_winding_up(&self) -> bool {
        self.windup > 0.0
    }
    pub fn windup_remaining(&self) -> f32 {
        self.windup
    }
    pub fn is_recovering(&self) -> bool {
        self.recovery > 0.0
    }
    pub fn recovery_remaining(&self) -> f32 {
        self.recovery
    }
    pub fn stun_source(&self) -> Option<u64> {
        (self.recovery > 0.0).then_some(self.stun_source).flatten()
    }
    pub fn stun_remaining(&self) -> f32 {
        if self.stun_source().is_some() {
            self.recovery
        } else {
            0.0
        }
    }
    pub(super) fn stun(&mut self, source: u64, duration: f32) {
        self.windup = 0.0;
        self.attack_target = None;
        self.recovery = duration;
        self.stun_source = Some(source);
    }
    fn reset(&mut self) {
        *self = if self.passive {
            Self::target_dummy()
        } else {
            Self::default()
        };
    }
}
pub(in crate::simulation) fn advance_enemies(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut players: Players,
    mut enemies: Enemies,
    mut ids: ResMut<ActionIds>,
) {
    let dt = time.delta_secs();
    for (
        entity,
        mut body,
        mut motion,
        mut facing,
        mut movement,
        mut actions,
        mut health,
        mut enemy,
        spawn,
        boss,
        mut slow,
    ) in &mut enemies
    {
        if boss.is_some() {
            continue;
        }
        actions.clear();
        enemy.cooldown = decay(enemy.cooldown, dt);
        defeat_if_fallen(entity, &body, &mut health, &mut feedback);
        let was_dead = !health.is_alive();
        if health.tick(dt) {
            body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
            motion.reset();
            enemy.reset();
            *slow = SlowEffect::default();
            feedback.push(
                entity,
                CombatEffect::Respawn,
                body.aim_position(),
                body.aim_position(),
                0.8,
            );
        }
        if was_dead {
            movement.stop();
            enemy.windup = 0.0;
            enemy.attack_target = None;
            continue;
        }
        if enemy.recovery > 0.0 {
            enemy.recovery = decay(enemy.recovery, dt);
            if enemy.recovery == 0.0 {
                enemy.stun_source = None;
            }
            movement.stop();
            continue;
        }
        if enemy.is_passive() || !motion.is_grounded() {
            movement.stop();
            enemy.windup = 0.0;
            enemy.attack_target = None;
            continue;
        }
        if enemy.windup > 0.0 {
            movement.stop();
            enemy.windup = decay(enemy.windup, dt);
            if enemy.windup == 0.0 {
                enemy.cooldown = ATTACK_COOLDOWN;
                feedback.push(
                    entity,
                    CombatEffect::Melee,
                    body.aim_position(),
                    body.aim_position()
                        + Vec3::new(facing.direction().x, 0.0, facing.direction().y) * REACH,
                    REACH,
                );
                if let Some(target) = enemy.attack_target.take() {
                    let parried = land_attack(
                        (entity, &body),
                        (target, enemy.attack_id),
                        &world,
                        &mut players,
                        &mut feedback,
                    );
                    if parried {
                        let id = enemy.attack_id;
                        enemy.stun(id, PARRIED_STUN);
                        movement.stop();
                    }
                }
            }
            continue;
        }
        let target = players
            .iter()
            .filter(|(_, _, _, _, _, _, h, _, _, _, _, _)| h.is_alive())
            .map(|(e, b, _, _, _, _, _, _, _, _, _, _)| (e, b.world_position(), b.aim_position()))
            .filter(|(_, p, _)| p.distance(body.world_position()) < SIGHT)
            .min_by(|a, b| {
                a.1.distance_squared(body.world_position())
                    .total_cmp(&b.1.distance_squared(body.world_position()))
            });
        let Some((target, position, aim)) = target else {
            movement.stop();
            continue;
        };
        let delta = position - body.world_position();
        let dir = delta.xz().normalize_or_zero();
        facing.set(dir);
        if delta.length() <= ENGAGE_DISTANCE
            && world.visible(body.aim_position(), aim, entity, target)
        {
            movement.stop();
            if enemy.cooldown == 0.0 {
                enemy.windup = WINDUP;
                enemy.attack_target = Some(target);
                enemy.attack_id = ids.next();
                feedback.push(
                    entity,
                    CombatEffect::EnemyWindup,
                    body.aim_position(),
                    aim,
                    REACH,
                );
            }
        } else {
            movement.set(dir);
        }
    }
}

/// A finished wind-up reaches its chosen player only if they are still alive,
/// in reach and visible. Returns whether the player parried it.
fn land_attack(
    attacker: (Entity, &Body),
    attack: (Entity, u64),
    world: &CombatWorld,
    players: &mut Players,
    feedback: &mut CombatFeedback,
) -> bool {
    let (entity, body) = attacker;
    let (target, attack_id) = attack;
    let Ok((
        _,
        target_body,
        target_motion,
        target_facing,
        _,
        mut target_actions,
        mut target_health,
        mut target_combat,
        mut target_intent,
        _,
        mut defense,
        mut stamina,
    )) = players.get_mut(target)
    else {
        return false;
    };
    if !target_health.is_alive()
        || body.world_position().distance(target_body.world_position()) > REACH
        || !world.visible(
            body.aim_position(),
            target_body.aim_position(),
            entity,
            target,
        )
    {
        return false;
    }
    let result = apply_hit(
        HitContext {
            source: entity,
            source_owner: entity,
            pair: None,
            melee_stage: None,
            target,
            action_id: attack_id,
            hit_index: 0,
            damage: DAMAGE,
            origin: body.aim_position(),
        },
        &target_body,
        target_facing,
        &target_motion,
        &mut target_health,
        Some(&mut defense),
        Some(&mut stamina),
        false,
        feedback,
    );
    if result == HitOutcome::Damaged {
        target_combat.interrupt_melee_chain();
    }
    if !target_health.is_alive() {
        target_combat.clear_targets();
        target_intent.clear();
        target_actions.clear();
    }
    result == HitOutcome::Parried
}
