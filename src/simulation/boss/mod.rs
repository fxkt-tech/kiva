//! Fixed-step Choulingke encounter. Its decisions are the source behaviour
//! trees, transcribed in [`source`]; this module carries them out on Kiva's
//! bodies: strikes swing their baked blade paths, movement goes through the
//! shared motor. What could not be recovered (the skill slot -> clip mapping,
//! damage, the phase-change script) is Kiva adaptation and says so.
use bevy::prelude::*;

mod source;

use super::combat::{CombatWorld, Hurtbox, strike, ticks_between};
use super::{
    ActionIntent, AttackClock, Body, CombatEffect, CombatFeedback, Combatant, Facing, Health,
    HitProbes, HitTrack, MotionState, MoveIntent, SpawnPoint, TrackId,
};
use source::{Mind, Plan, Sight};

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

/// Unrecovered damage and pacing are explicit local balance values. Where and
/// when a strike can hit is not tuned here: it is the strike's baked blade path.
struct KivaTuning;
impl KivaTuning {
    /// The share of the telegraph during which the Boss still turns to its target.
    const TRACKING: f32 = 0.65;
    /// The Boss closes in until the target is this deep inside the distance
    /// at which its strike's blade passes straight ahead of it: while it
    /// telegraphs, then while it swings.
    const TELEGRAPH_STOP: f32 = 0.75;
    const SWING_STOP: f32 = 0.875;
    /// How long the Boss collects itself after being knocked off the ground.
    const AIRBORNE_RECOVERY: f32 = 0.65;
    /// The source's `LockAnimLRAction` circles at `move_speed_rate` 0.7.
    const CIRCLE_SPEED: f32 = 0.7;
    /// A `LockAnimDistanceAction` gives up after this long.
    const APPROACH_TIMEOUT: f32 = 5.0;
    const TRANSITION: f32 = 2.2;
    const RESPAWN: f32 = 3.0;
    const DAMAGE_P1: f32 = 14.0;
    const DAMAGE_P2: f32 = 18.0;
}

/// Kiva adaptation: which strike (a row of [`Boss::STRIKES`]) each source
/// action 4..=60 swings. The source slot -> clip mapping is unrecovered, so
/// these are chosen: phase-one actions swing phase-one clips, phase-two
/// actions phase-two clips, and the reactive counters (4, 6..9, 34, 47) the
/// shortest ones.
const STRIKE_OF: [u8; 57] = [
    7, 5, 5, 7, 5, 7, // 4..=9
    0, 1, 2, 3, 3, 4, 4, 6, 2, 5, 1, // 10..=20
    8, 12, 14, 9, 10, 11, 15, 11, 15, 12, 13, 10, 14, // 21..=33
    16, // 34
    0, 1, 2, 0, 1, 4, 0, 3, 6, 2, 6, 1, // 35..=46
    16, 11, 12, 13, 9, 14, 14, 10, 15, 13, 8, 10, 9, 12, // 47..=60
];

#[derive(Component)]
pub struct Boss {
    phase: BossPhase,
    state: BossState,
    action: Option<u16>,
    elapsed: f32,
    duration: f32,
    sequence: u64,
    mind: Mind,
    strike_resolved: bool,
    engaged: bool,
    stun_source: Option<u64>,
    /// The strike in progress: its row of [`Self::STRIKES`] and the time
    /// since its telegraph began.
    strike: Option<(u8, f32)>,
    /// How near the strike in progress needs its target, feet to feet, for
    /// its blade to pass through it.
    engage: f32,
    /// How near an approach (action 2 or 3) closes in.
    approach: f32,
    /// Strafe to the Boss's left while circling.
    circle_left: bool,
    /// What the Boss does once its circling ends.
    after_circle: Option<u16>,
    /// The circling which ends the phase tree's attack round resets the count.
    resets_count: bool,
    /// Phase one marks its target below 65% health (buff 450097).
    marked: bool,
    /// The target guarded the last strike (`GedangCondition`).
    guarded: bool,
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
            mind: Mind::default(),
            strike_resolved: false,
            engaged: false,
            stun_source: None,
            strike: None,
            engage: 0.0,
            approach: 5.0,
            circle_left: false,
            after_circle: None,
            resets_count: false,
            marked: false,
            guarded: false,
        }
    }
}

impl Boss {
    /// Two health bars are a Kiva adaptation, not the source 65/75/30% buff checks.
    pub const PHASE_HEALTH: f32 = 650.0;
    /// Every strike's whole timeline in seconds: telegraph, swings and
    /// recovery. Each plays one named source clip at its own speed (client
    /// attack clip table); these are the lengths of those spans.
    pub const STRIKES: [f32; 17] = [
        2.0, 3.0, 3.2, 2.9, 2.6, 1.7, 4.3, 1.6, // phase-one clips
        3.6, 2.9, 2.8, 4.8, 3.2, 2.3, 3.3, 2.7, 1.6, // phase-two clips
    ];
    /// The strike a source action swings, if it is one which strikes.
    pub fn strike_of(action: u16) -> Option<u8> {
        (4..=60)
            .contains(&action)
            .then(|| STRIKE_OF[usize::from(action - 4)])
    }
    /// The timeline length of a strike.
    pub fn strike_duration(strike: u8) -> Option<f32> {
        Self::STRIKES.get(usize::from(strike)).copied()
    }
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
        self.action.and_then(source::slot)
    }
    /// The source tree's `attack_count`: strikes counted since it last circled.
    pub fn attack_count(&self) -> u8 {
        self.mind.attack_count
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
    /// The current strike's single timeline, from its telegraph through its
    /// recovery. Interruptions (stun, knock-up, phase change) end it; a
    /// combo starts the next strike's timeline.
    pub fn attack_clock(&self) -> Option<AttackClock> {
        let (strike, elapsed) = self.strike?;
        let duration = Self::strike_duration(strike)?;
        (elapsed <= duration).then_some(AttackClock {
            action_id: self.sequence,
            track: TrackId::Boss { strike },
            elapsed,
            duration,
        })
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
        self.strike = None;
    }

    /// The next phase of the same strike, keeping its timeline.
    fn continue_strike(&mut self, state: BossState, duration: f32) {
        let strike = self.strike;
        self.enter(state, duration);
        self.strike = strike;
    }

    fn strike_elapsed(&self) -> f32 {
        self.strike.map_or(0.0, |(_, elapsed)| elapsed)
    }

    /// The baked blade path of the strike in progress.
    fn strike_track(&self) -> &'static HitTrack {
        let (strike, _) = self.strike.expect("a strike is in progress");
        TrackId::Boss { strike }
            .hit_track()
            .expect("every Boss strike is baked")
    }

    /// The timeline length of the strike in progress.
    fn strike_duration_now(&self) -> f32 {
        self.strike
            .and_then(|(strike, _)| Self::strike_duration(strike))
            .unwrap_or(0.0)
    }

    fn reset_fight(&mut self) {
        let sequence = self.sequence;
        *self = Self::default();
        self.sequence = sequence;
    }

    fn sight(&self, distance: f32, health: &Health) -> Sight {
        Sight {
            phase: self.phase,
            distance,
            marked: self.marked,
            health: health.current() / health.maximum().max(1.0),
        }
    }

    /// The phase tree's `ConditionPriorityNode`, after the combo branch:
    /// counter a guard, attack while the round lasts, else circle.
    fn decide(&mut self, sight: Sight) -> Plan {
        if std::mem::take(&mut self.guarded) {
            return Plan::Act(self.mind.counter_guard());
        }
        if self.mind.attack_count < source::attack_limit(self.phase) {
            return self.mind.choose(sight);
        }
        // `LockTimer2`: circle for 1-2 s, then count a fresh round.
        let seconds = self.mind.between(1.0, 2.0);
        self.resets_count = true;
        Plan::Circle {
            seconds,
            then: None,
        }
    }

    fn circle(&mut self, seconds: f32, then: Option<u16>) {
        // `lastlockdirection` alternates the side.
        self.mind.last_lock_left = !self.mind.last_lock_left;
        self.circle_left = self.mind.last_lock_left;
        self.after_circle = then;
        self.enter(BossState::Strafing, seconds);
    }

    fn begin_action(&mut self, action: u16) {
        self.action = Some(action);
        self.sequence += 1;
        self.strike_resolved = false;
        self.engage = 0.0;
        self.mind.dispatch(action, self.marked);
        match action {
            // Action 1: `LockAnimLRAction` for 3 s.
            1 => self.circle(3.0, None),
            // Actions 2 / 3: `LockAnimDistanceAction` to 6 / 5 m.
            2 | 3 => {
                self.approach = if action == 2 { 6.0 } else { 5.0 };
                self.enter(BossState::Pursuing, KivaTuning::APPROACH_TIMEOUT);
            }
            _ => {
                let strike = Self::strike_of(action).expect("every source action 4..=60 strikes");
                self.strike = Some((strike, 0.0));
                // The telegraph lasts until the blade becomes dangerous.
                let telegraph = self.strike_track().strike_time();
                self.continue_strike(BossState::Telegraph, telegraph);
            }
        }
    }

    fn carry_out(&mut self, plan: Plan) {
        match plan {
            Plan::Act(action) => self.begin_action(action),
            Plan::Circle { seconds, then } => {
                self.action = None;
                self.circle(seconds, then);
            }
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
    &'static mut super::Energy,
);

/// Runs after current physical snapshots and player combat, with the same pause gate.
pub(super) fn advance(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut probes: ResMut<HitProbes>,
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
        boss.mind.tick(dt);
        if !health.is_alive() {
            movement.stop();
            actions.clear();
            if boss.phase == BossPhase::One {
                // A fresh phase tree: its own blackboard, timers and opening.
                boss.phase = BossPhase::Two;
                boss.mind = Mind::default();
                boss.guarded = false;
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
        if let Some((_, elapsed)) = &mut boss.strike {
            *elapsed += dt;
        }
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
            .filter(|(_, _, _, health, ..)| health.is_alive())
            .min_by(|a, b| {
                a.1.world_position()
                    .distance_squared(body.world_position())
                    .total_cmp(&b.1.world_position().distance_squared(body.world_position()))
            })
            .map(|(entity, ..)| entity);
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
            mut target_energy,
        ) = players.get_mut(target).unwrap();
        let offset = target_body.world_position() - body.world_position();
        let distance = offset.length();
        let direction = offset.xz().normalize_or(Vec2::Y);
        boss.engaged = true;
        if !motion.is_grounded() {
            movement.stop();
            if boss.state != BossState::Stunned {
                boss.enter(BossState::Recovering, KivaTuning::AIRBORNE_RECOVERY);
            }
            continue;
        }
        if boss.phase == BossPhase::One && health.current() <= health.maximum() * 0.65 {
            boss.marked = true;
        }
        let sight = boss.sight(distance, &health);
        // A decision taken this step, announced once it has started.
        let mut decided = None;
        match boss.state {
            BossState::Idle => decided = Some(boss.decide(sight)),
            BossState::Pursuing => {
                facing.set(direction);
                movement.set(direction);
                if distance <= boss.approach || boss.elapsed >= boss.duration {
                    movement.stop();
                    boss.enter(BossState::Idle, 0.0);
                }
            }
            BossState::Strafing => {
                facing.set(direction);
                let sign = if boss.circle_left { -1.0 } else { 1.0 };
                movement.set(
                    Vec2::new(-direction.y, direction.x)
                        * (KivaTuning::CIRCLE_SPEED * sign / body.speed),
                );
                if boss.elapsed >= boss.duration {
                    movement.stop();
                    if std::mem::take(&mut boss.resets_count) {
                        boss.mind.attack_count = 0;
                    }
                    match boss.after_circle.take() {
                        Some(action) => decided = Some(Plan::Act(action)),
                        None => boss.enter(BossState::Idle, 0.0),
                    }
                }
            }
            BossState::Telegraph => {
                let track = boss.strike_track();
                if boss.engage == 0.0 {
                    let target = Hurtbox::of(target_body, &target_motion);
                    // A strike which never passes in front still closes in.
                    boss.engage = track
                        .frontal_reach(body.scale(), &target)
                        .max(body.radius() + target.radius);
                }
                let reach = boss.engage;
                if boss.elapsed < boss.duration * KivaTuning::TRACKING {
                    facing.set(direction);
                }
                // Past the tracking share the direction is committed, but
                // the Boss keeps stepping in so the swing can connect.
                let forward = facing.direction();
                movement.set(if distance > reach * KivaTuning::TELEGRAPH_STOP {
                    forward * 0.75
                } else {
                    Vec2::ZERO
                });
                if boss.strike_elapsed() + 1e-4 >= track.strike_time() {
                    let swings = HitTrack::time(track.last_active()) - track.strike_time();
                    boss.continue_strike(BossState::Attacking, swings);
                    feedback.push(
                        entity,
                        CombatEffect::Melee,
                        body.aim_position(),
                        body.aim_position() + Vec3::new(forward.x, 0.0, forward.y) * track.reach,
                        track.reach,
                    );
                }
            }
            BossState::Attacking => {
                let track = boss.strike_track();
                let reach = boss.engage;
                let forward = facing.direction();
                movement.set(if distance > reach * KivaTuning::SWING_STOP {
                    forward * 0.95
                } else {
                    Vec2::ZERO
                });
                let elapsed = boss.strike_elapsed();
                let mut finished = false;
                for tick in ticks_between(elapsed - dt, elapsed) {
                    finished |= tick >= track.last_active();
                    if boss.strike_resolved || !track.is_active(tick) {
                        continue;
                    }
                    // The blade hits only where it actually passes.
                    let Some(contact) = strike(
                        track,
                        tick,
                        (entity, &body),
                        forward,
                        (target, target_body, &target_motion),
                        &world,
                        &mut probes,
                    ) else {
                        continue;
                    };
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
                            contact: Some(contact),
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
                    let riposte = super::combat::settle_defense(
                        outcome,
                        (target, target_body),
                        &mut target_energy,
                    );
                    if let Some(riposte) = riposte {
                        let source = boss.sequence;
                        super::combat::apply_hit(
                            riposte.context(entity, source),
                            &body,
                            &facing,
                            &motion,
                            &mut health,
                            None,
                            None,
                            boss.is_invulnerable(),
                            &mut feedback,
                        );
                        boss.stun(source, 0.80);
                        movement.stop();
                    }
                    if outcome == super::HitOutcome::Blocked {
                        boss.guarded = true;
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
                if boss.state == BossState::Attacking && finished {
                    movement.stop();
                    // The skill's combo check (`CanComboCancel`): the combo
                    // tree may cancel the recovery into a follow-up.
                    let action = boss.action.expect("a strike has its action");
                    match boss.mind.combo(action, sight) {
                        Some(next) => decided = Some(Plan::Act(next)),
                        None => {
                            let recovery = (boss.strike_duration_now() - elapsed).max(0.0);
                            boss.continue_strike(BossState::Recovering, recovery);
                        }
                    }
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
        if let Some(plan) = decided {
            facing.set(direction);
            boss.carry_out(plan);
            if boss.state == BossState::Telegraph {
                feedback.push(
                    entity,
                    CombatEffect::EnemyWindup,
                    body.aim_position(),
                    target_body.aim_position(),
                    boss.strike_track().reach,
                );
            }
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
                Combatant::new(WeaponPair::HammerMusket).without_auto_target(),
                CombatIntent::default(),
                SpawnPoint(Vec2::ZERO),
            ))
            .id();
        let position = Vec2::new(0.0, -4.0);
        let boss = app
            .world_mut()
            .spawn((
                Body::new(position, 5.2, 0.3),
                Boss {
                    state: BossState::Recovering,
                    duration: 1.0,
                    ..default()
                },
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
        // Establish support before starting the AI, so attack timing begins here.
        *app.world_mut().get_mut::<Boss>(boss).unwrap() = Boss::default();
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

    /// Ticks from the opening telegraph until the opening strike first
    /// reaches an undefended player. The encounter is deterministic, so a
    /// rehearsal on an identical fixture tells when the real one will land:
    /// a source strike swings several times and its first swing may whiff.
    fn ticks_to_contact() -> usize {
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        for ticks in 1..600 {
            tick(&mut app, 1);
            if health(&app, player) < 10_000.0 {
                return ticks;
            }
        }
        panic!("the opening strike never reached the player");
    }

    fn strike_count(app: &App, boss: Entity) -> usize {
        app.world()
            .resource::<CombatFeedback>()
            .events()
            .filter(|event| event.actor == boss && event.kind == CombatEffect::Melee)
            .count()
    }

    /// Start `action` now, in place of whatever the trees would choose.
    fn force(app: &mut App, boss: Entity, action: u16) {
        app.world_mut()
            .get_mut::<Boss>(boss)
            .unwrap()
            .begin_action(action);
    }

    fn state(app: &App, boss: Entity) -> BossState {
        app.world().get::<Boss>(boss).unwrap().state()
    }

    /// Run until the strike in progress has finished swinging.
    fn finish_swing(app: &mut App, boss: Entity) {
        wait_for(app, boss, BossState::Attacking);
        while state(app, boss) == BossState::Attacking {
            tick(app, 1);
        }
    }

    #[test]
    fn every_source_action_moves_or_swings_a_baked_strike() {
        for action in 1..=60 {
            match Boss::strike_of(action) {
                Some(strike) => assert!(
                    TrackId::Boss { strike }.hit_track().is_some(),
                    "action {action} swings unbaked strike {strike}"
                ),
                None => assert!(action <= 3, "action {action} does nothing"),
            }
        }
        assert!(
            (0..Boss::STRIKES.len() as u8).all(|strike| STRIKE_OF.contains(&strike)),
            "a strike no action swings"
        );
    }

    #[test]
    fn phase_one_circles_first_then_opens_with_its_source_strike_and_hits_once() {
        let (mut app, player, boss) = arena();
        tick(&mut app, 1);
        let snapshot = app.world().get::<Boss>(boss).unwrap();
        assert_eq!(snapshot.state(), BossState::Strafing);
        assert_eq!(snapshot.action_id(), None);
        assert!((snapshot.duration() - 2.0).abs() < 1e-5);
        wait_for(&mut app, boss, BossState::Telegraph);
        let snapshot = app.world().get::<Boss>(boss).unwrap();
        assert_eq!(snapshot.action_id(), Some(45));
        assert_eq!(snapshot.skill_slot(), Some(90021641));
        assert_eq!(snapshot.attack_count(), 1);
        assert_eq!(health(&app, player), 10_000.0);
        finish_swing(&mut app, boss);
        // However many times its blade passes, one strike lands once.
        assert_eq!(health(&app, player), 10_000.0 - KivaTuning::DAMAGE_P1);
        assert_eq!(strike_count(&app, boss), 1);
        let feedback = app.world().resource::<CombatFeedback>();
        assert_eq!(
            feedback
                .events()
                .filter(|event| event.actor == player && event.kind == CombatEffect::Hit)
                .count(),
            1
        );
    }

    #[test]
    fn a_skill_whose_combo_condition_holds_cancels_into_its_follow_up() {
        let (mut app, _, boss) = arena();
        // Opening strike 45 checks combo tag 641: within 8 m it chains into
        // combo 14, which swings 43 within 6 m, else 16.
        wait_for(&mut app, boss, BossState::Telegraph);
        let first = app.world().get::<Boss>(boss).unwrap().sequence();
        finish_swing(&mut app, boss);
        let snapshot = app.world().get::<Boss>(boss).unwrap();
        assert_eq!(
            snapshot.state(),
            BossState::Telegraph,
            "no recovery between"
        );
        assert_eq!(snapshot.sequence(), first + 1);
        assert!(matches!(snapshot.action_id(), Some(43 | 16)));
    }

    #[test]
    fn a_strike_without_a_combo_recovers_for_the_rest_of_its_timeline() {
        let (mut app, _, boss) = arena();
        force(&mut app, boss, 18);
        let duration = Boss::strike_duration(Boss::strike_of(18).unwrap()).unwrap();
        finish_swing(&mut app, boss);
        let snapshot = app.world().get::<Boss>(boss).unwrap();
        assert_eq!(snapshot.state(), BossState::Recovering);
        let left = duration - snapshot.strike_elapsed();
        assert!((snapshot.duration() - left).abs() < 0.02);
        // The clock keeps running through the recovery.
        assert!(snapshot.attack_clock().is_some());
    }

    #[test]
    fn a_full_attack_round_circles_for_one_to_two_seconds_then_counts_afresh() {
        for phase in [BossPhase::One, BossPhase::Two] {
            let (mut app, _, boss) = arena();
            {
                let mut boss = app.world_mut().get_mut::<Boss>(boss).unwrap();
                boss.phase = phase;
                let sight = boss.sight(3.0, &Health::new(Boss::PHASE_HEALTH));
                boss.mind.choose(sight); // the phase's opening
                boss.mind.attack_count = source::attack_limit(phase);
            }
            tick(&mut app, 1);
            let snapshot = app.world().get::<Boss>(boss).unwrap();
            assert_eq!(snapshot.state(), BossState::Strafing);
            assert!((1.0..=2.0).contains(&snapshot.duration()));
            let seconds = snapshot.duration();
            tick(&mut app, (seconds * 60.0) as usize + 2);
            // A fresh round: at most the strike just chosen is counted.
            assert!(app.world().get::<Boss>(boss).unwrap().attack_count() <= 1);
        }
    }

    #[test]
    fn a_guarded_strike_is_answered_by_one_of_the_two_guard_counters() {
        let (mut app, _, boss) = arena();
        let mut counters = Vec::new();
        for _ in 0..2 {
            let mut boss = app.world_mut().get_mut::<Boss>(boss).unwrap();
            boss.guarded = true;
            let sight = boss.sight(3.0, &Health::new(Boss::PHASE_HEALTH));
            match boss.decide(sight) {
                Plan::Act(action) => counters.push(action),
                plan => panic!("{plan:?}"),
            }
            assert!(!boss.guarded);
        }
        counters.sort();
        assert_eq!(counters, vec![6, 7], "the counter alternates");
    }

    #[test]
    fn tap_deflect_is_immune_without_stunning_boss_and_back_takes_full_damage() {
        let contact = ticks_to_contact();
        for front in [true, false] {
            let (mut app, player, boss) = arena();
            wait_for(&mut app, boss, BossState::Telegraph);
            tick(&mut app, contact - 3);
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
        // The strike under way, with the player placed and, for the parry, guarding.
        let fixture = |interrupted: bool| {
            let (mut app, player, boss) = arena();
            // A short strike, which ends while the Boss still lunges, and
            // whose skill (tag 435) has no combo, so it recovers in place.
            force(&mut app, boss, 5);
            assert_eq!(state(&app, boss), BossState::Telegraph);
            let forward = app.world().get::<Facing>(boss).unwrap().direction();
            let position = app.world().get::<Body>(boss).unwrap().world_position();
            // The ordinary miss stays beyond the blade through recovery.
            // A parry meets the blade at the far end of the lunge, while moving.
            let distance = if interrupted { 5.2 } else { 8.0 };
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
            (app, player, boss)
        };
        // A rehearsal which keeps guarding tells when the blade arrives; the
        // guard does not change how the Boss moves.
        let arrival = {
            let (mut app, player, _) = fixture(true);
            let blocked = |app: &App| {
                app.world()
                    .resource::<CombatFeedback>()
                    .events()
                    .any(|e| e.actor == player && e.kind == CombatEffect::Block)
            };
            (1..120)
                .find(|_| {
                    tick(&mut app, 1);
                    blocked(&app)
                })
                .expect("the guarded strike reaches the player")
        };
        for interrupted in [false, true] {
            let (mut app, player, boss) = fixture(interrupted);
            if interrupted {
                // Release just before the blade arrives.
                tick(&mut app, arrival - 2);
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
            // The parry also strikes back and earns one sun; a miss does neither.
            let riposte = if interrupted { 10.0 } else { 0.0 };
            assert_eq!(health(&app, boss), Boss::PHASE_HEALTH - riposte);
            assert_eq!(
                app.world()
                    .get::<super::super::Energy>(player)
                    .unwrap()
                    .sun(),
                u8::from(interrupted)
            );
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
        let contact = ticks_to_contact();
        wait_for(&mut app, boss, BossState::Telegraph);
        // Protection begins a few steps into the dodge.
        tick(&mut app, contact - 8);
        // Dodge into the swing, so the blade still passes through the player.
        let toward = (app.world().get::<Body>(boss).unwrap().world_position()
            - app.world().get::<Body>(player).unwrap().world_position())
        .xz()
        .normalize();
        app.world_mut()
            .get_mut::<MoveIntent>(player)
            .unwrap()
            .set(toward);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_dodge();
        tick(&mut app, 8);
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_invulnerable()
        );
        // Only the dodged strike: a combo may follow it.
        finish_swing(&mut app, boss);
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
        // The third bullet is still on its way for a step or two.
        for _ in 0..4 {
            if health(&app, boss) == Boss::PHASE_HEALTH - 104.0 {
                break;
            }
            tick(&mut app, 1);
        }
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
        let action = snapshot.action_id().unwrap();
        let (elapsed, cooldown) = (snapshot.elapsed(), snapshot.mind.cooldown(action));
        assert!(cooldown > 0.0);
        let position = app.world().get::<Body>(boss).unwrap().world_position();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        tick(&mut app, 60);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().elapsed(), elapsed);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().mind.cooldown(action),
            cooldown
        );
        assert_eq!(
            app.world().get::<Body>(boss).unwrap().world_position(),
            position
        );
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        app.world_mut().resource_mut::<HitStop>().request_sniper();
        tick(&mut app, 3);
        assert_eq!(app.world().get::<Boss>(boss).unwrap().elapsed(), elapsed);
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().mind.cooldown(action),
            cooldown
        );
        assert_eq!(
            app.world().get::<Body>(boss).unwrap().world_position(),
            position
        );
        tick(&mut app, 1);
        assert!(app.world().get::<Boss>(boss).unwrap().elapsed() > elapsed);
        assert!(app.world().get::<Boss>(boss).unwrap().mind.cooldown(action) < cooldown);
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
            // Wide enough that circling cannot take the Boss round it.
            Vec3::new(60.0, 3.0, 0.3),
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
        let contact = ticks_to_contact();
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        tick(&mut app, contact - 20);
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
        // Read the guard's cost as the blow lands, before stamina recovers.
        tick(&mut app, 7);
        assert_eq!(health(&app, player), 10_000.0 - KivaTuning::DAMAGE_P1 * 0.5);
        // The guard is what the phase tree's `GedangCondition` answers.
        assert!(app.world().get::<Boss>(boss).unwrap().guarded);
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
        let contact = ticks_to_contact();
        let (mut app, player, boss) = arena();
        wait_for(&mut app, boss, BossState::Telegraph);
        tick(&mut app, contact - 12);
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
