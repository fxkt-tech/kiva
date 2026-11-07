//! Builtin champion wiring is exercised through the real content and commands.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::{gameplay::prepare_world, simulation::*};

struct Arena {
    app: App,
    events: MessageCursor<SimulationEvent>,
}

impl Arena {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        prepare_world(app.world_mut());
        app.insert_resource(NavigationMap::new(100.0, 1.0, vec![]));
        app.world_mut().resource_mut::<SimulationControl>().paused = false;
        Self {
            app,
            events: default(),
        }
    }

    fn hero(&mut self, id: u16, team: Team, position: Vec2) -> Entity {
        spawn_character(
            self.app.world_mut(),
            CharacterSpawnSpec {
                character: CharacterId(id),
                team,
                position,
                loadout: LoadoutSelection::CharacterDefault,
                respawn_delay: Some(3.0),
            },
        )
        .unwrap()
    }

    fn target(&mut self, kind: UnitKind, position: Vec2) -> Entity {
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "Target".into(),
                team: Team::Enemy,
                position,
                stats: UnitStats {
                    kind,
                    max_health: 1_000.0,
                    mana_regen: 0.0,
                    base_attributes: BaseAttributes {
                        move_speed: 0.0,
                        attack_speed: 0.0,
                        ..default()
                    },
                    ..default()
                },
                abilities: [None; 4],
                respawn_delay: None,
            },
        )
        .unwrap()
    }

    fn command(&mut self, unit: Entity, command: UnitCommand) {
        self.app
            .world_mut()
            .resource_mut::<CommandQueue>()
            .submit(unit, command);
    }

    fn cast(&mut self, unit: Entity, slot: usize, target: AbilityTarget) {
        self.command(unit, UnitCommand::CastAbility { slot, target });
    }

    fn ticks(&mut self, count: usize) {
        for _ in 0..count {
            self.app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn health(&self, unit: Entity) -> f32 {
        self.app.world().get::<UnitState>(unit).unwrap().health()
    }

    fn committed(&mut self, unit: Entity, ability: AbilityId) -> bool {
        self.events
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .any(|event| {
                matches!(event, SimulationEvent::AbilityCommitted {
                unit: actor, ability: spell, ..
            } if *actor == unit && *spell == ability)
            })
    }
}

#[test]
fn real_roster_preserves_base_stats_and_applies_learned_standard_once() {
    let mut arena = Arena::new();
    let jarvan = arena.hero(1, Team::Ally, Vec2::ZERO);
    let ashe = arena.hero(2, Team::Ally, Vec2::new(-4.0, 0.0));
    let j = arena.app.world().get::<Unit>(jarvan).unwrap();
    let a = arena.app.world().get::<Unit>(ashe).unwrap();
    assert_eq!(j.definition().max_health, 640.0);
    assert_eq!(a.definition().max_health, 610.0);
    assert_eq!(j.attributes().attack_range(), 1.75);
    assert_eq!(a.attributes().attack_range(), 6.0);
    assert!((j.attributes().attack_speed() - 0.7896).abs() < 1e-5);
    assert!((a.attributes().attack_speed() - 0.658).abs() < 1e-5);

    arena.cast(jarvan, 2, AbilityTarget::Point(Vec2::new(4.0, 0.0)));
    arena.ticks(35);
    let j = arena.app.world().get::<Unit>(jarvan).unwrap();
    let a = arena.app.world().get::<Unit>(ashe).unwrap();
    assert!((j.attributes().attack_speed() - 0.9212).abs() < 1e-5);
    assert!((a.attributes().attack_speed() - 0.7896).abs() < 1e-5);
    arena.ticks(490);
    let j = arena.app.world().get::<Unit>(jarvan).unwrap();
    let a = arena.app.world().get::<Unit>(ashe).unwrap();
    assert!((j.attributes().attack_speed() - 0.7896).abs() < 1e-5);
    assert!((a.attributes().attack_speed() - 0.658).abs() < 1e-5);
}

#[test]
fn real_dragon_strike_hits_through_multiple_enemies_and_reduces_armor_after_damage() {
    let mut arena = Arena::new();
    let jarvan = arena.hero(1, Team::Ally, Vec2::ZERO);
    let front = arena.target(UnitKind::Minion, Vec2::new(3.0, 0.0));
    let back = arena.target(UnitKind::Hero, Vec2::new(6.0, 0.0));
    let changes = &mut arena.app.world_mut().resource_mut::<AttributeChanges>();
    let source = changes.allocate_source().unwrap();
    changes.upsert(
        back,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                armor: 100.0,
                ..default()
            },
            ..default()
        },
    );
    arena.cast(jarvan, 0, AbilityTarget::Direction(Vec2::X));
    arena.ticks(30);
    assert!(arena.committed(jarvan, AbilityId(11)));
    assert!((arena.health(front) - 910.0).abs() < 1e-4);
    assert!((arena.health(back) - 955.0).abs() < 1e-4);
    assert_eq!(
        arena
            .app
            .world()
            .get::<Unit>(back)
            .unwrap()
            .attributes()
            .armor(),
        90.0
    );
    arena.ticks(190);
    assert_eq!(
        arena
            .app
            .world()
            .get::<Unit>(back)
            .unwrap()
            .attributes()
            .armor(),
        100.0
    );
}

#[test]
fn real_standard_then_strike_dashes_to_the_flag_and_knocks_up_enemies_on_the_path() {
    let mut arena = Arena::new();
    let jarvan = arena.hero(1, Team::Ally, Vec2::ZERO);
    let target = arena.target(UnitKind::Hero, Vec2::new(3.0, 0.0));
    arena.cast(jarvan, 2, AbilityTarget::Point(Vec2::new(6.0, 0.0)));
    arena.ticks(35);
    arena.cast(jarvan, 0, AbilityTarget::Direction(Vec2::X));
    arena.ticks(50);
    assert!(
        arena
            .app
            .world()
            .get::<UnitPose>(jarvan)
            .unwrap()
            .position()
            .distance(Vec2::new(6.0, 0.0))
            < 1e-4
    );
    assert!((arena.health(target) - 910.0).abs() < 1e-4);
    assert!(
        arena
            .app
            .world()
            .get::<UnitStatuses>(target)
            .unwrap()
            .iter()
            .any(|status| status.kind() == StatusKind::Airborne)
    );
}

#[test]
fn real_martial_cadence_uses_pre_attack_health_and_does_not_repeat_inside_target_cooldown() {
    let mut arena = Arena::new();
    let jarvan = arena.hero(1, Team::Ally, Vec2::ZERO);
    let target = arena.target(UnitKind::Hero, Vec2::X);
    arena.command(jarvan, UnitCommand::AttackTarget(target));
    arena.ticks(30);
    assert!((arena.health(target) - 856.0).abs() < 1e-4);
    arena.ticks(75);
    assert!((arena.health(target) - 792.0).abs() < 1e-4);
}

#[test]
fn real_volley_can_hit_several_targets_but_overlapping_arrows_only_damage_each_once() {
    let mut arena = Arena::new();
    let ashe = arena.hero(2, Team::Ally, Vec2::ZERO);
    let center = arena.target(UnitKind::Hero, Vec2::new(5.0, 0.0));
    let side = arena.target(UnitKind::Minion, Vec2::new(5.0, 1.2));
    arena.cast(ashe, 1, AbilityTarget::Direction(Vec2::X));
    arena.ticks(70);
    assert!(arena.committed(ashe, AbilityId(22)));
    assert!((arena.health(center) - 940.0).abs() < 1e-4);
    assert!((arena.health(side) - 940.0).abs() < 1e-4);
    assert!(
        arena
            .app
            .world()
            .get::<UnitStatuses>(center)
            .unwrap()
            .iter()
            .any(|status| status.kind() == StatusKind::Slow)
    );
}

#[test]
fn real_crystal_arrow_ignores_front_minion_and_stuns_the_hero() {
    let mut arena = Arena::new();
    let ashe = arena.hero(2, Team::Ally, Vec2::ZERO);
    let front = arena.target(UnitKind::Minion, Vec2::new(3.0, 0.0));
    let hero = arena.target(UnitKind::Hero, Vec2::new(10.0, 0.0));
    arena.cast(ashe, 3, AbilityTarget::Direction(Vec2::X));
    arena.ticks(65);
    assert!(arena.committed(ashe, AbilityId(24)));
    assert_eq!(arena.health(front), 1_000.0);
    assert!((arena.health(hero) - 800.0).abs() < 1e-4);
    assert!(
        arena
            .app
            .world()
            .get::<UnitStatuses>(hero)
            .unwrap()
            .iter()
            .any(|status| status.kind() == StatusKind::Stun)
    );
}

#[test]
fn real_crystal_arrow_leaves_bounded_temporary_vision_when_it_misses_at_the_map_edge() {
    let mut arena = Arena::new();
    let ashe = arena.hero(2, Team::Ally, Vec2::ZERO);
    arena.cast(ashe, 3, AbilityTarget::Direction(Vec2::X));
    let mut seen_projectile = false;
    for _ in 0..500 {
        arena.ticks(1);
        let world = arena.app.world_mut();
        let flying = world.query::<&Projectile>().iter(world).next().is_some();
        if flying {
            seen_projectile = true;
        } else if seen_projectile {
            break;
        }
    }
    assert!(
        seen_projectile,
        "the arrow must really fly before terminating"
    );
    let world = arena.app.world_mut();
    assert_eq!(world.query::<&Projectile>().iter(world).count(), 0);
    let point = world
        .query::<&BattlefieldObject>()
        .iter(world)
        .find(|object| object.kind() == ObjectKind::Reveal)
        .expect("a missed arrow must retain terminal vision")
        .position();
    assert!(point.x > 98.0, "the arrow must reach the actual map edge");
    assert!(point_visible(world, Team::Ally, point + Vec2::Y * 3.4));
    assert!(
        !point_visible(world, Team::Ally, point + Vec2::Y * 3.8),
        "terminal sight uses vision radius, not the larger damage radius"
    );
    arena.ticks(61);
    assert!(!point_visible(arena.app.world(), Team::Ally, point));
}

#[test]
fn real_aegis_shields_without_damaging_enemies_and_counts_nearby_heroes() {
    let mut arena = Arena::new();
    let jarvan = arena.hero(1, Team::Ally, Vec2::ZERO);
    let hero = arena.target(UnitKind::Hero, Vec2::new(3.0, 0.0));
    let minion = arena.target(UnitKind::Minion, Vec2::new(4.0, 0.0));
    arena.cast(jarvan, 1, AbilityTarget::SelfCast);
    arena.ticks(2);
    let shield = arena
        .app
        .world()
        .get::<UnitMechanics>(jarvan)
        .unwrap()
        .shield();
    assert!((shield - 68.32).abs() < 1e-4);
    assert_eq!(arena.health(hero), 1_000.0);
    assert_eq!(arena.health(minion), 1_000.0);
    arena.ticks(250);
    assert_eq!(
        arena
            .app
            .world()
            .get::<UnitMechanics>(jarvan)
            .unwrap()
            .shield(),
        0.0
    );
}

#[test]
fn real_focus_requires_four_attacks_then_activates_and_consumes_its_stacks() {
    let mut arena = Arena::new();
    let ashe = arena.hero(2, Team::Ally, Vec2::ZERO);
    let target = arena.target(UnitKind::Hero, Vec2::new(5.0, 0.0));
    assert_eq!(
        preview_cast(
            arena.app.world(),
            ashe,
            CastSource::AbilitySlot(0),
            AbilityTarget::SelfCast
        ),
        Err(Rejection::InsufficientStacks)
    );
    arena.command(ashe, UnitCommand::AttackTarget(target));
    for _ in 0..400 {
        arena.ticks(1);
        if arena
            .app
            .world()
            .get::<UnitMechanics>(ashe)
            .unwrap()
            .focus()
            == 4
        {
            break;
        }
    }
    assert_eq!(
        arena
            .app
            .world()
            .get::<UnitMechanics>(ashe)
            .unwrap()
            .focus(),
        4
    );
    arena.command(ashe, UnitCommand::Stop);
    arena.cast(ashe, 0, AbilityTarget::SelfCast);
    arena.ticks(2);
    assert!(arena.committed(ashe, AbilityId(21)));
    let mechanics = arena.app.world().get::<UnitMechanics>(ashe).unwrap();
    assert!(mechanics.empowered());
    assert_eq!(mechanics.focus(), 0);
    assert_eq!(mechanics.arrows(), 5);
    assert!(
        (arena
            .app
            .world()
            .get::<Unit>(ashe)
            .unwrap()
            .attributes()
            .attack_speed()
            - 0.7896)
            .abs()
            < 1e-5
    );
}

#[test]
fn real_hawkshot_reveals_an_unknown_enemy_without_dealing_damage() {
    let mut arena = Arena::new();
    let ashe = arena.hero(2, Team::Ally, Vec2::ZERO);
    let target = arena.target(UnitKind::Hero, Vec2::new(40.0, 0.0));
    assert!(!team_can_see(arena.app.world(), Team::Ally, target));
    arena.cast(ashe, 2, AbilityTarget::Point(Vec2::new(40.0, 0.0)));
    arena.ticks(210);
    assert!(arena.committed(ashe, AbilityId(23)));
    assert!(team_can_see(arena.app.world(), Team::Ally, target));
    assert_eq!(arena.health(target), 1_000.0);
    assert_eq!(
        arena
            .app
            .world()
            .get::<UnitMechanics>(ashe)
            .unwrap()
            .charges(2),
        Some(0)
    );
    arena.command(ashe, UnitCommand::AttackTarget(target));
    arena.ticks(330);
    assert!(!team_can_see(arena.app.world(), Team::Ally, target));
    let last_position = arena.app.world().get::<UnitPose>(ashe).unwrap().position();
    arena.ticks(60);
    assert_eq!(
        arena.app.world().get::<UnitPose>(ashe).unwrap().position(),
        last_position,
        "an accepted attack order must stop tracking when scout vision expires"
    );
}

#[test]
fn real_cataclysm_creates_a_blocking_ring_and_recast_removes_it_without_another_mana_cost() {
    let mut arena = Arena::new();
    let jarvan = arena.hero(1, Team::Ally, Vec2::ZERO);
    let target = arena.target(UnitKind::Hero, Vec2::new(4.0, 0.0));
    arena.cast(jarvan, 3, AbilityTarget::Unit(target));
    arena.ticks(85);
    assert!(arena.committed(jarvan, AbilityId(14)));
    assert!((arena.health(target) - 800.0).abs() < 1e-4);
    assert!(!arena.app.world().resource::<NavigationMap>().line_clear(
        Vec2::new(4.0, 0.0),
        Vec2::new(10.0, 0.0),
        0.45
    ));
    let mana = arena.app.world().get::<UnitState>(jarvan).unwrap().mana();
    arena.cast(jarvan, 3, AbilityTarget::SelfCast);
    arena.ticks(2);
    assert!(arena.app.world().resource::<NavigationMap>().line_clear(
        Vec2::new(4.0, 0.0),
        Vec2::new(10.0, 0.0),
        0.45
    ));
    assert!(arena.app.world().get::<UnitState>(jarvan).unwrap().mana() >= mana);
}
