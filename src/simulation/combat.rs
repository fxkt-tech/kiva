//! Fixed-step combat. Device input and rendered transforms never enter this module.

use std::collections::VecDeque;

use avian3d::prelude::{
    Forces, ReadRigidBodyForces, Sensor, SpatialQuery, SpatialQueryFilter, WriteRigidBodyForces,
};
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{
    ActionIntent, Body, Facing, HitStop, MotionState, MoveIntent, PhysicsPose, STANDING_HEIGHT,
    SurfaceHit, WeaponMode, WeaponSet, character_collider,
};

const TARGET_RANGE: f32 = 22.0;
const HAMMER_RANGE: f32 = 4.5;
const HAMMER_RADIUS: f32 = 3.0;
const HAMMER_KNOCKUP_HEIGHT: f32 = 3.0;
const LEAP_DURATION: f32 = 0.9;
const SPIN_RADIUS: f32 = 2.8;
const RESPAWN_DELAY: f32 = 3.0;
const HISTORY_LIMIT: usize = 128;
const ATTACK: u8 = 1;
const SKILL: u8 = 2;
const COMBO: u8 = 4;
const SWITCH: u8 = 8;
const CYCLE: u8 = 16;

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

    fn tick(&mut self, dt: f32) -> bool {
        self.hit_remaining = (self.hit_remaining - dt).max(0.0);
        if self.is_alive() {
            return false;
        }
        self.respawn_remaining = (self.respawn_remaining - dt).max(0.0);
        if self.respawn_remaining == 0.0 {
            self.current = self.maximum;
            true
        } else {
            false
        }
    }

    fn damage(&mut self, amount: f32) -> bool {
        if !self.is_alive() {
            return false;
        }
        self.current = (self.current - amount).max(0.0);
        self.hit_remaining = 0.18;
        if !self.is_alive() {
            self.respawn_remaining = RESPAWN_DELAY;
        }
        true
    }
}

#[derive(Component, Clone, Copy)]
pub struct SpawnPoint(pub Vec2);

/// Each enemy owns its attack timing; no client timer can create damage.
#[derive(Component)]
pub struct Enemy {
    passive: bool,
    cooldown: f32,
    windup: f32,
    attack_target: Option<Entity>,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            passive: false,
            cooldown: 0.8,
            windup: 0.0,
            attack_target: None,
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

    fn reset(&mut self) {
        *self = if self.passive {
            Self::target_dummy()
        } else {
            Self::default()
        };
    }

    pub fn is_winding_up(&self) -> bool {
        self.windup > 0.0
    }
    pub fn windup_remaining(&self) -> f32 {
        self.windup
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatEffect {
    Melee,
    Shot,
    SniperShot,
    SniperImpact,
    Hammer,
    Cannon,
    Spin,
    Hit,
    Block,
    Dodge,
    Defeat,
    Respawn,
    EnemyWindup,
}

#[derive(Clone, Copy, Debug)]
pub struct CombatEvent {
    pub sequence: u64,
    /// Performer for attacks; recipient for hit, defense, defeat and respawn.
    pub actor: Entity,
    pub kind: CombatEffect,
    pub position: Vec3,
    pub end: Vec3,
    pub radius: f32,
}

/// Persistent, bounded history lets rendering consume all fixed ticks since its last frame.
#[derive(Resource, Default)]
pub struct CombatFeedback {
    next_sequence: u64,
    events: VecDeque<CombatEvent>,
}

impl CombatFeedback {
    pub fn events(&self) -> impl Iterator<Item = &CombatEvent> {
        self.events.iter()
    }

    fn push(&mut self, actor: Entity, kind: CombatEffect, position: Vec3, end: Vec3, radius: f32) {
        self.next_sequence += 1;
        self.events.push_back(CombatEvent {
            sequence: self.next_sequence,
            actor,
            kind,
            position,
            end,
            radius,
        });
        if self.events.len() > HISTORY_LIMIT {
            self.events.pop_front();
        }
    }
}

/// Held movement is separate; these commands remain latched until a fixed tick.
#[derive(Component, Default)]
pub struct CombatIntent {
    commands: u8,
    requested_set: Option<WeaponSet>,
    aim: Option<Vec2>,
}

impl CombatIntent {
    pub fn set_aim(&mut self, point: Vec2) {
        self.aim = point.is_finite().then_some(point);
    }
    pub fn request_attack(&mut self) {
        self.commands |= ATTACK;
    }
    pub fn request_skill(&mut self) {
        self.commands |= SKILL;
    }
    pub fn request_combo(&mut self) {
        self.commands |= COMBO;
    }
    pub fn request_switch_weapon(&mut self) {
        self.commands |= SWITCH;
    }
    pub fn request_set(&mut self, set: WeaponSet) {
        self.requested_set = Some(set);
    }
    pub fn request_cycle_target(&mut self) {
        self.commands |= CYCLE;
    }
    pub fn clear(&mut self) {
        self.commands = 0;
        self.requested_set = None;
        self.aim = None;
    }
}

#[derive(Clone, Copy)]
enum Link {
    Hammer(Entity),
    Cannon,
}

#[derive(Clone, Copy)]
struct ComboWindow {
    link: Link,
    remaining: f32,
}

/// The fixed-step timeline for the hammer's committed sniper follow-up.
/// Presentation reads this snapshot; it cannot advance or discharge the shot.
#[derive(Clone, Copy, Debug)]
pub struct SniperSequence {
    target: Entity,
    elapsed: f32,
    aim_position: Vec3,
    fired: bool,
}

impl SniperSequence {
    pub const AIM_DURATION: f32 = 0.30;
    pub const HOLD_DURATION: f32 = 0.14;
    pub const RETURN_DURATION: f32 = 0.28;
    pub const TOTAL_DURATION: f32 =
        Self::AIM_DURATION + Self::HOLD_DURATION + Self::RETURN_DURATION;

    pub fn target(self) -> Entity {
        self.target
    }

    pub fn elapsed(self) -> f32 {
        self.elapsed
    }

    pub fn aim_position(self) -> Vec3 {
        self.aim_position
    }

    pub fn has_fired(self) -> bool {
        self.fired
    }
}

#[derive(Component)]
pub struct Combatant {
    set: WeaponSet,
    mode: WeaponMode,
    target: Option<Entity>,
    aim: Vec3,
    aim_surface: Option<SurfaceHit>,
    attack_cooldown: f32,
    skill_cooldown: f32,
    combo: Option<ComboWindow>,
    sniper: Option<SniperSequence>,
    spin_remaining: f32,
    spin_hits: Vec<Entity>,
    status: &'static str,
    status_remaining: f32,
}

impl Combatant {
    pub fn new(set: WeaponSet) -> Self {
        Self {
            set,
            mode: WeaponMode::Melee,
            target: None,
            aim: Vec3::NEG_Z * 6.0,
            aim_surface: None,
            attack_cooldown: 0.0,
            skill_cooldown: 0.0,
            combo: None,
            sniper: None,
            spin_remaining: 0.0,
            spin_hits: Vec::new(),
            status: "准备就绪",
            status_remaining: 0.0,
        }
    }
    pub fn weapon_set(&self) -> WeaponSet {
        self.set
    }
    pub fn weapon_mode(&self) -> WeaponMode {
        self.mode
    }
    pub fn weapon_name(&self) -> &'static str {
        self.set.weapon_name(self.mode)
    }
    pub fn persona(&self) -> &'static str {
        self.mode.persona()
    }
    pub fn target(&self) -> Option<Entity> {
        self.target
    }
    pub fn combo_remaining(&self) -> f32 {
        self.combo.map_or(0.0, |combo| combo.remaining)
    }
    pub fn cooldown(&self) -> f32 {
        self.skill_cooldown
    }
    pub fn cooldown_remaining(&self) -> f32 {
        self.skill_cooldown
    }
    pub fn attack_cooldown(&self) -> f32 {
        self.attack_cooldown
    }
    pub fn aim(&self) -> Vec3 {
        self.aim
    }
    pub fn aim_surface(&self) -> Option<SurfaceHit> {
        self.aim_surface
    }
    pub fn is_spinning(&self) -> bool {
        self.spin_remaining > 0.0
    }
    pub fn sniper_sequence(&self) -> Option<SniperSequence> {
        self.sniper
    }
    pub fn is_sniping(&self) -> bool {
        self.sniper.is_some()
    }
    pub fn status(&self) -> &'static str {
        self.status
    }

    /// A roster change invalidates entity references, while preserving the loadout and cooldowns.
    pub fn clear_targets(&mut self) {
        self.target = None;
        self.cancel_link();
    }

    fn notify(&mut self, status: &'static str) {
        self.status = status;
        self.status_remaining = 1.4;
    }

    fn cancel_link(&mut self) {
        self.combo = None;
        self.sniper = None;
        self.spin_remaining = 0.0;
        self.spin_hits.clear();
    }
}

type PlayerData<'a> = (
    Entity,
    &'a mut Body,
    &'a mut MotionState,
    &'a Facing,
    &'a mut MoveIntent,
    &'a mut ActionIntent,
    &'a mut Health,
    &'a mut Combatant,
    &'a mut CombatIntent,
    &'a SpawnPoint,
);
type EnemyData<'a> = (
    Entity,
    &'a mut Body,
    &'a mut MotionState,
    &'a mut Facing,
    &'a mut MoveIntent,
    &'a mut ActionIntent,
    &'a mut Health,
    &'a mut Enemy,
    &'a SpawnPoint,
);
type Players<'w, 's> = Query<'w, 's, PlayerData<'static>, Without<Enemy>>;
type Enemies<'w, 's> = Query<'w, 's, EnemyData<'static>, With<Enemy>>;

#[derive(Clone, Copy)]
struct Target {
    entity: Entity,
    position: Vec3,
}

fn point(position: Vec3) -> Vec3 {
    position + Vec3::Y * 0.9
}

#[derive(SystemParam)]
pub(super) struct CombatWorld<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    sensors: Query<'w, 's, (), With<Sensor>>,
    actors: Query<'w, 's, (), With<Body>>,
}

impl CombatWorld<'_, '_> {
    fn hit(&self, start: Vec3, end: Vec3, excluded: &[Entity]) -> Option<SurfaceHit> {
        let delta = end - start;
        let direction = Dir3::new(delta).ok()?;
        self.spatial
            .cast_ray_predicate(
                start,
                direction,
                delta.length(),
                true,
                &SpatialQueryFilter::from_excluded_entities(excluded.iter().copied()),
                &|entity| !self.sensors.contains(entity),
            )
            .map(|hit| SurfaceHit {
                entity: hit.entity,
                position: start + *direction * hit.distance,
                normal: hit.normal,
            })
    }

    fn visible(&self, start: Vec3, end: Vec3, observer: Entity, target: Entity) -> bool {
        self.hit(start, end, &[observer, target])
            .is_none_or(|hit| hit.position.distance(end) < 0.03)
    }

    fn landing(
        &self,
        observer: Entity,
        origin: Vec3,
        requested: Vec2,
        radius: f32,
    ) -> Option<SurfaceHit> {
        let offset = (requested - origin.xz()).clamp_length_max(8.0);
        let start = origin + Vec3::new(offset.x, 8.0, offset.y);
        let filter = SpatialQueryFilter::from_excluded_entities([observer]);
        let hit = self.spatial.cast_ray_predicate(
            start,
            Dir3::NEG_Y,
            16.0,
            true,
            &filter,
            &|entity| !self.actors.contains(entity) && !self.sensors.contains(entity),
        )?;
        if hit.normal.y < std::f32::consts::FRAC_1_SQRT_2 {
            return None;
        }
        let surface = SurfaceHit {
            entity: hit.entity,
            position: start - Vec3::Y * hit.distance,
            normal: hit.normal,
        };
        let standing = character_collider(radius, STANDING_HEIGHT);
        let center = surface.position + Vec3::Y * (STANDING_HEIGHT * 0.5 + 0.04);
        let clearance = SpatialQueryFilter::from_excluded_entities([observer, surface.entity]);
        if self
            .spatial
            .shape_intersections(&standing, center, Quat::IDENTITY, &clearance)
            .iter()
            .any(|entity| !self.sensors.contains(*entity))
        {
            return None;
        }
        Some(surface)
    }
}

fn select_target(
    targets: &[Target],
    current: Option<Entity>,
    origin: Vec3,
    facing: Vec2,
    cycle: bool,
) -> Option<Entity> {
    if cycle {
        let index = current.and_then(|current| targets.iter().position(|t| t.entity == current));
        return targets
            .get(index.map_or(0, |index| (index + 1) % targets.len().max(1)))
            .map(|target| target.entity);
    }
    if targets.iter().any(|target| Some(target.entity) == current) {
        return current;
    }
    targets
        .iter()
        .filter_map(|target| {
            let offset = (target.position - origin).xz();
            let alignment = offset.normalize_or_zero().dot(facing);
            (alignment > 0.35).then_some((target.entity, alignment - offset.length() * 0.01))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(entity, _)| entity)
}

fn hurt_enemy(target: Entity, amount: f32, enemies: &mut Enemies, feedback: &mut CombatFeedback) {
    let Ok((_, body, _, _, _, _, mut health, _, _)) = enemies.get_mut(target) else {
        return;
    };
    if health.damage(amount) {
        let position = point(body.world_position());
        feedback.push(target, CombatEffect::Hit, position, position, 0.45);
        if !health.is_alive() {
            feedback.push(target, CombatEffect::Defeat, position, position, 0.8);
        }
    }
}

fn link_is_valid(combo: ComboWindow, motion: &MotionState, enemies: &mut Enemies) -> bool {
    match combo.link {
        Link::Hammer(target) => {
            enemies
                .get_mut(target)
                .is_ok_and(|(_, _, motion, _, _, _, health, _, _)| {
                    health.is_alive() && !motion.is_grounded()
                })
        }
        Link::Cannon => motion.is_leaping(),
    }
}

fn fall_defeat(entity: Entity, body: &Body, health: &mut Health, feedback: &mut CombatFeedback) {
    if body.world_position().y < -12.0 && health.is_alive() {
        health.damage(health.maximum());
        let position = point(body.world_position());
        feedback.push(entity, CombatEffect::Defeat, position, position, 0.9);
    }
}

fn push_props(
    world: &CombatWorld,
    props: &mut Query<(Entity, Forces), With<PhysicsPose>>,
    actor: Entity,
    origin: Vec3,
    radius: f32,
    strength: f32,
) {
    for (entity, mut forces) in props {
        let position = forces.position();
        let delta = position - origin;
        if delta.length() <= radius && world.visible(origin, position, actor, entity) {
            let direction = (delta.normalize_or(Vec3::Y) + Vec3::Y * 0.35).normalize();
            forces.apply_linear_impulse_at_point(direction * strength, position + Vec3::Y * 0.3);
        }
    }
}

pub(super) fn advance(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut hit_stop: ResMut<HitStop>,
    mut players: Players,
    mut enemies: Enemies,
    mut props: Query<(Entity, Forces), With<PhysicsPose>>,
) {
    let dt = time.delta_secs();

    for (
        entity,
        mut body,
        mut motion,
        _,
        mut movement,
        mut actions,
        mut health,
        mut enemy,
        spawn,
    ) in &mut enemies
    {
        fall_defeat(entity, &body, &mut health, &mut feedback);
        if health.tick(dt) {
            body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
            motion.reset();
            actions.clear();
            movement.stop();
            enemy.reset();
            let position = point(body.world_position());
            feedback.push(entity, CombatEffect::Respawn, position, position, 0.8);
        }
    }

    for (
        entity,
        mut body,
        mut motion,
        facing,
        mut movement,
        mut actions,
        mut health,
        mut actor,
        mut intent,
        spawn,
    ) in &mut players
    {
        fall_defeat(entity, &body, &mut health, &mut feedback);
        if health.tick(dt) {
            body.teleport_to(Vec3::new(spawn.0.x, 0.0, spawn.0.y));
            motion.reset();
            actions.clear();
            movement.stop();
            intent.clear();
            actor.cancel_link();
            actor.target = None;
            actor.attack_cooldown = 0.0;
            actor.skill_cooldown = 0.0;
            actor.notify("已恢复，继续练习");
            let position = point(body.world_position());
            feedback.push(entity, CombatEffect::Respawn, position, position, 0.8);
        }
        if !health.is_alive() {
            intent.clear();
            actions.clear();
            movement.stop();
            actor.cancel_link();
            actor.notify("倒地，3 秒后恢复");
            continue;
        }

        actor.attack_cooldown = (actor.attack_cooldown - dt).max(0.0);
        actor.skill_cooldown = (actor.skill_cooldown - dt).max(0.0);
        actor.status_remaining = (actor.status_remaining - dt).max(0.0);
        if actor.status_remaining == 0.0 {
            actor.status = "准备就绪";
        }
        if let Some(mut combo) = actor.combo {
            combo.remaining -= dt;
            if combo.remaining <= 0.0 || !link_is_valid(combo, &motion, &mut enemies) {
                actor.combo = None;
                actor.notify("联动窗口结束");
            } else {
                actor.combo = Some(combo);
            }
        }

        let origin = body.world_position();
        let direction = facing.direction();
        if !motion.is_leaping() {
            let requested_aim = intent.aim.unwrap_or(origin.xz() + direction * 6.0);
            // A leap commits its landing point; later look input only changes facing.
            actor.aim_surface = world.landing(entity, origin, requested_aim, body.radius());
            if let Some(surface) = actor.aim_surface {
                actor.aim = surface.position;
            }
        }
        let mut targets: Vec<_> = enemies
            .iter()
            .filter(|(target, body, _, _, _, _, health, _, _)| {
                health.is_alive()
                    && body.world_position().distance(origin) <= TARGET_RANGE
                    && world.visible(point(origin), point(body.world_position()), entity, *target)
            })
            .map(|(entity, body, _, _, _, _, _, _, _)| Target {
                entity,
                position: body.world_position(),
            })
            .collect();
        targets.sort_by_key(|target| target.entity.to_bits());
        let commands = intent.commands;
        intent.commands = 0;
        if let Some(set) = intent.requested_set.take()
            && (actor.set != set || actor.is_sniping())
        {
            actor.set = set;
            actor.mode = WeaponMode::Melee;
            actor.cancel_link();
            actor.notify("已切换武器套装");
        }
        if commands & SWITCH != 0 {
            actor.mode = actor.mode.other();
            actor.cancel_link();
            actor.notify("已切换人格与武器");
        }
        actor.target = select_target(
            &targets,
            actor.target,
            origin,
            direction,
            commands & CYCLE != 0,
        );

        if let Some(mut sniper) = actor.sniper.take() {
            sniper.elapsed += dt;
            let target = enemies.get_mut(sniper.target).ok().map(
                |(_, body, motion, _, _, _, health, _, _)| {
                    (
                        point(body.world_position()),
                        health.is_alive(),
                        !motion.is_grounded(),
                    )
                },
            );
            if let Some((end, alive, airborne)) = target {
                let can_aim = !motion.is_dodging()
                    && !motion.is_blocking()
                    && !motion.is_sliding()
                    && !actor.is_spinning();
                let mut keep = true;
                if !sniper.fired && (!alive || !airborne || !can_aim) {
                    actor.notify("狙击联动已取消");
                    keep = false;
                } else {
                    if alive {
                        sniper.aim_position = end;
                    }
                    if !sniper.fired && sniper.elapsed >= SniperSequence::AIM_DURATION {
                        if world.visible(point(origin), end, entity, sniper.target) {
                            sniper.fired = true;
                            actor.attack_cooldown = 0.9;
                            actor.notify("妹妹 · 联动狙击");
                            feedback.push(entity, CombatEffect::Shot, point(origin), end, 0.12);
                            hurt_enemy(sniper.target, 85.0, &mut enemies, &mut feedback);
                            if actor.target == Some(sniper.target)
                                && enemies.get(sniper.target).is_ok_and(
                                    |(_, _, _, _, _, _, health, _, _)| !health.is_alive(),
                                )
                            {
                                // Target selection precedes damage. Clear a killed
                                // selection before hit stop freezes the next ticks;
                                // the fired sequence keeps its own final aim point.
                                actor.target = None;
                            }
                            feedback.push(
                                entity,
                                CombatEffect::SniperShot,
                                point(origin),
                                end,
                                0.12,
                            );
                            feedback.push(
                                sniper.target,
                                CombatEffect::SniperImpact,
                                end,
                                point(origin),
                                0.65,
                            );
                            hit_stop.request_sniper();
                        } else {
                            actor.notify("狙击视线被障碍阻挡");
                            keep = false;
                        }
                    }
                }
                if keep && sniper.elapsed < SniperSequence::TOTAL_DURATION {
                    actor.sniper = Some(sniper);
                }
            } else {
                actor.notify("狙击目标已失效");
            }
        }

        if actor.is_spinning() {
            actor.spin_remaining = (actor.spin_remaining - dt).max(0.0);
            if !motion.is_leaping() {
                actor.spin_remaining = 0.0;
            } else {
                for target in &targets {
                    if target.position.distance(origin) <= SPIN_RADIUS
                        && !actor.spin_hits.contains(&target.entity)
                    {
                        actor.spin_hits.push(target.entity);
                        hurt_enemy(target.entity, 42.0, &mut enemies, &mut feedback);
                    }
                }
                for (prop, mut forces) in &mut props {
                    let position = forces.position();
                    if position.distance(point(origin)) <= SPIN_RADIUS
                        && !actor.spin_hits.contains(&prop)
                        && world.visible(point(origin), position, entity, prop)
                    {
                        actor.spin_hits.push(prop);
                        let impulse = (position - point(origin)).normalize_or(Vec3::Y) * 30.0;
                        forces.apply_linear_impulse_at_point(impulse, position + Vec3::Y * 0.3);
                    }
                }
            }
        }

        // Commands have already been consumed; do not queue an extra action or
        // replace the aiming feedback when Tab/attack is pressed repeatedly.
        let commands = if actor.is_sniping() {
            commands & !(ATTACK | SKILL | COMBO)
        } else {
            commands
        };
        let can_act = !motion.is_dodging()
            && !motion.is_blocking()
            && !motion.is_sliding()
            && !actor.is_spinning()
            && !actor.is_sniping();
        let selected = targets
            .iter()
            .find(|target| Some(target.entity) == actor.target);

        if commands & SKILL != 0 {
            if !can_act || !motion.is_grounded() {
                actor.notify("落地并结束防御动作后使用技能");
            } else if actor.skill_cooldown > 0.0 {
                actor.notify("起手技能冷却中");
            } else {
                match (actor.set, actor.mode) {
                    (WeaponSet::HammerSniper, WeaponMode::Melee) => {
                        if let Some(target) = selected {
                            if target.position.distance(origin) <= HAMMER_RANGE {
                                for nearby in &targets {
                                    if nearby.position.distance(target.position) <= HAMMER_RADIUS
                                        && world.visible(
                                            point(target.position),
                                            point(nearby.position),
                                            target.entity,
                                            nearby.entity,
                                        )
                                    {
                                        hurt_enemy(
                                            nearby.entity,
                                            20.0,
                                            &mut enemies,
                                            &mut feedback,
                                        );
                                        if let Ok((_, _, mut state, _, _, _, health, _, _)) =
                                            enemies.get_mut(nearby.entity)
                                            && health.is_alive()
                                        {
                                            state.launch_up(
                                                (2.0 * super::actions::GRAVITY
                                                    * HAMMER_KNOCKUP_HEIGHT)
                                                    .sqrt(),
                                            );
                                        }
                                    }
                                }
                                actor.combo = Some(ComboWindow {
                                    link: Link::Hammer(target.entity),
                                    remaining: 0.85,
                                });
                                actor.skill_cooldown = 3.0;
                                actor.attack_cooldown = 0.45;
                                actor.notify("击飞！Tab 衔接狙击");
                                push_props(
                                    &world,
                                    &mut props,
                                    entity,
                                    point(target.position),
                                    HAMMER_RADIUS,
                                    65.0,
                                );
                                let position = point(target.position);
                                feedback.push(
                                    entity,
                                    CombatEffect::Hammer,
                                    position,
                                    position,
                                    HAMMER_RADIUS,
                                );
                            } else {
                                actor.notify("锤击目标距离过远");
                            }
                        } else {
                            actor.notify("面向敌人或按 Q 锁定目标");
                        }
                    }
                    (WeaponSet::AxeCannon, WeaponMode::Ranged) => {
                        if actor.aim_surface.is_none() || actor.aim.distance(origin) < 0.5 {
                            actor.notify("落点不可站立或被障碍占据");
                        } else if motion.start_leap_to(origin, actor.aim, LEAP_DURATION) {
                            actor.combo = Some(ComboWindow {
                                link: Link::Cannon,
                                remaining: LEAP_DURATION,
                            });
                            actor.skill_cooldown = 3.5;
                            actor.attack_cooldown = LEAP_DURATION;
                            actor.notify("火炮跃迁！空中 Tab 衔接旋斧");
                            feedback.push(
                                entity,
                                CombatEffect::Cannon,
                                point(origin),
                                point(actor.aim),
                                1.5,
                            );
                        }
                    }
                    _ => actor.notify("滚轮切换至锤子或火炮释放起手技"),
                }
            }
        }

        if commands & COMBO != 0 {
            // Tab is an immediate attempt, never a queued attack after defense ends.
            if let Some(combo) = actor.combo.take() {
                if can_act && link_is_valid(combo, &motion, &mut enemies) {
                    match combo.link {
                        Link::Hammer(target) => {
                            if let Ok((_, target_body, _, _, _, _, _, _, _)) =
                                enemies.get_mut(target)
                            {
                                let end = point(target_body.world_position());
                                if world.visible(point(origin), end, entity, target) {
                                    actor.mode = WeaponMode::Ranged;
                                    actor.attack_cooldown = 0.9;
                                    actor.sniper = Some(SniperSequence {
                                        target,
                                        elapsed: 0.0,
                                        aim_position: end,
                                        fired: false,
                                    });
                                    actor.notify("妹妹 · 瞄准浮空目标");
                                } else {
                                    actor.notify("狙击视线被障碍阻挡");
                                }
                            }
                        }
                        Link::Cannon => {
                            actor.mode = WeaponMode::Melee;
                            actor.spin_remaining = combo.remaining;
                            actor.spin_hits.clear();
                            actor.notify("哥哥 · 空中旋斧");
                            feedback.push(
                                entity,
                                CombatEffect::Spin,
                                point(origin),
                                point(actor.aim),
                                SPIN_RADIUS,
                            );
                        }
                    }
                } else {
                    actor.notify("联动窗口已失效");
                }
            } else {
                actor.notify("先用 E 打开联动窗口");
            }
        }

        if commands & ATTACK != 0 && commands & (SKILL | COMBO) == 0 {
            if !can_act || motion.is_leaping() {
                actor.notify("当前动作中无法普攻");
            } else if actor.attack_cooldown > 0.0 {
                actor.notify("攻击冷却中");
            } else {
                let attack = actor.set.attack(actor.mode);
                actor.attack_cooldown = attack.cooldown;
                let target =
                    selected.filter(|target| target.position.distance(origin) <= attack.range);
                let forward = Vec3::new(direction.x, 0.0, direction.y);
                let missed_end = point(origin) + forward * attack.range;
                let hit = world.hit(
                    point(origin),
                    target.map_or(missed_end, |target| point(target.position)),
                    &[entity],
                );
                let end = target.map_or(hit.map_or(missed_end, |hit| hit.position), |target| {
                    point(target.position)
                });
                if let Some(hit) = hit
                    && let Ok((_, mut forces)) = props.get_mut(hit.entity)
                {
                    let impulse = (end - point(origin)).normalize_or(forward)
                        * if actor.mode == WeaponMode::Melee {
                            35.0
                        } else {
                            18.0
                        };
                    forces.apply_linear_impulse_at_point(impulse, hit.position);
                }
                let effect = if actor.mode == WeaponMode::Melee {
                    CombatEffect::Melee
                } else {
                    CombatEffect::Shot
                };
                feedback.push(entity, effect, point(origin), end, attack.range);
                if let Some(target) = target {
                    if attack.splash > 0.0 {
                        for nearby in &targets {
                            if nearby.position.distance(target.position) <= attack.splash
                                && world.visible(
                                    point(target.position),
                                    point(nearby.position),
                                    target.entity,
                                    nearby.entity,
                                )
                            {
                                hurt_enemy(
                                    nearby.entity,
                                    attack.damage,
                                    &mut enemies,
                                    &mut feedback,
                                );
                            }
                        }
                        feedback.push(entity, CombatEffect::Cannon, end, end, attack.splash);
                        push_props(&world, &mut props, entity, end, attack.splash, 40.0);
                    } else {
                        hurt_enemy(target.entity, attack.damage, &mut enemies, &mut feedback);
                    }
                    actor.notify("命中");
                } else {
                    actor.notify("未命中：调整方向或接近目标");
                }
            }
        }
    }

    advance_enemies(dt, &world, &mut players, &mut enemies, &mut feedback);
}

fn advance_enemies(
    dt: f32,
    world: &CombatWorld,
    players: &mut Players,
    enemies: &mut Enemies,
    feedback: &mut CombatFeedback,
) {
    for (entity, body, motion, mut facing, mut movement, mut actions, health, mut enemy, _) in
        enemies
    {
        actions.clear();
        enemy.cooldown = (enemy.cooldown - dt).max(0.0);
        if enemy.is_passive() || !health.is_alive() || !motion.is_grounded() {
            movement.stop();
            enemy.windup = 0.0;
            enemy.attack_target = None;
            continue;
        }

        if enemy.windup > 0.0 {
            movement.stop();
            enemy.windup = (enemy.windup - dt).max(0.0);
            if enemy.windup == 0.0 {
                enemy.cooldown = 1.25;
                feedback.push(
                    entity,
                    CombatEffect::Melee,
                    point(body.world_position()),
                    point(
                        body.world_position()
                            + Vec3::new(facing.direction().x, 0.0, facing.direction().y) * 2.0,
                    ),
                    2.0,
                );
                if let Some(target) = enemy.attack_target.take()
                    && let Ok((
                        _,
                        target_body,
                        target_motion,
                        target_facing,
                        _,
                        _,
                        mut target_health,
                        _,
                        _,
                        _,
                    )) = players.get_mut(target)
                {
                    let offset = body.world_position() - target_body.world_position();
                    if target_health.is_alive()
                        && offset.length() <= 2.0
                        && world.visible(
                            point(body.world_position()),
                            point(target_body.world_position()),
                            entity,
                            target,
                        )
                    {
                        let position = point(target_body.world_position());
                        if target_motion.is_invulnerable() {
                            feedback.push(target, CombatEffect::Dodge, position, position, 0.65);
                        } else {
                            let blocked = target_motion.is_blocking()
                                && target_facing
                                    .direction()
                                    .dot(offset.xz().normalize_or_zero())
                                    > 0.25;
                            target_health.damage(if blocked { 2.0 } else { 12.0 });
                            feedback.push(
                                target,
                                if blocked {
                                    CombatEffect::Block
                                } else {
                                    CombatEffect::Hit
                                },
                                position,
                                position,
                                0.65,
                            );
                            if !target_health.is_alive() {
                                feedback.push(
                                    target,
                                    CombatEffect::Defeat,
                                    position,
                                    position,
                                    0.9,
                                );
                            }
                        }
                    }
                }
            }
            continue;
        }

        let target = players
            .iter()
            .filter(|(_, _, _, _, _, _, health, _, _, _)| health.is_alive())
            .map(|(entity, body, _, _, _, _, _, _, _, _)| (entity, body.world_position()))
            .filter(|(_, position)| position.distance(body.world_position()) < 12.0)
            .min_by(|a, b| {
                a.1.distance_squared(body.world_position())
                    .total_cmp(&b.1.distance_squared(body.world_position()))
            });
        let Some((target, position)) = target else {
            movement.stop();
            continue;
        };
        let offset = position - body.world_position();
        facing.set(offset.xz());
        if offset.length() > 1.6 {
            movement.set(offset.xz().normalize_or_zero());
        } else {
            movement.stop();
            if enemy.cooldown == 0.0
                && world.visible(
                    point(body.world_position()),
                    point(position),
                    entity,
                    target,
                )
            {
                enemy.windup = 0.5;
                enemy.attack_target = Some(target);
                let position = point(body.world_position());
                feedback.push(entity, CombatEffect::EnemyWindup, position, position, 2.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::simulation::{
        CollisionWorld, SimulationActive, SimulationPlugin, SolidBox, spawn_box, teleport_actor,
    };

    fn arena() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(30.0, vec![]));
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        app.finish();
        app
    }

    fn player(app: &mut App, set: WeaponSet) -> Entity {
        app.world_mut()
            .spawn((
                Body::new(Vec2::ZERO, 6.0, 0.35),
                MoveIntent::default(),
                MotionState::default(),
                ActionIntent::default(),
                Facing::default(),
                Health::new(100.0),
                Combatant::new(set),
                CombatIntent::default(),
                SpawnPoint(Vec2::ZERO),
            ))
            .id()
    }

    fn enemy(app: &mut App, position: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                // Keep target placement stable while exercising the real AI and schedule.
                Body::new(position, 0.0001, 0.35),
                MoveIntent::default(),
                MotionState::default(),
                ActionIntent::default(),
                Facing::default(),
                Health::new(100.0),
                Enemy::default(),
                SpawnPoint(position),
            ))
            .id()
    }

    fn ticks(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn health(app: &App, entity: Entity) -> f32 {
        app.world().get::<Health>(entity).unwrap().current()
    }

    fn request(app: &mut App, entity: Entity, action: impl FnOnce(&mut CombatIntent)) {
        action(&mut app.world_mut().get_mut::<CombatIntent>(entity).unwrap());
    }

    #[test]
    fn hammer_knockup_rises_three_meters_above_the_launch_surface() {
        for elevation in [0.0, 1.5] {
            let mut app = arena();
            if elevation > 0.0 {
                spawn_box(
                    app.world_mut(),
                    Vec3::new(0.0, elevation * 0.5, -2.0),
                    Vec3::new(8.0, elevation, 8.0),
                    false,
                );
            }
            let player = player(&mut app, WeaponSet::HammerSniper);
            let target = enemy(&mut app, Vec2::new(0.0, -3.0));
            app.world_mut()
                .entity_mut(target)
                .insert(Enemy::target_dummy());
            teleport_actor(app.world_mut(), player, Vec3::new(0.0, elevation, 0.0));
            teleport_actor(app.world_mut(), target, Vec3::new(0.0, elevation, -3.0));
            ticks(&mut app, 12);
            let origin = app.world().get::<Body>(target).unwrap().world_position().y;
            assert!(
                app.world()
                    .get::<MotionState>(target)
                    .unwrap()
                    .is_grounded()
            );
            request(&mut app, player, CombatIntent::request_skill);
            let mut peak = origin;
            for _ in 0..100 {
                ticks(&mut app, 1);
                peak = peak.max(app.world().get::<Body>(target).unwrap().world_position().y);
            }
            assert!(
                (peak - origin - 3.0).abs() < 0.04,
                "origin={origin}, peak={peak}"
            );
            assert!(
                app.world()
                    .get::<MotionState>(target)
                    .unwrap()
                    .is_grounded()
            );
            assert!(
                (app.world().get::<Body>(target).unwrap().world_position().y - origin).abs() < 0.03
            );
        }
    }

    #[test]
    fn target_dummy_remains_passive_after_defeat_and_respawn() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -1.3));
        app.world_mut()
            .entity_mut(target)
            .insert(Enemy::target_dummy());
        app.world_mut()
            .get_mut::<Health>(target)
            .unwrap()
            .damage(100.0);
        ticks(&mut app, 500);
        assert!(app.world().get::<Health>(target).unwrap().is_alive());
        assert!(app.world().get::<Enemy>(target).unwrap().is_passive());
        assert_eq!(health(&app, player), 100.0);
        assert!(
            !app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| {
                    event.actor == target
                        && matches!(event.kind, CombatEffect::EnemyWindup | CombatEffect::Melee)
                })
        );
    }

    #[test]
    fn hammer_launches_a_group_and_tab_snipes_the_original_target_once() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let selected = enemy(&mut app, Vec2::new(0.0, -3.0));
        let nearby = enemy(&mut app, Vec2::new(2.0, -3.0));
        let outside = enemy(&mut app, Vec2::new(5.5, -3.0));

        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 1);
        assert_eq!(health(&app, selected), 80.0);
        assert_eq!(health(&app, nearby), 80.0);
        assert_eq!(health(&app, outside), 100.0);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .combo_remaining()
                > 0.0
        );
        ticks(&mut app, 5);
        assert!(app.world().get::<MotionState>(selected).unwrap().height() > 0.0);

        request(&mut app, player, |intent| {
            intent.request_cycle_target();
            intent.request_combo();
        });
        ticks(&mut app, 1);
        assert_eq!(health(&app, selected), 80.0);
        assert_eq!(health(&app, nearby), 80.0);
        let actor = app.world().get::<Combatant>(player).unwrap();
        assert_eq!(actor.weapon_mode(), WeaponMode::Ranged);
        assert_eq!(actor.combo_remaining(), 0.0);
        assert_eq!(actor.sniper_sequence().unwrap().target(), selected);
        assert!(!actor.sniper_sequence().unwrap().has_fired());
        assert!(
            !app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| event.kind == CombatEffect::Shot)
        );
        request(&mut app, player, |intent| {
            intent.request_cycle_target();
            intent.request_combo();
            intent.request_attack();
        });
        ticks(&mut app, 16);
        assert_eq!(health(&app, selected), 80.0);
        ticks(&mut app, 4);
        assert_eq!(health(&app, selected), 0.0);
        assert_eq!(health(&app, nearby), 80.0);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .sniper_sequence()
                .unwrap()
                .has_fired()
        );
        ticks(&mut app, 35);
        assert!(!app.world().get::<Combatant>(player).unwrap().is_sniping());
        let shots = app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .filter(|event| event.kind == CombatEffect::Shot)
            .count();
        assert_eq!(shots, 1);
    }

    #[test]
    fn changing_set_or_expiring_the_window_cannot_release_an_old_link() {
        for switch in [true, false] {
            let mut app = arena();
            let player = player(&mut app, WeaponSet::HammerSniper);
            let target = enemy(&mut app, Vec2::new(0.0, -3.0));
            request(&mut app, player, CombatIntent::request_skill);
            ticks(&mut app, 1);
            if switch {
                request(&mut app, player, |intent| {
                    intent.request_set(WeaponSet::AxeCannon)
                });
                ticks(&mut app, 1);
            } else {
                ticks(&mut app, 60);
            }
            request(&mut app, player, CombatIntent::request_combo);
            ticks(&mut app, 1);
            assert_eq!(health(&app, target), 80.0);
            assert_eq!(
                app.world()
                    .get::<Combatant>(player)
                    .unwrap()
                    .combo_remaining(),
                0.0
            );
        }
    }

    fn sniper_arena(elevation: f32) -> (App, Entity, Entity) {
        let mut app = arena();
        if elevation > 0.0 {
            spawn_box(
                app.world_mut(),
                Vec3::new(0.0, elevation * 0.5, -2.0),
                Vec3::new(8.0, elevation, 8.0),
                false,
            );
        }
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -3.0));
        app.world_mut()
            .entity_mut(target)
            .insert((Enemy::target_dummy(), Health::new(500.0)));
        teleport_actor(app.world_mut(), player, Vec3::new(0.0, elevation, 0.0));
        teleport_actor(app.world_mut(), target, Vec3::new(0.0, elevation, -3.0));
        ticks(&mut app, 12);
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 4);
        request(&mut app, player, CombatIntent::request_combo);
        ticks(&mut app, 1);
        assert!(app.world().get::<Combatant>(player).unwrap().is_sniping());
        assert_eq!(health(&app, target), 480.0);
        (app, player, target)
    }

    fn shot_count(app: &App, player: Entity) -> usize {
        app.world()
            .resource::<CombatFeedback>()
            .events()
            .filter(|event| event.actor == player && event.kind == CombatEffect::Shot)
            .count()
    }

    #[test]
    fn sniper_sequence_delays_its_only_shot_and_preserves_relative_elevation() {
        for elevation in [0.0, 1.5] {
            let (mut app, player, target) = sniper_arena(elevation);
            let sequence = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .sniper_sequence()
                .unwrap();
            assert_eq!(sequence.elapsed(), 0.0);
            assert_eq!(sequence.target(), target);
            assert!(sequence.aim_position().y > elevation + 0.9);
            assert_eq!(shot_count(&app, player), 0);
            ticks(&mut app, 16);
            assert_eq!(shot_count(&app, player), 0);
            assert_eq!(health(&app, target), 480.0);
            let current = app.world().get::<Body>(target).unwrap().world_position() + Vec3::Y * 0.9;
            let sequence = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .sniper_sequence()
                .unwrap();
            assert!(sequence.aim_position().distance(current) < 0.001);
            ticks(&mut app, 4);
            assert_eq!(shot_count(&app, player), 1);
            assert_eq!(health(&app, target), 395.0);
            let sequence = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .sniper_sequence()
                .unwrap();
            assert!(sequence.has_fired());
            assert!(sequence.elapsed() >= SniperSequence::AIM_DURATION);
            request(&mut app, player, |intent| {
                intent.request_combo();
                intent.request_attack();
                intent.request_skill();
            });
            ticks(&mut app, 30);
            assert_eq!(shot_count(&app, player), 1);
            assert_eq!(health(&app, target), 395.0);
            assert!(!app.world().get::<Combatant>(player).unwrap().is_sniping());
        }
    }

    #[test]
    fn pausing_before_sniper_discharge_freezes_the_committed_sequence() {
        let (mut app, player, target) = sniper_arena(0.0);
        ticks(&mut app, 8);
        let before = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .sniper_sequence()
            .unwrap();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        ticks(&mut app, 180);
        let frozen = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .sniper_sequence()
            .unwrap();
        assert_eq!(frozen.elapsed(), before.elapsed());
        assert_eq!(frozen.aim_position(), before.aim_position());
        assert_eq!(health(&app, target), 480.0);
        assert_eq!(shot_count(&app, player), 0);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        ticks(&mut app, 12);
        assert_eq!(health(&app, target), 395.0);
        assert_eq!(shot_count(&app, player), 1);
    }

    #[test]
    fn pending_sniper_shots_cancel_without_replay_when_the_commitment_becomes_invalid() {
        for cause in [
            "weapon",
            "set",
            "same-set",
            "target-despawn",
            "target-death",
            "player-death",
            "land",
            "block",
            "dodge",
            "mode-change",
            "wall",
        ] {
            let (mut app, player, target) = sniper_arena(0.0);
            ticks(&mut app, 4);
            match cause {
                "weapon" => request(&mut app, player, CombatIntent::request_switch_weapon),
                "set" => request(&mut app, player, |intent| {
                    intent.request_set(WeaponSet::AxeCannon)
                }),
                "same-set" => request(&mut app, player, |intent| {
                    intent.request_set(WeaponSet::HammerSniper)
                }),
                "target-despawn" => {
                    app.world_mut().despawn(target);
                }
                "target-death" => {
                    app.world_mut()
                        .get_mut::<Health>(target)
                        .unwrap()
                        .damage(500.0);
                }
                "player-death" => {
                    app.world_mut()
                        .get_mut::<Health>(player)
                        .unwrap()
                        .damage(100.0);
                }
                "land" => teleport_actor(app.world_mut(), target, Vec3::new(0.0, 0.0, -3.0)),
                "block" => app
                    .world_mut()
                    .get_mut::<ActionIntent>(player)
                    .unwrap()
                    .set_held(false, true),
                "dodge" => app
                    .world_mut()
                    .get_mut::<ActionIntent>(player)
                    .unwrap()
                    .request_dodge(),
                "mode-change" => crate::gameplay::set_enemy_mode(
                    app.world_mut(),
                    crate::gameplay::EnemyMode::Combat,
                ),
                "wall" => {
                    spawn_box(
                        app.world_mut(),
                        Vec3::new(0.0, 3.0, -1.5),
                        Vec3::new(4.0, 8.0, 0.5),
                        false,
                    );
                }
                _ => unreachable!(),
            }
            ticks(&mut app, 25);
            assert!(
                !app.world().get::<Combatant>(player).unwrap().is_sniping(),
                "cause: {cause}"
            );
            assert_eq!(shot_count(&app, player), 0, "cause: {cause}");
            app.world_mut()
                .get_mut::<ActionIntent>(player)
                .unwrap()
                .clear();
            ticks(&mut app, 50);
            assert_eq!(shot_count(&app, player), 0, "replayed after cause: {cause}");
        }
    }

    #[test]
    fn cannon_tab_spin_hits_enemies_along_the_arc_only_once_then_lands() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::AxeCannon);
        // Keep the flight corridor clear of full-height capsules while both
        // enemies remain inside the actual three-dimensional spin volume.
        let early = enemy(&mut app, Vec2::new(1.0, -2.0));
        // The intended landing needs capsule clearance, while the nearby enemy
        // remains inside the spin volume at the end of the leap.
        let late = enemy(&mut app, Vec2::new(1.0, -6.0));
        let outside = enemy(&mut app, Vec2::new(5.0, -6.0));
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.set_aim(Vec2::new(0.0, -6.0));
            intent.request_skill();
        });
        ticks(&mut app, 1);
        assert!(app.world().get::<MotionState>(player).unwrap().is_leaping());
        ticks(&mut app, 5);
        request(&mut app, player, CombatIntent::request_combo);
        ticks(&mut app, 1);
        assert!(app.world().get::<Combatant>(player).unwrap().is_spinning());
        ticks(&mut app, 60);

        let landed_position = app.world().get::<Body>(player).unwrap().position();
        // The capsule settles through finite acceleration after touchdown;
        // require a stopping error smaller than its radius.
        assert!((landed_position.y + 6.0).abs() < 0.2, "{landed_position:?}");
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_grounded()
        );
        assert_eq!(health(&app, early), 58.0);
        assert_eq!(health(&app, late), 58.0);
        assert_eq!(health(&app, outside), 100.0);
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().weapon_mode(),
            WeaponMode::Melee
        );
    }

    #[test]
    fn cannon_leap_stops_before_a_wall_and_does_not_restart_in_air() {
        let mut app = arena();
        app.insert_resource(CollisionWorld::new(
            30.0,
            vec![SolidBox::new(Vec2::new(0.0, -3.0), Vec2::new(2.0, 0.3))],
        ));
        let player = player(&mut app, WeaponSet::AxeCannon);
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.set_aim(Vec2::new(0.0, -6.0));
            intent.request_skill();
        });
        ticks(&mut app, 10);
        let height = app.world().get::<MotionState>(player).unwrap().height();
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 60);
        assert!(height > 0.0);
        let body = app.world().get::<Body>(player).unwrap();
        let feet = body.world_position();
        // Check the real capsule against the wall's near face. Contacts permit
        // up to the 5mm CCD skin, unlike the old strict planar overlap oracle.
        assert!(
            feet.z - body.radius() >= -2.7 - 0.005,
            "capsule crossed the wall face: {feet:?}"
        );
        assert!(feet.z < -2.3, "leap did not reach the wall: {feet:?}");
        assert!(feet.y.abs() < 0.005);
        assert!(body.velocity().xz().length() < 0.001);
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_grounded()
        );
    }

    #[test]
    fn cannon_commits_its_landing_marker_despite_new_aim_and_facing_during_flight() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::AxeCannon);
        let landing = Vec2::new(0.0, -6.0);
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.set_aim(landing);
            intent.request_skill();
        });
        ticks(&mut app, 1);
        assert!(app.world().get::<MotionState>(player).unwrap().is_leaping());
        let committed = app.world().get::<Combatant>(player).unwrap().aim();
        assert_eq!(committed.xz(), landing);
        for index in 0..60 {
            if !app.world().get::<MotionState>(player).unwrap().is_leaping() {
                break;
            }
            let direction = if index % 2 == 0 { Vec2::X } else { Vec2::Y };
            let position = app.world().get::<Body>(player).unwrap().position();
            app.world_mut()
                .get_mut::<Facing>(player)
                .unwrap()
                .set(direction);
            request(&mut app, player, |intent| {
                intent.set_aim(position + direction * 6.0);
            });
            ticks(&mut app, 1);
            if app.world().get::<MotionState>(player).unwrap().is_leaping() {
                assert_eq!(
                    app.world().get::<Combatant>(player).unwrap().aim(),
                    committed
                );
            }
        }
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_grounded()
        );
        let body = app.world().get::<Body>(player).unwrap();
        // Touchdown is resolved by the CCD/contact solver, within one fixed
        // step of horizontal travel rather than an exact destination snap.
        assert!(body.position().distance(landing) <= 6.0 / LEAP_DURATION / 60.0);
        assert!(body.world_position().y.abs() <= 0.0051);
        ticks(&mut app, 30);
        let body = app.world().get::<Body>(player).unwrap();
        // The finite ground motor then brakes without teleporting to the mark.
        assert!(
            body.position().distance(landing) < 0.2,
            "landing failed to stop near the committed destination: {:?}",
            body.world_position()
        );
        assert!(body.velocity().xz().length() < 0.001);
    }

    #[test]
    fn attempting_a_link_while_blocking_consumes_it_without_delaying_the_shot() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -3.0));
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 1);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .set_held(false, true);
        request(&mut app, player, CombatIntent::request_combo);
        ticks(&mut app, 1);
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .combo_remaining(),
            0.0
        );
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .clear();
        request(&mut app, player, CombatIntent::request_combo);
        ticks(&mut app, 1);
        assert_eq!(health(&app, target), 80.0);
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().weapon_mode(),
            WeaponMode::Melee
        );
    }

    #[test]
    fn front_block_reduces_damage_but_turning_away_does_not() {
        let incoming = |block, facing: Vec2| {
            let mut app = arena();
            let player = player(&mut app, WeaponSet::HammerSniper);
            enemy(&mut app, Vec2::new(0.0, -1.2));
            app.world_mut()
                .get_mut::<Facing>(player)
                .unwrap()
                .set(facing);
            app.world_mut()
                .get_mut::<ActionIntent>(player)
                .unwrap()
                .set_held(false, block);
            ticks(&mut app, 85);
            health(&app, player)
        };
        assert_eq!(incoming(false, Vec2::NEG_Y), 88.0);
        assert_eq!(incoming(true, Vec2::NEG_Y), 98.0);
        assert_eq!(incoming(true, Vec2::Y), 88.0);
    }

    #[test]
    fn dodge_invulnerability_only_rejects_attacks_during_its_effective_window() {
        for (windup, frames, protected) in [(1.0 / 60.0, 1, false), (0.06, 4, true)] {
            let mut app = arena();
            let player = player(&mut app, WeaponSet::HammerSniper);
            let enemy = enemy(&mut app, Vec2::new(0.0, -1.2));
            {
                let mut brain = app.world_mut().get_mut::<Enemy>(enemy).unwrap();
                brain.windup = windup;
                brain.attack_target = Some(player);
            }
            app.world_mut()
                .get_mut::<ActionIntent>(player)
                .unwrap()
                .request_dodge();
            ticks(&mut app, frames);
            assert_eq!(
                app.world()
                    .get::<MotionState>(player)
                    .unwrap()
                    .is_invulnerable(),
                protected
            );
            assert!(
                app.world()
                    .get::<Body>(player)
                    .unwrap()
                    .position()
                    .distance(Vec2::new(0.0, -1.2))
                    < 2.0
            );
            assert_eq!(health(&app, player), if protected { 100.0 } else { 88.0 });
            assert_eq!(
                app.world()
                    .resource::<CombatFeedback>()
                    .events()
                    .any(|event| event.kind == CombatEffect::Dodge),
                protected
            );
        }
    }

    #[test]
    fn death_removes_a_target_and_respawn_restores_health_at_its_spawn() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -3.0));
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 1);
        request(&mut app, player, CombatIntent::request_combo);
        ticks(&mut app, 21);
        assert!(!app.world().get::<Health>(target).unwrap().is_alive());
        assert!(app.world().resource::<HitStop>().is_active());
        assert_ne!(
            app.world().get::<Combatant>(player).unwrap().target(),
            Some(target)
        );
        ticks(&mut app, 1);
        assert_ne!(
            app.world().get::<Combatant>(player).unwrap().target(),
            Some(target)
        );
        ticks(&mut app, 181);
        assert_eq!(health(&app, target), 100.0);
        assert!(
            app.world()
                .get::<Body>(target)
                .unwrap()
                .position()
                .distance(Vec2::new(0.0, -3.0))
                < 0.01
        );
    }

    #[test]
    fn paused_simulation_preserves_commands_and_stops_enemy_damage() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -1.2));
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 180);
        assert_eq!(health(&app, player), 100.0);
        assert_eq!(health(&app, target), 100.0);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        ticks(&mut app, 1);
        assert_eq!(health(&app, target), 80.0);
    }

    #[test]
    fn commands_queued_while_dead_are_discarded_on_the_respawn_tick() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -3.0));
        {
            let mut health = app.world_mut().get_mut::<Health>(player).unwrap();
            health.damage(100.0);
            health.respawn_remaining = 1.0 / 60.0;
        }
        request(&mut app, player, |intent| {
            intent.request_skill();
            intent.request_set(WeaponSet::AxeCannon);
        });
        ticks(&mut app, 1);
        assert_eq!(health(&app, player), 100.0);
        assert_eq!(health(&app, target), 100.0);
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().weapon_set(),
            WeaponSet::HammerSniper
        );
    }

    #[test]
    fn crouching_keeps_existing_attacks_and_front_block_available() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -1.2));
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_crouch_toggle();
        request(&mut app, player, CombatIntent::request_attack);
        ticks(&mut app, 1);
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_crouched()
        );
        assert!(health(&app, target) < 100.0);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .set_held(false, true);
        ticks(&mut app, 85);
        let motion = app.world().get::<MotionState>(player).unwrap();
        assert!(motion.is_crouched() && motion.is_blocking());
        assert!(health(&app, player) > 95.0);
    }

    #[test]
    fn sliding_rejects_weapon_actions_until_the_actor_stands() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -1.2));
        app.world_mut()
            .get_mut::<MoveIntent>(player)
            .unwrap()
            .set(Vec2::X);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .set_held(true, false);
        ticks(&mut app, 30);
        let origin = app.world().get::<Body>(player).unwrap().position();
        app.world_mut()
            .get_mut::<Body>(target)
            .unwrap()
            .teleport(origin + Vec2::NEG_Y * 1.2);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_crouch_toggle();
        request(&mut app, player, CombatIntent::request_attack);
        ticks(&mut app, 1);
        assert!(app.world().get::<MotionState>(player).unwrap().is_sliding());
        assert_eq!(health(&app, target), 100.0);
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .attack_cooldown(),
            0.0
        );
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 1);
        assert_eq!(health(&app, target), 100.0);
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .combo_remaining(),
            0.0
        );
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_crouch_toggle();
        request(&mut app, player, CombatIntent::request_attack);
        ticks(&mut app, 1);
        assert!(!app.world().get::<MotionState>(player).unwrap().is_sliding());
        assert!(health(&app, target) < 100.0);
    }

    #[test]
    fn cannon_leap_exits_crouch_through_the_existing_skill_path() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::AxeCannon);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_crouch_toggle();
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.set_aim(Vec2::NEG_Y * 6.0);
            intent.request_skill();
        });
        ticks(&mut app, 1);
        let motion = app.world().get::<MotionState>(player).unwrap();
        assert!(motion.is_leaping());
        assert!(!motion.is_crouched());
    }

    #[test]
    fn feedback_identifies_each_attacker_and_hit_recipient_independently() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let enemy = enemy(&mut app, Vec2::new(0.0, -1.2));
        request(&mut app, player, CombatIntent::request_attack);
        ticks(&mut app, 90);
        let feedback = app.world().resource::<CombatFeedback>();
        for actor in [player, enemy] {
            assert!(
                feedback
                    .events()
                    .any(|event| event.actor == actor && event.kind == CombatEffect::Melee)
            );
            assert!(
                feedback
                    .events()
                    .any(|event| event.actor == actor && event.kind == CombatEffect::Hit)
            );
        }
        assert!(
            feedback
                .events()
                .any(|event| event.actor == enemy && event.kind == CombatEffect::EnemyWindup)
        );
    }

    #[test]
    fn cannon_uses_box_top_surface_and_lands_above_world_zero() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::AxeCannon);
        let support = spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 0.75, -6.0),
            Vec3::new(2.0, 1.5, 2.0),
            false,
        );
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.set_aim(Vec2::new(0.0, -6.0));
        });
        ticks(&mut app, 2);
        let surface = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .aim_surface()
            .unwrap();
        assert_eq!(surface.entity, support);
        assert!((surface.position.y - 1.5).abs() < 0.001);
        assert!(surface.normal.dot(Vec3::Y) > 0.999);
        request(&mut app, player, CombatIntent::request_skill);
        ticks(&mut app, 1);
        assert!(app.world().get::<MotionState>(player).unwrap().is_leaping());
        ticks(&mut app, 100);
        let body = app.world().get::<Body>(player).unwrap();
        assert!(
            (body.world_position().y - 1.5).abs() < 0.12,
            "{:?}",
            body.world_position()
        );
        assert!((body.position().y + 6.0).abs() < 0.25);
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_grounded()
        );
    }

    #[test]
    fn missing_landing_surface_is_explicit_and_does_not_launch() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::AxeCannon);
        teleport_actor(app.world_mut(), player, Vec3::new(28.0, 0.0, 0.0));
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.set_aim(Vec2::new(36.0, 0.0));
            intent.request_skill();
        });
        ticks(&mut app, 1);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .aim_surface()
                .is_none()
        );
        assert!(!app.world().get::<MotionState>(player).unwrap().is_leaping());
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().cooldown(),
            0.0
        );
    }

    #[test]
    fn moving_box_changes_visibility_and_can_receive_a_weapon_impulse() {
        use avian3d::prelude::{LinearVelocity, Position};
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -6.0));
        let blocker = spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 0.6, -2.0),
            Vec3::new(1.2, 1.2, 1.0),
            true,
        );
        request(&mut app, player, |intent| {
            intent.request_switch_weapon();
            intent.request_attack();
        });
        ticks(&mut app, 1);
        assert_eq!(health(&app, target), 100.0);
        assert!(app.world().get::<LinearVelocity>(blocker).unwrap().length() > 0.01);
        app.world_mut().get_mut::<Position>(blocker).unwrap().0 = Vec3::new(3.0, 0.6, -2.0);
        app.world_mut()
            .get_mut::<LinearVelocity>(blocker)
            .unwrap()
            .0 = Vec3::ZERO;
        ticks(&mut app, 60);
        request(&mut app, player, CombatIntent::request_attack);
        ticks(&mut app, 1);
        assert!(health(&app, target) < 100.0);
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().target(),
            Some(target)
        );
    }

    #[test]
    fn enemies_can_attack_at_equal_elevation_above_the_old_absolute_height_limit() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let enemy = enemy(&mut app, Vec2::new(0.0, -1.2));
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 1.5, -1.0),
            Vec3::new(6.0, 3.0, 6.0),
            false,
        );
        teleport_actor(app.world_mut(), player, Vec3::new(0.0, 3.0, 0.0));
        teleport_actor(app.world_mut(), enemy, Vec3::new(0.0, 3.0, -1.2));
        ticks(&mut app, 100);
        assert!(app.world().get::<Body>(player).unwrap().world_position().y > 2.9);
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_grounded()
        );
        assert!(health(&app, player) < 100.0);
    }

    #[test]
    fn melee_cannot_hit_a_target_only_close_in_horizontal_projection() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        let target = enemy(&mut app, Vec2::new(0.0, -1.0));
        teleport_actor(app.world_mut(), target, Vec3::new(0.0, 6.0, -1.0));
        request(&mut app, player, CombatIntent::request_attack);
        ticks(&mut app, 1);
        assert_eq!(
            app.world().get::<Combatant>(player).unwrap().target(),
            Some(target)
        );
        assert_eq!(health(&app, target), 100.0);
    }

    #[test]
    fn falling_out_of_the_courtyard_uses_defeat_and_resets_the_physical_respawn() {
        use avian3d::prelude::{LinearVelocity, Position};
        let mut app = arena();
        let player = player(&mut app, WeaponSet::HammerSniper);
        ticks(&mut app, 1);
        teleport_actor(app.world_mut(), player, Vec3::new(32.0, -13.0, 0.0));
        ticks(&mut app, 1);
        assert_eq!(health(&app, player), 0.0);
        request(&mut app, player, |intent| {
            intent.request_attack();
            intent.request_set(WeaponSet::AxeCannon);
        });
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_crouch_toggle();
        ticks(&mut app, 190);
        assert_eq!(health(&app, player), 100.0);
        assert!(
            app.world()
                .get::<Body>(player)
                .unwrap()
                .world_position()
                .length()
                < 0.05
        );
        let physical = app.world().get::<Position>(player).unwrap().0;
        assert!((physical.y - STANDING_HEIGHT * 0.5).abs() < 0.05);
        assert!(app.world().get::<LinearVelocity>(player).unwrap().length() < 0.05);
        assert!(
            !app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_crouched()
        );
        let combat = app.world().get::<Combatant>(player).unwrap();
        assert_eq!(combat.weapon_set(), WeaponSet::HammerSniper);
        assert_eq!(combat.attack_cooldown(), 0.0);
        let events = app.world().resource::<CombatFeedback>();
        assert_eq!(
            events
                .events()
                .filter(|event| event.actor == player && event.kind == CombatEffect::Defeat)
                .count(),
            1
        );
        assert_eq!(
            events
                .events()
                .filter(|event| event.actor == player && event.kind == CombatEffect::Respawn)
                .count(),
            1
        );
    }

    #[test]
    fn feedback_is_ordered_and_bounded_across_many_fixed_ticks() {
        let mut feedback = CombatFeedback::default();
        for _ in 0..200 {
            feedback.push(
                Entity::PLACEHOLDER,
                CombatEffect::Hit,
                Vec3::ZERO,
                Vec3::ZERO,
                1.0,
            );
        }
        let sequences: Vec<_> = feedback.events().map(|event| event.sequence).collect();
        assert_eq!(sequences.len(), HISTORY_LIMIT);
        assert_eq!(sequences[0], 73);
        assert_eq!(sequences.last(), Some(&200));
    }
}
