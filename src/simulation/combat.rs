//! Fixed-step combat. Device input and rendered transforms never enter this module.

use std::collections::VecDeque;

use bevy::prelude::*;

use super::{
    ActionIntent, Body, CollisionWorld, Facing, MotionState, MoveIntent, WeaponMode, WeaponSet,
};

const TARGET_RANGE: f32 = 22.0;
const HAMMER_RANGE: f32 = 4.5;
const HAMMER_RADIUS: f32 = 3.0;
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
    cooldown: f32,
    windup: f32,
    attack_target: Option<Entity>,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            cooldown: 0.8,
            windup: 0.0,
            attack_target: None,
        }
    }
}

impl Enemy {
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

#[derive(Component)]
pub struct Combatant {
    set: WeaponSet,
    mode: WeaponMode,
    target: Option<Entity>,
    aim: Vec2,
    attack_cooldown: f32,
    skill_cooldown: f32,
    combo: Option<ComboWindow>,
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
            aim: Vec2::NEG_Y * 6.0,
            attack_cooldown: 0.0,
            skill_cooldown: 0.0,
            combo: None,
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
    pub fn aim(&self) -> Vec2 {
        self.aim
    }
    pub fn is_spinning(&self) -> bool {
        self.spin_remaining > 0.0
    }
    pub fn status(&self) -> &'static str {
        self.status
    }

    fn notify(&mut self, status: &'static str) {
        self.status = status;
        self.status_remaining = 1.4;
    }

    fn cancel_link(&mut self) {
        self.combo = None;
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
    position: Vec2,
    height: f32,
}

fn point(position: Vec2, height: f32) -> Vec3 {
    Vec3::new(position.x, height + 0.9, position.y)
}

fn visible(world: &CollisionWorld, start: Vec2, end: Vec2) -> bool {
    world.slide(start, end - start, 0.02).distance(end) < 0.05
}

fn select_target(
    targets: &[Target],
    current: Option<Entity>,
    origin: Vec2,
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
            let offset = target.position - origin;
            let alignment = offset.normalize_or_zero().dot(facing);
            (alignment > 0.35).then_some((target.entity, alignment - offset.length() * 0.01))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(entity, _)| entity)
}

fn hurt_enemy(target: Entity, amount: f32, enemies: &mut Enemies, feedback: &mut CombatFeedback) {
    let Ok((_, body, motion, _, _, _, mut health, _, _)) = enemies.get_mut(target) else {
        return;
    };
    if health.damage(amount) {
        let position = point(body.position(), motion.height());
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

pub(super) fn advance(
    time: Res<Time<Fixed>>,
    world: Res<CollisionWorld>,
    mut feedback: ResMut<CombatFeedback>,
    mut players: Players,
    mut enemies: Enemies,
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
        if health.tick(dt) {
            body.teleport(spawn.0);
            motion.reset();
            actions.clear();
            movement.stop();
            *enemy = Enemy::default();
            let position = point(spawn.0, 0.0);
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
        if health.tick(dt) {
            body.teleport(spawn.0);
            motion.reset();
            actions.clear();
            movement.stop();
            intent.clear();
            actor.cancel_link();
            actor.target = None;
            actor.attack_cooldown = 0.0;
            actor.skill_cooldown = 0.0;
            actor.notify("已恢复，继续练习");
            let position = point(spawn.0, 0.0);
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

        let origin = body.position();
        let direction = facing.direction();
        if !motion.is_leaping() {
            let requested_aim = intent.aim.unwrap_or(origin + direction * 6.0);
            // A leap commits its landing point; later look input only changes facing.
            actor.aim = world.slide(origin, (requested_aim - origin).clamp_length_max(8.0), 0.35);
        }
        let mut targets: Vec<_> = enemies
            .iter()
            .filter(|(_, body, _, _, _, _, health, _, _)| {
                health.is_alive()
                    && body.position().distance(origin) <= TARGET_RANGE
                    && visible(&world, origin, body.position())
            })
            .map(|(entity, body, motion, _, _, _, _, _, _)| Target {
                entity,
                position: body.position(),
                height: motion.height(),
            })
            .collect();
        targets.sort_by_key(|target| target.entity.to_bits());
        let commands = intent.commands;
        intent.commands = 0;
        if let Some(set) = intent.requested_set.take()
            && actor.set != set
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
            }
        }

        let can_act = !motion.is_dodging() && !motion.is_blocking() && !actor.is_spinning();
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
                                    if nearby.position.distance(target.position) <= HAMMER_RADIUS {
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
                                            state.launch_up(8.0);
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
                                let position = point(target.position, 0.0);
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
                        if actor.aim.distance(origin) < 0.5 {
                            actor.notify("落点被障碍阻挡");
                        } else if motion.start_leap(origin, actor.aim, LEAP_DURATION, 2.3) {
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
                                point(origin, 0.0),
                                point(actor.aim, 0.0),
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
                            if let Ok((_, target_body, target_motion, _, _, _, _, _, _)) =
                                enemies.get_mut(target)
                            {
                                let end = point(target_body.position(), target_motion.height());
                                if visible(&world, origin, target_body.position()) {
                                    actor.mode = WeaponMode::Ranged;
                                    actor.attack_cooldown = 0.9;
                                    actor.notify("妹妹 · 联动狙击");
                                    feedback.push(
                                        entity,
                                        CombatEffect::Shot,
                                        point(origin, motion.height()),
                                        end,
                                        0.12,
                                    );
                                    hurt_enemy(target, 85.0, &mut enemies, &mut feedback);
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
                                point(origin, motion.height()),
                                point(actor.aim, 0.0),
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
                let target = selected.filter(|target| {
                    target.position.distance(origin) <= attack.range
                        && (target.height - motion.height()).abs() < 2.5
                });
                let end = target.map_or(
                    point(
                        world.slide(origin, direction * attack.range, 0.02),
                        motion.height(),
                    ),
                    |target| point(target.position, target.height),
                );
                let effect = if actor.mode == WeaponMode::Melee {
                    CombatEffect::Melee
                } else {
                    CombatEffect::Shot
                };
                feedback.push(
                    entity,
                    effect,
                    point(origin, motion.height()),
                    end,
                    attack.range,
                );
                if let Some(target) = target {
                    if attack.splash > 0.0 {
                        for nearby in &targets {
                            if nearby.position.distance(target.position) <= attack.splash {
                                hurt_enemy(
                                    nearby.entity,
                                    attack.damage,
                                    &mut enemies,
                                    &mut feedback,
                                );
                            }
                        }
                        feedback.push(entity, CombatEffect::Cannon, end, end, attack.splash);
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
    world: &CollisionWorld,
    players: &mut Players,
    enemies: &mut Enemies,
    feedback: &mut CombatFeedback,
) {
    for (entity, body, motion, mut facing, mut movement, mut actions, health, mut enemy, _) in
        enemies
    {
        actions.clear();
        enemy.cooldown = (enemy.cooldown - dt).max(0.0);
        if !health.is_alive() || !motion.is_grounded() {
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
                    point(body.position(), motion.height()),
                    point(body.position() + facing.direction() * 2.0, motion.height()),
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
                    let offset = body.position() - target_body.position();
                    if target_health.is_alive()
                        && offset.length() <= 2.0
                        && target_motion.height() < 1.1
                        && visible(world, body.position(), target_body.position())
                    {
                        let position = point(target_body.position(), target_motion.height());
                        if target_motion.is_invulnerable() {
                            feedback.push(target, CombatEffect::Dodge, position, position, 0.65);
                        } else {
                            let blocked = target_motion.is_blocking()
                                && target_facing.direction().dot(offset.normalize_or_zero()) > 0.25;
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
            .map(|(entity, body, _, _, _, _, _, _, _, _)| (entity, body.position()))
            .filter(|(_, position)| position.distance(body.position()) < 12.0)
            .min_by(|a, b| {
                a.1.distance_squared(body.position())
                    .total_cmp(&b.1.distance_squared(body.position()))
            });
        let Some((target, position)) = target else {
            movement.stop();
            continue;
        };
        let offset = position - body.position();
        facing.set(offset);
        if offset.length() > 1.6 {
            movement.set(offset.normalize_or_zero());
        } else {
            movement.stop();
            if enemy.cooldown == 0.0 && visible(world, body.position(), position) {
                enemy.windup = 0.5;
                enemy.attack_target = Some(target);
                let position = point(body.position(), motion.height());
                feedback.push(entity, CombatEffect::EnemyWindup, position, position, 2.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::simulation::{SimulationActive, SimulationPlugin, SolidBox};

    fn arena() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(30.0, vec![]));
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
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
        assert_eq!(health(&app, selected), 0.0);
        assert_eq!(health(&app, nearby), 80.0);
        let actor = app.world().get::<Combatant>(player).unwrap();
        assert_eq!(actor.weapon_mode(), WeaponMode::Ranged);
        assert_eq!(actor.combo_remaining(), 0.0);
        request(&mut app, player, CombatIntent::request_combo);
        ticks(&mut app, 1);
        assert_eq!(health(&app, nearby), 80.0);
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

    #[test]
    fn cannon_tab_spin_hits_enemies_along_the_arc_only_once_then_lands() {
        let mut app = arena();
        let player = player(&mut app, WeaponSet::AxeCannon);
        let early = enemy(&mut app, Vec2::new(0.0, -2.0));
        let late = enemy(&mut app, Vec2::new(0.0, -6.0));
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

        assert!((app.world().get::<Body>(player).unwrap().position().y + 6.0).abs() < 0.01);
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
        let position = app.world().get::<Body>(player).unwrap().position();
        assert!(position.y >= -2.351);
        assert!(
            app.world()
                .resource::<CollisionWorld>()
                .is_clear(position, 0.35)
        );
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
                assert_eq!(app.world().get::<Combatant>(player).unwrap().aim(), landing);
            }
        }
        assert!(
            app.world()
                .get::<MotionState>(player)
                .unwrap()
                .is_grounded()
        );
        assert!(
            app.world()
                .get::<Body>(player)
                .unwrap()
                .position()
                .distance(landing)
                < 0.001
        );
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
        ticks(&mut app, 1);
        assert!(!app.world().get::<Health>(target).unwrap().is_alive());
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
