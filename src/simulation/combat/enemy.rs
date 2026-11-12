//! The ordinary enemy: chase, wind up, swing its blade, recover. The Boss owns its own
//! component and is skipped here.
use super::hit::{
    CombatWorld, Enemies, Players, Riposte, apply_hit, defeat_if_fallen, settle_defense,
};
use super::sweep::{HitTrack, strike, ticks_between};
use super::{ActionIds, CombatEffect, CombatFeedback, HitContext, HitOutcome, HitProbes};
use crate::simulation::defense::decay;
use crate::simulation::{AttackClock, Body, SlowEffect, TrackId};
use bevy::prelude::*;

/// Kiva's ordinary enemy tuning.
const FIRST_ATTACK_DELAY: f32 = 0.8;
const ATTACK_COOLDOWN: f32 = 1.25;
/// The whole attack: windup, cuts and follow-through. When the blade is
/// dangerous within it comes from the baked path.
pub(super) const ATTACK_DURATION: f32 = 0.84;
const DAMAGE: f32 = 12.0;
/// The enemy closes in until the player is this deep inside its blade's reach.
const ENGAGE_DEPTH: f32 = 0.75;
const SIGHT: f32 = 12.0;
const PARRIED_STUN: f32 = 0.80;

#[derive(Component)]
#[require(SlowEffect)]
pub struct Enemy {
    pub(super) passive: bool,
    pub(super) cooldown: f32,
    pub(super) recovery: f32,
    pub(super) stun_source: Option<u64>,
    pub(super) attack_id: u64,
    /// Time since the current attack's windup began, through its follow-through.
    pub(super) attack_elapsed: Option<f32>,
    /// Players the current attack has already hit; each is hit once.
    pub(super) struck: Vec<Entity>,
}
impl Default for Enemy {
    fn default() -> Self {
        Self {
            passive: false,
            cooldown: FIRST_ATTACK_DELAY,
            recovery: 0.0,
            stun_source: None,
            attack_id: 0,
            attack_elapsed: None,
            struck: Vec::new(),
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
    /// Attacking, before the blade becomes dangerous.
    pub fn is_winding_up(&self) -> bool {
        self.windup_remaining() > 0.0
    }
    pub fn windup_remaining(&self) -> f32 {
        self.attack_elapsed
            .map_or(0.0, |elapsed| (track().strike_time() - elapsed).max(0.0))
    }
    pub fn is_recovering(&self) -> bool {
        self.recovery > 0.0
    }
    pub fn recovery_remaining(&self) -> f32 {
        self.recovery
    }
    /// The attack's single timeline: windup, then follow-through.
    pub fn attack_clock(&self) -> Option<AttackClock> {
        self.attack_elapsed.map(|elapsed| AttackClock {
            action_id: self.attack_id,
            track: TrackId::Enemy,
            elapsed,
            duration: ATTACK_DURATION,
        })
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
    /// Abandon the attack in progress, if any.
    fn stop_attack(&mut self) {
        self.attack_elapsed = None;
        self.struck.clear();
    }
    pub(super) fn stun(&mut self, source: u64, duration: f32) {
        self.stop_attack();
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
/// The ordinary enemy's baked blade path.
fn track() -> &'static HitTrack {
    crate::simulation::TrackId::Enemy
        .hit_track()
        .expect("the ordinary enemy's attack is baked")
}

pub(in crate::simulation) fn advance_enemies(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut probes: ResMut<HitProbes>,
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
            enemy.stop_attack();
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
            enemy.stop_attack();
            continue;
        }
        if let Some(previous) = enemy.attack_elapsed {
            // The attack is committed: stand, swing, and hit whatever the
            // blade touches.
            movement.stop();
            let elapsed = previous + dt;
            let track = track();
            let forward = facing.direction();
            for tick in ticks_between(previous, elapsed) {
                if tick == track.first_active() {
                    enemy.cooldown = ATTACK_COOLDOWN;
                    feedback.push(
                        entity,
                        CombatEffect::Melee,
                        body.aim_position(),
                        body.aim_position() + Vec3::new(forward.x, 0.0, forward.y) * track.reach,
                        track.reach,
                    );
                }
                if !track.is_active(tick) {
                    continue;
                }
                let touched: Vec<_> = players
                    .iter()
                    .filter(|(player, _, _, _, _, _, health, ..)| {
                        health.is_alive() && !enemy.struck.contains(player)
                    })
                    .filter_map(|(player, target_body, target_motion, ..)| {
                        strike(
                            track,
                            tick,
                            (entity, &body),
                            forward,
                            (player, target_body, target_motion),
                            &world,
                            &mut probes,
                        )
                        .map(|contact| (player, contact))
                    })
                    .collect();
                for (player, contact) in touched {
                    enemy.struck.push(player);
                    let id = enemy.attack_id;
                    let riposte = land_attack(
                        (entity, &body),
                        (player, id, contact),
                        &mut players,
                        &mut feedback,
                    );
                    if let Some(riposte) = riposte {
                        apply_hit(
                            riposte.context(entity, id),
                            &body,
                            &facing,
                            &motion,
                            &mut health,
                            None,
                            None,
                            false,
                            &mut feedback,
                        );
                        enemy.stun(id, PARRIED_STUN);
                    }
                }
                if enemy.attack_elapsed.is_none() {
                    break;
                }
            }
            if enemy.attack_elapsed.is_some() {
                if elapsed < ATTACK_DURATION {
                    enemy.attack_elapsed = Some(elapsed);
                } else {
                    enemy.stop_attack();
                }
            }
            continue;
        }
        let target = players
            .iter()
            .filter(|(_, _, _, _, _, _, h, _, _, _, _, _, _)| h.is_alive())
            .map(|(e, b, ..)| (e, b.world_position(), b.aim_position(), b.radius()))
            .filter(|(_, p, ..)| p.distance(body.world_position()) < SIGHT)
            .min_by(|a, b| {
                a.1.distance_squared(body.world_position())
                    .total_cmp(&b.1.distance_squared(body.world_position()))
            });
        let Some((target, position, aim, radius)) = target else {
            movement.stop();
            continue;
        };
        let delta = position - body.world_position();
        let dir = delta.xz().normalize_or_zero();
        facing.set(dir);
        let engage = (track().reach * body.scale() + radius) * ENGAGE_DEPTH;
        if delta.length() <= engage && world.visible(body.aim_position(), aim, entity, target) {
            movement.stop();
            if enemy.cooldown == 0.0 {
                enemy.attack_elapsed = Some(0.0);
                enemy.struck.clear();
                enemy.attack_id = ids.next();
                feedback.push(
                    entity,
                    CombatEffect::EnemyWindup,
                    body.aim_position(),
                    aim,
                    track().reach,
                );
            }
        } else {
            movement.set(dir);
        }
    }
}

/// The blade touched this player at `contact`: resolve their defense and the
/// damage. Returns the riposte when the player parried it.
fn land_attack(
    attacker: (Entity, &Body),
    attack: (Entity, u64, Vec3),
    players: &mut Players,
    feedback: &mut CombatFeedback,
) -> Option<Riposte> {
    let (entity, body) = attacker;
    let (target, attack_id, contact) = attack;
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
        (mut energy, _),
    )) = players.get_mut(target)
    else {
        return None;
    };
    if !target_health.is_alive() {
        return None;
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
            contact: Some(contact),
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
    settle_defense(result, (target, &target_body), &mut energy)
}
