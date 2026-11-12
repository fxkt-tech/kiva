use super::*;
use crate::simulation::*;
use bevy::prelude::*;
use std::time::Duration;
#[path = "energy_tests.rs"]
mod energy_tests;
#[path = "melee_chain_tests.rs"]
mod melee_chain_tests;
fn tick(app: &mut App, n: usize) {
    for _ in 0..n {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
fn arena(pair: WeaponPair, position: Vec3, elevation: f32) -> (App, Entity, Entity) {
    arena_with_target_scale(pair, position, elevation, 1.0)
}
fn arena_with_target_scale(
    pair: WeaponPair,
    position: Vec3,
    elevation: f32,
    scale: f32,
) -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin))
        .insert_resource(CollisionWorld::new(50.0, vec![]));
    app.finish();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f64(1.0 / 60.0));
    if elevation > 0.0 {
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, elevation - 0.5, 0.0),
            Vec3::new(60.0, 1.0, 60.0),
            false,
        );
    }
    let p = app
        .world_mut()
        .spawn((
            Body::at(Vec3::Y * elevation, 6.0, 0.35),
            MoveIntent::default(),
            MotionState::default(),
            ActionIntent::default(),
            Facing::default(),
            Health::new(1000.0),
            SpawnPoint(Vec2::ZERO),
            Combatant::new(pair),
            CombatIntent::default(),
        ))
        .id();
    let e = dummy_with_scale(&mut app, position + Vec3::Y * elevation, scale);
    tick(&mut app, 12);
    (app, p, e)
}
fn dummy(app: &mut App, position: Vec3) -> Entity {
    dummy_with_scale(app, position, 1.0)
}
fn dummy_with_scale(app: &mut App, position: Vec3, scale: f32) -> Entity {
    app.world_mut()
        .spawn((
            Body::at(position, 2.0, 0.3).scaled(scale),
            MoveIntent::default(),
            MotionState::default(),
            ActionIntent::default(),
            Facing::default(),
            Health::new(1000.0),
            SpawnPoint(position.xz()),
            Enemy::target_dummy(),
        ))
        .id()
}
fn hp(app: &App, e: Entity) -> f32 {
    app.world().get::<Health>(e).unwrap().current()
}
fn request(app: &mut App, p: Entity, f: impl FnOnce(&mut CombatIntent)) {
    f(&mut app.world_mut().get_mut::<CombatIntent>(p).unwrap());
}
fn edge(app: &mut App, p: Entity, e: DefenseEdge) {
    app.world_mut()
        .get_mut::<DefenseState>(p)
        .unwrap()
        .push_edge(e);
}
fn lock(app: &mut App, p: Entity, e: Entity) {
    request(app, p, CombatIntent::request_toggle_target);
    tick(app, 1);
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), Some(e));
}
fn hammer(app: &mut App, p: Entity, e: Entity) {
    lock(app, p, e);
    request(app, p, CombatIntent::request_melee_skill);
    tick(app, 13);
    assert_eq!(hp(app, e), 980.0);
    assert!(app.world().get::<Combatant>(p).unwrap().combo_remaining() > 0.0);
}
fn start_combo(app: &mut App, p: Entity) {
    request(app, p, CombatIntent::request_combo);
    tick(app, 1);
    assert!(app.world().get::<Combatant>(p).unwrap().is_combo_active());
}
fn shots(app: &App, p: Entity) -> Vec<CombatEvent> {
    app.world()
        .resource::<CombatFeedback>()
        .events()
        .filter(|e| e.source_owner == p && e.kind == CombatEffect::Shot)
        .copied()
        .collect()
}
/// One incoming hit: the enemy turns to the player and its blade passes
/// through them in this very step. The swing is started at the tick whose
/// sweep reaches the player, so defense timing stays exact to the step.
fn strike(app: &mut App, p: Entity, e: Entity) {
    use super::sweep::{HitTrack, Hurtbox, Stance, sweep};
    let track = crate::simulation::TrackId::Enemy.hit_track().unwrap();
    let world = app.world();
    let (attacker, victim) = (world.get::<Body>(e).unwrap(), world.get::<Body>(p).unwrap());
    let toward = (victim.world_position() - attacker.world_position()).xz();
    let mut stance = Stance::of(attacker, toward);
    stance.previous_feet = stance.feet;
    let hurtbox = Hurtbox::of(victim, world.get::<MotionState>(p).unwrap());
    let reaches = (track.first_active()..=track.last_active())
        .find(|&tick| {
            track.is_active(tick) && sweep(track, tick, &stance, &hurtbox, |_| {}).is_some()
        })
        .expect("the enemy's blade reaches the player");
    app.world_mut().get_mut::<Facing>(e).unwrap().set(toward);
    let mut enemy = app.world_mut().get_mut::<Enemy>(e).unwrap();
    enemy.passive = false;
    enemy.recovery = 0.0;
    enemy.attack_elapsed = Some(HitTrack::time(reaches - 1));
    enemy.struck.clear();
    enemy.attack_id += 1;
    let events = |app: &App| {
        app.world()
            .resource::<CombatFeedback>()
            .events()
            .last()
            .map_or(0, |event| event.sequence)
    };
    let before = events(app);
    tick(app, 1);
    // Whatever the defense made of it, the swing reached the player.
    assert!(
        app.world()
            .resource::<CombatFeedback>()
            .events()
            .any(|event| event.sequence > before && event.actor == p),
        "the swing touched the player"
    );
    let mut enemy = app.world_mut().get_mut::<Enemy>(e).unwrap();
    if enemy.stun_source.is_none() {
        enemy.attack_elapsed = None;
    }
    enemy.struck.clear();
    enemy.passive = true;
}
/// How many steps after its press the player's ordinary attack `stage` first
/// touches `target`, as both stand now: the press is admitted in the first
/// step, and the blade's tick `n` is swept in step `n + 1`.
fn contact_step(app: &App, attacker: Entity, target: Entity, stage: MeleeStage) -> usize {
    use super::sweep::{Hurtbox, Stance, sweep};
    let world = app.world();
    let pair = world.get::<Combatant>(attacker).unwrap().weapon_pair();
    let track = crate::simulation::TrackId::PlayerMelee { pair, stage }
        .hit_track()
        .unwrap();
    let body = world.get::<Body>(attacker).unwrap();
    let mut stance = Stance::of(body, world.get::<Facing>(attacker).unwrap().direction());
    stance.previous_feet = stance.feet;
    let hurtbox = Hurtbox::of(
        world.get::<Body>(target).unwrap(),
        world.get::<MotionState>(target).unwrap(),
    );
    let tick = (track.first_active()..=track.last_active())
        .find(|&tick| {
            track.is_active(tick) && sweep(track, tick, &stance, &hurtbox, |_| {}).is_some()
        })
        .expect("the player's blade reaches the target");
    usize::from(tick) + 1
}
#[test]
fn lock_is_explicit_toggle_parity_and_rejected_skills_cost_no_cooldown() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    tick(&mut app, 20);
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), None);
    request(&mut app, p, CombatIntent::request_melee_skill);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 1);
    let c = app.world().get::<Combatant>(p).unwrap();
    assert_eq!(c.melee_cooldown(), 0.0);
    assert_eq!(c.ranged_cooldown(), 0.0);
    request(&mut app, p, |i| {
        i.request_toggle_target();
        i.request_toggle_target();
    });
    tick(&mut app, 1);
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), None);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_toggle_target);
    tick(&mut app, 60);
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), None);
}
#[test]
fn melee_has_a_real_windup_and_uses_facing_without_a_lock() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.6, 0.0);
    let back = dummy(&mut app, Vec3::Z * 1.6);
    tick(&mut app, 2);
    // Nothing is hurt until the blade arrives; the enemy behind never is.
    let contact = contact_step(&app, p, e, MeleeStage::One);
    assert!(contact > 3, "the attack hit without a windup");
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, contact - 1);
    assert_eq!(hp(&app, e), 1000.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 976.0);
    assert_eq!(hp(&app, back), 1000.0);
    tick(&mut app, 40);
    assert_eq!(hp(&app, e), 976.0);
}
#[test]
fn the_blade_hits_only_what_it_reaches_and_never_through_a_wall() {
    // Straight ahead, but beyond the blade: the old 2.8 m cone reached this.
    let (mut app, p, far) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 2.4, 0.0);
    // Within the swing to the right front, well off the facing axis.
    let side = dummy(&mut app, Vec3::new(1.0, 0.0, -0.9));
    tick(&mut app, 2);
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 45);
    assert_eq!(hp(&app, far), 1000.0);
    assert_eq!(hp(&app, side), 976.0);
    let hit = app
        .world()
        .resource::<CombatFeedback>()
        .events()
        .find(|event| event.kind == CombatEffect::Hit && event.actor == side)
        .copied()
        .unwrap();
    // The event marks where the blade met the body, on its surface.
    let body = app.world().get::<Body>(side).unwrap();
    let planar = hit.position.xz().distance(body.world_position().xz());
    assert!(
        (planar - body.radius()).abs() < 0.05,
        "contact {planar} m from the axis"
    );

    // A thin wall between the player and an enemy in easy reach stops the blade.
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    spawn_box(
        app.world_mut(),
        Vec3::new(0.0, 1.0, -0.6),
        Vec3::new(4.0, 2.0, 0.16),
        false,
    );
    tick(&mut app, 4);
    for _ in 0..3 {
        request(&mut app, p, CombatIntent::request_attack);
        tick(&mut app, 45);
    }
    assert_eq!(hp(&app, e), 1000.0);
}
#[test]
fn hammer_three_shots_each_have_their_own_fixed_marker_and_companion_identity() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    start_combo(&mut app, p);
    tick(&mut app, 17);
    assert_eq!(hp(&app, e), 980.0);
    assert!(shots(&app, p).is_empty());
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 952.0);
    tick(&mut app, 7);
    assert_eq!(hp(&app, e), 952.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 924.0);
    tick(&mut app, 6);
    assert_eq!(hp(&app, e), 924.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 896.0);
    assert!(app.world().resource::<HitStop>().is_active());
    let companion = app
        .world()
        .get::<Combatant>(p)
        .unwrap()
        .companion()
        .unwrap();
    let events = shots(&app, p);
    assert_eq!(events.len(), 3);
    for (i, e) in events.iter().enumerate() {
        assert_eq!(e.actor, companion);
        assert_eq!(e.hit_index, i as u8);
        assert_ne!(e.actor, p);
        assert!(e.position.is_finite());
    }
    assert!(app.world().get::<Body>(companion).is_none());
    assert!(app.world().get::<Health>(companion).is_none());
    tick(&mut app, 45);
    assert_eq!(hp(&app, e), 896.0);
    assert!(!app.world().get::<Combatant>(p).unwrap().is_combo_active());
}
#[test]
fn ordinary_musket_q_stuns_once_and_keeps_e_cooldown_independent() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 11);
    assert_eq!(hp(&app, e), 1000.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 960.0);
    let enemy = app.world().get::<Enemy>(e).unwrap();
    assert!(enemy.stun_remaining() > 0.4);
    assert!(enemy.stun_source().is_some());
    let c = app.world().get::<Combatant>(p).unwrap();
    assert_eq!(c.melee_cooldown(), 0.0);
    assert!(c.ranged_cooldown() > 3.7);
    assert_eq!(c.combo_remaining(), 0.0);
    assert!(!app.world().resource::<HitStop>().is_active());
}
#[test]
fn original_target_survives_unlock_and_natural_landing_after_tab() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    start_combo(&mut app, p);
    request(&mut app, p, CombatIntent::request_toggle_target);
    teleport_actor(app.world_mut(), e, Vec3::NEG_Z * 4.0);
    tick(&mut app, 34);
    assert_eq!(hp(&app, e), 896.0);
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), None);
    assert_eq!(shots(&app, p).len(), 3);
}
#[test]
fn third_shot_is_cancelled_if_the_second_kills_and_selected_target_clears_immediately() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    app.world_mut().get_mut::<Health>(e).unwrap().damage(940.0);
    start_combo(&mut app, p);
    tick(&mut app, 26);
    assert_eq!(hp(&app, e), 0.0);
    assert_eq!(shots(&app, p).len(), 2);
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), None);
    assert!(
        app.world()
            .get::<Combatant>(p)
            .unwrap()
            .sequence()
            .unwrap()
            .cancelled
    );
    tick(&mut app, 40);
    assert_eq!(shots(&app, p).len(), 2);
}
#[test]
fn missing_target_and_los_stop_old_hits_without_retargeting() {
    for remove in [true, false] {
        let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
        hammer(&mut app, p, e);
        start_combo(&mut app, p);
        if remove {
            app.world_mut().despawn(e);
        } else {
            spawn_box(
                app.world_mut(),
                Vec3::new(0.0, 3.0, -2.0),
                Vec3::new(8.0, 6.0, 0.3),
                false,
            );
        }
        tick(&mut app, 40);
        if remove {
            assert!(shots(&app, p).is_empty());
        } else {
            assert_eq!(hp(&app, e), 980.0);
        }
        assert!(!app.world().resource::<HitStop>().is_active());
    }
}
#[test]
fn launch_peak_is_relative_to_actual_surface_including_high_platforms() {
    for elevation in [0.0, 3.0] {
        let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, elevation);
        let base = app.world().get::<Body>(e).unwrap().world_position().y;
        hammer(&mut app, p, e);
        let mut peak = base;
        for _ in 0..100 {
            tick(&mut app, 1);
            peak = peak.max(app.world().get::<Body>(e).unwrap().world_position().y);
        }
        assert!(
            (peak - base - 3.0).abs() < 0.13,
            "{elevation}: {}",
            peak - base
        );
        assert!(app.world().get::<MotionState>(e).unwrap().is_grounded());
    }
}
#[test]
fn unaccepted_hammer_window_expires_on_landing_and_never_replays_a_tab() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    teleport_actor(app.world_mut(), e, Vec3::NEG_Z * 4.0);
    tick(&mut app, 20);
    request(&mut app, p, CombatIntent::request_combo);
    tick(&mut app, 70);
    assert!(shots(&app, p).is_empty());
    assert!(!app.world().get::<Combatant>(p).unwrap().is_combo_active());
}
#[test]
fn attack_admission_discards_movement_new_attacks_and_equipment_during_commitment() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.6, 0.0);
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 1);
    for _ in 0..12 {
        request(&mut app, p, |i| {
            i.request_ranged_skill();
            i.request_pair(WeaponPair::ChainAxeBow);
        });
        app.world_mut()
            .get_mut::<ActionIntent>(p)
            .unwrap()
            .request_jump();
        tick(&mut app, 1);
    }
    tick(&mut app, 40);
    assert_eq!(
        app.world().get::<Combatant>(p).unwrap().weapon_pair(),
        WeaponPair::HammerMusket
    );
    assert_eq!(app.world().get::<MotionState>(p).unwrap().jumps_used(), 0);
    assert_eq!(hp(&app, e), 976.0);
    assert_eq!(shots(&app, p).len(), 0);
}
#[test]
fn independent_shared_slot_cooldowns_survive_switching_pairs() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    tick(&mut app, 20);
    let cd = app.world().get::<Combatant>(p).unwrap().melee_cooldown();
    request(&mut app, p, |i| i.request_pair(WeaponPair::ChainAxeBow));
    tick(&mut app, 1);
    let c = app.world().get::<Combatant>(p).unwrap();
    assert_eq!(c.weapon_pair(), WeaponPair::ChainAxeBow);
    assert!(c.melee_cooldown() > cd - 0.02);
    assert_eq!(c.ranged_cooldown(), 0.0);
    assert_eq!(c.combo_remaining(), 0.0);
}
#[test]
fn chain_sweep_without_lock_damages_and_slow_reaches_the_actual_motor() {
    let (mut app, p, e) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * 2.5, 0.0);
    // Out of the blade's reach, the enemy is hit by the shockwave in the
    // step the baked swing lands (the press is admitted one step earlier).
    let track = crate::simulation::TrackId::PlayerSkill(WeaponPair::ChainAxeBow.melee_skill())
        .hit_track()
        .unwrap();
    let impact = usize::from(super::sweep::HitTrack::tick(track.impact_time())) + 1;
    request(&mut app, p, CombatIntent::request_melee_skill);
    tick(&mut app, impact - 1);
    assert_eq!(hp(&app, e), 1000.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 976.0);
    let slow = app.world().get::<SlowEffect>(e).unwrap();
    assert_eq!(slow.factor(), 0.65);
    assert!(slow.remaining() > 1.4);
    app.world_mut().get_mut::<Enemy>(e).unwrap().passive = true;
    app.world_mut()
        .get_mut::<MoveIntent>(e)
        .unwrap()
        .set(Vec2::X);
    tick(&mut app, 1);
    assert!(app.world().get::<Body>(e).unwrap().velocity().x <= 1.31);
    tick(&mut app, 100);
    assert_eq!(app.world().get::<SlowEffect>(e).unwrap().factor(), 1.0);
}
#[test]
fn axe_shot_has_separate_throw_hit_return_and_source_bound_stun() {
    let (mut app, p, e) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * 8.0, 0.0);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 9);
    assert!(!app.world().get::<Combatant>(p).unwrap().axe_away());
    tick(&mut app, 1);
    assert!(app.world().get::<Combatant>(p).unwrap().axe_away());
    assert_eq!(hp(&app, e), 1000.0);
    tick(&mut app, 8);
    assert_eq!(hp(&app, e), 1000.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 965.0);
    let c = app.world().get::<Combatant>(p).unwrap();
    let window = c.combo_window().unwrap();
    assert_eq!(
        app.world().get::<Enemy>(e).unwrap().stun_source(),
        Some(window.source_action_id)
    );
    tick(&mut app, 28);
    assert!(!app.world().get::<Combatant>(p).unwrap().axe_away());
}
fn chain_setup() -> (App, Entity, Entity) {
    let (mut app, p, e) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * 8.0, 0.0);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 19);
    assert_eq!(hp(&app, e), 965.0);
    (app, p, e)
}
#[test]
fn chain_pull_uses_physics_and_three_separate_arrows_from_a_fixed_sister() {
    let (mut app, p, e) = chain_setup();
    start_combo(&mut app, p);
    let origin = app.world().get::<Body>(p).unwrap().world_position();
    let sister = app
        .world()
        .get::<Combatant>(p)
        .unwrap()
        .active_action()
        .unwrap()
        .companion_position;
    tick(&mut app, 7);
    assert_eq!(hp(&app, e), 965.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 947.0);
    tick(&mut app, 8);
    assert_eq!(hp(&app, e), 947.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 929.0);
    tick(&mut app, 9);
    assert_eq!(hp(&app, e), 929.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 911.0);
    let pos = app.world().get::<Body>(p).unwrap().world_position();
    assert!(origin.distance(pos) > 3.0);
    let action = app
        .world()
        .get::<Combatant>(p)
        .unwrap()
        .active_action()
        .unwrap();
    assert_eq!(action.companion_position, sister);
    assert_eq!(shots(&app, p).len(), 4);
    tick(&mut app, 50);
    assert!(!app.world().get::<Combatant>(p).unwrap().axe_away());
    assert!(!app.world().get::<Combatant>(p).unwrap().is_combo_active());
    assert!(
        app.world()
            .get::<Body>(p)
            .unwrap()
            .world_position()
            .distance(app.world().get::<Body>(e).unwrap().world_position())
            >= 1.20
    );
}
#[test]
fn chain_combo_requires_the_original_q_stun_source() {
    let (mut app, p, e) = chain_setup();
    app.world_mut()
        .get_mut::<Enemy>(e)
        .unwrap()
        .stun(99_999, 0.90);
    request(&mut app, p, CombatIntent::request_combo);
    tick(&mut app, 1);
    assert!(!app.world().get::<Combatant>(p).unwrap().is_combo_active());
    assert_eq!(hp(&app, e), 965.0);
}
#[test]
fn pull_blocked_by_wall_or_large_height_cancels_and_returns_axe_without_late_arrows() {
    for wall in [false, true] {
        let (mut app, p, e) = chain_setup();
        start_combo(&mut app, p);
        if wall {
            spawn_box(
                app.world_mut(),
                Vec3::new(0.0, 2.0, -2.0),
                Vec3::new(8.0, 4.0, 0.25),
                false,
            );
        } else {
            teleport_actor(app.world_mut(), e, Vec3::new(0.0, 5.0, -8.0));
        }
        tick(&mut app, 70);
        assert!(!app.world().get::<Combatant>(p).unwrap().is_combo_active());
        assert!(!app.world().get::<Combatant>(p).unwrap().axe_away());
        assert!(shots(&app, p).len() < 4);
        if wall {
            assert!(app.world().get::<Body>(p).unwrap().world_position().z > -1.60);
        }
    }
}
#[test]
fn ordered_short_tap_preserves_deflect_but_never_creates_release_parry() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    edge(&mut app, p, DefenseEdge::Released);
    tick(&mut app, 1);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_deflecting());
    strike(&mut app, p, e);
    assert_eq!(hp(&app, p), 1000.0);
    assert!(!app.world().get::<Enemy>(e).unwrap().is_recovering());
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), 100.0);
    tick(&mut app, 11);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_active());
}
#[test]
fn held_guard_blocks_more_than_three_hits_and_exact_last_cost_but_never_partially_spends() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 13);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_guarding());
    for n in 1..=5 {
        strike(&mut app, p, e);
        assert_eq!(hp(&app, p), 1000.0 - 6.0 * n as f32);
        assert_eq!(
            app.world().get::<Stamina>(p).unwrap().current(),
            100.0 - 20.0 * n as f32
        );
    }
    strike(&mut app, p, e);
    assert_eq!(hp(&app, p), 958.0);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), 0.0);
    assert!(!app.world().get::<Enemy>(e).unwrap().is_recovering());
}
#[test]
fn release_from_effective_guard_parries_and_stuns_but_cancel_does_not() {
    for cancel in [false, true] {
        let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
        edge(&mut app, p, DefenseEdge::Pressed);
        tick(&mut app, 13);
        edge(
            &mut app,
            p,
            if cancel {
                DefenseEdge::Cancelled
            } else {
                DefenseEdge::Released
            },
        );
        tick(&mut app, 1);
        strike(&mut app, p, e);
        assert_eq!(hp(&app, p), if cancel { 988.0 } else { 1000.0 });
        assert_eq!(
            app.world().get::<Enemy>(e).unwrap().stun_source().is_some(),
            !cancel
        );
    }
}
#[test]
fn back_hits_ignore_all_three_defense_states() {
    for kind in 0..3 {
        let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::Z * 1.2, 0.0);
        edge(&mut app, p, DefenseEdge::Pressed);
        tick(&mut app, if kind == 0 { 1 } else { 13 });
        if kind == 2 {
            edge(&mut app, p, DefenseEdge::Released);
            tick(&mut app, 1);
        }
        strike(&mut app, p, e);
        assert_eq!(hp(&app, p), 988.0);
        assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), 100.0);
    }
}
#[test]
fn clear_during_zero_tick_pause_cancels_without_ghost_release_or_timer_reset() {
    let (mut app, p, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 13);
    let cd = app
        .world()
        .get::<DefenseState>(p)
        .unwrap()
        .deflect_cooldown();
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    app.world_mut().get_mut::<DefenseState>(p).unwrap().clear();
    edge(&mut app, p, DefenseEdge::Released);
    tick(&mut app, 120);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_guarding());
    app.world_mut().resource_mut::<SimulationActive>().0 = true;
    tick(&mut app, 1);
    let d = app.world().get::<DefenseState>(p).unwrap();
    assert!(!d.is_active());
    assert_eq!(d.parry_cooldown(), 0.0);
    assert!(d.deflect_cooldown() > cd - 0.02);
}
#[test]
fn air_and_action_right_presses_are_consumed_and_never_queued() {
    let (mut app, p, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 3.0, 0.0);
    app.world_mut()
        .get_mut::<ActionIntent>(p)
        .unwrap()
        .request_jump();
    tick(&mut app, 1);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 80);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_active());
    assert_eq!(
        app.world()
            .get::<DefenseState>(p)
            .unwrap()
            .deflect_cooldown(),
        0.0
    );
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 1);
    edge(&mut app, p, DefenseEdge::Released);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 50);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_active());
}
#[test]
fn defense_moves_slowly_crouches_without_slide_and_jump_exits() {
    let (mut app, p, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 20.0, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    app.world_mut()
        .get_mut::<MoveIntent>(p)
        .unwrap()
        .set(Vec2::X);
    app.world_mut()
        .get_mut::<ActionIntent>(p)
        .unwrap()
        .set_shift_held(true);
    tick(&mut app, 20);
    assert!((app.world().get::<Body>(p).unwrap().velocity().x - 2.1).abs() < 0.02);
    assert!(!app.world().get::<MotionState>(p).unwrap().is_sprinting());
    app.world_mut()
        .get_mut::<ActionIntent>(p)
        .unwrap()
        .request_crouch_toggle();
    tick(&mut app, 3);
    let m = app.world().get::<MotionState>(p).unwrap();
    assert!(m.is_crouched());
    assert!(!m.is_sliding());
    assert!((app.world().get::<Body>(p).unwrap().velocity().x - 1.6).abs() < 0.02);
    app.world_mut()
        .get_mut::<ActionIntent>(p)
        .unwrap()
        .request_jump();
    tick(&mut app, 1);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_active());
    assert_eq!(app.world().get::<MotionState>(p).unwrap().jumps_used(), 1);
}
#[test]
fn stamina_recovers_only_after_quiet_and_attack_or_incoming_restarts_wait() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 13);
    strike(&mut app, p, e);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), 80.0);
    tick(&mut app, 58);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), 80.0);
    tick(&mut app, 31);
    let value = app.world().get::<Stamina>(p).unwrap().current();
    assert!((value - 87.5).abs() < 0.6);
    edge(&mut app, p, DefenseEdge::Cancelled);
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 20);
    let value = app.world().get::<Stamina>(p).unwrap().current();
    tick(&mut app, 35);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), value);
    strike(&mut app, p, e);
    tick(&mut app, 50);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), value);
}
#[test]
fn pause_freezes_action_stamina_cooldowns_and_clear_keeps_committed_timeline() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    start_combo(&mut app, p);
    tick(&mut app, 9);
    let before = app.world().get::<Combatant>(p).unwrap().sequence().unwrap();
    let cd = app.world().get::<Combatant>(p).unwrap().melee_cooldown();
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    app.world_mut().get_mut::<CombatIntent>(p).unwrap().clear();
    app.world_mut().get_mut::<DefenseState>(p).unwrap().clear();
    tick(&mut app, 120);
    let c = app.world().get::<Combatant>(p).unwrap();
    assert_eq!(c.sequence().unwrap().elapsed, before.elapsed);
    assert_eq!(c.melee_cooldown(), cd);
    assert!(shots(&app, p).is_empty());
    app.world_mut().resource_mut::<SimulationActive>().0 = true;
    tick(&mut app, 30);
    assert_eq!(shots(&app, p).len(), 3);
}
#[test]
fn death_cancels_pending_shots_and_respawn_clears_every_gesture_and_refills() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    start_combo(&mut app, p);
    app.world_mut().get_mut::<Health>(p).unwrap().damage(1000.0);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 181);
    assert_eq!(hp(&app, p), 1000.0);
    assert_eq!(shots(&app, p).len(), 0);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), 100.0);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_active());
    assert_eq!(app.world().get::<Combatant>(p).unwrap().target(), None);
}
#[test]
fn feedback_history_remains_bounded_ordered_and_companion_does_not_duplicate() {
    let (mut app, p, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 10.0, 0.0);
    for _ in 0..150 {
        request(&mut app, p, CombatIntent::request_melee_skill);
        tick(&mut app, 1);
    }
    let events: Vec<_> = app.world().resource::<CombatFeedback>().events().collect();
    assert_eq!(events.len(), 128);
    assert!(events.windows(2).all(|w| w[0].sequence < w[1].sequence));
    assert_eq!(
        app.world_mut()
            .query::<&Companion>()
            .iter(app.world())
            .count(),
        1
    );
}

#[test]
fn pull_arrives_with_braking_without_contact_or_pushing_the_target() {
    for distance in [3.0, 8.0, 11.5] {
        let (mut app, p, e) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * distance, 0.0);
        lock(&mut app, p, e);
        request(&mut app, p, CombatIntent::request_ranged_skill);
        tick(&mut app, 19);
        start_combo(&mut app, p);
        let target_start = app.world().get::<Body>(e).unwrap().world_position();
        let mut minimum = f32::MAX;
        for _ in 0..70 {
            tick(&mut app, 1);
            let player = app.world().get::<Body>(p).unwrap().world_position();
            let target = app.world().get::<Body>(e).unwrap().world_position();
            minimum = minimum.min(player.xz().distance(target.xz()));
            assert!(
                target.xz().distance(target_start.xz()) < 0.03,
                "pull pushed target at distance {distance}"
            );
        }
        assert!(
            minimum >= 1.30,
            "minimum separation {minimum} at initial {distance}"
        );
        if distance <= 8.0 {
            assert!(
                minimum <= 1.55,
                "did not reach stopping distance: {minimum}"
            );
        }
    }
}
#[test]
fn post_marker_availability_and_half_open_defense_boundaries_are_current() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    assert!(
        app.world()
            .get::<Combatant>(p)
            .unwrap()
            .availability()
            .combo
    );
    tick(&mut app, 20);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 1);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_deflecting());
    tick(&mut app, 11);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_deflecting());
    tick(&mut app, 1);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_guarding());
    edge(&mut app, p, DefenseEdge::Released);
    tick(&mut app, 1);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_parrying());
    tick(&mut app, 10);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_parrying());
    tick(&mut app, 1);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_active());
}
#[test]
fn cooldown_press_and_insufficient_stamina_do_not_manufacture_extra_windows() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    edge(&mut app, p, DefenseEdge::Released);
    tick(&mut app, 13);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 1);
    assert!(!app.world().get::<DefenseState>(p).unwrap().is_deflecting());
    assert_eq!(
        app.world().get::<DefenseState>(p).unwrap().kind(),
        DefenseKind::Waiting
    );
    tick(&mut app, 12);
    assert!(app.world().get::<DefenseState>(p).unwrap().is_guarding());
    for _ in 0..5 {
        strike(&mut app, p, e);
    }
    tick(&mut app, 100);
    let before = app.world().get::<Stamina>(p).unwrap().current();
    assert!(before > 0.0 && before < 20.0);
    strike(&mut app, p, e);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), before);
    assert_eq!(hp(&app, p), 958.0);
}
#[test]
fn every_projectile_rechecks_visibility_and_does_not_replay_a_skipped_shot() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    start_combo(&mut app, p);
    let wall = spawn_box(
        app.world_mut(),
        Vec3::new(0.0, 3.0, -2.0),
        Vec3::new(8.0, 6.0, 0.3),
        false,
    );
    tick(&mut app, 18);
    assert_eq!(hp(&app, e), 980.0);
    assert_eq!(shots(&app, p).len(), 1);
    app.world_mut().despawn(wall);
    tick(&mut app, 15);
    assert_eq!(hp(&app, e), 924.0);
    assert_eq!(shots(&app, p).len(), 3);
    tick(&mut app, 50);
    assert_eq!(hp(&app, e), 924.0);
}
#[test]
fn dying_dummy_revives_passive_and_scale_survives_teleport_and_landing() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    app.world_mut().get_mut::<Health>(e).unwrap().damage(1000.0);
    tick(&mut app, 181);
    assert_eq!(hp(&app, e), 1000.0);
    assert!(app.world().get::<Enemy>(e).unwrap().is_passive());
    tick(&mut app, 180);
    assert_eq!(hp(&app, p), 1000.0);
    assert!(!app.world().get::<Enemy>(e).unwrap().is_winding_up());
}
#[test]
fn mode_switch_while_paused_clears_sister_synchronously_and_preserves_stamina() {
    use crate::gameplay::{EnemyMode, set_enemy_mode};
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.2, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 13);
    strike(&mut app, p, e);
    edge(&mut app, p, DefenseEdge::Cancelled);
    tick(&mut app, 1);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 1);
    let c = app.world().get::<Combatant>(p).unwrap();
    let companion = c.companion().unwrap();
    assert!(
        app.world()
            .get::<CompanionState>(companion)
            .unwrap()
            .visible
    );
    let cooldown = c.ranged_cooldown();
    let stamina = app.world().get::<Stamina>(p).unwrap().current();
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    set_enemy_mode(app.world_mut(), EnemyMode::Combat);
    let c = app.world().get::<Combatant>(p).unwrap();
    assert!(c.active_action().is_none());
    assert_eq!(c.ranged_cooldown(), cooldown);
    assert_eq!(app.world().get::<Stamina>(p).unwrap().current(), stamina);
    assert!(
        !app.world()
            .get::<CompanionState>(companion)
            .unwrap()
            .visible
    );
}
#[test]
fn action_markers_crossed_with_completion_are_not_dropped() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.6, 0.0);
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 1);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f64(0.70));
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 976.0);
    assert!(
        app.world()
            .get::<Combatant>(p)
            .unwrap()
            .active_action()
            .is_none()
    );
}

#[test]
fn late_tab_keeps_three_shots_when_gravity_naturally_lands_the_original_target() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    tick(&mut app, 40);
    assert!(!app.world().get::<MotionState>(e).unwrap().is_grounded());
    assert!(app.world().get::<Combatant>(p).unwrap().combo_remaining() > 0.0);
    start_combo(&mut app, p);
    tick(&mut app, 32);
    assert!(app.world().get::<MotionState>(e).unwrap().is_grounded());
    assert_eq!(hp(&app, e), 924.0);
    tick(&mut app, 1);
    assert_eq!(hp(&app, e), 896.0);
    assert_eq!(shots(&app, p).len(), 3);
}
#[test]
fn many_crossed_combo_markers_execute_in_order_once_even_past_completion() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 4.0, 0.0);
    hammer(&mut app, p, e);
    start_combo(&mut app, p);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f64(1.0));
    tick(&mut app, 1);
    let events = shots(&app, p);
    assert_eq!(
        events.iter().map(|e| e.hit_index).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(hp(&app, e), 896.0);
    assert!(!app.world().get::<Combatant>(p).unwrap().is_combo_active());
}

#[test]
fn stationary_skill_range_matches_admission_for_scaled_targets_on_high_ground() {
    type SkillCase = (WeaponPair, f32, f32, fn(&mut CombatIntent), bool);
    let skills: [SkillCase; 3] = [
        (
            WeaponPair::HammerMusket,
            22.0,
            40.0,
            CombatIntent::request_ranged_skill,
            false,
        ),
        (
            WeaponPair::ChainAxeBow,
            12.0,
            35.0,
            CombatIntent::request_ranged_skill,
            false,
        ),
        (
            WeaponPair::HammerMusket,
            4.5,
            20.0,
            CombatIntent::request_melee_skill,
            true,
        ),
    ];
    for (pair, range, damage, skill, melee) in skills {
        for elevation in [0.0, 3.0] {
            for outside in [false, true] {
                let (mut app, p, e) =
                    arena_with_target_scale(pair, Vec3::NEG_Z * (range - 0.01), elevation, 1.45);
                lock(&mut app, p, e);
                if outside {
                    teleport_actor(
                        app.world_mut(),
                        e,
                        Vec3::new(0.0, elevation, -(range + 0.01)),
                    );
                    tick(&mut app, 2);
                }
                request(&mut app, p, skill);
                tick(&mut app, 22);
                let combat = app.world().get::<Combatant>(p).unwrap();
                let cooldown = if melee {
                    combat.melee_cooldown()
                } else {
                    combat.ranged_cooldown()
                };
                assert_eq!(
                    hp(&app, e),
                    if outside { 1000.0 } else { 1000.0 - damage },
                    "{pair:?} / {range}m / {elevation}m / outside={outside}"
                );
                assert_eq!(cooldown > 0.0, !outside);
            }
        }
    }
}
