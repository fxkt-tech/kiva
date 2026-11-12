//! After physics: the player's defeat and respawn, then the committed action's
//! markers. Each skill shape has one resolver which reads its numbers from the
//! skill table; every marker settles exactly once.
use super::hit::{
    Blow, CombatWorld, Enemies, Players, defeat_if_fallen, hit_enemy, target_lost, target_valid,
    valid_shot,
};
use super::runtime::ActionRuntime;
use super::skills::{Area, Centre, Control, MELEE_ARC, Shape, Shot, Volley};
use super::{CombatEffect, CombatFeedback, Combatant, Health};
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
}

pub(in crate::simulation) fn resolve_actions(
    time: Res<Time<Fixed>>,
    world: CombatWorld,
    mut feedback: ResMut<CombatFeedback>,
    mut hit_stop: ResMut<HitStop>,
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
        };
        let spec = runtime.snapshot.kind.spec();
        match spec.shape {
            Shape::Arc => strike_arc(&striker, &mut runtime, &mut stage),
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

/// The ordinary attack: one frontal arc with the weapon stage's own numbers.
fn strike_arc(striker: &Striker, runtime: &mut ActionRuntime, stage: &mut Stage) {
    let snapshot = runtime.snapshot;
    let profile = snapshot
        .pair
        .melee_profile(snapshot.melee_stage.expect("melee has a committed stage"));
    if !runtime.marker(0, profile.hit_time) {
        return;
    }
    let (owner, body, forward) = (striker.owner, striker.body, striker.forward);
    let range = profile.range;
    stage.feedback.action(
        owner,
        owner,
        &snapshot,
        0,
        CombatEffect::Melee,
        body.aim_position(),
        body.aim_position() + forward * range,
        range,
    );
    let targets: Vec<_> = stage
        .enemies
        .iter()
        .filter(|(_, b, _, _, _, _, h, _, _, boss, _)| {
            target_valid(h, boss.as_deref())
                && b.world_position().distance(body.world_position()) <= range
                && (b.world_position() - body.world_position())
                    .xz()
                    .normalize_or_zero()
                    .dot(forward.xz())
                    > MELEE_ARC
        })
        .filter(|(t, b, _, _, _, _, _, _, _, _, _)| {
            stage
                .world
                .visible(body.aim_position(), b.aim_position(), owner, *t)
        })
        .map(|(e, ..)| e)
        .collect();
    for target in targets {
        hit_enemy(
            Blow {
                target,
                source: owner,
                index: 0,
                damage: profile.damage,
                control: Control::None,
                origin: body.aim_position(),
            },
            owner,
            &snapshot,
            stage.enemies,
            stage.feedback,
        );
    }
}

/// One marker hits every visible enemy around the actor or the locked target.
/// Controlling the original target opens the skill's combo window.
fn strike_area(
    area: &Area,
    cast_range: f32,
    striker: &Striker,
    runtime: &mut ActionRuntime,
    combat: &mut Combatant,
    stage: &mut Stage,
) {
    let snapshot = runtime.snapshot;
    if !runtime.marker(0, area.time) {
        return;
    }
    let (owner, body) = (striker.owner, striker.body);
    // `centre` measures the radius from feet; `shown` is where the effect appears.
    let (centre, shown) = match area.centre {
        Centre::Actor => (body.world_position(), body.aim_position()),
        Centre::Target => {
            let Some(target) = snapshot.target else {
                return;
            };
            let source = body.aim_position();
            if valid_shot(
                stage.world,
                source,
                (owner, body),
                target,
                cast_range,
                stage.enemies,
            )
            .is_none()
            {
                return;
            }
            let centre = stage.enemies.get_mut(target).unwrap().1.world_position();
            (centre, centre)
        }
    };
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
    let targets: Vec<_> = stage
        .enemies
        .iter()
        .filter(|(_, b, _, _, _, _, h, _, _, boss, _)| {
            target_valid(h, boss.as_deref()) && b.world_position().distance(centre) <= area.radius
        })
        .filter(|(t, b, _, _, _, _, _, _, _, _, _)| {
            stage
                .world
                .visible(body.aim_position(), b.aim_position(), owner, *t)
        })
        .map(|(e, ..)| e)
        .collect();
    for target in targets {
        let controlled = hit_enemy(
            Blow {
                target,
                source: owner,
                index: 0,
                damage: area.damage,
                control: area.control,
                origin: body.aim_position(),
            },
            owner,
            &snapshot,
            stage.enemies,
            stage.feedback,
        );
        if controlled
            && snapshot.target == Some(target)
            && let Some(remaining) = area.window
        {
            combat.combo = Some(window(&snapshot, target, remaining));
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
    let stunned = hit_enemy(
        Blow {
            target,
            source: performer,
            index: 0,
            damage: shot.damage,
            control: Control::Stun(shot.stun),
            origin: source,
        },
        owner,
        &snapshot,
        stage.enemies,
        stage.feedback,
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
        let alive = hit_enemy(
            Blow {
                target,
                source: performer,
                index: index as u8,
                damage: volley.damage,
                control: Control::None,
                origin: source,
            },
            owner,
            &snapshot,
            stage.enemies,
            stage.feedback,
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
    for (_, _, _, _, _, _, _, mut combat, _, _, _, _) in players {
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
