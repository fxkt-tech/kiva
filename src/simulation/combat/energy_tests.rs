//! Sun, moon, rage, the riposte and weapon durability through the real fixed step.
use super::*;

fn energy(app: &App, player: Entity) -> &Energy {
    app.world().get::<Energy>(player).unwrap()
}
fn charge(app: &mut App, player: Entity, sun: u8, moon: u8, rage: f32) {
    *app.world_mut().get_mut::<Energy>(player).unwrap() = Energy::with(sun, moon, rage);
}
fn combat(app: &App, player: Entity) -> &Combatant {
    app.world().get::<Combatant>(player).unwrap()
}
fn durability(app: &App, player: Entity, pair: WeaponPair) -> u8 {
    app.world().get::<Arsenal>(player).unwrap().durability(pair)
}
/// One whole ordinary attack, from the press until it has retired.
fn attack(app: &mut App, player: Entity) {
    request(app, player, CombatIntent::request_attack);
    tick(app, 45);
    assert!(combat(app, player).active_action().is_none());
}

#[test]
fn every_damaging_hit_adds_one_sun_and_a_full_sun_empowers_one_melee_skill() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 2.0, 0.0);
    for hits in 1..=3 {
        attack(&mut app, p);
        assert_eq!(energy(&app, p).sun(), hits);
    }
    assert_eq!(hp(&app, e), 1000.0 - 3.0 * 24.0);
    // An attack which reaches nobody earns nothing.
    teleport_actor(app.world_mut(), e, Vec3::NEG_Z * 20.0);
    tick(&mut app, 2);
    attack(&mut app, p);
    assert_eq!(energy(&app, p).sun(), 3);
    teleport_actor(app.world_mut(), e, Vec3::NEG_Z * 2.0);
    tick(&mut app, 12);

    charge(&mut app, p, Energy::SUN_MAX, 0, 0.0);
    let before = hp(&app, e);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_melee_skill);
    tick(&mut app, 13);
    // 20 × 1.5; the meter is spent, this hit starts it again, rage grows by one.
    assert_eq!(hp(&app, e), before - 30.0);
    assert_eq!(energy(&app, p).sun(), 1);
    assert_eq!(energy(&app, p).rage(), 1.0);
    assert!(!energy(&app, p).is_enraged());

    // The next one, three seconds later, is ordinary again.
    tick(&mut app, 200);
    let before = hp(&app, e);
    request(&mut app, p, CombatIntent::request_melee_skill);
    tick(&mut app, 13);
    assert_eq!(hp(&app, e), before - 20.0);
    assert_eq!(energy(&app, p).rage(), 1.0);
}

#[test]
fn a_deflect_adds_one_moon_and_a_full_moon_empowers_one_ranged_skill() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.5, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    edge(&mut app, p, DefenseEdge::Released);
    tick(&mut app, 1);
    strike(&mut app, p, e);
    assert_eq!(hp(&app, p), 1000.0);
    assert_eq!(energy(&app, p).moon(), 1);
    assert_eq!(energy(&app, p).sun(), 0);
    // A hit which lands on the player earns nothing.
    tick(&mut app, 30);
    strike(&mut app, p, e);
    assert_eq!(hp(&app, p), 988.0);
    assert_eq!(energy(&app, p).moon(), 1);

    charge(&mut app, p, 0, Energy::MOON_MAX, 0.0);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 12);
    // 40 × 1.5; the ranged skill builds two rage.
    assert_eq!(hp(&app, e), 1000.0 - 60.0);
    assert_eq!(energy(&app, p).moon(), 0);
    assert_eq!(energy(&app, p).rage(), 2.0);
}

#[test]
fn a_parry_strikes_back_for_a_little_damage_stuns_and_adds_one_sun() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 1.5, 0.0);
    edge(&mut app, p, DefenseEdge::Pressed);
    tick(&mut app, 13);
    edge(&mut app, p, DefenseEdge::Released);
    tick(&mut app, 1);
    strike(&mut app, p, e);
    assert_eq!(hp(&app, p), 1000.0);
    assert_eq!(hp(&app, e), 990.0);
    assert!(app.world().get::<Enemy>(e).unwrap().stun_source().is_some());
    assert_eq!(energy(&app, p).sun(), 1);
    assert_eq!(energy(&app, p).moon(), 0);
    let events: Vec<_> = app
        .world()
        .resource::<CombatFeedback>()
        .events()
        .map(|event| (event.kind, event.actor))
        .collect();
    let parry = events
        .iter()
        .position(|event| *event == (CombatEffect::Parry, p))
        .expect("the player parried");
    assert_eq!(events[parry + 1], (CombatEffect::Hit, e));
}

#[test]
fn rage_needs_a_full_meter_then_doubles_attacks_empowers_skills_and_chains_combos() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 2.0, 0.0);
    charge(&mut app, p, 0, 0, Energy::RAGE_MAX - 1.0);
    request(&mut app, p, CombatIntent::request_rage);
    tick(&mut app, 1);
    assert!(!energy(&app, p).is_enraged());
    assert!(!energy(&app, p).rage_ready());

    charge(&mut app, p, 0, 0, Energy::RAGE_MAX);
    assert!(energy(&app, p).rage_ready());
    request(&mut app, p, CombatIntent::request_rage);
    tick(&mut app, 1);
    assert!(energy(&app, p).is_enraged());
    assert!(!energy(&app, p).rage_ready());
    assert_eq!(combat(&app, p).status(), CombatStatus::Enraged);
    assert_eq!(
        app.world()
            .resource::<CombatFeedback>()
            .events()
            .filter(|event| event.kind == CombatEffect::Enrage && event.actor == p)
            .count(),
        1
    );

    attack(&mut app, p);
    assert_eq!(hp(&app, e), 1000.0 - 48.0);
    // The skill is empowered without a full sun, spends none and builds no rage.
    let sun = energy(&app, p).sun();
    let rage = energy(&app, p).rage();
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_melee_skill);
    tick(&mut app, 13);
    assert_eq!(hp(&app, e), 1000.0 - 48.0 - 30.0);
    assert_eq!(energy(&app, p).sun(), sun + 1);
    assert!(energy(&app, p).rage() < rage);
    // The launch opened the window; the combo follows without a Tab.
    tick(&mut app, 1);
    assert!(combat(&app, p).is_combo_active());

    // The state lasts twenty seconds of rule time and ends empty.
    tick(&mut app, 1100);
    assert!(energy(&app, p).is_enraged());
    tick(&mut app, 100);
    assert!(!energy(&app, p).is_enraged());
    assert_eq!(energy(&app, p).rage(), 0.0);
    attack(&mut app, p);
    let before = hp(&app, e);
    attack(&mut app, p);
    assert_eq!(hp(&app, e), before - 24.0);
}

#[test]
fn starting_actions_wears_the_pair_and_a_worn_out_pair_rotates_once_idle() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 2.0, 0.0);
    let (hammer, chain) = (WeaponPair::HammerMusket, WeaponPair::ChainAxeBow);
    assert_eq!(durability(&app, p, hammer), Arsenal::DURABILITY);
    attack(&mut app, p);
    assert_eq!(durability(&app, p, hammer), 49);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 40);
    assert_eq!(durability(&app, p, hammer), 39);
    assert_eq!(durability(&app, p, chain), Arsenal::DURABILITY);
    // A rejected request costs nothing.
    request(&mut app, p, CombatIntent::request_ranged_skill);
    tick(&mut app, 1);
    assert_eq!(durability(&app, p, hammer), 39);

    // The last point: the attack still completes with the worn pair.
    app.world_mut()
        .get_mut::<Arsenal>(p)
        .unwrap()
        .set(hammer, 1);
    let before = hp(&app, e);
    request(&mut app, p, CombatIntent::request_attack);
    tick(&mut app, 10);
    assert_eq!(hp(&app, e), before - 24.0);
    assert_eq!(durability(&app, p, hammer), 0);
    assert_eq!(combat(&app, p).weapon_pair(), hammer);
    tick(&mut app, 35);
    assert_eq!(combat(&app, p).weapon_pair(), chain);
    assert_eq!(combat(&app, p).status(), CombatStatus::WeaponExhausted);
    assert_eq!(durability(&app, p, hammer), Arsenal::DURABILITY);
}

#[test]
fn a_worn_out_pair_waits_for_its_open_combo_window_and_z_cycles_without_repair() {
    let (mut app, p, e) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 2.0, 0.0);
    let (hammer, chain) = (WeaponPair::HammerMusket, WeaponPair::ChainAxeBow);
    app.world_mut()
        .get_mut::<Arsenal>(p)
        .unwrap()
        .set(hammer, 10);
    lock(&mut app, p, e);
    request(&mut app, p, CombatIntent::request_melee_skill);
    tick(&mut app, 36);
    // The skill has retired, but its window still belongs to the hammer.
    assert!(combat(&app, p).active_action().is_none());
    assert!(combat(&app, p).combo_remaining() > 0.0);
    assert_eq!(durability(&app, p, hammer), 0);
    assert_eq!(combat(&app, p).weapon_pair(), hammer);
    tick(&mut app, 60);
    assert_eq!(combat(&app, p).combo_remaining(), 0.0);
    assert_eq!(combat(&app, p).weapon_pair(), chain);
    assert_eq!(durability(&app, p, hammer), Arsenal::DURABILITY);

    app.world_mut()
        .get_mut::<Arsenal>(p)
        .unwrap()
        .set(chain, 30);
    tick(&mut app, 120);
    request(&mut app, p, CombatIntent::request_next_pair);
    tick(&mut app, 1);
    assert_eq!(combat(&app, p).weapon_pair(), hammer);
    assert_eq!(combat(&app, p).status(), CombatStatus::PairSwitched);
    assert_eq!(durability(&app, p, chain), 30);
    request(&mut app, p, CombatIntent::request_next_pair);
    tick(&mut app, 1);
    assert_eq!(combat(&app, p).weapon_pair(), chain);
}

#[test]
fn respawning_empties_the_meters_ends_rage_and_repairs_every_pair() {
    let (mut app, p, _) = arena(WeaponPair::HammerMusket, Vec3::NEG_Z * 2.0, 0.0);
    charge(&mut app, p, 4, 7, Energy::RAGE_MAX);
    request(&mut app, p, CombatIntent::request_rage);
    tick(&mut app, 1);
    assert!(energy(&app, p).is_enraged());
    app.world_mut()
        .get_mut::<Arsenal>(p)
        .unwrap()
        .set(WeaponPair::ChainAxeBow, 3);
    // Falling out of the arena defeats the player; three seconds later they respawn.
    teleport_actor(app.world_mut(), p, Vec3::new(0.0, -13.0, 0.0));
    tick(&mut app, 2);
    assert_eq!(hp(&app, p), 0.0);
    tick(&mut app, 190);
    assert_eq!(hp(&app, p), 1000.0);
    let energy = energy(&app, p);
    assert_eq!((energy.sun(), energy.moon(), energy.rage()), (0, 0, 0.0));
    assert!(!energy.is_enraged());
    assert_eq!(
        durability(&app, p, WeaponPair::ChainAxeBow),
        Arsenal::DURABILITY
    );
}
