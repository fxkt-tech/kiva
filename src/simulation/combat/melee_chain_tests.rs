use super::*;
use crate::simulation::{MotionFeedback, MotionKind};

fn active(app: &App, player: Entity) -> Option<ActionSnapshot> {
    app.world()
        .get::<Combatant>(player)
        .unwrap()
        .active_action()
}
fn finish_action(app: &mut App, player: Entity) -> usize {
    let id = active(app, player).expect("committed action").action_id;
    for ticks in 1..=100 {
        tick(app, 1);
        if active(app, player).is_none_or(|action| action.action_id != id) {
            return ticks;
        }
    }
    panic!("action {id} did not retire");
}
fn ordinary_events(app: &App, player: Entity, kind: CombatEffect) -> Vec<CombatEvent> {
    app.world()
        .resource::<CombatFeedback>()
        .events()
        .filter(|event| event.actor == player && event.kind == kind)
        .copied()
        .collect()
}
fn buffered_first(app: &mut App, player: Entity) -> u64 {
    request(app, player, CombatIntent::request_attack);
    tick(app, 1);
    let id = active(app, player).unwrap().action_id;
    request(app, player, CombatIntent::request_attack);
    tick(app, 1);
    id
}

#[test]
fn three_stages_preserve_each_marker_damage_and_immutable_history_for_both_pairs() {
    for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
        let (mut app, player, enemy) = arena(pair, Vec3::NEG_Z * 1.2, 0.0);
        let (damage, duration_ticks) = match pair {
            WeaponPair::HammerMusket => (24.0, 36),
            WeaponPair::ChainAxeBow => (18.0, 24),
        };
        request(&mut app, player, CombatIntent::request_attack);
        tick(&mut app, 1);
        let mut ids = Vec::new();
        for (index, stage) in [MeleeStage::One, MeleeStage::Two, MeleeStage::Three]
            .into_iter()
            .enumerate()
        {
            let action = active(&app, player).unwrap();
            assert_eq!(action.melee_stage, Some(stage));
            assert_eq!(action.elapsed, 0.0);
            ids.push(action.action_id);
            // Many edges within one sample/tick may request only one successor.
            for _ in 0..30 {
                request(&mut app, player, CombatIntent::request_attack);
            }
            // The stage hurts in the very step its blade reaches the enemy,
            // once, however long the blade then stays in them. This step is
            // the stage's first: the blade's tick n is swept n steps later.
            let contact = contact_step(&app, player, enemy, stage) - 1;
            tick(&mut app, 1);
            tick(&mut app, contact - 2);
            assert_eq!(hp(&app, enemy), 1000.0 - damage * index as f32);
            tick(&mut app, 1);
            assert_eq!(hp(&app, enemy), 1000.0 - damage * (index + 1) as f32);
            let remaining_ticks = finish_action(&mut app, player);
            assert_eq!(contact + remaining_ticks, duration_ticks);
            assert_eq!(hp(&app, enemy), 1000.0 - damage * (index + 1) as f32);
            assert!(active(&app, player).is_none());
            assert_eq!(
                ordinary_events(&app, player, CombatEffect::MeleeStart).len(),
                index + 1
            );
            if stage != MeleeStage::Three {
                tick(&mut app, 1);
            }
        }
        tick(&mut app, 40);
        assert!(active(&app, player).is_none());
        let starts = ordinary_events(&app, player, CombatEffect::MeleeStart);
        let swings = ordinary_events(&app, player, CombatEffect::Melee);
        assert_eq!(starts.len(), 3);
        assert_eq!(swings.len(), 3);
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        for (index, (start, swing)) in starts.iter().zip(&swings).enumerate() {
            assert_eq!(start.action_id, ids[index]);
            assert_eq!(swing.action_id, start.action_id);
            assert_eq!(swing.melee_stage, start.melee_stage);
            assert_eq!(swing.pair, Some(pair));
            assert_eq!(swing.source_owner, player);
            assert_eq!(swing.hit_index, 0);
            let hit = app
                .world()
                .resource::<CombatFeedback>()
                .events()
                .find(|event| {
                    event.actor == enemy
                        && event.kind == CombatEffect::Hit
                        && event.action_id == start.action_id
                })
                .unwrap();
            assert_eq!(hit.melee_stage, start.melee_stage);
            assert_eq!(hit.pair, Some(pair));
            assert_eq!(hit.source_owner, player);
            assert_eq!(hit.hit_index, 0);
        }
    }
}

#[test]
fn single_click_does_not_auto_chain_and_grace_resets_after_expiry() {
    let (mut app, player, enemy) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    finish_action(&mut app, player);
    tick(&mut app, 10);
    assert_eq!(
        ordinary_events(&app, player, CombatEffect::MeleeStart).len(),
        1
    );
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::Two)
    );
    finish_action(&mut app, player);
    tick(&mut app, 28);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::One)
    );
    tick(&mut app, 120);
    assert_eq!(
        ordinary_events(&app, player, CombatEffect::MeleeStart).len(),
        3
    );
    assert_eq!(hp(&app, enemy), 928.0);
}

#[test]
fn hit_stop_freezes_the_uncommitted_grace_window() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    finish_action(&mut app, player);
    tick(&mut app, 24);
    app.world_mut().resource_mut::<HitStop>().request_sniper();
    tick(&mut app, 3);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::Two)
    );
}

#[test]
fn paused_clear_latches_cancellation_without_cancelling_the_committed_attack() {
    let (mut app, player, enemy) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    let id = buffered_first(&mut app, player);
    let elapsed = active(&app, player).unwrap().elapsed;
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    request(&mut app, player, CombatIntent::clear);
    tick(&mut app, 60);
    assert_eq!(active(&app, player).unwrap().elapsed, elapsed);
    app.world_mut().resource_mut::<SimulationActive>().0 = true;
    tick(&mut app, 1);
    assert_eq!(active(&app, player).unwrap().action_id, id);
    finish_action(&mut app, player);
    tick(&mut app, 10);
    assert!(active(&app, player).is_none());
    assert_eq!(hp(&app, enemy), 976.0);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::One)
    );
}

#[test]
fn explicit_cancel_survives_other_requests_until_the_rule_tick() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    buffered_first(&mut app, player);
    request(&mut app, player, CombatIntent::cancel_melee_chain);
    request(&mut app, player, CombatIntent::request_toggle_target);
    tick(&mut app, 1);
    finish_action(&mut app, player);
    tick(&mut app, 1);
    assert!(active(&app, player).is_none());
}

#[test]
fn new_valid_skill_pair_and_defense_win_over_a_buffered_successor() {
    for choice in 0..3 {
        let (mut app, player, enemy) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
        lock(&mut app, player, enemy);
        buffered_first(&mut app, player);
        finish_action(&mut app, player);
        match choice {
            0 => request(&mut app, player, CombatIntent::request_melee_skill),
            1 => request(&mut app, player, |intent| {
                intent.request_pair(WeaponPair::ChainAxeBow)
            }),
            _ => edge(&mut app, player, DefenseEdge::Pressed),
        }
        tick(&mut app, 1);
        assert_eq!(
            ordinary_events(&app, player, CombatEffect::MeleeStart).len(),
            1
        );
        match choice {
            0 => {
                assert_eq!(active(&app, player).unwrap().kind, ActionKind::HammerSlam);
                assert_eq!(active(&app, player).unwrap().melee_stage, None);
                finish_action(&mut app, player);
            }
            1 => assert_eq!(
                app.world().get::<Combatant>(player).unwrap().weapon_pair(),
                WeaponPair::ChainAxeBow
            ),
            _ => {
                assert!(
                    app.world()
                        .get::<DefenseState>(player)
                        .unwrap()
                        .is_deflecting()
                );
                edge(&mut app, player, DefenseEdge::Cancelled);
                tick(&mut app, 1);
            }
        }
        request(&mut app, player, CombatIntent::request_attack);
        tick(&mut app, 1);
        assert_eq!(
            active(&app, player).unwrap().melee_stage,
            Some(MeleeStage::One)
        );
    }
}

#[test]
fn accepted_jump_or_dodge_gets_the_motor_before_an_automatic_successor() {
    for dodge in [false, true] {
        let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
        buffered_first(&mut app, player);
        finish_action(&mut app, player);
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            if dodge {
                intent.set_shift_held(true);
                intent.request_dodge();
            } else {
                intent.request_jump();
            }
        }
        tick(&mut app, 1);
        assert!(active(&app, player).is_none());
        let motion = app.world().get::<MotionState>(player).unwrap();
        if dodge {
            assert!(motion.is_dodging());
            tick(&mut app, 16);
        } else {
            assert_eq!(motion.jumps_used(), 1);
        }
        request(&mut app, player, CombatIntent::request_attack);
        tick(&mut app, 1);
        assert_eq!(
            active(&app, player).unwrap().melee_stage,
            Some(MeleeStage::One)
        );
    }
}

#[test]
fn a_dodge_becoming_ready_this_tick_preempts_the_buffered_successor() {
    let (mut app, player, _) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * 10.0, 0.0);
    {
        let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
        intent.set_shift_held(true);
        intent.request_dodge();
    }
    tick(&mut app, 23);
    assert!(!app.world().get::<MotionState>(player).unwrap().is_dodging());
    // Offset the fixed clock by half a step so the old dodge's cooldown
    // expires in the motor on the first tick after this real attack retires.
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f64(1.0 / 120.0));
    tick(&mut app, 1);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f64(1.0 / 60.0));
    buffered_first(&mut app, player);
    finish_action(&mut app, player);
    let remaining = app
        .world()
        .get::<MotionState>(player)
        .unwrap()
        .dodge_cooldown();
    assert!(remaining > 0.0 && remaining < 1.0 / 60.0, "{remaining}");
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .request_dodge();
    tick(&mut app, 1);
    assert!(active(&app, player).is_none());
    assert!(app.world().get::<MotionState>(player).unwrap().is_dodging());
    assert_eq!(
        ordinary_events(&app, player, CombatEffect::MeleeStart).len(),
        1
    );
    assert_eq!(
        app.world()
            .resource::<MotionFeedback>()
            .events()
            .filter(|event| event.actor == player && event.kind == MotionKind::Dodge)
            .count(),
        2
    );
}

#[test]
fn ordinary_damage_clears_followups_but_cannot_revoke_the_current_marker() {
    let (mut app, player, enemy) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    let id = buffered_first(&mut app, player);
    strike(&mut app, player, enemy);
    assert_eq!(hp(&app, player), 988.0);
    assert_eq!(active(&app, player).unwrap().action_id, id);
    finish_action(&mut app, player);
    tick(&mut app, 1);
    assert!(active(&app, player).is_none());
    assert_eq!(hp(&app, enemy), 976.0);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::One)
    );
}

#[test]
fn mode_cleanup_and_death_cannot_revive_an_old_followup_after_respawn() {
    for died in [false, true] {
        let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
        buffered_first(&mut app, player);
        if died {
            app.world_mut()
                .get_mut::<Health>(player)
                .unwrap()
                .damage(1000.0);
            tick(&mut app, 185);
            assert_eq!(hp(&app, player), 1000.0);
        } else {
            app.world_mut()
                .get_mut::<Combatant>(player)
                .unwrap()
                .clear_targets();
            // Mode cleanup preserves the already started ordinary cooldown.
            assert!(
                app.world()
                    .get::<Combatant>(player)
                    .unwrap()
                    .attack_cooldown()
                    > 0.0
            );
            tick(&mut app, 40);
        }
        assert!(active(&app, player).is_none());
        request(&mut app, player, CombatIntent::request_attack);
        tick(&mut app, 1);
        assert_eq!(
            active(&app, player).unwrap().melee_stage,
            Some(MeleeStage::One)
        );
    }
}

#[test]
fn a_whiff_keeps_stage_markers_without_synthesizing_impacts() {
    for blocked in [false, true] {
        let (mut app, player, enemy) = arena(
            WeaponPair::ChainAxeBow,
            Vec3::NEG_Z * if blocked { 2.0 } else { 10.0 },
            0.0,
        );
        if blocked {
            spawn_box(
                app.world_mut(),
                Vec3::new(0.0, 0.9, -1.0),
                Vec3::new(3.0, 1.8, 0.2),
                false,
            );
        }
        for _ in 0..3 {
            request(&mut app, player, CombatIntent::request_attack);
            tick(&mut app, 1);
            finish_action(&mut app, player);
        }
        assert_eq!(hp(&app, enemy), 1000.0);
        assert_eq!(ordinary_events(&app, player, CombatEffect::Melee).len(), 3);
        assert!(
            !app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| event.kind == CombatEffect::Hit)
        );
    }
}

#[test]
fn a_valid_slide_cancels_the_pending_attack_and_does_not_lock_out_the_crouch_edge() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 10.0, 0.0);
    app.world_mut()
        .get_mut::<MoveIntent>(player)
        .unwrap()
        .set(Vec2::X);
    {
        let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
        intent.set_movement_held(true);
        intent.set_shift_held(true);
        intent.request_dodge();
    }
    tick(&mut app, 18);
    assert!(
        app.world()
            .get::<MotionState>(player)
            .unwrap()
            .is_sprinting()
    );
    buffered_first(&mut app, player);
    finish_action(&mut app, player);
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 1);
    assert!(active(&app, player).is_none());
    assert!(app.world().get::<MotionState>(player).unwrap().is_sliding());
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 1);
    request(&mut app, player, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::One)
    );
}

#[test]
fn ordinary_crouch_applies_before_a_buffered_attack_without_losing_its_stage() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 10.0, 0.0);
    buffered_first(&mut app, player);
    finish_action(&mut app, player);
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 1);
    assert!(active(&app, player).is_none());
    assert!(
        app.world()
            .get::<MotionState>(player)
            .unwrap()
            .is_crouched()
    );
    tick(&mut app, 1);
    assert_eq!(
        active(&app, player).unwrap().melee_stage,
        Some(MeleeStage::Two)
    );
    assert!(
        app.world()
            .get::<MotionState>(player)
            .unwrap()
            .is_crouched()
    );
}

#[test]
fn defense_snapshot_survives_an_entire_missed_parry_window_and_cancel_invalidates_it() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 8.0, 0.0);
    edge(&mut app, player, DefenseEdge::Pressed);
    tick(&mut app, 14);
    let pressed = app.world().get::<DefenseState>(player).unwrap().snapshot();
    assert_eq!(pressed.kind, DefenseKind::Guard);
    assert_eq!(pressed.last_gesture_kind, DefenseKind::Deflect);
    edge(&mut app, player, DefenseEdge::Released);
    tick(&mut app, 1);
    let parry = app.world().get::<DefenseState>(player).unwrap().snapshot();
    assert_eq!(parry.kind, DefenseKind::Parry);
    assert!(parry.gesture_id > pressed.gesture_id);
    assert_eq!(parry.gesture_elapsed, 0.0);
    tick(&mut app, 13);
    let after = app.world().get::<DefenseState>(player).unwrap().snapshot();
    assert_eq!(after.kind, DefenseKind::Idle);
    assert_eq!(after.last_gesture_kind, DefenseKind::Parry);
    assert_eq!(after.gesture_id, parry.gesture_id);
    assert!(after.gesture_elapsed > 0.18 && after.gesture_elapsed < 0.6);
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    tick(&mut app, 50);
    assert_eq!(
        app.world().get::<DefenseState>(player).unwrap().snapshot(),
        after
    );
    app.world_mut().resource_mut::<SimulationActive>().0 = true;
    edge(&mut app, player, DefenseEdge::Cancelled);
    tick(&mut app, 1);
    let cancelled = app.world().get::<DefenseState>(player).unwrap().snapshot();
    assert_eq!(cancelled.kind, DefenseKind::Idle);
    assert_eq!(cancelled.last_gesture_kind, DefenseKind::Idle);
    assert_eq!(cancelled.gesture_id, after.gesture_id);
    assert!(
        !app.world()
            .resource::<CombatFeedback>()
            .events()
            .any(|event| event.kind == CombatEffect::Parry)
    );
}

#[test]
fn motion_feedback_uses_accepted_jumps_and_one_real_landing_not_requested_third_jump() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 10.0, 0.0);
    for _ in 0..3 {
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .request_jump();
        tick(&mut app, 1);
    }
    let jumps: Vec<_> = app
        .world()
        .resource::<MotionFeedback>()
        .events()
        .filter(|event| event.actor == player && event.kind == MotionKind::Jump)
        .map(|event| event.jump_count)
        .collect();
    assert_eq!(jumps, [1, 2]);
    tick(&mut app, 100);
    let events: Vec<_> = app
        .world()
        .resource::<MotionFeedback>()
        .events()
        .copied()
        .collect();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == MotionKind::Land)
            .count(),
        1
    );
    assert!(
        events
            .iter()
            .all(|event| event.actor == player && event.position.is_finite())
    );
    assert!(
        events
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence)
    );
}

#[test]
fn actual_stride_feedback_stops_at_idle_and_excludes_teleports_and_blocked_input() {
    let (mut app, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 10.0, 0.0);
    app.world_mut()
        .get_mut::<MoveIntent>(player)
        .unwrap()
        .set(Vec2::X);
    tick(&mut app, 50);
    let count = app.world().resource::<MotionFeedback>().events().count();
    assert!(
        app.world()
            .resource::<MotionFeedback>()
            .events()
            .any(|event| event.kind == MotionKind::Footstep)
    );
    app.world_mut()
        .get_mut::<MoveIntent>(player)
        .unwrap()
        .stop();
    tick(&mut app, 20);
    assert_eq!(
        app.world().resource::<MotionFeedback>().events().count(),
        count
    );
    app.world_mut()
        .get_mut::<MoveIntent>(player)
        .unwrap()
        .set(Vec2::X);
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .request_jump();
    tick(&mut app, 8);
    assert!(
        !app.world()
            .get::<MotionState>(player)
            .unwrap()
            .is_grounded()
    );
    let count = app.world().resource::<MotionFeedback>().events().count();
    teleport_actor(app.world_mut(), player, Vec3::X * 15.0);
    tick(&mut app, 2);
    assert_eq!(
        app.world().resource::<MotionFeedback>().events().count(),
        count
    );

    let (mut blocked, player, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 10.0, 0.0);
    spawn_box(
        blocked.world_mut(),
        Vec3::new(1.0, 0.9, 0.0),
        Vec3::new(0.3, 1.8, 4.0),
        false,
    );
    blocked
        .world_mut()
        .get_mut::<MoveIntent>(player)
        .unwrap()
        .set(Vec2::X);
    tick(&mut blocked, 120);
    assert!(
        !blocked
            .world()
            .resource::<MotionFeedback>()
            .events()
            .any(|event| event.kind == MotionKind::Footstep)
    );
}
