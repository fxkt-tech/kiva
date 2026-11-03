pub mod core;

use super::{camera::OrbitCamera, input::Controls, sound::Sounds};
use bevy::prelude::*;
use core::{
    Attack, AttackKind, DODGE_COOLDOWN, DODGE_DURATION, JumpState, damped_motion, dodge_protected,
    dodge_step, guarded_damage, inside_attack, jump_step,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Side {
    Player,
    Dummy,
}
impl Side {
    pub fn home(self) -> Vec3 {
        match self {
            Self::Player => Vec3::new(0.0, 0.0, 1.5),
            Self::Dummy => Vec3::new(0.0, 0.0, -0.6),
        }
    }
    pub fn yaw(self) -> f32 {
        if self == Self::Player {
            0.0
        } else {
            std::f32::consts::PI
        }
    }
}

#[derive(Debug)]
pub(super) enum Action {
    Idle,
    Guard,
    Attack(Attack),
    Stagger(f32),
    Down(f32),
    Dodge { elapsed: f32, direction: Vec3 },
}

#[derive(Component)]
pub(super) struct Fighter {
    pub side: Side,
    pub health: f32,
    pub facing: f32,
    pub action: Action,
    pub flash: f32,
    pub hit_stop: f32,
    pub velocity: Vec3,
    pub moving: bool,
    pub walk_phase: f32,
    pub vertical_speed: f32,
    pub jumps: JumpState,
    pub airborne: bool,
    pub dodge_cooldown: f32,
    pub protected: bool,
}
impl Fighter {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            health: 100.0,
            facing: side.yaw(),
            action: Action::Idle,
            flash: 0.0,
            hit_stop: 0.0,
            velocity: Vec3::ZERO,
            moving: false,
            walk_phase: 0.0,
            vertical_speed: 0.0,
            jumps: JumpState::default(),
            airborne: false,
            dodge_cooldown: 0.0,
            protected: false,
        }
    }
    pub fn forward(&self) -> Vec3 {
        Quat::from_rotation_y(self.facing) * Vec3::NEG_Z
    }
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }
    pub fn label(&self) -> String {
        match &self.action {
            Action::Idle => {
                if self.airborne {
                    format!("AIRBORNE {}/2", self.jumps.used())
                } else if self.moving {
                    "MOVING".into()
                } else {
                    "READY".into()
                }
            }
            Action::Guard => "GUARD / FRONT 120 DEG".into(),
            Action::Attack(attack) => match attack.kind {
                AttackKind::Light(index) => format!(
                    "CUT {} / {}",
                    index + 1,
                    if attack.can_chain() {
                        "CHAIN NOW"
                    } else {
                        attack.phase()
                    }
                ),
                AttackKind::Training => format!("TRAINING STRIKE / {}", attack.phase()),
            },
            Action::Stagger(_) => "STAGGERED".into(),
            Action::Down(_) => "DOWN / RECOVERING".into(),
            Action::Dodge { .. } => "DODGING".into(),
        }
    }
}

#[derive(Resource)]
pub(super) struct CombatStatus {
    pub last: String,
    pub hits: u32,
    pub blocks: u32,
}
impl Default for CombatStatus {
    fn default() -> Self {
        Self {
            last: "Close the gap. Click near the end of a cut to chain.".into(),
            hits: 0,
            blocks: 0,
        }
    }
}

pub(super) struct Impact {
    pub position: Vec3,
    pub damage: f32,
    pub blocked: bool,
    pub target_side: Side,
    pub heavy: bool,
}
#[derive(Resource, Default)]
pub(super) struct Impacts(pub Vec<Impact>);

struct Snapshot {
    entity: Entity,
    side: Side,
    position: Vec3,
    alive: bool,
}
struct Hit {
    source: Entity,
    target: Entity,
    damage: f32,
    heavy: bool,
    origin: Vec3,
}

fn bounded(position: Vec3, other: Option<Vec3>) -> Vec3 {
    let mut result = Vec3::new(
        position.x.clamp(-6.8, 6.8),
        position.y,
        position.z.clamp(-6.8, 6.8),
    );
    if let Some(other) = other {
        let offset = Vec3::new(result.x - other.x, 0.0, result.z - other.z);
        if (result.y - other.y).abs() < 0.9 && offset.length_squared() < 0.68 * 0.68 {
            let corrected = other + offset.try_normalize().unwrap_or(Vec3::Z) * 0.68;
            result.x = corrected.x;
            result.z = corrected.z;
        }
    }
    result
}

pub(super) fn simulate(
    controls: Res<Controls>,
    time: Res<Time>,
    orbit: Res<OrbitCamera>,
    mut fighters: Query<(Entity, &mut Fighter, &mut Transform)>,
    mut impacts: ResMut<Impacts>,
    mut status: ResMut<CombatStatus>,
    mut sounds: ResMut<Sounds>,
) {
    impacts.0.clear();
    sounds.0.clear();
    if controls.reset {
        sounds.0.push("reset");
        for (_, mut fighter, mut transform) in &mut fighters {
            let side = fighter.side;
            *fighter = Fighter::new(side);
            *transform = Transform::from_translation(side.home())
                .with_rotation(Quat::from_rotation_y(side.yaw()));
        }
        *status = CombatStatus::default();
        return;
    }
    if !controls.active {
        for (_, mut fighter, _) in &mut fighters {
            fighter.moving = false;
            if matches!(fighter.action, Action::Guard) {
                fighter.action = Action::Idle;
            }
        }
        return;
    }
    let dt = time.delta_secs();
    let snapshots: Vec<_> = fighters
        .iter()
        .map(|(entity, fighter, transform)| Snapshot {
            entity,
            side: fighter.side,
            position: transform.translation,
            alive: fighter.alive(),
        })
        .collect();
    let mut hits = Vec::new();
    for (entity, mut fighter, mut transform) in &mut fighters {
        let fighter = &mut *fighter;
        fighter.flash = (fighter.flash - dt).max(0.0);
        fighter.moving = false;
        fighter.protected = false;
        fighter.dodge_cooldown = (fighter.dodge_cooldown - dt).max(0.0);
        if fighter.hit_stop > 0.0 {
            fighter.hit_stop = (fighter.hit_stop - dt).max(0.0);
            continue;
        }
        let opponent = snapshots.iter().find(|target| target.side != fighter.side);
        let obstacle = opponent
            .filter(|target| target.alive)
            .map(|target| target.position);
        let can_act = matches!(&fighter.action, Action::Idle | Action::Guard)
            || matches!(&fighter.action, Action::Attack(attack) if attack.phase() == "RECOVERY");
        if fighter.side == Side::Player && can_act {
            if controls.dodge && !fighter.airborne && fighter.dodge_cooldown <= 0.0 {
                let movement = Quat::from_rotation_y(orbit.yaw)
                    * Vec3::new(controls.movement.x, 0.0, -controls.movement.y);
                let direction = if movement.length_squared() > 0.0 {
                    movement
                } else {
                    -fighter.forward()
                };
                fighter.action = Action::Dodge {
                    elapsed: 0.0,
                    direction,
                };
                fighter.dodge_cooldown = DODGE_COOLDOWN;
                fighter.velocity = Vec3::ZERO;
                sounds.0.push("dodge");
            } else if controls.jump && matches!(fighter.action, Action::Idle | Action::Guard) {
                if let Some(speed) = fighter.jumps.try_jump() {
                    fighter.vertical_speed = speed;
                    fighter.airborne = true;
                    fighter.action = Action::Idle;
                    sounds.0.push("jump");
                }
            }
        }
        if fighter.airborne {
            let (height, speed) = jump_step(transform.translation.y, fighter.vertical_speed, dt);
            transform.translation.y = height;
            fighter.vertical_speed = speed;
            if height == 0.0 && speed == 0.0 {
                fighter.airborne = false;
                fighter.jumps.land();
                sounds.0.push("land");
            }
        }
        let was_guarding = matches!(fighter.action, Action::Guard);
        let (movement, retention) = damped_motion(dt, 12.0);
        transform.translation = bounded(
            transform.translation + fighter.velocity * movement,
            obstacle,
        );
        fighter.velocity *= retention;
        let mut next_action = None;
        match &mut fighter.action {
            Action::Dodge { elapsed, direction } => {
                let previous = *elapsed;
                *elapsed += dt;
                fighter.protected = dodge_protected(previous, *elapsed);
                transform.translation = bounded(
                    transform.translation + *direction * dodge_step(previous, *elapsed),
                    obstacle,
                );
                if *elapsed >= DODGE_DURATION {
                    next_action = Some(Action::Idle);
                }
            }
            Action::Down(remaining) => {
                *remaining -= dt;
                if *remaining <= 0.0 {
                    let side = fighter.side;
                    *fighter = Fighter::new(side);
                    transform.translation = side.home();
                    sounds.0.push("revive");
                }
            }
            Action::Stagger(remaining) => {
                *remaining -= dt;
                if *remaining <= 0.0 {
                    next_action = Some(Action::Idle);
                }
            }
            Action::Attack(attack) => {
                if fighter.side == Side::Player && controls.attack {
                    attack.buffer_next();
                }
                let previous = attack.elapsed;
                let step = attack.tick(dt);
                if previous < attack.spec().windup && attack.elapsed >= attack.spec().windup {
                    sounds.0.push(match attack.kind {
                        AttackKind::Light(0) => "swing1",
                        AttackKind::Light(1) => "swing2",
                        AttackKind::Light(_) => "swing3",
                        AttackKind::Training => "swing3",
                    });
                }
                let forward = Quat::from_rotation_y(fighter.facing) * Vec3::NEG_Z;
                transform.translation = bounded(
                    transform.translation + forward * step.forward_step,
                    obstacle,
                );
                if step.active {
                    for target in &snapshots {
                        if target.side == fighter.side || !target.alive {
                            continue;
                        }
                        let mut delta = target.position - transform.translation;
                        if delta.y.abs() > 1.1 {
                            continue;
                        }
                        delta.y = 0.0;
                        let dot = forward.dot(delta.normalize_or_zero());
                        if inside_attack(delta.length(), dot, attack.spec())
                            && attack.claim_target(target.entity.to_bits())
                        {
                            hits.push(Hit {
                                source: entity,
                                target: target.entity,
                                damage: attack.spec().damage,
                                heavy: matches!(attack.kind, AttackKind::Light(2)),
                                origin: transform.translation,
                            });
                        }
                    }
                }
                if step.finished {
                    if fighter.side == Side::Player && !attack.has_hit() {
                        status.last = "MISS / Move closer and face your target.".into();
                    }
                    next_action = Some(if fighter.side == Side::Player && controls.guard {
                        Action::Guard
                    } else if let Some(kind) = attack.next_combo() {
                        Action::Attack(Attack::new(kind))
                    } else {
                        Action::Idle
                    });
                }
            }
            Action::Idle | Action::Guard => {
                if fighter.side == Side::Player {
                    if controls.guard {
                        next_action = Some(Action::Guard);
                        fighter.facing = orbit.yaw;
                    } else if controls.attack && !fighter.airborne {
                        fighter.facing = orbit.yaw;
                        next_action = Some(Action::Attack(Attack::new(AttackKind::Light(0))));
                    } else {
                        next_action = Some(Action::Idle);
                    }
                    if !controls.attack || controls.guard {
                        let movement = Quat::from_rotation_y(orbit.yaw)
                            * Vec3::new(controls.movement.x, 0.0, -controls.movement.y);
                        if movement.length_squared() > 0.0 {
                            fighter.moving = true;
                            let previous_step = (fighter.walk_phase / std::f32::consts::PI) as i32;
                            fighter.walk_phase += dt * if controls.guard { 5.0 } else { 9.0 };
                            if !fighter.airborne
                                && (fighter.walk_phase / std::f32::consts::PI) as i32
                                    != previous_step
                            {
                                sounds.0.push("step");
                            }
                            if !controls.guard {
                                fighter.facing = (-movement.x).atan2(-movement.z);
                            }
                            transform.translation = bounded(
                                transform.translation
                                    + movement * dt * if controls.guard { 1.6 } else { 3.8 },
                                obstacle,
                            );
                        }
                    }
                } else if controls.training_attack && opponent.is_some_and(|target| target.alive) {
                    let direction = opponent.unwrap().position - transform.translation;
                    fighter.facing = (-direction.x).atan2(-direction.z);
                    next_action = Some(Action::Attack(Attack::new(AttackKind::Training)));
                    status.last = "Incoming strike! Guard or dodge as the blade arrives.".into();
                    sounds.0.push("windup");
                }
            }
        }
        if matches!(&next_action, Some(Action::Guard)) && !was_guarding {
            sounds.0.push("guard-up");
        }
        if let Some(action) = next_action {
            fighter.action = action;
        }
        transform.rotation = Quat::from_rotation_y(fighter.facing);
    }
    for hit in hits {
        let Ok((_, mut target, transform)) = fighters.get_mut(hit.target) else {
            continue;
        };
        if !target.alive() {
            continue;
        }
        if target.protected {
            status.last = "EVADED / The strike passed through your dodge window.".into();
            sounds.0.push("evade");
            continue;
        }
        let mut incoming = hit.origin - transform.translation;
        incoming.y = 0.0;
        let incoming = incoming.normalize_or_zero();
        let (damage, blocked) = guarded_damage(
            hit.damage,
            matches!(target.action, Action::Guard),
            target.forward().dot(incoming),
        );
        target.health = (target.health - damage).max(0.0);
        sounds.0.push(if blocked { "guard" } else { "hit" });
        target.flash = 0.18;
        target.hit_stop = if blocked { 0.035 } else { 0.055 };
        target.velocity = -incoming
            * if blocked {
                0.8
            } else if hit.heavy {
                3.6
            } else {
                2.2
            };
        if blocked {
            status.blocks += 1;
        } else {
            status.hits += 1;
        }
        if !target.alive() {
            target.action = Action::Down(2.0);
            sounds.0.push("down");
            status.last = if target.side == Side::Dummy {
                "DUMMY DOWN / Respawning in 2 seconds."
            } else {
                "YOU ARE DOWN / Recovering in 2 seconds."
            }
            .into();
        } else if blocked {
            status.last = format!("BLOCK / {:.0} damage taken (80% reduced)", damage);
        } else {
            target.action = Action::Stagger(0.24);
            status.last = format!(
                "{} / {:.0} damage",
                if hit.heavy { "HEAVY HIT" } else { "HIT" },
                damage
            );
        }
        impacts.0.push(Impact {
            position: transform.translation + Vec3::Y * 1.25,
            damage,
            blocked,
            target_side: target.side,
            heavy: hit.heavy,
        });
        if let Ok((_, mut source, _)) = fighters.get_mut(hit.source) {
            source.hit_stop = 0.045;
        }
    }
}
