use super::*;
use crate::game::input::ButtonFrame;
use std::time::Duration;

struct Arena {
    world: World,
    schedule: Schedule,
    player: Entity,
    dummy: Entity,
}
impl Arena {
    fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(Controls::default());
        world.insert_resource(Time::<()>::default());
        world.insert_resource(OrbitCamera::default());
        world.insert_resource(Impacts::default());
        world.insert_resource(CombatStatus::default());
        world.insert_resource(Sounds::default());
        world.insert_resource(SwordWaves::default());
        let player = world
            .spawn((
                Fighter::new(Side::Player),
                Transform::from_translation(Side::Player.home()),
            ))
            .id();
        let dummy = world
            .spawn((
                Fighter::new(Side::Dummy),
                Transform::from_translation(Side::Dummy.home()),
            ))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(simulate);
        Self {
            world,
            schedule,
            player,
            dummy,
        }
    }
    fn controls() -> Controls {
        Controls {
            active: true,
            ..default()
        }
    }
    fn tick(&mut self, dt: f32, controls: Controls) {
        self.world.insert_resource(controls);
        self.world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        self.schedule.run(&mut self.world);
    }
    fn wait(&mut self, seconds: f32) {
        for _ in 0..(seconds * 100.0).round() as usize {
            self.tick(0.01, Self::controls());
        }
    }
    fn press(&mut self, axis: AttackAxis) {
        let mut controls = Self::controls();
        controls.attacks[button_index(axis)] = ButtonFrame {
            pressed: true,
            held: true,
            released: false,
        };
        self.tick(0.01, controls);
    }
    fn release(&mut self, axis: AttackAxis) {
        let mut controls = Self::controls();
        controls.attacks[button_index(axis)].released = true;
        self.tick(0.01, controls);
    }
    fn begin_sprint(&mut self, direction: Vec2) {
        let mut input = Self::controls();
        input.dodge = true;
        input.sprint = true;
        input.movement = direction;
        self.tick(0.01, input);
        for _ in 0..35 {
            let mut input = Self::controls();
            input.sprint = true;
            input.movement = direction;
            self.tick(0.01, input);
        }
    }
    fn fighter(&self, entity: Entity) -> &Fighter {
        self.world.get::<Fighter>(entity).unwrap()
    }
}

#[test]
fn right_click_is_vertical_attack_and_never_a_held_guard() {
    let mut arena = Arena::new();
    arena.press(AttackAxis::Vertical);
    arena.release(AttackAxis::Vertical);
    assert!(
        matches!(arena.fighter(arena.player).action, Action::Attack(ref a) if a.kind == AttackKind::Light(AttackAxis::Vertical, 0))
    );
    arena.wait(0.30);
    assert_eq!(arena.fighter(arena.dummy).health, 90.0);
}

#[test]
fn charge_crouch_cancel_does_not_fire_on_the_later_mouse_release() {
    let mut arena = Arena::new();
    arena.press(AttackAxis::Horizontal);
    for _ in 0..60 {
        let mut controls = Arena::controls();
        controls.attacks[0].held = true;
        arena.tick(0.01, controls);
    }
    assert_eq!(arena.fighter(arena.player).armor(), Armor::Blue);
    let mut controls = Arena::controls();
    controls.crouch_pressed = true;
    arena.tick(0.01, controls);
    arena.release(AttackAxis::Horizontal);
    arena.wait(0.8);
    assert!(matches!(arena.fighter(arena.player).action, Action::Idle));
    assert_eq!(arena.fighter(arena.dummy).health, 100.0);
    assert!(arena.world.resource::<SwordWaves>().active.is_empty());
}

#[test]
fn charge_release_spawns_a_wave_that_hits_only_once() {
    let mut arena = Arena::new();
    arena.press(AttackAxis::Vertical);
    for _ in 0..60 {
        let mut controls = Arena::controls();
        controls.attacks[1].held = true;
        arena.tick(0.01, controls);
    }
    arena.release(AttackAxis::Vertical);
    arena.wait(0.6);
    assert_eq!(arena.fighter(arena.dummy).health, 70.0);
    assert_eq!(arena.world.resource::<CombatStatus>().hits, 1);
}

#[test]
fn combined_parry_does_not_leave_an_attack_queued() {
    let mut arena = Arena::new();
    let mut controls = Arena::controls();
    controls.parry = true;
    controls.attacks = [ButtonFrame {
        pressed: true,
        held: true,
        released: false,
    }; 2];
    arena.tick(0.01, controls);
    assert!(matches!(
        arena.fighter(arena.player).action,
        Action::Parry(_)
    ));
    arena.wait(0.8);
    assert!(matches!(arena.fighter(arena.player).action, Action::Idle));
    assert_eq!(arena.fighter(arena.dummy).health, 100.0);
}

#[test]
fn close_blue_strike_can_be_parried_then_punished_with_a_gold_riposte() {
    let mut arena = Arena::new();
    let mut controls = Arena::controls();
    controls.training_attack = true;
    arena.tick(0.01, controls);
    arena.wait(0.50);
    let mut controls = Arena::controls();
    controls.parry = true;
    arena.tick(0.01, controls);
    arena.wait(0.18);
    assert_eq!(arena.fighter(arena.player).health, 100.0);
    assert_eq!(arena.world.resource::<CombatStatus>().blocks, 1);
    assert!(arena.fighter(arena.dummy).disarmed > 0.0);
    arena.press(AttackAxis::Horizontal);
    assert!(
        matches!(arena.fighter(arena.player).action, Action::Attack(ref a) if a.kind == AttackKind::Riposte)
    );
    arena.wait(0.35);
    assert_eq!(arena.fighter(arena.dummy).health, 65.0);
}

#[test]
fn normal_attack_punishes_parry_and_blue_charge_resists_its_stagger() {
    for blue in [false, true] {
        let mut arena = Arena::new();
        arena.world.get_mut::<Fighter>(arena.player).unwrap().action = if blue {
            Action::Charge(Charge {
                axis: AttackAxis::Horizontal,
                combo: 0,
                dodge_followup: false,
                elapsed: 0.7,
            })
        } else {
            Action::Parry(0.06)
        };
        let mut attack = Attack::new(AttackKind::Training(false));
        attack.elapsed = 0.64;
        arena.world.get_mut::<Fighter>(arena.dummy).unwrap().action = Action::Attack(attack);
        let mut controls = Arena::controls();
        controls.attacks[0].held = blue;
        arena.tick(0.03, controls);
        assert_eq!(arena.fighter(arena.player).health, 80.0);
        assert_eq!(
            matches!(arena.fighter(arena.player).action, Action::Charge(_)),
            blue
        );
        assert_eq!(
            matches!(arena.fighter(arena.player).action, Action::Stagger(_)),
            !blue
        );
    }
}

#[test]
fn equal_normal_attacks_clash_without_query_order_advantage() {
    let mut arena = Arena::new();
    for (entity, z) in [(arena.player, 0.8), (arena.dummy, -0.8)] {
        let mut attack = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 0));
        attack.elapsed = 0.13;
        arena.world.get_mut::<Fighter>(entity).unwrap().action = Action::Attack(attack);
        arena
            .world
            .get_mut::<Transform>(entity)
            .unwrap()
            .translation
            .z = z;
    }
    arena.tick(0.03, Arena::controls());
    for entity in [arena.player, arena.dummy] {
        assert_eq!(arena.fighter(entity).health, 100.0);
        assert!(matches!(arena.fighter(entity).action, Action::Stagger(_)));
    }
}

#[test]
fn uppercut_launches_both_actors_and_an_air_attack_is_available() {
    let mut arena = Arena::new();
    let mut controls = Arena::controls();
    controls.crouch_pressed = true;
    controls.attacks[1].pressed = true;
    arena.tick(0.01, controls);
    let windup = Attack::new(AttackKind::Crouch(AttackAxis::Vertical))
        .spec()
        .windup;
    arena.wait(windup - 0.03);
    assert!(!arena.fighter(arena.player).airborne);
    assert_eq!(arena.fighter(arena.dummy).health, 100.0);
    arena.wait(0.05);
    assert!(arena.fighter(arena.player).airborne);
    assert!(arena.fighter(arena.dummy).airborne);
    arena.wait(0.35);
    arena.press(AttackAxis::Horizontal);
    assert!(
        matches!(arena.fighter(arena.player).action, Action::Attack(ref a) if a.kind == AttackKind::Air(AttackAxis::Horizontal))
    );
}

#[test]
fn distant_wave_is_reflected_without_disarming_the_distant_attacker() {
    let mut arena = Arena::new();
    arena
        .world
        .get_mut::<Transform>(arena.dummy)
        .unwrap()
        .translation
        .z = -4.0;
    arena.world.get_mut::<Fighter>(arena.dummy).unwrap().action = Action::Parry(0.08);
    let origin = Side::Player.home() + Vec3::Y;
    let mut waves = arena.world.resource_mut::<SwordWaves>();
    waves.spawn(
        arena.player,
        Side::Player,
        origin,
        Vec3::NEG_Z,
        AttackAxis::Vertical,
        30.0,
    );
    waves.active[0].distance = 5.0;
    arena.tick(0.05, Arena::controls());
    assert_eq!(arena.world.resource::<CombatStatus>().blocks, 1);
    assert_eq!(arena.fighter(arena.player).disarmed, 0.0);
    arena.wait(0.55);
    assert_eq!(arena.fighter(arena.player).health, 88.0);
}

#[test]
fn pause_cancels_a_charge_and_reset_clears_waves_and_resources() {
    let mut arena = Arena::new();
    arena.press(AttackAxis::Horizontal);
    arena.tick(0.1, Controls::default());
    assert!(matches!(arena.fighter(arena.player).action, Action::Idle));
    arena
        .world
        .get_mut::<Fighter>(arena.player)
        .unwrap()
        .stamina
        .spend(80.0);
    arena.world.resource_mut::<SwordWaves>().spawn(
        arena.player,
        Side::Player,
        Vec3::Y,
        Vec3::NEG_Z,
        AttackAxis::Horizontal,
        30.0,
    );
    let mut controls = Arena::controls();
    controls.reset = true;
    arena.tick(0.01, controls);
    assert_eq!(arena.fighter(arena.player).stamina.value, 100.0);
    assert!(arena.world.resource::<SwordWaves>().active.is_empty());
}

// Drive real input frames and let the dummy try to parry on the first free
// frame. Every follow-up must connect before it can act again.
#[test]
fn b_dodge_b_uppercut_air_slash_plunge_connects_without_a_gap() {
    for dt in [1.0 / 60.0, 1.0 / 144.0, 1.0 / 30.0] {
        let mut arena = Arena::new();
        arena.press(AttackAxis::Vertical);
        arena.release(AttackAxis::Vertical);
        for _ in 0..100 {
            arena.tick(dt, Arena::controls());
            if arena.fighter(arena.dummy).health < 100.0 {
                break;
            }
        }
        assert_eq!(arena.fighter(arena.dummy).health, 90.0);
        // Buffer Shift during the first hit's freeze, then B during the dash.
        let mut input = Arena::controls();
        input.dodge = true;
        input.sprint = true;
        arena.tick(dt, input);
        let mut saw_dodge = false;
        let mut sent_b = false;
        let mut sent_uppercut = false;
        let mut sent_air = false;
        let mut sent_plunge = false;
        for _ in 0..300 {
            let mut input = Arena::controls();
            input.training_parry = true;
            if matches!(arena.fighter(arena.player).action, Action::Dodge { .. }) {
                saw_dodge = true;
                if !sent_b {
                    input.attacks[1].pressed = true;
                    sent_b = true;
                }
            }
            let hits = arena.world.resource::<CombatStatus>().hits;
            if hits == 2 && !sent_uppercut {
                input.crouch_pressed = true;
                input.attacks[1].pressed = true;
                sent_uppercut = true;
            } else if hits == 4 && !sent_air {
                input.attacks[0].pressed = true;
                sent_air = true;
            } else if hits == 6 && !sent_plunge {
                input.attacks[1].pressed = true;
                sent_plunge = true;
            }
            arena.tick(dt, input);
            assert!(
                !matches!(arena.fighter(arena.dummy).action, Action::Parry(_)),
                "dt {dt}: target escaped after {hits} hits; player {:?}",
                arena.fighter(arena.player).action
            );
            if arena.world.resource::<CombatStatus>().hits == 8 {
                break;
            }
        }
        assert!(
            saw_dodge && sent_b && sent_uppercut && sent_air && sent_plunge,
            "missing transition at dt {dt}"
        );
        assert_eq!(arena.world.resource::<CombatStatus>().hits, 8, "dt {dt}");
        assert_eq!(arena.fighter(arena.dummy).health, 36.0);
    }
}

#[test]
fn buffered_dodge_waits_for_recovery_and_does_not_grant_attack_invulnerability() {
    let mut arena = Arena::new();
    let mut attack = Attack::new(AttackKind::Light(AttackAxis::Vertical, 0));
    attack.elapsed = 0.15;
    arena.world.get_mut::<Fighter>(arena.player).unwrap().action = Action::Attack(attack);
    let mut input = Arena::controls();
    input.dodge = true;
    arena.tick(0.01, input);
    assert!(matches!(
        arena.fighter(arena.player).action,
        Action::Attack(_)
    ));
    arena.wait(0.12);
    assert!(matches!(
        arena.fighter(arena.player).action,
        Action::Dodge { .. }
    ));
    let mut input = Arena::controls();
    input.attacks[1].pressed = true;
    arena.tick(0.01, input);
    arena.wait(0.13);
    let fighter = arena.fighter(arena.player);
    assert!(
        matches!(fighter.action, Action::Attack(ref a) if a.kind == AttackKind::Light(AttackAxis::Vertical, 0))
    );
    assert!(
        !fighter.protected,
        "starting an attack must end dodge protection"
    );
}

#[test]
fn dodge_followup_can_still_hold_a_charge_and_release_a_wave() {
    let mut arena = Arena::new();
    let mut input = Arena::controls();
    input.dodge = true;
    arena.tick(0.01, input);
    arena.wait(0.04);
    arena.press(AttackAxis::Vertical);
    for _ in 0..75 {
        let mut input = Arena::controls();
        input.attacks[1].held = true;
        arena.tick(0.01, input);
    }
    assert!(matches!(
        arena.fighter(arena.player).action,
        Action::Charge(_)
    ));
    assert_eq!(arena.fighter(arena.player).armor(), Armor::Blue);
    arena.release(AttackAxis::Vertical);
    assert!(
        matches!(arena.fighter(arena.player).action, Action::Attack(ref a) if a.kind == AttackKind::Focus(AttackAxis::Vertical))
    );
    arena.wait(0.6);
    assert_eq!(arena.fighter(arena.dummy).health, 70.0);
}

#[test]
fn buffered_inputs_expire_and_pause_discards_them() {
    for pause in [false, true] {
        let mut arena = Arena::new();
        arena
            .world
            .get_mut::<Fighter>(arena.player)
            .unwrap()
            .hit_stop = 0.35;
        let mut input = Arena::controls();
        input.crouch_pressed = true;
        input.attacks[1].pressed = true;
        arena.tick(0.01, input);
        if pause {
            arena.tick(0.01, Controls::default());
        }
        arena.wait(0.65);
        assert!(matches!(arena.fighter(arena.player).action, Action::Idle));
        assert_eq!(arena.fighter(arena.dummy).health, 100.0);
    }
}

#[test]
fn one_air_left_click_hits_twice_and_plays_two_swings() {
    for dt in [1.0 / 144.0, 1.0 / 30.0, 0.5] {
        let mut arena = Arena::new();
        for (entity, z) in [(arena.player, 0.5), (arena.dummy, -0.5)] {
            let mut fighter = arena.world.get_mut::<Fighter>(entity).unwrap();
            fighter.airborne = true;
            fighter.vertical_speed = core::JUMP_SPEED;
            arena
                .world
                .get_mut::<Transform>(entity)
                .unwrap()
                .translation = Vec3::new(0.0, 2.0, z);
        }
        arena.press(AttackAxis::Horizontal);
        let mut swings = 0;
        let mut elapsed = 0.0;
        while elapsed < 1.0 {
            arena.tick(dt, Arena::controls());
            swings += arena
                .world
                .resource::<Sounds>()
                .0
                .iter()
                .filter(|&&cue| cue == "swing1")
                .count();
            elapsed += dt;
        }
        assert_eq!(arena.world.resource::<CombatStatus>().hits, 2, "dt {dt}");
        assert_eq!(arena.fighter(arena.dummy).health, 88.0, "dt {dt}");
        assert_eq!(swings, 2, "dt {dt}");
    }
}

#[test]
fn multi_hit_moves_deal_their_budget_once_across_frame_rates() {
    for (kind, count, damage) in [
        (AttackKind::Light(AttackAxis::Horizontal, 1), 3, 15.0),
        (AttackKind::Crouch(AttackAxis::Vertical), 2, 14.0),
        (AttackKind::Air(AttackAxis::Vertical), 2, 18.0),
        (AttackKind::Riposte, 2, 35.0),
    ] {
        for dt in [1.0 / 144.0, 1.0 / 30.0, 0.5] {
            let mut arena = Arena::new();
            for (entity, z) in [(arena.player, 0.5), (arena.dummy, -0.5)] {
                let mut fighter = arena.world.get_mut::<Fighter>(entity).unwrap();
                let aerial = matches!(kind, AttackKind::Air(_));
                fighter.airborne = aerial;
                fighter.vertical_speed = if aerial { core::JUMP_SPEED } else { 0.0 };
                arena
                    .world
                    .get_mut::<Transform>(entity)
                    .unwrap()
                    .translation = Vec3::new(0.0, if aerial { 2.0 } else { 0.0 }, z);
            }
            arena.world.get_mut::<Fighter>(arena.player).unwrap().action =
                Action::Attack(Attack::new(kind));
            let mut elapsed = 0.0;
            while elapsed < 2.0 {
                arena.tick(dt, Arena::controls());
                elapsed += dt;
            }
            assert_eq!(
                arena.world.resource::<CombatStatus>().hits,
                count,
                "{kind:?}, dt {dt}"
            );
            assert_eq!(
                arena.fighter(arena.dummy).health,
                100.0 - damage,
                "{kind:?}, dt {dt}"
            );
        }
    }
}

#[test]
fn uppercut_second_hit_keeps_the_existing_launch_velocity() {
    let mut arena = Arena::new();
    let mut input = Arena::controls();
    input.crouch_pressed = true;
    input.attacks[1].pressed = true;
    arena.tick(0.01, input);
    for _ in 0..100 {
        arena.tick(0.01, Arena::controls());
        if arena.world.resource::<CombatStatus>().hits == 2 {
            assert!(
                arena.fighter(arena.dummy).vertical_speed < 6.0,
                "second hit relaunched the target"
            );
            assert!(arena.fighter(arena.dummy).airborne);
            assert_eq!(arena.fighter(arena.dummy).health, 86.0);
            return;
        }
    }
    panic!("uppercut failed to land both hits");
}

#[test]
fn uppercut_second_hit_can_launch_when_the_first_missed() {
    let mut arena = Arena::new();
    arena
        .world
        .get_mut::<Transform>(arena.dummy)
        .unwrap()
        .translation
        .z = -4.0;
    let mut input = Arena::controls();
    input.crouch_pressed = true;
    input.attacks[1].pressed = true;
    arena.tick(0.01, input);
    arena.wait(0.44);
    assert_eq!(arena.world.resource::<CombatStatus>().hits, 0);
    arena
        .world
        .get_mut::<Transform>(arena.dummy)
        .unwrap()
        .translation
        .z = 0.0;
    arena.wait(0.08);
    assert_eq!(arena.world.resource::<CombatStatus>().hits, 1);
    assert_eq!(arena.fighter(arena.dummy).health, 93.0);
    assert!(arena.fighter(arena.dummy).airborne);
}

#[test]
fn crouch_toggles_on_each_press_and_survives_key_release() {
    let mut arena = Arena::new();
    let mut input = Arena::controls();
    input.crouch_pressed = true;
    arena.tick(0.01, input);
    assert!(arena.fighter(arena.player).crouched);
    arena.wait(0.3);
    assert!(
        arena.fighter(arena.player).crouched,
        "releasing C must not stand up"
    );
    let mut input = Arena::controls();
    input.crouch_pressed = true;
    arena.tick(0.01, input);
    assert!(!arena.fighter(arena.player).crouched);
}

#[test]
fn crouched_right_click_uses_uppercut_without_holding_c() {
    let mut arena = Arena::new();
    let mut input = Arena::controls();
    input.crouch_pressed = true;
    arena.tick(0.01, input);
    arena.wait(0.1);
    arena.press(AttackAxis::Vertical);
    assert!(matches!(arena.fighter(arena.player).action,
        Action::Attack(ref attack) if attack.kind == AttackKind::Crouch(AttackAxis::Vertical)));
}

#[test]
fn tapped_c_starts_slide_and_slide_finishes_crouched() {
    let mut arena = Arena::new();
    arena.begin_sprint(Vec2::X);
    let mut input = Arena::controls();
    input.movement = Vec2::X;
    input.crouch_pressed = true;
    arena.tick(0.01, input);
    arena.wait(0.10);
    assert!(matches!(
        arena.fighter(arena.player).action,
        Action::Slide { .. }
    ));
    arena.wait(0.6);
    assert!(matches!(arena.fighter(arena.player).action, Action::Idle));
    assert!(arena.fighter(arena.player).crouched);
}

#[test]
fn sprint_stays_on_after_shift_release_until_all_directions_are_released() {
    let mut arena = Arena::new();
    let mut input = Arena::controls();
    input.dodge = true;
    input.sprint = true;
    input.movement = Vec2::X;
    arena.tick(0.01, input);
    for _ in 0..35 {
        let mut input = Arena::controls();
        input.sprint = true;
        input.movement = Vec2::X;
        arena.tick(0.01, input);
    }
    assert!(arena.fighter(arena.player).sprinting);
    let mut input = Arena::controls();
    input.movement = Vec2::X;
    arena.tick(0.01, input);
    assert!(
        arena.fighter(arena.player).sprinting,
        "Shift release must not stop sprinting"
    );
    arena.tick(0.01, Arena::controls());
    assert!(!arena.fighter(arena.player).sprinting);
    let mut input = Arena::controls();
    input.movement = Vec2::X;
    arena.tick(0.01, input);
    assert!(
        !arena.fighter(arena.player).sprinting,
        "a new movement needs a new long Shift press"
    );
}

#[test]
fn sprint_survives_direction_changes_and_opposing_held_keys() {
    let mut arena = Arena::new();
    arena.begin_sprint(Vec2::X);
    let mut input = Arena::controls();
    input.movement_held = true;
    // W+S can cancel the vector while physical direction keys remain down.
    arena.tick(0.01, input);
    assert!(arena.fighter(arena.player).sprint_latched);
    let mut input = Arena::controls();
    input.movement_held = true;
    input.movement = Vec2::Y;
    arena.tick(0.01, input);
    assert!(arena.fighter(arena.player).sprinting);
    arena.tick(0.01, Arena::controls());
    assert!(!arena.fighter(arena.player).sprint_latched);
}

#[test]
fn short_shift_press_does_not_latch_sprint() {
    let mut arena = Arena::new();
    let mut input = Arena::controls();
    input.dodge = true;
    input.sprint = true;
    input.movement = Vec2::X;
    arena.tick(0.01, input);
    for _ in 0..50 {
        let mut input = Arena::controls();
        input.movement = Vec2::X;
        arena.tick(0.01, input);
    }
    assert!(!arena.fighter(arena.player).sprinting);
    assert!(!arena.fighter(arena.player).sprint_latched);
}

#[test]
fn pause_and_reset_clear_sticky_stances() {
    for reset in [false, true] {
        let mut arena = Arena::new();
        arena.begin_sprint(Vec2::X);
        let mut input = Arena::controls();
        input.crouch_pressed = true;
        input.movement = Vec2::X;
        arena.tick(0.01, input);
        assert!(arena.fighter(arena.player).crouched);
        assert!(arena.fighter(arena.player).sprint_latched);
        let mut input = if reset {
            Arena::controls()
        } else {
            Controls::default()
        };
        input.reset = reset;
        arena.tick(0.01, input);
        let fighter = arena.fighter(arena.player);
        assert!(!fighter.crouched);
        assert!(!fighter.sprint_latched);
        assert_eq!(fighter.sprint_hold_time, 0.0);
    }
}

#[test]
fn crouch_then_right_click_can_be_buffered_across_separate_frames() {
    let mut arena = Arena::new();
    let mut attack = Attack::new(AttackKind::Light(AttackAxis::Vertical, 0));
    attack.elapsed = 0.15;
    arena.world.get_mut::<Fighter>(arena.player).unwrap().action = Action::Attack(attack);
    let mut input = Arena::controls();
    input.crouch_pressed = true;
    arena.tick(0.01, input);
    arena.wait(0.02);
    arena.press(AttackAxis::Vertical);
    arena.wait(0.2);
    assert!(matches!(arena.fighter(arena.player).action,
        Action::Attack(ref attack) if attack.kind == AttackKind::Crouch(AttackAxis::Vertical)));
    assert!(arena.fighter(arena.player).crouched);
}

#[test]
fn enemy_ai_repeats_charged_horizontal_attacks_without_player_input() {
    let mut arena = Arena::new();
    arena.world.entity_mut(arena.dummy).insert(EnemyAi);
    arena.tick(0.01, Arena::controls());
    assert!(
        matches!(arena.fighter(arena.dummy).action,Action::Charge(ref charge) if charge.axis == AttackAxis::Horizontal)
    );
    let mut releases = 0;
    for _ in 0..300 {
        arena.tick(0.01, Arena::controls());
        releases += arena
            .world
            .resource::<Sounds>()
            .0
            .iter()
            .filter(|&&cue| cue == "sword-energy")
            .count();
    }
    assert!(releases >= 2, "AI must finish recovery and charge again");
    assert!(
        arena.fighter(arena.player).health < 100.0,
        "horizontal sword energy must hit the player"
    );
}

#[test]
fn enemy_ai_tracks_during_charge_and_locks_heading_on_release() {
    let mut arena = Arena::new();
    arena.world.entity_mut(arena.dummy).insert(EnemyAi);
    arena
        .world
        .get_mut::<Transform>(arena.player)
        .unwrap()
        .translation = Vec3::new(4.0, 0.0, -0.6);
    arena.world.resource_mut::<OrbitCamera>().heading = 0.7;
    arena.tick(0.01, Arena::controls());
    assert!((arena.fighter(arena.dummy).facing + std::f32::consts::FRAC_PI_2).abs() < 0.001);
    // Mouse release belongs to the player and must not release the enemy charge.
    arena.release(AttackAxis::Horizontal);
    assert!(matches!(
        arena.fighter(arena.dummy).action,
        Action::Charge(_)
    ));
    arena
        .world
        .get_mut::<Transform>(arena.player)
        .unwrap()
        .translation = Vec3::new(-4.0, 0.0, -0.6);
    arena.wait(0.80);
    assert!(matches!(arena.fighter(arena.dummy).action,
        Action::Attack(ref attack) if attack.kind == AttackKind::Focus(AttackAxis::Horizontal)));
    let facing = arena.fighter(arena.dummy).facing;
    assert!((facing - std::f32::consts::FRAC_PI_2).abs() < 0.001);
    arena
        .world
        .get_mut::<Transform>(arena.player)
        .unwrap()
        .translation = Vec3::new(0.0, 0.0, 4.0);
    arena.wait(0.05);
    assert_eq!(arena.fighter(arena.dummy).facing, facing);
}

#[test]
fn enemy_ai_restarts_after_interruption_and_training_override() {
    let mut arena = Arena::new();
    arena.world.entity_mut(arena.dummy).insert(EnemyAi);
    arena.tick(0.01, Arena::controls());
    arena.press(AttackAxis::Horizontal);
    arena.release(AttackAxis::Horizontal);
    arena.wait(0.20);
    assert!(arena.fighter(arena.dummy).health < 100.0);
    assert!(matches!(
        arena.fighter(arena.dummy).action,
        Action::Stagger(_)
    ));
    arena.wait(0.45);
    assert!(matches!(arena.fighter(arena.dummy).action,
        Action::Charge(ref charge) if charge.elapsed < CHARGE_READY));
    let mut input = Arena::controls();
    input.training_parry = true;
    arena.tick(0.01, input);
    assert!(matches!(
        arena.fighter(arena.dummy).action,
        Action::Parry(_)
    ));
    arena.wait(PARRY_DURATION + 0.05);
    assert!(matches!(
        arena.fighter(arena.dummy).action,
        Action::Charge(_)
    ));
}

#[test]
fn enemy_ai_waits_for_weapon_and_survives_pause_and_reset() {
    let mut arena = Arena::new();
    arena.world.entity_mut(arena.dummy).insert(EnemyAi);
    arena
        .world
        .get_mut::<Fighter>(arena.dummy)
        .unwrap()
        .disarmed = 0.30;
    arena.wait(0.20);
    assert!(matches!(arena.fighter(arena.dummy).action, Action::Idle));
    arena.wait(0.15);
    assert!(matches!(
        arena.fighter(arena.dummy).action,
        Action::Charge(_)
    ));
    arena.tick(0.01, Controls::default());
    assert!(matches!(arena.fighter(arena.dummy).action, Action::Idle));
    arena.tick(0.01, Arena::controls());
    assert!(matches!(arena.fighter(arena.dummy).action,
        Action::Charge(ref charge) if charge.elapsed < 0.02));
    let mut input = Arena::controls();
    input.reset = true;
    arena.tick(0.01, input);
    assert!(arena.world.get::<EnemyAi>(arena.dummy).is_some());
    arena.tick(0.01, Arena::controls());
    assert!(matches!(
        arena.fighter(arena.dummy).action,
        Action::Charge(_)
    ));
    let mut player = arena.world.get_mut::<Fighter>(arena.player).unwrap();
    player.health = 0.0;
    player.action = Action::Down(2.0);
    arena.tick(0.01, Arena::controls());
    assert!(matches!(arena.fighter(arena.dummy).action, Action::Idle));
}
