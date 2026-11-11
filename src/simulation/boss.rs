//! Fixed-step Choulingke encounter. Recovered source decisions and Kiva tuning
//! stay distinct: this is an adaptation, not an executable copy of the source AI.
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use super::{
    ActionIntent, Body, CombatEffect, CombatFeedback, Combatant, Facing, Health, MotionState,
    MoveIntent, SpawnPoint,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossPhase {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossState {
    Idle,
    Pursuing,
    Strafing,
    Telegraph,
    Attacking,
    Recovering,
    Stunned,
    Transition,
    Defeated,
}

/// Unrecovered hitboxes, damage and active frames are explicit local balance values.
/// Source distance fields retain their numeric values; no hidden model-height scaling.
struct KivaTuning;
impl KivaTuning {
    const TELEGRAPH: f32 = 0.72;
    const ACTIVE: f32 = 0.22;
    const RECOVERY: f32 = 0.65;
    const TRANSITION: f32 = 2.2;
    const RESPAWN: f32 = 3.0;
    const HIT_RANGE: f32 = 3.2;
    const HIT_HALF_ANGLE: f32 = 70.0_f32.to_radians();
    const HIT_HEIGHT: f32 = 0.9;
    const DAMAGE_P1: f32 = 14.0;
    const DAMAGE_P2: f32 = 18.0;
}

// Source: 51000025_CLK_fight_Action, cases 4..60. Cooldowns only apply when
// action_flow_modify=true; values are zero-weight expiry durations, not clip lengths.
const SOURCE_SLOTS: [u32; 57] = [
    90021629, 90021435, 90021632, 90021633, 90021626, 90021627, 90021593, 90021594, 90021595,
    90021596, 90021597, 90021598, 90021599, 90021600, 90021601, 90021602, 90021603, 90021604,
    90021605, 90021606, 90021607, 90021608, 90021609, 90021610, 90021611, 90021612, 90021613,
    90021614, 90021615, 90021616, 90021625, 90021631, 90021634, 90021635, 90021636, 90021637,
    90021628, 90021639, 90021640, 90021638, 90021622, 90021641, 90021623, 90021646, 90021647,
    90021648, 90021649, 90021650, 90021651, 90021652, 90021653, 90021654, 90021655, 90021656,
    90021657, 90021658, 90021645,
];
const SOURCE_COOLDOWNS: [f32; 57] = [
    0., 0., 0., 0., 0., 0., 40., 55., 50., 65., 60., 0., 50., 60., 60., 55., 70., 65., 0., 80.,
    65., 55., 0., 75., 0., 75., 75., 55., 55., 0., 0., 35., 45., 60., 65., 60., 55., 60., 45., 55.,
    85., 45., 85., 0., 65., 50., 65., 65., 50., 55., 55., 60., 45., 50., 40., 55., 0.,
];

// Exact normal-choice lists and weights from the P1/P2 main trees. Action1 is
// lateral repositioning; Action3 closes to source distance 6. Complex buff,
// grab and soul mechanics remain explicitly outside this adaptation.
const P1_NEAR: &[(u16, u32)] = &[(1, 1), (10, 50), (12, 50), (11, 50), (19, 1)];
const P1_MID: &[(u16, u32)] = &[(3, 10), (14, 50), (16, 50), (45, 50)];
const P2_NEAR: &[(u16, u32)] = &[(50, 50), (25, 50), (53, 50)];
const P2_MID: &[(u16, u32)] = &[(48, 50), (49, 50), (24, 50), (3, 10), (21, 50)];

#[derive(Component)]
pub struct Boss {
    phase: BossPhase,
    state: BossState,
    action: Option<u16>,
    elapsed: f32,
    duration: f32,
    sequence: u64,
    cooldowns: [f32; 61],
    random: u32,
    opening: bool,
    attacks_since_reposition: u8,
    strike_resolved: bool,
    engaged: bool,
    stun_source: Option<u64>,
}

impl Default for Boss {
    fn default() -> Self {
        Self {
            phase: BossPhase::One,
            state: BossState::Idle,
            action: None,
            elapsed: 0.0,
            duration: 0.0,
            sequence: 0,
            cooldowns: [0.0; 61],
            random: 0xc10c_51a1,
            opening: true,
            attacks_since_reposition: 0,
            strike_resolved: false,
            engaged: false,
            stun_source: None,
        }
    }
}

impl Boss {
    /// Two health bars are a Kiva adaptation, not the source 65/75/30% buff checks.
    pub const PHASE_HEALTH: f32 = 650.0;
    pub fn phase(&self) -> BossPhase {
        self.phase
    }
    pub fn state(&self) -> BossState {
        self.state
    }
    pub fn action_id(&self) -> Option<u16> {
        self.action
    }
    pub fn skill_slot(&self) -> Option<u32> {
        self.action.and_then(|action| {
            (4..=60)
                .contains(&action)
                .then(|| SOURCE_SLOTS[(action - 4) as usize])
        })
    }
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
    pub fn duration(&self) -> f32 {
        self.duration
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    /// The 9002xxxx slot -> source graph/clip mapping has not been recovered.
    pub fn clip_name(&self) -> Option<&'static str> {
        None
    }
    pub fn is_invulnerable(&self) -> bool {
        matches!(self.state, BossState::Transition | BossState::Defeated)
    }
    pub fn can_be_launched(&self) -> bool {
        !self.is_invulnerable()
    }

    pub fn stun_source(&self) -> Option<u64> {
        if self.state == BossState::Stunned {
            self.stun_source
        } else {
            None
        }
    }
    pub fn stun_remaining(&self) -> f32 {
        if self.state == BossState::Stunned {
            (self.duration - self.elapsed).max(0.0)
        } else {
            0.0
        }
    }
    pub(super) fn stun(&mut self, source: u64, duration: f32) {
        if !self.is_invulnerable() {
            self.enter(BossState::Stunned, duration);
            self.stun_source = Some(source);
            self.strike_resolved = true;
        }
    }
    fn enter(&mut self, state: BossState, duration: f32) {
        self.state = state;
        self.stun_source = None;
        self.elapsed = 0.0;
        self.duration = duration;
    }

    fn reset_fight(&mut self) {
        let sequence = self.sequence;
        *self = Self::default();
        self.sequence = sequence;
    }

    fn roll(&mut self, maximum: u32) -> u32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 17;
        self.random ^= self.random << 5;
        self.random % maximum
    }

    fn choose(&mut self, distance: f32) -> u16 {
        if distance > 12.0 {
            return 3;
        }
        let choices = match (self.phase, distance <= 6.0) {
            (BossPhase::One, true) => P1_NEAR,
            (BossPhase::One, false) => P1_MID,
            (BossPhase::Two, true) => P2_NEAR,
            (BossPhase::Two, false) => P2_MID,
        };
        let total: u32 = choices
            .iter()
            .filter(|(action, _)| self.cooldowns[*action as usize] == 0.0)
            .map(|(_, weight)| weight)
            .sum();
        if total == 0 {
            return 1; // Reposition while every source weight is suppressed.
        }
        let mut roll = self.roll(total);
        for &(action, weight) in choices {
            if self.cooldowns[action as usize] != 0.0 {
                continue;
            }
            if roll < weight {
                return action;
            }
            roll -= weight;
        }
        unreachable!("weighted choice exhausted")
    }

    fn begin_action(&mut self, action: u16) {
        self.action = Some(action);
        self.sequence += 1;
        self.strike_resolved = false;
        match action {
            1 | 2 => self.enter(BossState::Strafing, 2.0),
            3 => self.enter(BossState::Pursuing, 5.0),
            _ => self.enter(BossState::Telegraph, KivaTuning::TELEGRAPH),
        }
    }
}

type BossActors = (
    Entity,
    &'static mut Boss,
    &'static mut Body,
    &'static mut MotionState,
    &'static mut MoveIntent,
    &'static mut Facing,
    &'static mut ActionIntent,
    &'static mut Health,
    &'static SpawnPoint,
    &'static mut super::SlowEffect,
);
type Opponents = (
    Entity,
    &'static Body,
    &'static Facing,
    &'static mut Health,
    &'static mut MotionState,
    &'static mut ActionIntent,
    &'static mut super::DefenseState,
    &'static mut super::Stamina,
    &'static mut Combatant,
    &'static mut super::CombatIntent,
);

/// Runs after current physical snapshots and player combat, with the same pause gate.
pub(super) fn advance(
    time: Res<Time<Fixed>>,
    spatial: SpatialQuery,
    mut feedback: ResMut<CombatFeedback>,
    mut bosses: Query<BossActors, With<Boss>>,
    mut players: Query<Opponents, (With<Combatant>, Without<Boss>)>,
) {
    let dt = time.delta_secs();
    for (
        entity,
        mut boss,
        mut body,
        mut motion,
        mut movement,
        mut facing,
        mut actions,
        mut health,
        spawn,
        mut slow,
    ) in &mut bosses
    {
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
        for cooldown in &mut boss.cooldowns {
            *cooldown = (*cooldown - dt).max(0.0);
        }
        if !health.is_alive() {
            movement.stop();
            actions.clear();
            if boss.phase == BossPhase::One {
                boss.phase = BossPhase::Two;
                boss.cooldowns.fill(0.0);
                boss.opening = true;
                boss.attacks_since_reposition = 0;
                boss.action = None;
                boss.sequence += 1;
                boss.enter(BossState::Transition, KivaTuning::TRANSITION);
                health.refill();
                *slow = super::SlowEffect::default();
                body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
                motion.reset();
            } else if boss.state != BossState::Defeated {
                boss.enter(BossState::Defeated, KivaTuning::RESPAWN);
            } else {
                boss.elapsed += dt;
                if boss.elapsed >= boss.duration {
                    boss.reset_fight();
                    health.refill();
                    *slow = super::SlowEffect::default();
                    body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
                    motion.reset();
                    feedback.push(
                        entity,
                        CombatEffect::Respawn,
                        body.aim_position(),
                        body.aim_position(),
                        0.8,
                    );
                }
            }
            continue;
        }
        health.tick(dt);
        boss.elapsed += dt;
        if boss.state == BossState::Transition {
            movement.stop();
            if boss.elapsed >= boss.duration {
                boss.enter(BossState::Idle, 0.0);
                feedback.push(
                    entity,
                    CombatEffect::Respawn,
                    body.aim_position(),
                    body.aim_position(),
                    0.8,
                );
            }
            continue;
        }
        let target = players
            .iter()
            .filter(|(_, _, _, health, _, _, _, _, _, _)| health.is_alive())
            .min_by(|a, b| {
                a.1.world_position()
                    .distance_squared(body.world_position())
                    .total_cmp(&b.1.world_position().distance_squared(body.world_position()))
            })
            .map(|(entity, _, _, _, _, _, _, _, _, _)| entity);
        let Some(target) = target else {
            movement.stop();
            if boss.engaged {
                boss.reset_fight();
                health.refill();
                *slow = super::SlowEffect::default();
                body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
                motion.reset();
                actions.clear();
            }
            continue;
        };
        let (
            _,
            target_body,
            target_facing,
            mut target_health,
            target_motion,
            mut target_actions,
            mut target_defense,
            mut target_stamina,
            mut target_combat,
            mut target_intent,
        ) = players.get_mut(target).unwrap();
        let offset = target_body.world_position() - body.world_position();
        let distance = offset.length();
        let direction = offset.xz().normalize_or(Vec2::Y);
        if !boss.engaged {
            boss.engaged = true;
            boss.begin_action(1); // Source P1's once-only two-second lateral opening.
        }
        if !motion.is_grounded() {
            movement.stop();
            if boss.state != BossState::Stunned {
                boss.enter(BossState::Recovering, KivaTuning::RECOVERY);
            }
            continue;
        }
        match boss.state {
            BossState::Idle => {
                let limit = if boss.phase == BossPhase::One { 3 } else { 5 };
                let action = if boss.opening {
                    boss.opening = false;
                    if boss.phase == BossPhase::One { 45 } else { 21 }
                } else if boss.attacks_since_reposition >= limit {
                    boss.attacks_since_reposition = 0;
                    1
                } else {
                    boss.choose(distance)
                };
                facing.set(direction);
                boss.begin_action(action);
                if boss.state == BossState::Telegraph {
                    feedback.push(
                        entity,
                        CombatEffect::EnemyWindup,
                        body.aim_position(),
                        target_body.aim_position(),
                        KivaTuning::HIT_RANGE,
                    );
                }
            }
            BossState::Pursuing => {
                facing.set(direction);
                movement.set(direction);
                if distance <= 6.0 || boss.elapsed >= boss.duration {
                    movement.stop();
                    boss.enter(BossState::Idle, 0.0);
                }
            }
            BossState::Strafing => {
                facing.set(direction);
                let sign = if boss.random & 1 == 0 { 1.0 } else { -1.0 };
                movement.set(Vec2::new(-direction.y, direction.x) * (0.7 * sign / body.speed));
                if boss.elapsed >= boss.duration {
                    movement.stop();
                    boss.enter(BossState::Idle, 0.0);
                }
            }
            BossState::Telegraph => {
                if boss.elapsed < boss.duration * 0.65 {
                    facing.set(direction);
                    movement.set(if distance > 2.4 {
                        direction * 0.75
                    } else {
                        Vec2::ZERO
                    });
                } else {
                    movement.stop();
                }
                if boss.elapsed >= boss.duration {
                    let action = boss.action.unwrap();
                    boss.cooldowns[action as usize] = SOURCE_COOLDOWNS[(action - 4) as usize];
                    boss.attacks_since_reposition += 1;
                    boss.enter(BossState::Attacking, KivaTuning::ACTIVE);
                    feedback.push(
                        entity,
                        CombatEffect::Melee,
                        body.aim_position(),
                        body.aim_position()
                            + Vec3::new(facing.direction().x, 0.0, facing.direction().y)
                                * KivaTuning::HIT_RANGE,
                        KivaTuning::HIT_RANGE,
                    );
                }
            }
            BossState::Attacking => {
                let forward = facing.direction();
                movement.set(if distance > 2.8 {
                    forward * 0.95
                } else {
                    Vec2::ZERO
                });
                let aim_offset = target_body.aim_position() - body.aim_position();
                let in_arc = forward.dot(aim_offset.xz().normalize_or(forward))
                    >= KivaTuning::HIT_HALF_ANGLE.cos();
                let visible = Dir3::new(aim_offset).ok().is_none_or(|direction| {
                    spatial
                        .cast_ray(
                            body.aim_position(),
                            direction,
                            aim_offset.length(),
                            true,
                            &SpatialQueryFilter::from_excluded_entities([entity, target]),
                        )
                        .is_none()
                });
                if !boss.strike_resolved
                    && distance <= KivaTuning::HIT_RANGE
                    && aim_offset.y.abs() <= KivaTuning::HIT_HEIGHT
                    && in_arc
                    && visible
                {
                    boss.strike_resolved = true;
                    let damage = if boss.phase == BossPhase::One {
                        KivaTuning::DAMAGE_P1
                    } else {
                        KivaTuning::DAMAGE_P2
                    };
                    let outcome = super::combat::apply_hit(
                        super::HitContext {
                            source: entity,
                            source_owner: entity,
                            pair: None,
                            melee_stage: None,
                            target,
                            action_id: boss.sequence,
                            hit_index: 0,
                            damage,
                            origin: body.aim_position(),
                        },
                        target_body,
                        target_facing,
                        &target_motion,
                        &mut target_health,
                        Some(&mut target_defense),
                        Some(&mut target_stamina),
                        false,
                        &mut feedback,
                    );
                    if outcome == super::HitOutcome::Parried {
                        let source = boss.sequence;
                        boss.stun(source, 0.80);
                        movement.stop();
                    }
                    if outcome == super::HitOutcome::Damaged {
                        target_combat.interrupt_melee_chain();
                    }
                    if !target_health.is_alive() {
                        movement.stop();
                        target_actions.clear();
                        target_combat.clear_targets();
                        target_intent.clear();
                    }
                }
                if boss.state == BossState::Attacking && boss.elapsed >= boss.duration {
                    boss.enter(BossState::Recovering, KivaTuning::RECOVERY);
                    movement.stop();
                }
            }
            BossState::Recovering | BossState::Stunned => {
                movement.stop();
                if boss.elapsed >= boss.duration {
                    boss.enter(BossState::Idle, 0.0);
                }
            }
            BossState::Transition | BossState::Defeated => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{
        CollisionWorld, CombatIntent, Enemy, HitStop, SimulationActive, SimulationPlugin,
        WeaponPair, spawn_box,
    };
    use std::time::Duration;

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn arena() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(40.0, vec![]));
        app.finish();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        let player = app
            .world_mut()
            .spawn((
                Body::new(Vec2::ZERO, 6.0, 0.35),
                ActionIntent::default(),
                MoveIntent::default(),
                MotionState::default(),
                Facing::default(),
                Health::new(10_000.0),
                Combatant::new(WeaponPair::HammerMusket),
                CombatIntent::default(),
                SpawnPoint(Vec2::ZERO),
            ))
            .id();
        let position = Vec2::new(0.0, -4.0);
        let boss = app
            .world_mut()
            .spawn((
                Body::new(position, 5.2, 0.3),
                Boss::default(),
                Enemy::default(),
                ActionIntent::default(),
                MoveIntent::default(),
                MotionState::default(),
                Facing::default(),
                Health::new(Boss::PHASE_HEALTH),
                SpawnPoint(position),
            ))
            .id();
        tick(&mut app, 12);
        (app, player, boss)
    }

    fn wait_for(app: &mut App, boss: Entity, state: BossState) {
        for _ in 0..500 {
            if app.world().get::<Boss>(boss).unwrap().state() == state {
                return;
            }
            tick(app, 1);
        }
        panic!("boss never reached {state:?}");
    }

    fn health(app: &App, entity: Entity) -> f32 {
        app.world().get::<Health>(entity).unwrap().current()
    }

    #[test]
    fn source_choices_keep_distance_boundaries_weights_and_zero_weight_cooldowns() {
        let mut boss = Boss::default();
        assert_eq!(boss.choose(12.001), 3);
        for _ in 0..1000 {
            let near = boss.choose(6.0);
            let mid = boss.choose(6.001);
            assert!(P1_NEAR.iter().any(|&(id, _)| id == near));
            assert!(P1_MID.iter().any(|&(id, _)| id == mid));
        }
        for action in [10, 11, 12, 19] {
            boss.cooldowns[action] = 10.0;
        }
        assert_eq!(boss.choose(3.0), 1);
        for action in [14, 16, 45] {
            boss.cooldowns[action] = 10.0;
        }
        assert_eq!(boss.choose(12.0), 3);
        boss.phase = BossPhase::Two;
        for action in [50, 25, 53] {
            boss.cooldowns[action] = 10.0;
        }
        assert_eq!(boss.choose(3.0), 1);
        assert_eq!(SOURCE_SLOTS[45 - 4], 90021641);
        assert_eq!(SOURCE_COOLDOWNS[45 - 4], 45.0);
    }

    #[test]
    fn real_encounter_telegraphs_source_opening_and_hits_only_once_per_active_window() {
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        let snapshot = app.world().get::<Boss>(boss).unwrap();
        assert_eq!(snapshot.action_id(), Some(45));
        assert_eq!(snapshot.skill_slot(), Some(90021641));
        assert_eq!(snapshot.clip_name(), None);
        assert!(snapshot.duration() > 0.5);
        assert_eq!(health(&app, player), 10_000.0);
        wait_for(&mut app, boss, BossState::Attacking);
        wait_for(&mut app, boss, BossState::Recovering);
        assert_eq!(health(&app, player), 10_000.0 - KivaTuning::DAMAGE_P1);
        let feedback = app.world().resource::<CombatFeedback>();
        assert_eq!(
            feedback
                .events()
                .filter(|event| event.actor == boss && event.kind == CombatEffect::EnemyWindup)
                .count(),
            1
        );
        assert_eq!(
            feedback
                .events()
                .filter(|event| event.actor == boss && event.kind == CombatEffect::Melee)
                .count(),
            1
        );
        assert_eq!(
            feedback
                .events()
                .filter(|event| event.actor == player && event.kind == CombatEffect::Hit)
                .count(),
            1
        );
    }

    #[test]
    fn tap_deflect_is_immune_without_stunning_boss_and_back_takes_full_damage() {
        for front in [true, false] {
            let (mut app, player, boss) = arena();
            wait_for(&mut app, boss, BossState::Attacking);
            let direction = app.world().get::<Body>(boss).unwrap().world_position()
                - app.world().get::<Body>(player).unwrap().world_position();
            app.world_mut()
                .get_mut::<Facing>(player)
                .unwrap()
                .set(direction.xz() * if front { 1.0 } else { -1.0 });
            app.world_mut()
                .get_mut::<super::super::DefenseState>(player)
                .unwrap()
                .push_edge(super::super::DefenseEdge::Pressed);
            tick(&mut app, 10);
            assert_eq!(
                health(&app, player),
                if front {
                    10_000.0
                } else {
                    10_000.0 - KivaTuning::DAMAGE_P1
                }
            );
            assert_ne!(
                app.world().get::<Boss>(boss).unwrap().state(),
                BossState::Stunned
            );
            assert_eq!(
                app.world()
                    .resource::<CombatFeedback>()
                    .events()
                    .filter(|event| event.actor == player && event.kind == CombatEffect::Deflect)
                    .count(),
                usize::from(front)
            );
        }
    }

    #[test]
    fn recovery_and_release_parry_clear_the_lunge_before_the_next_motor_step() {
        for interrupted in [false, true] {
            let (mut app, player, boss) = arena();
            wait_for(&mut app, boss, BossState::Telegraph);
            let forward = app.world().get::<Facing>(boss).unwrap().direction();
            let position = app.world().get::<Body>(boss).unwrap().world_position();
            // The ordinary miss stays beyond the hit volume through recovery.
            // A parry meets the lunge just inside its outer edge, while moving.
            let distance = if interrupted { 5.3 } else { 8.0 };
            super::super::teleport_actor(
                app.world_mut(),
                player,
                position + Vec3::new(forward.x, 0.0, forward.y) * distance,
            );
            app.world_mut()
                .get_mut::<Facing>(player)
                .unwrap()
                .set(-forward);
            if interrupted {
                tick(&mut app, 2); // Teleport clears support; establish it before the physical press.
                app.world_mut()
                    .get_mut::<super::super::DefenseState>(player)
                    .unwrap()
                    .push_edge(super::super::DefenseEdge::Pressed);
                tick(&mut app, 13);
                assert!(
                    app.world()
                        .get::<super::super::DefenseState>(player)
                        .unwrap()
                        .is_guarding()
                );
            }
            wait_for(&mut app, boss, BossState::Attacking);
            if interrupted {
                app.world_mut()
                    .get_mut::<super::super::DefenseState>(player)
                    .unwrap()
                    .push_edge(super::super::DefenseEdge::Released);
            }
            let boundary = if interrupted {
                BossState::Stunned
            } else {
                BossState::Recovering
            };
            wait_for(&mut app, boss, boundary);
            assert_eq!(
                app.world().get::<MoveIntent>(boss).unwrap().0,
                Vec2::ZERO,
                "{boundary:?} retained the lunge intent for another motor step"
            );
            let body = app.world().get::<Body>(boss).unwrap();
            let position = body.world_position();
            let speed = body.velocity().xz().length();
            assert!(
                speed > 1.0,
                "fixture did not reach the moving boundary: {speed}"
            );
            tick(&mut app, 1);
            let body = app.world().get::<Body>(boss).unwrap();
            assert!(
                body.velocity().xz().length() < speed - 1.0,
                "{boundary:?} did not brake on its first motor step"
            );
            assert!(
                body.world_position().xz().distance(position.xz()) < speed / 60.0 * 0.9,
                "{boundary:?} completed another full-speed lunge step"
            );
        }
    }

    #[test]
    fn a_dodge_protected_by_the_existing_motor_rejects_the_boss_active_hit_once() {
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        tick(&mut app, 33);
        let side = app
            .world()
            .get::<Body>(boss)
            .unwrap()
            .world_position()
            .x
            .signum();
        app.world_mut()
            .get_mut::<MoveIntent>(player)
            .unwrap()
            .set(Vec2::X * side);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_dodge();
        wait_for(&mut app, boss, BossState::Attacking);
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_invulnerable()
        );
        wait_for(&mut app, boss, BossState::Recovering);
        assert_eq!(health(&app, player), 10_000.0);
        assert_eq!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .filter(|event| event.actor == player && event.kind == CombatEffect::Dodge)
                .count(),
            1
        );
    }

    #[test]
    fn hammer_launch_and_three_companion_shots_damage_the_active_boss() {
        let (mut app, player, boss) = arena();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(&mut app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        tick(&mut app, 13);
        assert_eq!(health(&app, boss), Boss::PHASE_HEALTH - 20.0);
        assert!(!app.world().get::<MotionState>(boss).unwrap().is_grounded());
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        tick(&mut app, 34);
        assert_eq!(health(&app, boss), Boss::PHASE_HEALTH - 104.0);
        assert_eq!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .filter(|e| e.source_owner == player && e.kind == CombatEffect::SniperShot)
                .count(),
            3
        );
    }

    #[test]
    fn pause_and_hit_stop_freeze_boss_timeline_cooldowns_and_real_physical_pose() {
        let (mut app, _, boss) = arena();
        wait_for(&mut app, boss, BossState::Attacking);
        let snapshot = app.world().get::<Boss>(boss).unwrap();
        let (elapsed, cooldowns) = (snapshot.elapsed(), snapshot.cooldowns);
        let position = app.world().get::<Body>(boss).unwrap().world_position();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        tick(&mut app, 60);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().elapsed(), elapsed);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().cooldowns, cooldowns);
        assert_eq!(
            app.world().get::<Body>(boss).unwrap().world_position(),
            position
        );
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        app.world_mut().resource_mut::<HitStop>().request_sniper();
        tick(&mut app, 3);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().elapsed(), elapsed);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().cooldowns, cooldowns);
        assert_eq!(
            app.world().get::<Body>(boss).unwrap().world_position(),
            position
        );
        tick(&mut app, 1);
        assert!(app.world().get::<Boss>(boss).unwrap().elapsed() > elapsed);
        assert!(app.world().get::<Boss>(boss).unwrap().cooldowns[45] < cooldowns[45]);
    }

    #[test]
    fn two_health_bars_transition_with_damage_and_knockup_immunity_then_reset_after_defeat() {
        let (mut app, player, boss) = arena();
        app.world_mut()
            .get_mut::<Health>(boss)
            .unwrap()
            .damage(Boss::PHASE_HEALTH);
        tick(&mut app, 1);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().phase(),
            BossPhase::Two
        );
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Transition
        );
        assert_eq!(health(&app, boss), Boss::PHASE_HEALTH);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        tick(&mut app, 4);
        assert_eq!(health(&app, boss), Boss::PHASE_HEALTH);
        assert!(app.world().get::<MotionState>(boss).unwrap().is_grounded());
        wait_for(&mut app, boss, BossState::Telegraph);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().action_id(), Some(21));
        app.world_mut()
            .get_mut::<Health>(boss)
            .unwrap()
            .damage(Boss::PHASE_HEALTH);
        tick(&mut app, 1);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Defeated
        );
        tick(&mut app, 120);
        assert_eq!(health(&app, boss), 0.0);
        tick(&mut app, 70);
        assert_eq!(health(&app, boss), Boss::PHASE_HEALTH);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().phase(),
            BossPhase::One
        );
        assert!(
            app.world()
                .get::<Body>(boss)
                .unwrap()
                .world_position()
                .y
                .abs()
                < 0.05
        );
    }

    #[test]
    fn walls_block_the_boss_motor_and_strikes_instead_of_client_position_corrections() {
        let (mut app, player, boss) = arena();
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 1.5, -1.5),
            Vec3::new(8.0, 3.0, 0.3),
            false,
        );
        tick(&mut app, 600);
        assert_eq!(health(&app, player), 10_000.0);
        assert!(app.world().get::<Body>(boss).unwrap().world_position().z < -1.94);
        assert!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| event.actor == boss && event.kind == CombatEffect::Melee)
        );
    }

    #[test]
    fn player_death_resets_one_encounter_and_discards_the_pending_boss_strike() {
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        app.world_mut()
            .get_mut::<Health>(player)
            .unwrap()
            .damage(10_000.0);
        tick(&mut app, 1);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Idle
        );
        tick(&mut app, 100);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Idle
        );
        assert_eq!(app.world().get::<Boss>(boss).unwrap().action_id(), None);
        assert_eq!(
            app.world().get::<Body>(boss).unwrap().world_position().xz(),
            Vec2::new(0.0, -4.0)
        );
    }
    #[test]
    fn chain_q_stun_belongs_only_to_boss_and_phase_change_cancels_remaining_arrows() {
        let (mut app, player, boss) = arena();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_pair(super::super::WeaponPair::ChainAxeBow);
        tick(&mut app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(&mut app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_ranged_skill();
        tick(&mut app, 19);
        let window = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .combo_window()
            .unwrap();
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Stunned
        );
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().stun_source(),
            Some(window.source_action_id)
        );
        assert_eq!(app.world().get::<Enemy>(boss).unwrap().stun_source(), None);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        tick(&mut app, 9);
        let count = app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .filter(|e| e.source_owner == player && e.kind == CombatEffect::Shot)
            .count();
        assert_eq!(count, 2); // Q departure plus first arrow.
        app.world_mut()
            .get_mut::<Health>(boss)
            .unwrap()
            .damage(Boss::PHASE_HEALTH);
        tick(&mut app, 1);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Transition
        );
        assert_eq!(app.world().get::<Boss>(boss).unwrap().stun_source(), None);
        let actor = app.world().get::<Combatant>(player).unwrap();
        assert_eq!(actor.target(), None);
        assert!(!actor.axe_away());
        assert!(actor.sequence().unwrap().cancelled);
        tick(&mut app, 60);
        assert_eq!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .filter(|e| e.source_owner == player && e.kind == CombatEffect::Shot)
                .count(),
            count
        );
    }
    #[test]
    fn boss_slow_changes_real_motor_and_is_cleared_on_phase_reset() {
        let (mut app, player, boss) = arena();
        super::super::teleport_actor(app.world_mut(), player, Vec3::new(0.0, 0.0, 7.0));
        tick(&mut app, 2);
        {
            let mut b = app.world_mut().get_mut::<Boss>(boss).unwrap();
            b.enter(BossState::Pursuing, 5.0);
        }
        app.world_mut()
            .get_mut::<super::super::SlowEffect>(boss)
            .unwrap()
            .apply(0.65, 1.5);
        tick(&mut app, 8);
        let speed = app
            .world()
            .get::<Body>(boss)
            .unwrap()
            .velocity()
            .xz()
            .length();
        assert!(
            (speed - 5.2 * 0.65).abs() < 0.06,
            "actual slowed motor speed {speed}"
        );
        app.world_mut()
            .get_mut::<Health>(boss)
            .unwrap()
            .damage(Boss::PHASE_HEALTH);
        tick(&mut app, 1);
        assert_eq!(
            app.world()
                .get::<super::super::SlowEffect>(boss)
                .unwrap()
                .factor(),
            1.0
        );
    }
    #[test]
    fn sustained_boss_guard_uses_the_same_cost_and_half_damage_as_normal_enemies() {
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        app.world_mut()
            .get_mut::<super::super::DefenseState>(player)
            .unwrap()
            .push_edge(super::super::DefenseEdge::Pressed);
        tick(&mut app, 13);
        assert!(
            app.world()
                .get::<super::super::DefenseState>(player)
                .unwrap()
                .is_guarding()
        );
        wait_for(&mut app, boss, BossState::Attacking);
        wait_for(&mut app, boss, BossState::Recovering);
        assert_eq!(health(&app, player), 10_000.0 - KivaTuning::DAMAGE_P1 * 0.5);
        assert_eq!(
            app.world()
                .get::<super::super::Stamina>(player)
                .unwrap()
                .current(),
            80.0
        );
        assert_eq!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .filter(|e| e.actor == player && e.kind == CombatEffect::Block)
                .count(),
            1
        );
    }
    #[test]
    fn a_real_boss_hit_clears_only_the_players_uncommitted_melee_successor() {
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        tick(&mut app, 18);
        let before = health(&app, player);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        tick(&mut app, 1);
        let action = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .active_action()
            .unwrap();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        tick(&mut app, 1);
        assert_eq!(health(&app, player), before);
        for _ in 0..30 {
            tick(&mut app, 1);
            if health(&app, player) < before {
                break;
            }
        }
        assert!(health(&app, player) < before);
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .unwrap()
                .action_id,
            action.action_id
        );
        while app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .active_action()
            .is_some()
        {
            tick(&mut app, 1);
        }
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .is_none()
        );
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        tick(&mut app, 1);
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .unwrap()
                .melee_stage,
            Some(super::super::MeleeStage::One)
        );
    }
}
