pub mod core;

use super::{camera::OrbitCamera, input::Controls, sound::Sounds};
use bevy::prelude::*;
use core::{
    Armor, Attack, AttackAxis, AttackKind, CHARGE_AUTO_RELEASE, CHARGE_READY, Charge,
    DODGE_DURATION, Defense, HitResult, JumpState, PARRY_DURATION, Stamina, can_clash,
    damped_motion, dodge_protected, dodge_step, inside_attack, jump_step, parry_window,
    resolve_hit, wave_intersects,
};
use std::collections::HashSet;

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
    Charge(Charge),
    Parry(f32),
    Attack(Attack),
    Stagger(f32),
    Down(f32),
    Dodge {
        elapsed: f32,
        direction: Vec3,
        held: bool,
    },
    Slide {
        elapsed: f32,
        direction: Vec3,
    },
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
    pub crouched: bool,
    pub sprinting: bool,
    pub walk_phase: f32,
    pub vertical_speed: f32,
    pub jumps: JumpState,
    pub airborne: bool,
    pub protected: bool,
    pub parry_active: bool,
    pub stamina: Stamina,
    pub riposte: f32,
    pub disarmed: f32,
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
            crouched: false,
            sprinting: false,
            walk_phase: 0.0,
            vertical_speed: 0.0,
            jumps: JumpState::default(),
            airborne: false,
            protected: false,
            parry_active: false,
            stamina: Stamina::default(),
            riposte: 0.0,
            disarmed: 0.0,
        }
    }
    pub fn forward(&self) -> Vec3 {
        Quat::from_rotation_y(self.facing) * Vec3::NEG_Z
    }
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }
    pub fn armor(&self) -> Armor {
        match &self.action {
            Action::Charge(charge) if charge.elapsed >= CHARGE_READY => Armor::Blue,
            Action::Attack(attack) if attack.phase() != "RECOVERY" => attack.kind.armor(),
            _ => Armor::Normal,
        }
    }
    fn defense(&self) -> Defense {
        if self.protected {
            Defense::Evade
        } else if self.parry_active {
            Defense::Parry
        } else if let Action::Attack(attack) = &self.action
            && attack.phase() == "STRIKE"
        {
            Defense::Striking(attack.kind.armor())
        } else {
            match self.armor() {
                Armor::Blue => Defense::Blue,
                Armor::Gold => Defense::Gold,
                _ => Defense::Open,
            }
        }
    }
    pub fn label(&self) -> String {
        if self.disarmed > 0.0 {
            return "DISARMED / TRAINING AUTO-RECOVERY".into();
        }
        match &self.action {
            Action::Idle if self.airborne => format!("AIRBORNE {}/2", self.jumps.used()),
            Action::Idle if self.riposte > 0.0 => "RIPOSTE READY / LMB".into(),
            Action::Idle if self.crouched => "CROUCHING".into(),
            Action::Idle if self.sprinting => "SPRINTING".into(),
            Action::Idle if self.moving => "MOVING".into(),
            Action::Idle => "READY".into(),
            Action::Charge(charge) => format!(
                "{} CHARGE / {}",
                charge.axis.label(),
                if charge.elapsed >= CHARGE_READY {
                    "BLUE READY"
                } else {
                    "BUILDING"
                }
            ),
            Action::Parry(_) => if self.parry_active {
                "PARRY / ACTIVE"
            } else {
                "PARRY / RECOVERY"
            }
            .into(),
            Action::Attack(attack) => format!(
                "{} / {}",
                attack.kind.label(),
                if attack.can_chain() {
                    "CHAIN NOW"
                } else {
                    attack.phase()
                }
            ),
            Action::Stagger(_) => "STAGGERED".into(),
            Action::Down(_) => "DOWN / RECOVERING".into(),
            Action::Dodge { .. } => "DODGING".into(),
            Action::Slide { .. } => "SLIDING".into(),
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
            last: "Longsword / LMB horizontal, RMB vertical. Hold either for sword energy.".into(),
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

pub(super) struct SwordWave {
    pub id: u64,
    source: Entity,
    side: Side,
    pub origin: Vec3,
    pub direction: Vec3,
    pub axis: AttackAxis,
    pub distance: f32,
    damage: f32,
    targets: HashSet<Entity>,
}
#[derive(Resource, Default)]
pub(super) struct SwordWaves {
    pub active: Vec<SwordWave>,
    next_id: u64,
}
impl SwordWaves {
    fn spawn(
        &mut self,
        source: Entity,
        side: Side,
        origin: Vec3,
        direction: Vec3,
        axis: AttackAxis,
        damage: f32,
    ) {
        self.next_id += 1;
        self.active.push(SwordWave {
            id: self.next_id,
            source,
            side,
            origin,
            direction,
            axis,
            distance: 0.0,
            damage,
            targets: HashSet::new(),
        });
    }
}
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
    kind: AttackKind,
    origin: Vec3,
    wave: Option<u64>,
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
fn movement(controls: &Controls, heading: f32) -> Vec3 {
    Quat::from_rotation_y(heading) * Vec3::new(controls.movement.x, 0.0, -controls.movement.y)
}
fn pressed_axis(controls: &Controls) -> Option<AttackAxis> {
    if controls.attacks[0].pressed {
        Some(AttackAxis::Horizontal)
    } else if controls.attacks[1].pressed {
        Some(AttackAxis::Vertical)
    } else {
        None
    }
}
fn button_index(axis: AttackAxis) -> usize {
    if axis == AttackAxis::Horizontal { 0 } else { 1 }
}
fn can_cancel(action: &Action) -> bool {
    matches!(
        action,
        Action::Idle | Action::Charge(_) | Action::Slide { .. }
    ) || matches!(action, Action::Attack(attack) if attack.phase() == "RECOVERY")
}
fn start_attack(fighter: &mut Fighter, kind: AttackKind, heading: f32) {
    fighter.facing = heading;
    fighter.action = Action::Attack(Attack::new(kind));
}
fn move_fighter(
    fighter: &mut Fighter,
    transform: &mut Transform,
    controls: &Controls,
    heading: f32,
    dt: f32,
    obstacle: Option<Vec3>,
) {
    let direction = movement(controls, heading);
    if direction.length_squared() == 0.0 {
        return;
    }
    fighter.moving = true;
    let charging = matches!(fighter.action, Action::Charge(_));
    fighter.sprinting = controls.sprint && !controls.crouch && !charging && !controls.walk;
    let speed = if charging || fighter.crouched {
        1.6
    } else if controls.walk {
        1.8
    } else if fighter.sprinting {
        6.0
    } else {
        3.8
    };
    fighter.walk_phase += dt * speed * 2.5;
    if !charging {
        fighter.facing = (-direction.x).atan2(-direction.z);
    }
    transform.translation = bounded(transform.translation + direction * speed * dt, obstacle);
}

pub(super) fn simulate(
    controls: Res<Controls>,
    time: Res<Time>,
    orbit: Res<OrbitCamera>,
    mut fighters: Query<(Entity, &mut Fighter, &mut Transform)>,
    mut impacts: ResMut<Impacts>,
    mut status: ResMut<CombatStatus>,
    mut sounds: ResMut<Sounds>,
    mut waves: ResMut<SwordWaves>,
) {
    impacts.0.clear();
    sounds.0.clear();
    if controls.reset {
        sounds.0.push("reset");
        waves.active.clear();
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
            fighter.crouched = false;
            fighter.sprinting = false;
            if matches!(fighter.action, Action::Charge(_) | Action::Parry(_)) {
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
        fighter.sprinting = false;
        fighter.protected = false;
        fighter.parry_active = false;
        fighter.stamina.tick(dt);
        fighter.riposte = (fighter.riposte - dt).max(0.0);
        fighter.disarmed = (fighter.disarmed - dt).max(0.0);
        fighter.crouched = fighter.side == Side::Player && controls.crouch && !fighter.airborne;
        if fighter.hit_stop > 0.0 {
            fighter.hit_stop = (fighter.hit_stop - dt).max(0.0);
            continue;
        }
        let opponent = snapshots.iter().find(|target| target.side != fighter.side);
        let obstacle = opponent
            .filter(|target| target.alive)
            .map(|target| target.position);
        if fighter.side == Side::Player && can_cancel(&fighter.action) {
            if controls.dodge && !fighter.airborne && fighter.stamina.spend(20.0) {
                let direction = movement(&controls, orbit.heading);
                let direction = if direction.length_squared() > 0.0 {
                    direction
                } else {
                    -fighter.forward()
                };
                fighter.action = Action::Dodge {
                    elapsed: 0.0,
                    direction,
                    held: false,
                };
                fighter.velocity = Vec3::ZERO;
                sounds.0.push("dodge");
            } else if controls.jump {
                if let Some(speed) = fighter.jumps.try_jump() {
                    fighter.vertical_speed = speed;
                    fighter.airborne = true;
                    fighter.action = Action::Idle;
                    fighter.crouched = false;
                    sounds.0.push("jump");
                }
            } else if controls.parry && fighter.disarmed <= 0.0 {
                fighter.facing = orbit.heading;
                fighter.action = Action::Parry(0.0);
                sounds.0.push("parry");
            } else if controls.crouch_pressed {
                let direction = movement(&controls, orbit.heading);
                fighter.action =
                    if controls.sprint && direction.length_squared() > 0.0 && !fighter.airborne {
                        Action::Slide {
                            elapsed: 0.0,
                            direction,
                        }
                    } else {
                        Action::Idle
                    };
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
        let (distance, retention) = damped_motion(dt, 12.0);
        transform.translation = bounded(
            transform.translation + fighter.velocity * distance,
            obstacle,
        );
        fighter.velocity *= retention;
        let mut next_action = None;
        match &mut fighter.action {
            Action::Dodge {
                elapsed,
                direction,
                held,
            } => {
                let previous = *elapsed;
                *elapsed += dt;
                if controls.sprint && *elapsed >= 0.16 {
                    *held = true;
                }
                // Holding Shift makes a short dash into sprint; tapping keeps the full dodge.
                let duration = if *held { 0.26 } else { DODGE_DURATION };
                fighter.protected = dodge_protected(previous, *elapsed);
                transform.translation = bounded(
                    transform.translation + *direction * dodge_step(previous, *elapsed),
                    obstacle,
                );
                if *elapsed >= duration {
                    next_action = Some(Action::Idle);
                }
            }
            Action::Slide { elapsed, direction } => {
                *elapsed += dt;
                transform.translation = bounded(
                    transform.translation
                        + *direction * dt * 5.0 * (1.0 - *elapsed / 0.55).max(0.0),
                    obstacle,
                );
                if *elapsed >= 0.55 || !controls.crouch {
                    next_action = Some(Action::Idle);
                }
                if let Some(axis) = pressed_axis(&controls) {
                    next_action = Some(Action::Attack(Attack::new(AttackKind::Crouch(axis))));
                    fighter.facing = orbit.heading;
                }
            }
            Action::Parry(elapsed) => {
                let previous = *elapsed;
                *elapsed += dt;
                fighter.parry_active = parry_window(previous, *elapsed);
                if *elapsed >= PARRY_DURATION {
                    next_action = Some(Action::Idle);
                }
            }
            Action::Charge(charge) => {
                let previous = charge.elapsed;
                charge.elapsed += dt;
                if previous < CHARGE_READY && charge.elapsed >= CHARGE_READY {
                    sounds.0.push("charge-ready");
                }
                fighter.facing = orbit.heading;
                let button = controls.attacks[button_index(charge.axis)];
                if button.released
                    || (!button.held && !button.pressed)
                    || charge.elapsed >= CHARGE_AUTO_RELEASE
                {
                    next_action = Some(Action::Attack(Attack::new(charge.release())));
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
                if fighter.side == Side::Player
                    && let Some(axis) = pressed_axis(&controls)
                {
                    attack.buffer_next(axis);
                }
                let previous = attack.elapsed;
                let step = attack.tick(dt);
                let forward = Quat::from_rotation_y(fighter.facing) * Vec3::NEG_Z;
                if previous < attack.spec().windup && attack.elapsed >= attack.spec().windup {
                    sounds.0.push(if attack.kind.armor() == Armor::Normal {
                        "swing1"
                    } else {
                        "swing3"
                    });
                    if let AttackKind::Focus(axis) = attack.kind {
                        waves.spawn(
                            entity,
                            fighter.side,
                            transform.translation + Vec3::Y * 1.0,
                            forward,
                            axis,
                            attack.spec().damage,
                        );
                        sounds.0.push("sword-energy");
                    }
                    if attack.kind == AttackKind::Crouch(AttackAxis::Vertical) && !fighter.airborne
                    {
                        fighter.vertical_speed = core::JUMP_SPEED;
                        fighter.airborne = true;
                        fighter.jumps.try_jump();
                        sounds.0.push("jump");
                    }
                    if attack.kind == AttackKind::Air(AttackAxis::Vertical) {
                        fighter.vertical_speed = -9.0;
                    }
                }
                transform.translation = bounded(
                    transform.translation + forward * step.forward_step,
                    obstacle,
                );
                if step.active && !matches!(attack.kind, AttackKind::Focus(_)) {
                    for target in &snapshots {
                        if target.side == fighter.side || !target.alive {
                            continue;
                        }
                        let mut delta = target.position - transform.translation;
                        if delta.y.abs() > 1.4 {
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
                                kind: attack.kind,
                                origin: transform.translation,
                                wave: None,
                            });
                        }
                    }
                }
                if step.finished {
                    if fighter.side == Side::Player
                        && !attack.has_hit()
                        && !matches!(attack.kind, AttackKind::Focus(_))
                    {
                        status.last = "MISS / Close the gap and face the target.".into();
                    }
                    next_action = Some(if let Some(kind) = attack.next_combo() {
                        if let AttackKind::Light(axis, combo) = kind
                            && controls.attacks[button_index(axis)].held
                        {
                            Action::Charge(Charge {
                                axis,
                                combo,
                                elapsed: 0.0,
                            })
                        } else {
                            Action::Attack(Attack::new(kind))
                        }
                    } else {
                        Action::Idle
                    });
                }
            }
            Action::Idle => {
                if fighter.side == Side::Player {
                    if fighter.disarmed <= 0.0
                        && !controls.parry
                        && let Some(axis) = pressed_axis(&controls)
                    {
                        fighter.facing = orbit.heading;
                        if fighter.riposte > 0.0 {
                            fighter.riposte = 0.0;
                            start_attack(fighter, AttackKind::Riposte, orbit.heading);
                        } else if fighter.airborne {
                            start_attack(fighter, AttackKind::Air(axis), orbit.heading);
                        } else if controls.crouch {
                            start_attack(fighter, AttackKind::Crouch(axis), orbit.heading);
                        } else {
                            fighter.action = Action::Charge(Charge {
                                axis,
                                combo: 0,
                                elapsed: 0.0,
                            });
                        }
                    }
                } else if fighter.disarmed <= 0.0 && opponent.is_some_and(|target| target.alive) {
                    let direction = opponent.unwrap().position - transform.translation;
                    let heading = (-direction.x).atan2(-direction.z);
                    if controls.training_parry {
                        fighter.facing = heading;
                        next_action = Some(Action::Parry(0.0));
                    } else if controls.training_attack || controls.training_normal {
                        start_attack(
                            fighter,
                            AttackKind::Training(controls.training_attack),
                            heading,
                        );
                        status.last = if controls.training_attack {
                            "Incoming BLUE / Parry on contact, or dodge."
                        } else {
                            "Incoming NORMAL / Beats parry; use blue armor or dodge."
                        }
                        .into();
                        sounds.0.push("windup");
                    }
                }
            }
        }
        if let Some(action) = next_action {
            fighter.action = action;
        }
        if fighter.side == Side::Player
            && matches!(fighter.action, Action::Idle | Action::Charge(_))
        {
            let step_before = (fighter.walk_phase / std::f32::consts::PI) as i32;
            move_fighter(
                fighter,
                &mut transform,
                &controls,
                orbit.heading,
                dt,
                obstacle,
            );
            if !fighter.airborne
                && (fighter.walk_phase / std::f32::consts::PI) as i32 != step_before
            {
                sounds.0.push("step");
            }
        }
        transform.rotation = Quat::from_rotation_y(fighter.facing);
    }
    // Re-snapshot positions after movement; a fast wave must test its whole path.
    for wave in &mut waves.active {
        let previous = wave.distance;
        wave.distance = (wave.distance + dt * 12.0).min(10.0);
        for (entity, fighter, transform) in &fighters {
            if fighter.side == wave.side || !fighter.alive() || wave.targets.contains(&entity) {
                continue;
            }
            let offset = transform.translation + Vec3::Y * 1.0 - wave.origin;
            let forward = offset.dot(wave.direction);
            let lateral = offset.dot(Vec3::new(wave.direction.z, 0.0, -wave.direction.x));
            if wave_intersects(
                previous,
                wave.distance,
                forward,
                lateral,
                offset.y,
                wave.axis,
            ) {
                wave.targets.insert(entity);
                hits.push(Hit {
                    source: wave.source,
                    target: entity,
                    damage: wave.damage,
                    kind: AttackKind::Focus(wave.axis),
                    origin: wave.origin,
                    wave: Some(wave.id),
                });
            }
        }
    }
    // Decide defenses before mutating either actor, so query iteration order
    // cannot turn simultaneous strikes into a one-sided interrupt.
    let mut results: Vec<_> = hits
        .iter()
        .map(|hit| {
            let (_, fighter, transform) = fighters.get(hit.target).unwrap();
            let incoming = (hit.origin - transform.translation)
                .with_y(0.0)
                .normalize_or_zero();
            let mut defense = fighter.defense();
            if fighter.forward().dot(incoming) < 0.0 {
                defense = match defense {
                    Defense::Parry | Defense::Striking(Armor::Normal) => Defense::Open,
                    Defense::Striking(Armor::Blue) => Defense::Blue,
                    Defense::Striking(Armor::Gold) => Defense::Gold,
                    other => other,
                };
            }
            if hit.wave.is_some() && matches!(defense, Defense::Striking(_)) {
                // Sword energy is countered with parry; swinging at it is not a clash.
                defense = match fighter.armor() {
                    Armor::Blue => Defense::Blue,
                    Armor::Gold => Defense::Gold,
                    _ => Defense::Open,
                };
            }
            resolve_hit(hit.kind.armor(), defense)
        })
        .collect();
    for (i, hit) in hits.iter().enumerate() {
        if hit.wave.is_some() || results[i] == HitResult::Evade {
            continue;
        }
        if let Some((j, _)) = hits.iter().enumerate().skip(i + 1).find(|(j, other)| {
            other.wave.is_none()
                && other.source == hit.target
                && other.target == hit.source
                && can_clash(hit.kind.armor(), other.kind.armor())
                && results[*j] != HitResult::Evade
        }) {
            results[i] = HitResult::Clash;
            results[j] = HitResult::Clash;
        }
    }
    let mut clash_pairs = HashSet::new();
    for (index, hit) in hits.into_iter().enumerate() {
        let Ok((_, mut target, transform)) = fighters.get_mut(hit.target) else {
            continue;
        };
        if !target.alive() {
            continue;
        }
        match results[index] {
            HitResult::Clash => {
                let pair = if hit.source.to_bits() < hit.target.to_bits() {
                    (hit.source, hit.target)
                } else {
                    (hit.target, hit.source)
                };
                if !clash_pairs.insert(pair) {
                    continue;
                }
                // Release the target borrow before applying the same clash to both.
                for entity in [hit.source, hit.target] {
                    if let Ok((_, mut fighter, transform)) = fighters.get_mut(entity) {
                        fighter.action = Action::Stagger(0.18);
                        fighter.hit_stop = 0.04;
                        impacts.0.push(Impact {
                            position: transform.translation + Vec3::Y,
                            damage: 0.0,
                            blocked: true,
                            target_side: fighter.side,
                            heavy: false,
                        });
                    }
                }
                sounds.0.push("guard");
                status.last = "CLASH / Equal attacks met.".into();
            }
            HitResult::Evade => {
                status.last = "EVADED".into();
                sounds.0.push("evade");
            }
            HitResult::Counter => {
                target.riposte = 1.8;
                target.action = Action::Idle;
                target.parry_active = false;
                let position = transform.translation;
                let side = target.side;
                impacts.0.push(Impact {
                    position: position + Vec3::Y,
                    damage: 0.0,
                    blocked: true,
                    target_side: side,
                    heavy: true,
                });
                status.blocks += 1;
                status.last = "PARRY SUCCESS / LMB for gold riposte.".into();
                sounds.0.push("guard");
                if let Some(id) = hit.wave {
                    if let Some(wave) = waves.active.iter_mut().find(|wave| wave.id == id) {
                        wave.distance = 10.0;
                        let direction = -wave.direction;
                        let axis = wave.axis;
                        waves.spawn(
                            hit.target,
                            side,
                            position + Vec3::Y,
                            direction,
                            axis,
                            hit.damage * 0.4,
                        );
                    }
                }
                if let Ok((_, mut source, source_transform)) = fighters.get_mut(hit.source)
                    && source_transform.translation.distance(position) < 2.8
                {
                    source.disarmed = 1.2;
                    source.action = Action::Stagger(1.2);
                    source.hit_stop = 0.06;
                }
            }
            HitResult::Damage { stagger } => {
                target.health = (target.health - hit.damage).max(0.0);
                target.flash = 0.18;
                target.hit_stop = 0.035;
                let incoming = (hit.origin - transform.translation)
                    .with_y(0.0)
                    .normalize_or_zero();
                target.velocity = -incoming
                    * if hit.kind.armor() == Armor::Normal {
                        1.5
                    } else {
                        3.0
                    };
                let heavy = hit.kind.armor() != Armor::Normal;
                if !target.alive() {
                    target.action = Action::Down(2.0);
                    sounds.0.push("down");
                    status.last = "DOWN / Training auto-recovery in 2 seconds.".into();
                } else if stagger {
                    target.action =
                        Action::Stagger(if hit.kind == AttackKind::Crouch(AttackAxis::Vertical) {
                            0.50
                        } else {
                            0.24
                        });
                    if hit.kind == AttackKind::Crouch(AttackAxis::Vertical) {
                        target.vertical_speed = 6.2;
                        target.airborne = true;
                    }
                    status.last = format!("{} / {:.0} damage", hit.kind.label(), hit.damage);
                } else {
                    status.last = format!("ARMOR / {:.0} damage, no stagger", hit.damage);
                }
                impacts.0.push(Impact {
                    position: transform.translation + Vec3::Y * 1.25,
                    damage: hit.damage,
                    blocked: false,
                    target_side: target.side,
                    heavy,
                });
                sounds.0.push("hit");
                status.hits += 1;
                if let Ok((_, mut source, _)) = fighters.get_mut(hit.source) {
                    source.hit_stop = 0.03;
                }
            }
        }
    }
    waves.active.retain(|wave| wave.distance < 10.0);
}

#[cfg(test)]
mod tests;
