//! Fixed-step action preparation and hit resolution. Rendering never creates damage.
use super::abilities::ActionRuntime;
use super::defense::decay;
use super::{
    ActionAvailability, ActionIntent, ActionKind, ActionSnapshot, Body, Boss, ComboWindow,
    Companion, CompanionState, DefenseKind, DefenseState, Facing, HitStop, MeleeStage, MotionState,
    MoveIntent, SequenceSnapshot, SlowEffect, Stamina, WeaponMode, WeaponPair,
};
use avian3d::prelude::{Sensor, SpatialQuery, SpatialQueryFilter};
use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::VecDeque;
const ATTACK: u8 = 1;
const MELEE_SKILL: u8 = 2;
const RANGED_SKILL: u8 = 4;
const COMBO: u8 = 8;
const TOGGLE: u8 = 16;
#[derive(Component)]
pub struct Health {
    current: f32,
    maximum: f32,
    hit_remaining: f32,
    respawn_remaining: f32,
}
impl Health {
    pub fn new(maximum: f32) -> Self {
        assert!(maximum.is_finite() && maximum > 0.0);
        Self {
            current: maximum,
            maximum,
            hit_remaining: 0.0,
            respawn_remaining: 0.0,
        }
    }
    pub fn current(&self) -> f32 {
        self.current
    }
    pub fn maximum(&self) -> f32 {
        self.maximum
    }
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }
    pub fn hit_remaining(&self) -> f32 {
        self.hit_remaining
    }
    pub fn respawn_remaining(&self) -> f32 {
        self.respawn_remaining
    }
    pub(super) fn refill(&mut self) {
        self.current = self.maximum;
        self.hit_remaining = 0.0;
        self.respawn_remaining = 0.0;
    }
    pub(super) fn tick(&mut self, dt: f32) -> bool {
        self.hit_remaining = decay(self.hit_remaining, dt);
        if self.is_alive() {
            return false;
        }
        self.respawn_remaining = decay(self.respawn_remaining, dt);
        if self.respawn_remaining == 0.0 {
            self.refill();
            true
        } else {
            false
        }
    }
    pub(super) fn damage(&mut self, amount: f32) -> bool {
        if !self.is_alive() {
            return false;
        }
        self.current = (self.current - amount).max(0.0);
        self.hit_remaining = 0.18;
        if !self.is_alive() {
            self.respawn_remaining = 3.0;
        }
        true
    }
}
#[derive(Component, Clone, Copy)]
pub struct SpawnPoint(pub Vec2);
#[derive(Component)]
#[require(SlowEffect)]
pub struct Enemy {
    passive: bool,
    cooldown: f32,
    windup: f32,
    recovery: f32,
    stun_source: Option<u64>,
    attack_target: Option<Entity>,
    attack_id: u64,
}
impl Default for Enemy {
    fn default() -> Self {
        Self {
            passive: false,
            cooldown: 0.8,
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
    fn stun(&mut self, source: u64, duration: f32) {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatEffect {
    MeleeStart,
    Melee,
    Shot,
    SniperShot,
    SniperImpact,
    Hammer,
    ChainSweep,
    AxeThrow,
    Hit,
    Deflect,
    Block,
    Parry,
    Dodge,
    Defeat,
    Respawn,
    EnemyWindup,
    Rejected,
}
#[derive(Clone, Copy, Debug)]
pub struct CombatEvent {
    pub sequence: u64,
    pub actor: Entity,
    pub source_owner: Entity,
    pub pair: Option<WeaponPair>,
    pub melee_stage: Option<MeleeStage>,
    pub action_id: u64,
    pub hit_index: u8,
    pub kind: CombatEffect,
    pub position: Vec3,
    pub end: Vec3,
    pub radius: f32,
}
#[derive(Resource, Default)]
pub struct CombatFeedback {
    next_sequence: u64,
    events: VecDeque<CombatEvent>,
}
impl CombatFeedback {
    pub fn events(&self) -> impl Iterator<Item = &CombatEvent> {
        self.events.iter()
    }
    pub(super) fn push(
        &mut self,
        actor: Entity,
        kind: CombatEffect,
        position: Vec3,
        end: Vec3,
        radius: f32,
    ) {
        self.emit(CombatEvent {
            sequence: 0,
            actor,
            source_owner: actor,
            pair: None,
            melee_stage: None,
            action_id: 0,
            hit_index: 0,
            kind,
            position,
            end,
            radius,
        });
    }
    fn emit(&mut self, mut event: CombatEvent) {
        self.next_sequence += 1;
        event.sequence = self.next_sequence;
        self.events.push_back(event);
        if self.events.len() > 128 {
            self.events.pop_front();
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn action(
        &mut self,
        actor: Entity,
        owner: Entity,
        action: &ActionSnapshot,
        index: u8,
        kind: CombatEffect,
        position: Vec3,
        end: Vec3,
        radius: f32,
    ) {
        self.emit(CombatEvent {
            sequence: 0,
            actor,
            source_owner: owner,
            pair: Some(action.pair),
            melee_stage: action.melee_stage,
            action_id: action.action_id,
            hit_index: index,
            kind,
            position,
            end,
            radius,
        });
    }
}
#[derive(Resource, Default)]
pub(super) struct ActionIds(u64);
impl ActionIds {
    pub(super) fn next(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}
#[derive(Component, Default)]
pub struct CombatIntent {
    commands: u8,
    pair: Option<WeaponPair>,
    cancel_melee_chain: bool,
}
impl CombatIntent {
    pub fn request_attack(&mut self) {
        self.commands |= ATTACK;
    }
    pub fn request_melee_skill(&mut self) {
        self.commands |= MELEE_SKILL;
    }
    pub fn request_ranged_skill(&mut self) {
        self.commands |= RANGED_SKILL;
    }
    pub fn request_combo(&mut self) {
        self.commands |= COMBO;
    }
    pub fn request_toggle_target(&mut self) {
        self.commands ^= TOGGLE;
    }
    pub fn request_pair(&mut self, pair: WeaponPair) {
        self.pair = Some(pair);
    }
    /// Drop only uncommitted ordinary-attack follow-ups at the next rule tick.
    /// This latch survives a paused frame with no FixedUpdate.
    pub fn cancel_melee_chain(&mut self) {
        self.cancel_melee_chain = true;
    }
    pub fn clear(&mut self) {
        *self = Self {
            cancel_melee_chain: true,
            ..Self::default()
        };
    }
}

#[derive(Default)]
struct MeleeChain {
    next_stage: MeleeStage,
    remaining: f32,
    buffered: bool,
    /// Only this committed action may open/extend the current chain. Clearing
    /// it on damage prevents its later retirement from reviving an old chain.
    eligible_action: Option<u64>,
}
impl MeleeChain {
    fn clear(&mut self) {
        *self = Self::default();
    }
    fn tick(&mut self, dt: f32) {
        if self.remaining > 0.0 {
            self.remaining = decay(self.remaining, dt);
            if self.remaining == 0.0 {
                self.clear();
            }
        }
    }
    fn begin(&mut self, action_id: u64, stage: MeleeStage) {
        self.clear();
        if let Some(next) = stage.next() {
            self.next_stage = next;
            self.eligible_action = Some(action_id);
        }
    }
    fn buffer(&mut self, action_id: u64) -> bool {
        if self.eligible_action == Some(action_id) {
            self.buffered = true;
            true
        } else {
            false
        }
    }
    fn finish(&mut self, action_id: u64) {
        if self.eligible_action == Some(action_id) {
            self.eligible_action = None;
            self.remaining = 0.45;
        }
    }
}
#[derive(Component)]
#[require(DefenseState, Stamina, super::motion_feedback::MotionFeedbackState)]
pub struct Combatant {
    pair: WeaponPair,
    target: Option<Entity>,
    attack_cooldown: f32,
    melee_cooldown: f32,
    ranged_cooldown: f32,
    combo: Option<ComboWindow>,
    action: Option<ActionRuntime>,
    melee_chain: MeleeChain,
    companion: Option<Entity>,
    axe_away: bool,
    status: &'static str,
    availability: ActionAvailability,
    pull_velocity: Option<Vec2>,
}
impl Combatant {
    pub fn new(pair: WeaponPair) -> Self {
        Self {
            pair,
            target: None,
            attack_cooldown: 0.0,
            melee_cooldown: 0.0,
            ranged_cooldown: 0.0,
            combo: None,
            action: None,
            melee_chain: MeleeChain::default(),
            companion: None,
            axe_away: false,
            status: "准备就绪",
            availability: default(),
            pull_velocity: None,
        }
    }
    pub fn weapon_pair(&self) -> WeaponPair {
        self.pair
    }
    pub fn weapon_name(&self) -> &'static str {
        self.pair.weapon_name(WeaponMode::Melee)
    }
    pub fn target(&self) -> Option<Entity> {
        self.target
    }
    pub fn melee_cooldown(&self) -> f32 {
        self.melee_cooldown
    }
    pub fn ranged_cooldown(&self) -> f32 {
        self.ranged_cooldown
    }
    pub fn attack_cooldown(&self) -> f32 {
        self.attack_cooldown
    }
    pub fn combo_remaining(&self) -> f32 {
        self.combo.map_or(0.0, |c| c.remaining)
    }
    pub fn combo_window(&self) -> Option<ComboWindow> {
        self.combo
    }
    pub fn active_action(&self) -> Option<ActionSnapshot> {
        self.action.as_ref().map(|a| a.snapshot)
    }
    pub fn sequence(&self) -> Option<SequenceSnapshot> {
        self.action.as_ref().and_then(ActionRuntime::sequence)
    }
    pub fn is_combo_active(&self) -> bool {
        self.sequence().is_some()
    }
    pub fn companion(&self) -> Option<Entity> {
        self.companion
    }
    pub fn axe_away(&self) -> bool {
        self.axe_away
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
    pub fn availability(&self) -> ActionAvailability {
        self.availability
    }
    pub fn clear_targets(&mut self) {
        self.target = None;
        self.combo = None;
        self.action = None;
        self.melee_chain.clear();
        self.axe_away = false;
        self.pull_velocity = None;
        self.availability = ActionAvailability::default();
    }
    /// Gameplay interruption clears pending follow-ups, not the current hit.
    pub(super) fn interrupt_melee_chain(&mut self) {
        self.melee_chain.clear();
    }
    pub(super) fn movement_locked(&self) -> bool {
        self.is_combo_active()
    }
    pub(super) fn pull_velocity(&self) -> Option<Vec2> {
        self.pull_velocity
    }
    pub(super) fn busy(&self) -> bool {
        self.action.is_some()
    }
    fn reject(
        &mut self,
        status: &'static str,
        entity: Entity,
        position: Vec3,
        feedback: &mut CombatFeedback,
    ) {
        self.status = status;
        feedback.push(entity, CombatEffect::Rejected, position, position, 0.0);
    }
}
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
pub(super) fn apply_hit(
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
pub(super) struct CombatWorld<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    sensors: Query<'w, 's, (), With<Sensor>>,
}
impl CombatWorld<'_, '_> {
    fn visible(&self, start: Vec3, end: Vec3, observer: Entity, target: Entity) -> bool {
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
    fn companion_pose(&self, body: &Body, facing: Vec2, owner: Entity) -> (Vec3, Vec3) {
        let origin = body.world_position();
        let forward = Vec3::new(facing.x, 0.0, facing.y);
        let side = Vec3::new(-facing.y, 0.0, facing.x);
        let desired = origin + side * 0.95 - forward * 0.45;
        for fraction in [1.0, 0.65, 0.3, 0.0] {
            let feet = origin.lerp(desired, fraction);
            let filter = SpatialQueryFilter::from_excluded_entities([owner]);
            let Some(hit) = self.spatial.cast_ray_predicate(
                feet + Vec3::Y * 1.5,
                Dir3::NEG_Y,
                3.0,
                true,
                &filter,
                &|e| !self.sensors.contains(e),
            ) else {
                continue;
            };
            if hit.normal.y < 0.65 {
                continue;
            }
            let feet = feet + Vec3::Y * (1.5 - hit.distance);
            let muzzle = feet + Vec3::Y * 1.2 + forward * 0.55;
            if self.visible(body.aim_position(), feet + Vec3::Y * 0.9, owner, owner)
                && self.visible(body.aim_position(), muzzle, owner, owner)
            {
                return (feet, muzzle);
            }
        }
        (origin, body.aim_position())
    }
}
pub(crate) fn ensure_companions(world: &mut World) {
    let missing: Vec<_> = world
        .query::<(Entity, &Combatant)>()
        .iter(world)
        .filter(|(_, c)| {
            c.companion
                .is_none_or(|e| world.get::<Companion>(e).is_none())
        })
        .map(|(e, _)| e)
        .collect();
    for owner in missing {
        let entity = world
            .spawn((Companion { owner }, CompanionState::default()))
            .id();
        world.get_mut::<Combatant>(owner).unwrap().companion = Some(entity);
    }
    let orphaned: Vec<_> = world
        .query::<(Entity, &Companion)>()
        .iter(world)
        .filter(|(_, c)| world.get::<Combatant>(c.owner).is_none())
        .map(|(e, _)| e)
        .collect();
    for e in orphaned {
        world.despawn(e);
    }
}
type CandidateData = (
    Entity,
    &'static Body,
    &'static MotionState,
    &'static Health,
    &'static Enemy,
    Option<&'static Boss>,
);
type PreparePlayer = (
    Entity,
    &'static Body,
    &'static MotionState,
    &'static mut Facing,
    &'static mut Combatant,
    &'static mut CombatIntent,
    &'static mut DefenseState,
    &'static Stamina,
    &'static mut ActionIntent,
    &'static Health,
);
fn target_valid(health: &Health, boss: Option<&Boss>) -> bool {
    health.is_alive() && boss.is_none_or(|b| !b.is_invulnerable())
}
fn window_valid(combo: ComboWindow, enemies: &Query<CandidateData, With<Enemy>>) -> bool {
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
fn availability_for(
    combat: &Combatant,
    motion: &MotionState,
    defense: &DefenseState,
    target_distance: Option<f32>,
) -> ActionAvailability {
    let ground = motion.is_grounded() && !motion.is_dodging() && !motion.is_sliding();
    let free =
        !combat.busy() && !defense.is_active() && !motion.is_dodging() && !motion.is_sliding();
    let in_range = |range| target_distance.is_some_and(|distance| distance <= range);
    let branch = combat.combo.is_some_and(|combo| {
        combat.action.as_ref().is_none_or(|action| {
            action.snapshot.action_id == combo.source_action_id
                && matches!(
                    action.snapshot.kind,
                    ActionKind::HammerSlam | ActionKind::AxeShot
                )
        })
    });
    ActionAvailability {
        attack: free && combat.attack_cooldown == 0.0 && !combat.axe_away,
        melee_skill: free
            && ground
            && combat.melee_cooldown == 0.0
            && !combat.axe_away
            && (combat.pair == WeaponPair::ChainAxeBow || in_range(4.5)),
        ranged_skill: free
            && ground
            && combat.ranged_cooldown == 0.0
            && in_range(if combat.pair == WeaponPair::HammerMusket {
                22.0
            } else {
                12.0
            }),
        combo: ground && !defense.is_active() && branch,
        defense: ground && !combat.busy(),
        equip: free && ground,
        movement_actions: !combat.busy(),
    }
}

pub(super) fn prepare(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut ids: ResMut<ActionIds>,
    mut feedback: ResMut<CombatFeedback>,
    enemies: Query<CandidateData, With<Enemy>>,
    mut players: Query<PreparePlayer, Without<Enemy>>,
    mut slows: Query<&mut SlowEffect>,
) {
    let dt = time.delta_secs();
    for mut slow in &mut slows {
        slow.tick(dt);
    }
    for (
        entity,
        body,
        motion,
        mut facing,
        mut combat,
        mut intent,
        mut defense,
        stamina,
        mut actions,
        health,
    ) in &mut players
    {
        let commands = std::mem::take(&mut intent.commands);
        let pair = intent.pair.take();
        if std::mem::take(&mut intent.cancel_melee_chain) {
            combat.melee_chain.clear();
        }
        combat.melee_chain.tick(dt);
        combat.attack_cooldown = decay(combat.attack_cooldown, dt);
        combat.melee_cooldown = decay(combat.melee_cooldown, dt);
        combat.ranged_cooldown = decay(combat.ranged_cooldown, dt);
        combat.pull_velocity = None;
        if !health.is_alive() {
            combat.clear_targets();
            defense.cancel();
            defense.clear();
            actions.clear();
            continue;
        }
        if combat.target.is_some_and(|e| {
            enemies
                .get(e)
                .map_or(true, |(_, _, _, h, _, b)| !target_valid(h, b))
        }) {
            combat.target = None;
        }
        if commands & TOGGLE != 0 {
            combat.target = if combat.target.is_some() {
                None
            } else {
                enemies
                    .iter()
                    .filter(|(_, b, _, h, _, boss)| {
                        target_valid(h, *boss)
                            && b.world_position().distance(body.world_position()) <= 22.0
                    })
                    .filter(|(e, b, _, _, _, _)| {
                        world.visible(body.aim_position(), b.aim_position(), entity, *e)
                    })
                    .filter_map(|(e, b, _, _, _, _)| {
                        let offset = (b.world_position() - body.world_position()).xz();
                        let align = facing.direction().dot(offset.normalize_or_zero());
                        (align > 0.35).then_some((e, align - offset.length() * 0.01))
                    })
                    .max_by(|a, b| {
                        a.1.total_cmp(&b.1)
                            .then_with(|| b.0.to_bits().cmp(&a.0.to_bits()))
                    })
                    .map(|(e, _)| e)
            };
        }
        if let Some(mut combo) = combat.combo {
            combo.remaining = decay(combo.remaining, dt);
            let valid = window_valid(combo, &enemies);
            combat.combo = (valid && combo.remaining > 0.0).then_some(combo);
        }
        if let Some(mut action) = combat.action.take() {
            action.previous = action.snapshot.elapsed;
            action.snapshot.elapsed += dt;
            if action.cancelled {
                action.return_remaining = decay(action.return_remaining, dt);
            }
            if let Some(target) = action.snapshot.target {
                if let Ok((_, b, _, h, _, boss)) = enemies.get(target) {
                    if target_valid(h, boss) {
                        action.snapshot.aim_position = b.aim_position();
                    } else {
                        action.cancel();
                    }
                } else {
                    action.cancel();
                }
            }
            if action.snapshot.kind == ActionKind::ChainCombo
                && !action.cancelled
                && action.snapshot.elapsed <= 0.65
                && !action.pull_stopped
                && let Some(target) = action.snapshot.target.and_then(|t| enemies.get(t).ok())
            {
                let offset = target.1.world_position() - body.world_position();
                let distance = offset.xz().length();
                if offset.y.abs() > 1.5 || !motion.is_grounded() {
                    action.cancel();
                } else if distance <= 1.4 {
                    action.pull_stopped = true;
                } else {
                    let dir = offset.xz().normalize_or_zero();
                    if body.step_displacement().dot(dir) < 0.012
                        && action.snapshot.elapsed > dt * 2.0
                    {
                        action.pull_stalled += dt;
                    } else {
                        action.pull_stalled = 0.0;
                    }
                    if action.pull_stalled + 1e-6 >= 0.12 {
                        action.cancel();
                    }
                }
            }
            if action.cancelled || action.snapshot.elapsed >= action.snapshot.duration {
                combat.axe_away = false;
            }
            let finished = if action.cancelled {
                action.return_remaining == 0.0 || !action.snapshot.kind.is_combo()
            } else {
                false // Final markers are resolved after physics before retiring this action.
            };
            if finished {
                combat.axe_away = false;
                combat.melee_chain.clear();
            } else {
                combat.action = Some(action);
            }
        }
        let busy = combat.action.is_some();
        let buffered_input = if commands & ATTACK != 0 {
            let action_id = combat
                .action
                .as_ref()
                .filter(|action| action.snapshot.kind == ActionKind::Melee && !action.cancelled)
                .map(|action| action.snapshot.action_id);
            action_id.is_some_and(|id| combat.melee_chain.buffer(id))
        } else {
            false
        };
        if busy {
            actions.discard_movement_actions();
        }
        let grounded = motion.is_grounded() && !motion.is_dodging() && !motion.is_sliding();
        let gesture_id = defense.snapshot().gesture_id;
        defense.advance(
            dt,
            grounded && !busy,
            actions.movement_requested(),
            motion,
            stamina,
        );
        if defense.snapshot().gesture_id != gesture_id || motion.is_dodging() || motion.is_sliding()
        {
            combat.melee_chain.clear();
        }
        let free = !busy && !defense.is_active() && !motion.is_dodging() && !motion.is_sliding();
        if let Some(pair) = pair
            && free
            && grounded
        {
            if combat.pair != pair {
                combat.melee_chain.clear();
            }
            combat.pair = pair;
            combat.combo = None;
            combat.axe_away = false;
            combat.status = "切换武器组";
        }
        let current_pair = combat.pair;
        let target_distance = combat
            .target
            .and_then(|t| enemies.get(t).ok())
            .filter(|(t, b, _, h, _, boss)| {
                target_valid(h, *boss)
                    && world.visible(body.aim_position(), b.aim_position(), entity, *t)
            })
            .map(|(_, b, _, _, _, _)| b.world_position().distance(body.world_position()));
        let availability = availability_for(&combat, motion, &defense, target_distance);
        let combo_admit = availability.combo;
        combat.availability = availability;
        // A pending input gets one admission attempt, after the previous
        // action's final markers retired it in advance(). New legal actions
        // retain priority; a pending movement edge is handled by the motor.
        let queued_attack = !busy && std::mem::take(&mut combat.melee_chain.buffered);
        let movement_has_priority = actions.postpones_buffered_attack(motion, dt);
        if queued_attack && movement_has_priority && actions.crouch_toggle_requested() {
            // Let C reach the motor. An accepted slide clears this chain;
            // an ordinary crouch preserves it for the following rule tick.
            combat.melee_chain.buffered = true;
        }
        let kind = if commands & COMBO != 0 && combo_admit {
            Some(if current_pair == WeaponPair::HammerMusket {
                ActionKind::HammerCombo
            } else {
                ActionKind::ChainCombo
            })
        } else if commands & MELEE_SKILL != 0 && availability.melee_skill {
            Some(if current_pair == WeaponPair::HammerMusket {
                ActionKind::HammerSlam
            } else {
                ActionKind::ChainSweep
            })
        } else if commands & RANGED_SKILL != 0 && availability.ranged_skill {
            Some(if current_pair == WeaponPair::HammerMusket {
                ActionKind::MusketShot
            } else {
                ActionKind::AxeShot
            })
        } else if (commands & ATTACK != 0 || (queued_attack && !movement_has_priority))
            && availability.attack
        {
            Some(ActionKind::Melee)
        } else {
            None
        };
        if let Some(kind) = kind {
            let melee_stage = (kind == ActionKind::Melee).then_some(combat.melee_chain.next_stage);
            let combo = if kind.is_combo() {
                combat.combo.take()
            } else {
                None
            };
            let target = if matches!(kind, ActionKind::Melee | ActionKind::ChainSweep) {
                None
            } else {
                combo.map(|c| c.original_target).or(combat.target)
            };
            if kind == ActionKind::ChainCombo
                && target
                    .and_then(|t| enemies.get(t).ok())
                    .is_none_or(|(_, b, _, _, _, _)| {
                        (b.world_position().y - body.world_position().y).abs() > 1.5
                    })
            {
                combat.reject("目标高差过大", entity, body.aim_position(), &mut feedback);
                continue;
            }
            let aim = target.and_then(|t| enemies.get(t).ok()).map_or(
                body.aim_position()
                    + Vec3::new(facing.direction().x, 0.0, facing.direction().y) * 3.0,
                |(_, b, _, _, _, _)| b.aim_position(),
            );
            if kind != ActionKind::Melee {
                facing.set((aim - body.aim_position()).xz());
            }
            let (companion_position, source_position) =
                world.companion_pose(body, facing.direction(), entity);
            let action_id = ids.next();
            let duration = melee_stage.map_or_else(
                || kind.duration(current_pair),
                |stage| current_pair.melee_profile(stage).duration,
            );
            combat.action = Some(ActionRuntime {
                snapshot: ActionSnapshot {
                    action_id,
                    kind,
                    pair: current_pair,
                    melee_stage,
                    target,
                    elapsed: 0.0,
                    duration,
                    shot_count: 0,
                    aim_position: aim,
                    source_position,
                    companion_position,
                },
                previous: 0.0,
                markers: 0,
                cancelled: false,
                return_remaining: 0.0,
                pull_stalled: 0.0,
                pull_stopped: false,
            });
            if let Some(stage) = melee_stage {
                combat.melee_chain.begin(action_id, stage);
                let snapshot = combat.action.as_ref().unwrap().snapshot;
                feedback.action(
                    entity,
                    entity,
                    &snapshot,
                    0,
                    CombatEffect::MeleeStart,
                    body.aim_position(),
                    body.aim_position(),
                    0.0,
                );
            } else {
                combat.melee_chain.clear();
            }
            if kind == ActionKind::ChainCombo {
                combat.axe_away = true;
            }
            combat.availability = ActionAvailability::default();
            match kind {
                ActionKind::Melee => combat.attack_cooldown = duration,
                ActionKind::HammerSlam | ActionKind::ChainSweep => combat.melee_cooldown = 3.0,
                ActionKind::MusketShot | ActionKind::AxeShot => combat.ranged_cooldown = 4.0,
                _ => {}
            }
            combat.status = match kind {
                ActionKind::Melee => "近战攻击",
                ActionKind::HammerSlam => "震地击飞",
                ActionKind::MusketShot => "妹妹鸟铳射击",
                ActionKind::ChainSweep => "链斧环扫",
                ActionKind::AxeShot => "妹妹射出链斧",
                ActionKind::HammerCombo => "妹妹三枪联动",
                ActionKind::ChainCombo => "拉近三箭联动",
            };
            actions.discard_movement_actions();
        } else if commands & (MELEE_SKILL | RANGED_SKILL | COMBO) != 0
            || (commands & ATTACK != 0 && !buffered_input)
        {
            combat.reject(
                if busy {
                    "动作执行中"
                } else if defense.is_active() {
                    "防御中无法进攻"
                } else {
                    "条件不足 / 冷却中"
                },
                entity,
                body.aim_position(),
                &mut feedback,
            );
        }
        if let Some(action) = combat.action.as_ref()
            && action.snapshot.kind == ActionKind::ChainCombo
            && !action.cancelled
            && !action.pull_stopped
            && action.snapshot.elapsed <= 0.65
        {
            combat.pull_velocity = action
                .snapshot
                .target
                .and_then(|t| enemies.get(t).ok())
                .map(|(_, b, _, _, _, _)| {
                    let delta = (b.world_position() - body.world_position()).xz();
                    // Brake early enough for the motor's finite 120 m/s² acceleration.
                    // A hard 18→0 target at 1.4m would overshoot into the victim.
                    let gap = (delta.length() - 1.4).max(0.0);
                    let speed = 18.0_f32.min((2.0 * 120.0 * gap).sqrt() * 0.9).min(gap / dt);
                    delta.normalize_or_zero() * speed
                });
        }
    }
}
type PlayerData = (
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
type EnemyData = (
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
type Players<'w, 's> = Query<'w, 's, PlayerData, Without<Enemy>>;
type Enemies<'w, 's> = Query<'w, 's, EnemyData, With<Enemy>>;
#[derive(Clone, Copy)]
enum Control {
    None,
    Launch,
    Stun(f32),
    Slow,
}
#[allow(clippy::too_many_arguments)]
fn hit_enemy(
    target: Entity,
    owner: Entity,
    source: Entity,
    action: &ActionSnapshot,
    index: u8,
    damage: f32,
    control: Control,
    origin: Vec3,
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
    )) = enemies.get_mut(target)
    else {
        return false;
    };
    let outcome = apply_hit(
        HitContext {
            source,
            source_owner: owner,
            pair: Some(action.pair),
            melee_stage: action.melee_stage,
            target,
            action_id: action.action_id,
            hit_index: index,
            damage,
            origin,
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
    match control {
        Control::None => {}
        Control::Launch => {
            motion.launch_up((2.0 * super::actions::GRAVITY * 3.0).sqrt());
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
        Control::Slow => slow.apply(0.65, 1.5),
    }
    true
}
fn valid_shot(
    world: &CombatWorld,
    source: Vec3,
    owner_body: &Body,
    owner: Entity,
    target: Entity,
    range: f32,
    enemies: &mut Enemies,
) -> Option<Vec3> {
    let Ok((_, body, _, _, _, _, health, _, _, boss, _)) = enemies.get_mut(target) else {
        return None;
    };
    (target_valid(&health, boss.as_deref())
        && owner_body.world_position().distance(body.world_position()) <= range
        && world.visible(source, body.aim_position(), owner, target))
    .then_some(body.aim_position())
}
pub(super) fn advance(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut hit_stop: ResMut<HitStop>,
    mut players: Players,
    mut enemies: Enemies,
    mut ids: ResMut<ActionIds>,
) {
    let dt = time.delta_secs();
    for (
        entity,
        mut body,
        mut motion,
        facing,
        mut movement,
        mut actions,
        mut health,
        mut combat,
        mut intent,
        spawn,
        mut defense,
        mut stamina,
    ) in &mut players
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
        let was_dead = !health.is_alive();
        if health.tick(dt) {
            body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
            motion.reset();
            movement.stop();
            actions.clear();
            intent.clear();
            combat.clear_targets();
            combat.attack_cooldown = 0.0;
            combat.melee_cooldown = 0.0;
            combat.ranged_cooldown = 0.0;
            defense.reset();
            stamina.refill();
            feedback.push(
                entity,
                CombatEffect::Respawn,
                body.aim_position(),
                body.aim_position(),
                0.8,
            );
        }
        if was_dead {
            combat.clear_targets();
            defense.cancel();
            continue;
        }
        let Some(mut runtime) = combat.action.take() else {
            continue;
        };
        if runtime.cancelled {
            combat.action = Some(runtime);
            continue;
        }
        let snapshot = runtime.snapshot;
        match snapshot.kind {
            ActionKind::Melee => {
                let profile = snapshot
                    .pair
                    .melee_profile(snapshot.melee_stage.expect("melee has a committed stage"));
                let range = profile.range;
                if runtime.marker(0, profile.hit_time) {
                    let facing = facing.direction();
                    let forward = Vec3::new(facing.x, 0.0, facing.y);
                    feedback.action(
                        entity,
                        entity,
                        &snapshot,
                        0,
                        CombatEffect::Melee,
                        body.aim_position(),
                        body.aim_position() + forward * range,
                        range,
                    );
                    let targets: Vec<_> = enemies
                        .iter()
                        .filter(|(_, b, _, _, _, _, h, _, _, boss, _)| {
                            target_valid(h, boss.as_deref())
                                && b.world_position().distance(body.world_position()) <= range
                                && (b.world_position() - body.world_position())
                                    .xz()
                                    .normalize_or_zero()
                                    .dot(forward.xz())
                                    > 0.25
                        })
                        .filter(|(t, b, _, _, _, _, _, _, _, _, _)| {
                            world.visible(body.aim_position(), b.aim_position(), entity, *t)
                        })
                        .map(|(e, ..)| e)
                        .collect();
                    for target in targets {
                        hit_enemy(
                            target,
                            entity,
                            entity,
                            &snapshot,
                            0,
                            profile.damage,
                            Control::None,
                            body.aim_position(),
                            &mut enemies,
                            &mut feedback,
                        );
                    }
                }
            }
            ActionKind::HammerSlam => {
                if runtime.marker(0, 0.20)
                    && let Some(target) = snapshot.target
                    && let Some(_) = valid_shot(
                        &world,
                        body.aim_position(),
                        &body,
                        entity,
                        target,
                        4.5,
                        &mut enemies,
                    )
                {
                    let center = enemies.get_mut(target).unwrap().1.world_position();
                    feedback.action(
                        entity,
                        entity,
                        &snapshot,
                        0,
                        CombatEffect::Hammer,
                        center,
                        center,
                        3.0,
                    );
                    let targets: Vec<_> = enemies
                        .iter()
                        .filter(|(_, b, _, _, _, _, h, _, _, boss, _)| {
                            target_valid(h, boss.as_deref())
                                && b.world_position().distance(center) <= 3.0
                        })
                        .filter(|(t, b, _, _, _, _, _, _, _, _, _)| {
                            world.visible(body.aim_position(), b.aim_position(), entity, *t)
                        })
                        .map(|(e, ..)| e)
                        .collect();
                    for t in targets {
                        let launched = hit_enemy(
                            t,
                            entity,
                            entity,
                            &snapshot,
                            0,
                            20.0,
                            Control::Launch,
                            body.aim_position(),
                            &mut enemies,
                            &mut feedback,
                        );
                        if t == target && launched {
                            combat.combo = Some(ComboWindow {
                                source_action_id: snapshot.action_id,
                                pair: snapshot.pair,
                                original_target: target,
                                remaining: 0.85,
                            });
                        }
                    }
                }
            }
            ActionKind::ChainSweep => {
                if runtime.marker(0, 0.15) {
                    feedback.action(
                        entity,
                        entity,
                        &snapshot,
                        0,
                        CombatEffect::ChainSweep,
                        body.aim_position(),
                        body.aim_position(),
                        3.0,
                    );
                    let targets: Vec<_> = enemies
                        .iter()
                        .filter(|(_, b, _, _, _, _, h, _, _, boss, _)| {
                            target_valid(h, boss.as_deref())
                                && b.world_position().distance(body.world_position()) <= 3.0
                        })
                        .filter(|(t, b, _, _, _, _, _, _, _, _, _)| {
                            world.visible(body.aim_position(), b.aim_position(), entity, *t)
                        })
                        .map(|(e, ..)| e)
                        .collect();
                    for t in targets {
                        hit_enemy(
                            t,
                            entity,
                            entity,
                            &snapshot,
                            0,
                            24.0,
                            Control::Slow,
                            body.aim_position(),
                            &mut enemies,
                            &mut feedback,
                        );
                    }
                }
            }
            ActionKind::MusketShot | ActionKind::AxeShot => {
                if snapshot.kind == ActionKind::AxeShot && runtime.marker(0, 0.15) {
                    combat.axe_away = true;
                    runtime.snapshot.shot_count += 1;
                    feedback.action(
                        combat.companion.unwrap_or(entity),
                        entity,
                        &snapshot,
                        0,
                        CombatEffect::Shot,
                        snapshot.source_position,
                        snapshot.aim_position,
                        0.22,
                    );
                    feedback.action(
                        combat.companion.unwrap_or(entity),
                        entity,
                        &snapshot,
                        0,
                        CombatEffect::AxeThrow,
                        snapshot.source_position,
                        snapshot.aim_position,
                        0.3,
                    );
                }
                let marker = if snapshot.kind == ActionKind::MusketShot {
                    0.18
                } else {
                    0.30
                };
                if runtime.marker(1, marker) {
                    let performer = combat.companion.unwrap_or(entity);
                    // The chain axe departed at .15; .30 applies its impact only.
                    if snapshot.kind == ActionKind::MusketShot {
                        feedback.action(
                            performer,
                            entity,
                            &snapshot,
                            0,
                            CombatEffect::Shot,
                            snapshot.source_position,
                            snapshot.aim_position,
                            0.22,
                        );
                        runtime.snapshot.shot_count += 1;
                    }
                    if let Some(target) = snapshot.target
                        && let Some(aim) = valid_shot(
                            &world,
                            snapshot.source_position,
                            &body,
                            entity,
                            target,
                            if snapshot.kind == ActionKind::MusketShot {
                                22.0
                            } else {
                                12.0
                            },
                            &mut enemies,
                        )
                    {
                        runtime.snapshot.aim_position = aim;
                        let chain = snapshot.kind == ActionKind::AxeShot;
                        let stunned = hit_enemy(
                            target,
                            entity,
                            performer,
                            &snapshot,
                            0,
                            if chain { 35.0 } else { 40.0 },
                            Control::Stun(if chain { 0.90 } else { 0.45 }),
                            snapshot.source_position,
                            &mut enemies,
                            &mut feedback,
                        );
                        if chain && stunned {
                            combat.combo = Some(ComboWindow {
                                source_action_id: snapshot.action_id,
                                pair: snapshot.pair,
                                original_target: target,
                                remaining: 0.90,
                            });
                        }
                    }
                }
            }
            ActionKind::HammerCombo | ActionKind::ChainCombo => {
                let hammer = snapshot.kind == ActionKind::HammerCombo;
                let markers = if hammer {
                    [0.30, 0.42, 0.54]
                } else {
                    [0.12, 0.28, 0.44]
                };
                for (index, marker) in markers.into_iter().enumerate() {
                    if !runtime.marker(index as u8, marker) {
                        continue;
                    }
                    let performer = combat.companion.unwrap_or(entity);
                    let target = snapshot.target.unwrap();
                    if enemies
                        .get_mut(target)
                        .map_or(true, |(_, _, _, _, _, _, h, _, _, b, _)| {
                            !target_valid(&h, b.as_deref())
                        })
                    {
                        runtime.cancel();
                        combat.axe_away = false;
                        break;
                    }
                    runtime.snapshot.shot_count += 1;
                    let aim = valid_shot(
                        &world,
                        snapshot.source_position,
                        &body,
                        entity,
                        target,
                        22.0,
                        &mut enemies,
                    );
                    feedback.action(
                        performer,
                        entity,
                        &snapshot,
                        index as u8,
                        CombatEffect::Shot,
                        snapshot.source_position,
                        aim.unwrap_or(runtime.snapshot.aim_position),
                        0.22,
                    );
                    if let Some(aim) = aim {
                        runtime.snapshot.aim_position = aim;
                        let alive = hit_enemy(
                            target,
                            entity,
                            performer,
                            &snapshot,
                            index as u8,
                            if hammer { 28.0 } else { 18.0 },
                            Control::None,
                            snapshot.source_position,
                            &mut enemies,
                            &mut feedback,
                        );
                        if hammer {
                            feedback.action(
                                performer,
                                entity,
                                &snapshot,
                                index as u8,
                                CombatEffect::SniperShot,
                                snapshot.source_position,
                                aim,
                                0.0,
                            );
                            feedback.action(
                                target,
                                entity,
                                &snapshot,
                                index as u8,
                                CombatEffect::SniperImpact,
                                aim,
                                snapshot.source_position,
                                0.0,
                            );
                            if index == 2 {
                                hit_stop.request_sniper();
                            }
                        }
                        if !alive {
                            runtime.cancel();
                            combat.axe_away = false;
                            break;
                        }
                    }
                }
            }
        }
        if !runtime.cancelled && runtime.snapshot.elapsed + 1e-6 >= runtime.snapshot.duration {
            combat.axe_away = false;
            if runtime.snapshot.kind == ActionKind::Melee {
                combat.melee_chain.finish(runtime.snapshot.action_id);
            }
        } else {
            combat.action = Some(runtime);
        }
    }
    // Remove a killed selected target in the damage tick, before HitStop freezes anything.
    for (_, _, _, _, _, _, _, mut combat, _, _, _, _) in &mut players {
        if combat.target.is_some_and(|t| {
            enemies
                .get_mut(t)
                .map_or(true, |(_, _, _, _, _, _, h, _, _, b, _)| {
                    !target_valid(&h, b.as_deref())
                })
        }) {
            combat.target = None;
        }
        if combat.combo.is_some_and(|c| {
            enemies
                .get_mut(c.original_target)
                .map_or(true, |(_, _, _, _, _, _, h, _, _, b, _)| {
                    !target_valid(&h, b.as_deref())
                })
        }) {
            combat.combo = None;
        }
    }
    advance_enemies(
        dt,
        &world,
        &mut players,
        &mut enemies,
        &mut feedback,
        &mut ids,
    );
}
fn advance_enemies(
    dt: f32,
    world: &CombatWorld,
    players: &mut Players,
    enemies: &mut Enemies,
    feedback: &mut CombatFeedback,
    ids: &mut ActionIds,
) {
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
    ) in enemies
    {
        if boss.is_some() {
            continue;
        }
        actions.clear();
        enemy.cooldown = decay(enemy.cooldown, dt);
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
                enemy.cooldown = 1.25;
                feedback.push(
                    entity,
                    CombatEffect::Melee,
                    body.aim_position(),
                    body.aim_position()
                        + Vec3::new(facing.direction().x, 0.0, facing.direction().y) * 2.0,
                    2.0,
                );
                if let Some(target) = enemy.attack_target.take()
                    && let Ok((
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
                    && target_health.is_alive()
                    && body.world_position().distance(target_body.world_position()) <= 2.0
                    && world.visible(
                        body.aim_position(),
                        target_body.aim_position(),
                        entity,
                        target,
                    )
                {
                    let result = apply_hit(
                        HitContext {
                            source: entity,
                            source_owner: entity,
                            pair: None,
                            melee_stage: None,
                            target,
                            action_id: enemy.attack_id,
                            hit_index: 0,
                            damage: 12.0,
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
                    if result == HitOutcome::Parried {
                        let id = enemy.attack_id;
                        enemy.stun(id, 0.80);
                        movement.stop();
                    }
                    if result == HitOutcome::Damaged {
                        target_combat.interrupt_melee_chain();
                    }
                    if !target_health.is_alive() {
                        target_combat.clear_targets();
                        target_intent.clear();
                        target_actions.clear();
                    }
                }
            }
            continue;
        }
        let target = players
            .iter()
            .filter(|(_, _, _, _, _, _, h, _, _, _, _, _)| h.is_alive())
            .map(|(e, b, _, _, _, _, _, _, _, _, _, _)| (e, b.world_position(), b.aim_position()))
            .filter(|(_, p, _)| p.distance(body.world_position()) < 12.0)
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
        if delta.length() <= 1.7 && world.visible(body.aim_position(), aim, entity, target) {
            movement.stop();
            if enemy.cooldown == 0.0 {
                enemy.windup = 0.42;
                enemy.attack_target = Some(target);
                enemy.attack_id = ids.next();
                feedback.push(
                    entity,
                    CombatEffect::EnemyWindup,
                    body.aim_position(),
                    aim,
                    2.0,
                );
            }
        } else {
            movement.set(dir);
        }
    }
}
pub(super) fn finish_resources(
    time: Res<Time<Fixed>>,
    mut players: Query<(&Combatant, &Health, &mut Stamina)>,
    mut companions: Query<(&Companion, &mut CompanionState)>,
) {
    for (combat, health, mut stamina) in &mut players {
        if health.is_alive() {
            if combat.busy() {
                stamina.disturb();
            } else {
                stamina.recover(time.delta_secs());
            }
        }
    }
    for (companion, mut state) in &mut companions {
        state.previous_position = state.position;
        let Ok((combat, health, _)) = players.get(companion.owner) else {
            state.visible = false;
            continue;
        };
        let Some(action) = combat.active_action() else {
            state.visible = false;
            continue;
        };
        state.visible = health.is_alive()
            && !combat.action.as_ref().unwrap().cancelled
            && matches!(
                action.kind,
                ActionKind::MusketShot
                    | ActionKind::AxeShot
                    | ActionKind::HammerCombo
                    | ActionKind::ChainCombo
            );
        if state.action_id != action.action_id {
            state.previous_position = action.companion_position;
        }
        state.position = action.companion_position;
        state.source_position = action.source_position;
        state.aim_position = action.aim_position;
        state.facing = (action.aim_position - action.source_position)
            .xz()
            .normalize_or(Vec2::NEG_Y);
        state.pair = action.pair;
        state.action_id = action.action_id;
        state.kind = action.kind;
        state.elapsed = action.elapsed;
        state.shot_count = action.shot_count;
    }
}

#[cfg(test)]
#[path = "combat_tests.rs"]
mod tests;

type AvailabilityPlayer = (
    Entity,
    &'static Body,
    &'static MotionState,
    &'static DefenseState,
    &'static Health,
    &'static mut Combatant,
);

pub(super) fn publish_availability(
    world: CombatWorld,
    enemies: Query<CandidateData, With<Enemy>>,
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
