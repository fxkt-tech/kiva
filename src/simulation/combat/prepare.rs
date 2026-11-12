//! Before physics: consume the player's commands, advance the committed action
//! and admit at most one new action. Each step of the tick is a named function.
use super::availability::{availability_for, window_valid};
use super::combatant::{ATTACK, COMBO, MELEE_SKILL, NEXT_PAIR, RANGED_SKILL, TOGGLE};
use super::companion::companion_pose;
use super::hit::{Candidates, CombatWorld, PlayerMeters, target_valid};
use super::runtime::ActionRuntime;
use super::skills::{LOCK_ALIGNMENT, LOCK_RANGE, Shape, Slot};
use super::{
    ActionIds, Arsenal, CombatEffect, CombatFeedback, CombatIntent, CombatStatus, Combatant, Enemy,
    Energy, Health,
};
use crate::simulation::defense::decay;
use crate::simulation::{
    ActionAvailability, ActionIntent, ActionKind, ActionSnapshot, Body, DefenseState, Facing,
    MeleeStage, MotionState, SlowEffect, Stamina, WeaponPair,
};
use bevy::prelude::*;

/// Without a target, a new action aims this far ahead of the actor.
const DEFAULT_AIM_DISTANCE: f32 = 3.0;

pub(in crate::simulation) fn tick_slows(time: Res<Time<Fixed>>, mut slows: Query<&mut SlowEffect>) {
    for mut slow in &mut slows {
        slow.tick(time.delta_secs());
    }
}

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
    PlayerMeters,
);

/// The acting player's read-only physical facts for this tick.
struct Actor<'a> {
    entity: Entity,
    body: &'a Body,
    motion: &'a MotionState,
}

pub(in crate::simulation) fn prepare(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut ids: ResMut<ActionIds>,
    mut feedback: ResMut<CombatFeedback>,
    enemies: Candidates,
    mut players: Query<PreparePlayer, Without<Enemy>>,
) {
    let dt = time.delta_secs();
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
        (mut energy, mut arsenal),
    ) in &mut players
    {
        let commands = std::mem::take(&mut intent.commands);
        let pair = intent.pair.take();
        let cancel_chain = std::mem::take(&mut intent.cancel_melee_chain);
        tick_timers(&mut combat, cancel_chain, dt);
        if !health.is_alive() {
            combat.clear_targets();
            defense.cancel();
            defense.clear();
            actions.clear();
            continue;
        }
        let actor = Actor {
            entity,
            body,
            motion,
        };
        update_target(
            &mut combat,
            commands & TOGGLE != 0,
            &actor,
            facing.direction(),
            &world,
            &enemies,
        );
        tick_combo_window(&mut combat, &enemies, dt);
        advance_action(&mut combat, &actor, &enemies, dt);

        let busy = combat.action.is_some();
        let buffered_input = commands & ATTACK != 0 && buffer_follow_up(&mut combat);
        if busy {
            actions.discard_movement_actions();
        }
        let grounded = motion.is_grounded() && !motion.is_dodging() && !motion.is_sliding();
        let moving = actions.movement_requested();
        advance_defense(
            &mut defense,
            &mut combat,
            motion,
            stamina,
            grounded && !busy,
            moving,
            dt,
        );
        let free = !busy && !defense.is_active() && !motion.is_dodging() && !motion.is_sliding();
        let requested = pair.or((commands & NEXT_PAIR != 0).then(|| combat.pair.next()));
        switch_weapons(
            &mut combat,
            &mut arsenal,
            requested,
            !busy,
            free && grounded,
        );
        let distance = target_distance(&combat, &actor, &world, &enemies);
        let availability = availability_for(&combat, motion, &defense, distance);
        combat.availability = availability;

        // A pending input gets one admission attempt, after the previous
        // action's final markers retired it in resolve_actions(). New legal
        // actions retain priority; a pending movement edge is handled by the motor.
        let queued_attack = !busy && std::mem::take(&mut combat.melee_chain.buffered);
        let movement_has_priority = actions.postpones_buffered_attack(motion, dt);
        if queued_attack && movement_has_priority && actions.crouch_toggle_requested() {
            // Let C reach the motor. An accepted slide clears this chain;
            // an ordinary crouch preserves it for the following rule tick.
            combat.melee_chain.buffered = true;
        }
        let attack = commands & ATTACK != 0 || (queued_attack && !movement_has_priority);
        // While enraged a prerequisite skill flows into its combo unprompted.
        let combo = commands & COMBO != 0 || energy.is_enraged();
        if let Some(slot) = admit(commands, combo, attack, availability) {
            let kind = combat.pair.action(slot);
            let Some(mut plan) = plan(kind, &mut combat, &mut facing, &actor, &world, &enemies)
            else {
                combat.reject(
                    CombatStatus::TargetOutOfReach,
                    entity,
                    body.aim_position(),
                    &mut feedback,
                );
                continue;
            };
            plan.damage_scale = damage_scale(slot, kind, &mut energy);
            arsenal.wear(combat.pair, kind.spec().wear);
            commit(
                slot,
                kind,
                plan,
                &mut combat,
                &actor,
                &mut ids,
                &mut feedback,
            );
            actions.discard_movement_actions();
        } else if commands & (MELEE_SKILL | RANGED_SKILL | COMBO) != 0
            || (commands & ATTACK != 0 && !buffered_input)
        {
            combat.reject(
                if busy {
                    CombatStatus::Busy
                } else if defense.is_active() {
                    CombatStatus::Guarding
                } else {
                    CombatStatus::Unavailable
                },
                entity,
                body.aim_position(),
                &mut feedback,
            );
        }
        if let Some(velocity) = pull_velocity(&combat, body, &enemies, dt) {
            combat.pull_velocity = Some(velocity);
        }
    }
}

/// Cooldowns, the follow-up grace and the previous tick's pull request.
fn tick_timers(combat: &mut Combatant, cancel_chain: bool, dt: f32) {
    if cancel_chain {
        combat.melee_chain.clear();
    }
    combat.melee_chain.tick(dt);
    combat.attack_cooldown = decay(combat.attack_cooldown, dt);
    combat.melee_cooldown = decay(combat.melee_cooldown, dt);
    combat.ranged_cooldown = decay(combat.ranged_cooldown, dt);
    combat.pull_velocity = None;
}

/// Advance the right-button gesture. A new gesture, a dodge or a slide drops
/// the ordinary attack's queued follow-up.
fn advance_defense(
    defense: &mut DefenseState,
    combat: &mut Combatant,
    motion: &MotionState,
    stamina: &Stamina,
    can_defend: bool,
    moving: bool,
    dt: f32,
) {
    let gesture_id = defense.snapshot().gesture_id;
    defense.advance(dt, can_defend, moving, motion, stamina);
    if defense.snapshot().gesture_id != gesture_id || motion.is_dodging() || motion.is_sliding() {
        combat.melee_chain.clear();
    }
}

/// A worn-out pair is rotated out and restored as soon as no action or combo
/// window depends on it; otherwise a selection is honoured while free on the
/// ground.
fn switch_weapons(
    combat: &mut Combatant,
    arsenal: &mut Arsenal,
    requested: Option<WeaponPair>,
    idle: bool,
    free_on_ground: bool,
) {
    if idle && combat.combo.is_none() && arsenal.exhausted(combat.pair) {
        let worn = combat.pair;
        arsenal.restore(worn);
        switch_pair(combat, worn.next());
        combat.status = CombatStatus::WeaponExhausted;
    } else if let Some(pair) = requested
        && free_on_ground
    {
        switch_pair(combat, pair);
    }
}

/// How much harder the new action hits: ordinary attacks while enraged, and
/// skills whose meter is full (which spends the meter).
fn damage_scale(slot: Slot, kind: ActionKind, energy: &mut Energy) -> f32 {
    let spec = kind.spec();
    match slot {
        Slot::Attack => energy.attack_scale(),
        Slot::MeleeSkill | Slot::RangedSkill if energy.empower(slot, spec.rage) => spec.empowered(),
        Slot::MeleeSkill | Slot::RangedSkill | Slot::Combo => 1.0,
    }
}

/// Equip the other weapon pair; windows and follow-ups belong to the old one.
fn switch_pair(combat: &mut Combatant, pair: WeaponPair) {
    if combat.pair != pair {
        combat.melee_chain.clear();
    }
    combat.pair = pair;
    combat.combo = None;
    combat.axe_away = false;
    combat.status = CombatStatus::PairSwitched;
}

/// Drop a target which can no longer be attacked, then apply a lock toggle:
/// unlock, or pick the visible enemy best aligned with the actor's facing.
fn update_target(
    combat: &mut Combatant,
    toggle: bool,
    actor: &Actor,
    facing: Vec2,
    world: &CombatWorld,
    enemies: &Candidates,
) {
    if combat.target.is_some_and(|e| {
        enemies
            .get(e)
            .map_or(true, |(_, _, _, h, _, b)| !target_valid(h, b))
    }) {
        combat.target = None;
    }
    if !toggle {
        return;
    }
    let (entity, body) = (actor.entity, actor.body);
    combat.target = if combat.target.is_some() {
        None
    } else {
        enemies
            .iter()
            .filter(|(_, b, _, h, _, boss)| {
                target_valid(h, *boss)
                    && b.world_position().distance(body.world_position()) <= LOCK_RANGE
            })
            .filter(|(e, b, _, _, _, _)| {
                world.visible(body.aim_position(), b.aim_position(), entity, *e)
            })
            .filter_map(|(e, b, _, _, _, _)| {
                let offset = (b.world_position() - body.world_position()).xz();
                let align = facing.dot(offset.normalize_or_zero());
                (align > LOCK_ALIGNMENT).then_some((e, align - offset.length() * 0.01))
            })
            .max_by(|a, b| {
                a.1.total_cmp(&b.1)
                    .then_with(|| b.0.to_bits().cmp(&a.0.to_bits()))
            })
            .map(|(e, _)| e)
    };
}

fn tick_combo_window(combat: &mut Combatant, enemies: &Candidates, dt: f32) {
    if let Some(mut combo) = combat.combo {
        combo.remaining = decay(combo.remaining, dt);
        let valid = window_valid(combo, enemies);
        combat.combo = (valid && combo.remaining > 0.0).then_some(combo);
    }
}

/// Move the committed action's clock, follow its target, supervise a pull and
/// drop a cancelled action once it has returned. Its markers are resolved
/// after physics, so an unfinished action always stays committed here.
fn advance_action(combat: &mut Combatant, actor: &Actor, enemies: &Candidates, dt: f32) {
    let Some(mut action) = combat.action.take() else {
        return;
    };
    action.previous = action.snapshot.elapsed;
    action.snapshot.elapsed += dt;
    if action.cancelled {
        action.return_remaining = decay(action.return_remaining, dt);
    }
    if let Some(target) = action.snapshot.target {
        match enemies.get(target) {
            Ok((_, b, _, h, _, boss)) if target_valid(h, boss) => {
                action.snapshot.aim_position = b.aim_position();
            }
            _ => action.cancel(),
        }
    }
    supervise_pull(&mut action, actor, enemies, dt);
    if action.cancelled || action.snapshot.elapsed >= action.snapshot.duration {
        combat.axe_away = false;
    }
    let returned =
        action.cancelled && (action.return_remaining == 0.0 || !action.snapshot.kind.is_combo());
    if returned {
        combat.axe_away = false;
        combat.melee_chain.clear();
    } else {
        combat.action = Some(action);
    }
}

/// Stop the pull on arrival; cancel it when the target is out of reach
/// vertically, the actor left the ground, or real contacts block progress.
fn supervise_pull(action: &mut ActionRuntime, actor: &Actor, enemies: &Candidates, dt: f32) {
    let Some(pull) = action.snapshot.kind.spec().pull() else {
        return;
    };
    if action.cancelled || action.snapshot.elapsed > pull.until || action.pull_stopped {
        return;
    }
    let Some(target) = action.snapshot.target.and_then(|t| enemies.get(t).ok()) else {
        return;
    };
    let offset = target.1.world_position() - actor.body.world_position();
    let distance = offset.xz().length();
    if offset.y.abs() > pull.max_height_gap || !actor.motion.is_grounded() {
        action.cancel();
    } else if distance <= pull.stop_distance {
        action.pull_stopped = true;
    } else {
        let dir = offset.xz().normalize_or_zero();
        if actor.body.step_displacement().dot(dir) < pull.stalled_step
            && action.snapshot.elapsed > dt * 2.0
        {
            action.pull_stalled += dt;
        } else {
            action.pull_stalled = 0.0;
        }
        if action.pull_stalled + 1e-6 >= pull.stall_limit {
            action.cancel();
        }
    }
}

/// An attack press during a committed ordinary attack buffers one follow-up.
fn buffer_follow_up(combat: &mut Combatant) -> bool {
    let action_id = combat
        .action
        .as_ref()
        .filter(|action| action.snapshot.kind == ActionKind::Melee && !action.cancelled)
        .map(|action| action.snapshot.action_id);
    action_id.is_some_and(|id| combat.melee_chain.buffer(id))
}

/// Distance to the locked target, while it is valid and visible.
fn target_distance(
    combat: &Combatant,
    actor: &Actor,
    world: &CombatWorld,
    enemies: &Candidates,
) -> Option<f32> {
    let body = actor.body;
    combat
        .target
        .and_then(|t| enemies.get(t).ok())
        .filter(|(t, b, _, h, _, boss)| {
            target_valid(h, *boss)
                && world.visible(body.aim_position(), b.aim_position(), actor.entity, *t)
        })
        .map(|(_, b, _, _, _, _)| b.world_position().distance(body.world_position()))
}

/// The highest-priority requested input which is available right now.
fn admit(
    commands: u8,
    combo: bool,
    attack: bool,
    availability: ActionAvailability,
) -> Option<Slot> {
    if combo && availability.combo {
        Some(Slot::Combo)
    } else if commands & MELEE_SKILL != 0 && availability.melee_skill {
        Some(Slot::MeleeSkill)
    } else if commands & RANGED_SKILL != 0 && availability.ranged_skill {
        Some(Slot::RangedSkill)
    } else if attack && availability.attack {
        Some(Slot::Attack)
    } else {
        None
    }
}

/// Where a new action is aimed and where the companion would stand.
struct Plan {
    melee_stage: Option<MeleeStage>,
    target: Option<Entity>,
    aim: Vec3,
    companion_position: Vec3,
    source_position: Vec3,
    damage_scale: f32,
}

/// Resolve the new action's target and aim. A combo consumes its window even
/// when the pull target then turns out to be vertically out of reach (`None`).
fn plan(
    kind: ActionKind,
    combat: &mut Combatant,
    facing: &mut Facing,
    actor: &Actor,
    world: &CombatWorld,
    enemies: &Candidates,
) -> Option<Plan> {
    let spec = kind.spec();
    let body = actor.body;
    let melee_stage = (kind == ActionKind::Melee).then_some(combat.melee_chain.next_stage);
    let combo = if kind.is_combo() {
        combat.combo.take()
    } else {
        None
    };
    let target = if spec.targeted() {
        combo.map(|c| c.original_target).or(combat.target)
    } else {
        None
    };
    let target_body = target
        .and_then(|t| enemies.get(t).ok())
        .map(|(_, b, _, _, _, _)| b);
    if let Some(pull) = spec.pull()
        && target_body.is_none_or(|b| {
            (b.world_position().y - body.world_position().y).abs() > pull.max_height_gap
        })
    {
        return None;
    }
    let ahead = Vec3::new(facing.direction().x, 0.0, facing.direction().y);
    let aim = target_body.map_or(
        body.aim_position() + ahead * DEFAULT_AIM_DISTANCE,
        Body::aim_position,
    );
    if !matches!(spec.shape, Shape::Blade) {
        facing.set((aim - body.aim_position()).xz());
    }
    let (companion_position, source_position) =
        companion_pose(world, body, facing.direction(), actor.entity);
    Some(Plan {
        melee_stage,
        target,
        aim,
        companion_position,
        source_position,
        damage_scale: 1.0,
    })
}

/// Install the planned action, announce an ordinary attack's start and start
/// the slot's cooldown.
fn commit(
    slot: Slot,
    kind: ActionKind,
    plan: Plan,
    combat: &mut Combatant,
    actor: &Actor,
    ids: &mut ActionIds,
    feedback: &mut CombatFeedback,
) {
    let spec = kind.spec();
    let action_id = ids.next();
    let duration = plan.melee_stage.map_or(spec.duration, |stage| {
        combat.pair.melee_profile(stage).duration
    });
    let snapshot = ActionSnapshot {
        action_id,
        kind,
        pair: combat.pair,
        melee_stage: plan.melee_stage,
        target: plan.target,
        elapsed: 0.0,
        duration,
        shot_count: 0,
        aim_position: plan.aim,
        source_position: plan.source_position,
        companion_position: plan.companion_position,
    };
    combat.action = Some(ActionRuntime::new(snapshot, plan.damage_scale));
    if let Some(stage) = plan.melee_stage {
        combat.melee_chain.begin(action_id, stage);
        let position = actor.body.aim_position();
        feedback.action(
            actor.entity,
            actor.entity,
            &snapshot,
            0,
            CombatEffect::MeleeStart,
            position,
            position,
            0.0,
        );
    } else {
        combat.melee_chain.clear();
    }
    // The chain which pulls is the axe itself: it leaves the hand at once.
    if spec.pull().is_some() {
        combat.axe_away = true;
    }
    combat.availability = ActionAvailability::default();
    match slot {
        Slot::Attack => combat.attack_cooldown = duration,
        Slot::MeleeSkill => combat.melee_cooldown = spec.cooldown,
        Slot::RangedSkill => combat.ranged_cooldown = spec.cooldown,
        Slot::Combo => {}
    }
    combat.status = CombatStatus::Started(kind);
}

/// The planar velocity the motor should chase while a pull is in progress.
fn pull_velocity(combat: &Combatant, body: &Body, enemies: &Candidates, dt: f32) -> Option<Vec2> {
    let action = combat.action.as_ref()?;
    let pull = action.snapshot.kind.spec().pull()?;
    if action.cancelled || action.pull_stopped || action.snapshot.elapsed > pull.until {
        return None;
    }
    let (_, target, _, _, _, _) = enemies.get(action.snapshot.target?).ok()?;
    let delta = (target.world_position() - body.world_position()).xz();
    // Brake early enough for the motor's finite acceleration. A hard
    // full-speed-to-zero target at the stop distance would overshoot into the victim.
    let gap = (delta.length() - pull.stop_distance).max(0.0);
    let speed = pull
        .speed
        .min((2.0 * pull.acceleration * gap).sqrt() * 0.9)
        .min(gap / dt);
    Some(delta.normalize_or_zero() * speed)
}
