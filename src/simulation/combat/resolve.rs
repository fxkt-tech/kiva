//! After physics: the player's defeat and respawn, then the committed action's
//! blade sweeps and markers. Each skill shape has one resolver which reads its
//! numbers from the skill table and its blade path from the baked hit tracks;
//! every marker settles exactly once and every enemy is struck once per action.
use super::hit::{
    Blow, CombatWorld, Enemies, Landed, Players, defeat_if_fallen, hit_enemy, target_lost,
    target_valid, valid_shot,
};
use super::runtime::ActionRuntime;
use super::skills::{Area, Centre, Control, Shape, Shot, Volley};
use super::sweep::{HitTrack, strike, ticks_between};
use super::{CombatEffect, CombatFeedback, Combatant, Health, HitProbes};
use crate::simulation::{ActionKind, ActionSnapshot, Body, ComboWindow, HitStop, Stamina};
use bevy::prelude::*;

/// Presentation radii of a fired shot and of the thrown chain axe.
const SHOT_RADIUS: f32 = 0.22;
const AXE_RADIUS: f32 = 0.3;

/// Who acts in this tick. `performer` is the companion for ranged skills.
struct Striker<'a> {
    owner: Entity,
    performer: Entity,
    body: &'a Body,
    forward: Vec3,
}

/// The shared surroundings every resolver queries and reports to.
struct Stage<'a, 'w, 's, 'world_w, 'world_s> {
    world: &'a CombatWorld<'world_w, 'world_s>,
    enemies: &'a mut Enemies<'w, 's>,
    feedback: &'a mut CombatFeedback,
    probes: &'a mut HitProbes,
    /// Blows which dealt damage in this tick; each one feeds the sun meter.
    damaging_hits: u8,
}

pub(in crate::simulation) fn resolve_actions(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut hit_stop: ResMut<HitStop>,
    mut probes: ResMut<HitProbes>,
    mut players: Players,
    mut enemies: Enemies,
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
        (mut energy, mut arsenal),
    ) in &mut players
    {
        defeat_if_fallen(entity, &body, &mut health, &mut feedback);
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
            energy.reset();
            arsenal.refill();
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
        let direction = facing.direction();
        let striker = Striker {
            owner: entity,
            performer: combat.companion.unwrap_or(entity),
            body: &body,
            forward: Vec3::new(direction.x, 0.0, direction.y),
        };
        let mut stage = Stage {
            world: &world,
            enemies: &mut enemies,
            feedback: &mut feedback,
            probes: &mut probes,
            damaging_hits: 0,
        };
        let spec = runtime.snapshot.kind.spec();
        match spec.shape {
            Shape::Blade => strike_blade(&striker, &mut runtime, &mut stage),
            Shape::Area(area) => {
                let range = spec.cast_range.unwrap_or(f32::INFINITY);
                strike_area(
                    &area,
                    range,
                    &striker,
                    &mut runtime,
                    &mut combat,
                    &mut stage,
                );
            }
            Shape::Shot(shot) => {
                let range = spec.cast_range.unwrap_or(f32::INFINITY);
                fire_shot(
                    &shot,
                    range,
                    &striker,
                    &mut runtime,
                    &mut combat,
                    &mut stage,
                );
            }
            Shape::Volley(volley) => {
                if fire_volley(&volley, &striker, &mut runtime, &mut combat, &mut stage) {
                    hit_stop.request_sniper();
                }
            }
        }
        energy.gain_sun(stage.damaging_hits);
        if !runtime.cancelled && runtime.snapshot.elapsed + 1e-6 >= runtime.snapshot.duration {
            combat.axe_away = false;
            if runtime.snapshot.kind == ActionKind::Melee {
                combat.melee_chain.finish(runtime.snapshot.action_id);
            }
        } else {
            combat.action = Some(runtime);
        }
    }
    forget_lost_targets(&mut players, &mut enemies);
}

/// The action's baked blade path; every blade-swinging action has one.
fn blade_track(snapshot: &ActionSnapshot) -> &'static HitTrack {
    let track = snapshot.clock().track;
    track
        .hit_track()
        .unwrap_or_else(|| panic!("{track:?} has no baked blade path"))
}

/// Every valid enemy the moving blade touches in this step which the action
/// has not hit yet, with the point of contact. `started` tells whether the
/// blade's first dangerous sweep is in this step.
fn blade_contacts(
    striker: &Striker,
    runtime: &ActionRuntime,
    stage: &mut Stage,
) -> (bool, Vec<(Entity, Vec3)>) {
    let track = blade_track(&runtime.snapshot);
    let mut started = false;
    let mut contacts: Vec<(Entity, Vec3)> = Vec::new();
    for tick in ticks_between(runtime.previous, runtime.snapshot.elapsed) {
        started |= tick == track.first_active();
        if !track.is_active(tick) {
            continue;
        }
        for (target, body, motion, _, _, _, health, _, _, boss, _) in stage.enemies.iter() {
            if !target_valid(health, boss)
                || runtime.struck.contains(&target)
                || contacts.iter().any(|(hit, _)| *hit == target)
            {
                continue;
            }
            let contact = strike(
                track,
                tick,
                (striker.owner, striker.body),
                striker.forward.xz(),
                (target, body, motion),
                stage.world,
                stage.probes,
            );
            if let Some(contact) = contact {
                contacts.push((target, contact));
            }
        }
    }
    (started, contacts)
}

/// The ordinary attack: the blade hits each enemy it touches once, with the
/// weapon stage's damage.
fn strike_blade(striker: &Striker, runtime: &mut ActionRuntime, stage: &mut Stage) {
    let snapshot = runtime.snapshot;
    let profile = snapshot
        .pair
        .melee_profile(snapshot.melee_stage.expect("melee has a committed stage"));
    let (owner, body) = (striker.owner, striker.body);
    let (started, contacts) = blade_contacts(striker, runtime, stage);
    if started {
        let reach = blade_track(&snapshot).reach;
        stage.feedback.action(
            owner,
            owner,
            &snapshot,
            0,
            CombatEffect::Melee,
            body.aim_position(),
            body.aim_position() + striker.forward * reach,
            reach,
        );
    }
    for (target, contact) in contacts {
        runtime.struck.push(target);
        land(
            Blow {
                target,
                source: owner,
                index: 0,
                damage: profile.damage * runtime.damage_scale,
                control: Control::None,
                origin: body.aim_position(),
                contact: Some(contact),
            },
            owner,
            &snapshot,
            stage,
        );
    }
}

/// The blade hits the enemies it touches; once its first swing has landed,
/// the shockwave hits every other visible enemy around the actor or the
/// locked target. Each enemy is hit once. Controlling the original target
/// opens the skill's combo window.
fn strike_area(
    area: &Area,
    cast_range: f32,
    striker: &Striker,
    runtime: &mut ActionRuntime,
    combat: &mut Combatant,
    stage: &mut Stage,
) {
    let snapshot = runtime.snapshot;
    let (owner, body) = (striker.owner, striker.body);
    let (_, contacts) = blade_contacts(striker, runtime, stage);
    let mut hits: Vec<_> = contacts
        .into_iter()
        .map(|(target, contact)| (target, Some(contact)))
        .collect();
    if runtime.marker(0, blade_track(&snapshot).impact_time())
        && let Some((centre, shown)) = area_centre(area, cast_range, striker, &snapshot, stage)
    {
        stage.feedback.action(
            owner,
            owner,
            &snapshot,
            0,
            area.effect,
            shown,
            shown,
            area.radius,
        );
        let reached: Vec<_> = stage
            .enemies
            .iter()
            .filter(|(target, b, _, _, _, _, h, _, _, boss, _)| {
                target_valid(h, boss.as_deref())
                    && b.world_position().distance(centre) <= area.radius
                    && !runtime.struck.contains(target)
                    && !hits.iter().any(|(hit, _)| hit == target)
            })
            .filter(|(t, b, _, _, _, _, _, _, _, _, _)| {
                stage
                    .world
                    .visible(body.aim_position(), b.aim_position(), owner, *t)
            })
            .map(|(target, ..)| (target, None))
            .collect();
        hits.extend(reached);
    }
    for (target, contact) in hits {
        runtime.struck.push(target);
        let controlled = land(
            Blow {
                target,
                source: owner,
                index: 0,
                damage: area.damage * runtime.damage_scale,
                control: area.control,
                origin: body.aim_position(),
                contact,
            },
            owner,
            &snapshot,
            stage,
        );
        if controlled
            && snapshot.target == Some(target)
            && let Some(remaining) = area.window
        {
            combat.combo = Some(window(&snapshot, target, remaining));
        }
    }
}

/// Where the shockwave is measured from (at the feet) and where it is shown.
/// `None` when it is centred on a target which can no longer be reached.
fn area_centre(
    area: &Area,
    cast_range: f32,
    striker: &Striker,
    snapshot: &ActionSnapshot,
    stage: &mut Stage,
) -> Option<(Vec3, Vec3)> {
    let (owner, body) = (striker.owner, striker.body);
    match area.centre {
        Centre::Actor => Some((body.world_position(), body.aim_position())),
        Centre::Target => {
            let target = snapshot.target?;
            valid_shot(
                stage.world,
                body.aim_position(),
                (owner, body),
                target,
                cast_range,
                stage.enemies,
            )?;
            let centre = stage.enemies.get_mut(target).ok()?.1.world_position();
            Some((centre, centre))
        }
    }
}

/// The companion's single shot. A thrown chain axe first leaves the hand and
/// lands later; stunning the target can open the skill's combo window.
fn fire_shot(
    shot: &Shot,
    range: f32,
    striker: &Striker,
    runtime: &mut ActionRuntime,
    combat: &mut Combatant,
    stage: &mut Stage,
) {
    let snapshot = runtime.snapshot;
    let (owner, performer) = (striker.owner, striker.performer);
    let source = snapshot.source_position;
    if let Some(depart) = shot.depart
        && runtime.marker(0, depart)
    {
        combat.axe_away = true;
        runtime.snapshot.shot_count += 1;
        for (kind, radius) in [
            (CombatEffect::Shot, SHOT_RADIUS),
            (CombatEffect::AxeThrow, AXE_RADIUS),
        ] {
            stage.feedback.action(
                performer,
                owner,
                &snapshot,
                0,
                kind,
                source,
                snapshot.aim_position,
                radius,
            );
        }
    }
    if !runtime.marker(1, shot.impact) {
        return;
    }
    // A thrown axe already announced its departure; this marker only lands it.
    if shot.depart.is_none() {
        stage.feedback.action(
            performer,
            owner,
            &snapshot,
            0,
            CombatEffect::Shot,
            source,
            snapshot.aim_position,
            SHOT_RADIUS,
        );
        runtime.snapshot.shot_count += 1;
    }
    let Some(target) = snapshot.target else {
        return;
    };
    let Some(aim) = valid_shot(
        stage.world,
        source,
        (owner, striker.body),
        target,
        range,
        stage.enemies,
    ) else {
        return;
    };
    runtime.snapshot.aim_position = aim;
    let stunned = land(
        Blow {
            target,
            source: performer,
            index: 0,
            damage: shot.damage * runtime.damage_scale,
            control: Control::Stun(shot.stun),
            origin: source,
            contact: None,
        },
        owner,
        &snapshot,
        stage,
    );
    if stunned && let Some(remaining) = shot.window {
        combat.combo = Some(window(&snapshot, target, remaining));
    }
}

/// The companion's three shots at the original target. Each re-checks the
/// target, range and line of sight; losing or defeating the target cancels
/// the rest. Returns whether the final sniper shot landed and wants a hit stop.
fn fire_volley(
    volley: &Volley,
    striker: &Striker,
    runtime: &mut ActionRuntime,
    combat: &mut Combatant,
    stage: &mut Stage,
) -> bool {
    let snapshot = runtime.snapshot;
    let (owner, performer) = (striker.owner, striker.performer);
    let source = snapshot.source_position;
    let last = volley.times.len() - 1;
    let mut hit_stop = false;
    for (index, time) in volley.times.into_iter().enumerate() {
        if !runtime.marker(index as u8, time) {
            continue;
        }
        let target = snapshot.target.unwrap();
        if target_lost(target, stage.enemies) {
            runtime.cancel();
            combat.axe_away = false;
            break;
        }
        runtime.snapshot.shot_count += 1;
        let aim = valid_shot(
            stage.world,
            source,
            (owner, striker.body),
            target,
            volley.range,
            stage.enemies,
        );
        stage.feedback.action(
            performer,
            owner,
            &snapshot,
            index as u8,
            CombatEffect::Shot,
            source,
            aim.unwrap_or(runtime.snapshot.aim_position),
            SHOT_RADIUS,
        );
        let Some(aim) = aim else {
            continue;
        };
        runtime.snapshot.aim_position = aim;
        let alive = land(
            Blow {
                target,
                source: performer,
                index: index as u8,
                damage: volley.damage * runtime.damage_scale,
                control: Control::None,
                origin: source,
                contact: None,
            },
            owner,
            &snapshot,
            stage,
        );
        if volley.sniper {
            for (actor, kind, from, to) in [
                (performer, CombatEffect::SniperShot, source, aim),
                (target, CombatEffect::SniperImpact, aim, source),
            ] {
                stage
                    .feedback
                    .action(actor, owner, &snapshot, index as u8, kind, from, to, 0.0);
            }
            hit_stop |= index == last;
        }
        if !alive {
            runtime.cancel();
            combat.axe_away = false;
            break;
        }
    }
    hit_stop
}

/// Land one blow and count it when it dealt damage. Returns whether the
/// target took it and is still standing.
fn land(blow: Blow, owner: Entity, action: &ActionSnapshot, stage: &mut Stage) -> bool {
    let landed = hit_enemy(blow, owner, action, stage.enemies, stage.feedback);
    if landed != Landed::Missed {
        stage.damaging_hits += 1;
    }
    landed == Landed::Survived
}

fn window(action: &ActionSnapshot, target: Entity, remaining: f32) -> ComboWindow {
    ComboWindow {
        source_action_id: action.action_id,
        pair: action.pair,
        original_target: target,
        remaining,
    }
}

/// Remove a killed selected target in the damage tick, before a hit stop
/// freezes anything.
fn forget_lost_targets(players: &mut Players, enemies: &mut Enemies) {
    for (_, _, _, _, _, _, _, mut combat, _, _, _, _, _) in players {
        if combat.target.is_some_and(|t| target_lost(t, enemies)) {
            combat.target = None;
        }
        if combat
            .combo
            .is_some_and(|c| target_lost(c.original_target, enemies))
        {
            combat.combo = None;
        }
    }
}

/// Stamina recovers only while the player is alive and not attacking.
pub(in crate::simulation) fn recover_stamina(
    time: Res<Time<Fixed>>,
    mut players: Query<(&Combatant, &Health, &mut Stamina)>,
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
}
