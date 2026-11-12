//! Shots in flight through the real fixed step: flight time, guidance, what
//! stops them, and the thrown weapon's way back.
use super::super::projectile::{Payload, Projectile};
use super::super::skills::{Control, Guidance, ProjectileSpec};
use super::*;

const SPEED: f32 = 30.0;

fn spec(guidance: Guidance) -> ProjectileSpec {
    ProjectileSpec {
        speed: SPEED,
        radius: 0.2,
        max_range: 30.0,
        guidance,
    }
}
const LOCKED: Guidance = Guidance::Locked { turn_rate: 12.0 };

/// Fire one shot from chest height at the player toward `aim`, as an action
/// of the player's which remembers `target`.
fn fire(
    app: &mut App,
    player: Entity,
    target: Option<Entity>,
    aim: Vec3,
    spec: ProjectileSpec,
) -> Entity {
    let origin = app.world().get::<Body>(player).unwrap().aim_position();
    let action = ActionSnapshot {
        action_id: 900,
        kind: ActionKind::MusketShot,
        pair: WeaponPair::HammerMusket,
        melee_stage: None,
        target,
        elapsed: 0.0,
        duration: 0.4,
        shot_count: 1,
        aim_position: aim,
        source_position: origin,
        companion_position: origin,
    };
    let payload = Payload {
        damage: 40.0,
        control: Control::None,
        window: None,
        sniper: false,
        hit_stop: false,
    };
    let performer = app
        .world()
        .get::<Combatant>(player)
        .unwrap()
        .companion()
        .unwrap();
    app.world_mut()
        .spawn(Projectile::launch(
            player,
            performer,
            action,
            0,
            (origin, aim),
            spec,
            payload,
            false,
        ))
        .id()
}
fn aim_at(app: &App, entity: Entity) -> Vec3 {
    app.world().get::<Body>(entity).unwrap().aim_position()
}
fn position(app: &App, projectile: Entity) -> Option<Vec3> {
    app.world()
        .get::<Projectile>(projectile)
        .map(Projectile::position)
}

#[test]
fn a_shot_takes_its_distance_over_its_speed_to_arrive() {
    for distance in [6.0, 12.0, 20.0] {
        let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * distance, 0.0);
        let aim = aim_at(&app, e);
        fire(&mut app, p, Some(e), aim, spec(Guidance::Straight));
        let steps = until(&mut app, 60, |app| hp(app, e) < 1000.0);
        // The gap between the muzzle and the target's surface, at 0.5 m per step.
        let gap = distance - 0.3 - 0.2;
        let expected = (gap / (SPEED / 60.0)).ceil() as usize;
        assert!(
            steps.abs_diff(expected) <= 1,
            "{distance} m took {steps} steps, expected about {expected}"
        );
        assert_eq!(hp(&app, e), 960.0);
        assert_eq!(flying(&app), 0);
        // The hit is reported where the shot met the body.
        let hit = app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .find(|event| event.kind == CombatEffect::Hit && event.actor == e)
            .copied()
            .unwrap();
        let body = app.world().get::<Body>(e).unwrap();
        assert!(hit.position.xz().distance(body.world_position().xz()) <= body.radius() + 0.21);
        assert_eq!(hit.action_id, 900);
    }
}

#[test]
fn a_locked_shot_follows_its_target_and_a_straight_one_does_not() {
    for (guidance, hits) in [(LOCKED, true), (Guidance::Straight, false)] {
        // The target is carried sideways at 6 m/s while the shot is on its way.
        let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 15.0, 0.0);
        let aim = aim_at(&app, e);
        let shot = fire(&mut app, p, Some(e), aim, spec(guidance));
        for step in 1..=60 {
            teleport_actor(app.world_mut(), e, Vec3::new(step as f32 * 0.1, 0.0, -15.0));
            tick(&mut app, 1);
            if position(&app, shot).is_none() {
                break;
            }
        }
        assert_eq!(hp(&app, e) < 1000.0, hits, "{guidance:?}");
    }
    // A locked shot also follows a target which is falling.
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 12.0, 0.0);
    teleport_actor(app.world_mut(), e, Vec3::new(0.0, 6.0, -12.0));
    tick(&mut app, 1);
    let aim = aim_at(&app, e);
    fire(&mut app, p, Some(e), aim, spec(LOCKED));
    until(&mut app, 60, |app| hp(app, e) < 1000.0);
    assert!(app.world().get::<Body>(e).unwrap().world_position().y < 5.9);
}

#[test]
fn a_straight_shot_needs_no_target_and_expires_at_its_range() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::new(8.0, 0.0, 0.0), 0.0);
    let origin = aim_at(&app, p);
    let shot = fire(
        &mut app,
        p,
        None,
        origin + Vec3::NEG_Z * 3.0,
        spec(Guidance::Straight),
    );
    let steps = until(&mut app, 80, |app| flying(app) == 0);
    // Thirty metres at half a metre per step, into empty air.
    assert_eq!(steps, 60);
    assert!(position(&app, shot).is_none());
    assert_eq!(hp(&app, e), 1000.0);
    assert!(
        app.world()
            .resource::<CombatFeedback>()
            .events()
            .all(|event| event.kind != CombatEffect::Hit && event.kind != CombatEffect::ShotImpact)
    );
}

#[test]
fn a_shot_hits_whatever_stands_in_its_way_first() {
    // Another enemy on the line takes the shot meant for the locked target.
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 12.0, 0.0);
    let between = dummy(&mut app, Vec3::NEG_Z * 6.0);
    tick(&mut app, 12);
    let aim = aim_at(&app, e);
    fire(&mut app, p, Some(e), aim, spec(LOCKED));
    land_shots(&mut app);
    assert_eq!(hp(&app, between), 960.0);
    assert_eq!(hp(&app, e), 1000.0);

    // A wall stops it and reports where; a defeated body does not.
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 12.0, 0.0);
    let fallen = dummy(&mut app, Vec3::NEG_Z * 4.0);
    spawn_box(
        app.world_mut(),
        Vec3::new(0.0, 2.0, -8.0),
        Vec3::new(6.0, 4.0, 0.3),
        false,
    );
    tick(&mut app, 12);
    app.world_mut()
        .get_mut::<Health>(fallen)
        .unwrap()
        .damage(1000.0);
    let aim = aim_at(&app, e);
    fire(&mut app, p, Some(e), aim, spec(LOCKED));
    land_shots(&mut app);
    assert_eq!(hp(&app, e), 1000.0);
    let stopped = app
        .world()
        .resource::<CombatFeedback>()
        .events()
        .find(|event| event.kind == CombatEffect::ShotImpact)
        .copied()
        .expect("the wall stopped the shot");
    assert!(
        (stopped.position.z + 7.85).abs() < 0.05,
        "stopped at {}",
        stopped.position
    );
}

#[test]
fn shots_freeze_with_the_simulation_and_leave_with_their_owner_or_the_roster() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 20.0, 0.0);
    let aim = aim_at(&app, e);
    let shot = fire(&mut app, p, Some(e), aim, spec(LOCKED));
    tick(&mut app, 5);
    let frozen = position(&app, shot).unwrap();
    assert!(frozen.z < -2.0);
    app.world_mut().resource_mut::<SimulationActive>().0 = false;
    tick(&mut app, 20);
    assert_eq!(position(&app, shot), Some(frozen));
    app.world_mut().resource_mut::<SimulationActive>().0 = true;
    tick(&mut app, 1);
    assert!(position(&app, shot).unwrap().z < frozen.z);

    // Replacing the enemies removes what was fired at the old ones.
    crate::gameplay::set_enemy_mode(app.world_mut(), crate::gameplay::EnemyMode::Combat);
    assert_eq!(flying(&app), 0);

    // A shot does not outlive its owner.
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 20.0, 0.0);
    let aim = aim_at(&app, e);
    fire(&mut app, p, Some(e), aim, spec(LOCKED));
    tick(&mut app, 3);
    app.world_mut().get_mut::<Health>(p).unwrap().damage(1000.0);
    tick(&mut app, 60);
    assert_eq!(flying(&app), 0);
    assert_eq!(hp(&app, e), 1000.0);
}

#[test]
fn the_thrown_axe_cannot_be_swung_until_it_is_back_in_the_hand() {
    let (mut app, p, e) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * 10.0, 0.0);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    until(&mut app, 60, |app| hp(app, e) < 1000.0);
    // The player ends up far from the axe, so the skill itself is over
    // while the axe is still on its way back.
    teleport_actor(app.world_mut(), p, Vec3::Z * 14.0);
    until(&mut app, 60, |app| {
        app.world()
            .get::<Combatant>(p)
            .unwrap()
            .active_action()
            .is_none()
    });
    let combat = app.world().get::<Combatant>(p).unwrap();
    assert!(combat.axe_away());
    assert!(!combat.availability().attack);
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 1);
    assert!(
        app.world()
            .get::<Combatant>(p)
            .unwrap()
            .active_action()
            .is_none()
    );
    // The other pair's weapon is in the hand and usable meanwhile.
    request(&mut app, p, |intent| {
        intent.request_pair(WeaponPair::HammerMusket)
    });
    tick(&mut app, 1);
    let combat = app.world().get::<Combatant>(p).unwrap();
    assert_eq!(combat.weapon_pair(), WeaponPair::HammerMusket);
    assert!(!combat.axe_away() && combat.availability().attack);
    request(&mut app, p, |intent| {
        intent.request_pair(WeaponPair::ChainAxeBow)
    });
    tick(&mut app, 1);
    assert!(app.world().get::<Combatant>(p).unwrap().axe_away());
    until(&mut app, 60, |app| flying(app) == 0);
    tick(&mut app, 1);
    let combat = app.world().get::<Combatant>(p).unwrap();
    assert!(!combat.axe_away() && combat.availability().attack);

    // A throw which reaches nothing comes back from the end of its range.
    let (mut app, p, e) = arena(WeaponPair::ChainAxeBow, Vec3::NEG_Z * 10.0, 0.0);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    until(&mut app, 30, |app| flying(app) == 1);
    teleport_actor(app.world_mut(), e, Vec3::new(0.0, 0.0, 40.0));
    app.world_mut().get_mut::<Health>(e).unwrap().damage(1000.0);
    let out_and_back = until(&mut app, 120, |app| flying(app) == 0);
    // Sixteen metres out and back at 35 m/s.
    assert!((50..=60).contains(&out_and_back), "{out_and_back} steps");
    assert_eq!(
        app.world()
            .get::<Combatant>(p)
            .unwrap()
            .combo_window()
            .map(|w| w.remaining),
        None
    );
    assert!(!app.world().get::<Combatant>(p).unwrap().axe_away());
}

#[test]
fn locked_skills_need_a_lock_and_every_row_states_how_its_shots_fly() {
    use super::super::skills::Shape;
    let mut shooting = 0;
    for kind in ActionKind::ALL {
        let row = kind.spec();
        let projectile = match row.shape {
            Shape::Shot(shot) => shot.projectile,
            Shape::Volley(volley) => volley.projectile,
            Shape::Blade | Shape::Area(_) => continue,
        };
        shooting += 1;
        assert!(projectile.speed > 0.0 && projectile.radius > 0.0);
        // The shot can cross the distance the skill may be cast from.
        if let Some(range) = row.cast_range {
            assert!(projectile.max_range > range, "{kind:?}");
        }
        // A skill whose shots lock on is only admitted with a locked target
        // in range, or follows a window which remembers its target.
        if matches!(projectile.guidance, Guidance::Locked { .. }) {
            assert!(row.cast_range.is_some() || kind.is_combo(), "{kind:?}");
        }
    }
    assert_eq!(shooting, 4);
}
