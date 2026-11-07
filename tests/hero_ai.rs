//! Tactical AI is exercised through real unit definitions and public commands.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::{gameplay::*, simulation::*};

struct Arena {
    app: App,
    hero: Entity,
    target: Entity,
    events: MessageCursor<SimulationEvent>,
    commits: Vec<(AbilityId, u64)>,
    releases: Vec<u64>,
    rejections: Vec<Rejection>,
}
impl Arena {
    fn new(abilities: Vec<AbilityDefinition>, loadout: AbilityLoadout, stats: UnitStats) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin, GameplayPlugin, AiPlugin));
        prepare_world(app.world_mut());
        for (index, definition) in abilities.into_iter().enumerate() {
            app.world_mut()
                .resource_mut::<AbilityCatalog>()
                .register(AbilityId(100 + index as u16), definition)
                .unwrap();
        }
        let character = CharacterDefinition {
            name: "Actual AI loadout".into(),
            description: String::new(),
            stats,
            default_loadout: loadout,
        };
        app.world_mut()
            .resource_scope(|world, mut catalog: Mut<CharacterCatalog>| {
                catalog
                    .register(
                        CharacterId(100),
                        character,
                        world.resource::<AbilityCatalog>(),
                    )
                    .unwrap();
            });
        let definition = app
            .world()
            .resource::<MatchDefinition>()
            .clone()
            .with_opponent(CharacterId(100));
        app.world_mut().insert_resource(definition);
        start_match(app.world_mut(), CharacterId(1)).unwrap();
        let hero = app
            .world()
            .resource::<MatchState>()
            .hero(Team::Enemy)
            .unwrap();
        app.world_mut().entity_mut(hero).insert(Controller::Human);
        app.world_mut().resource_mut::<SimulationControl>().paused = false;
        // Drain the real opening wave before isolating tactical scenarios.
        // The AI starts observing only after this setup; no pending/clock writes.
        for _ in 0..150 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        let creeps: Vec<_> = app
            .world_mut()
            .query::<(Entity, &UnitRole)>()
            .iter(app.world())
            .filter(|(_, role)| matches!(role, UnitRole::MeleeMinion | UnitRole::RangedMinion))
            .map(|(entity, _)| entity)
            .collect();
        for entity in creeps {
            app.world_mut().despawn(entity);
        }
        app.world_mut().entity_mut(hero).insert(Controller::HeroAi);
        let origin = app.world().get::<UnitPose>(hero).unwrap().position();
        let axis = app
            .world()
            .resource::<MapDefinition>()
            .lane_direction(Team::Enemy);
        let target = target(&mut app, origin + axis * 4.0, [None; 4], UnitRole::Hero);
        app.world_mut().resource_mut::<SimulationControl>().paused = false;
        Self {
            app,
            hero,
            target,
            events: default(),
            commits: vec![],
            releases: vec![],
            rejections: vec![],
        }
    }
    fn ticks(&mut self, count: usize) {
        for _ in 0..count {
            self.app.world_mut().run_schedule(FixedUpdate);
            let tick = self.app.world().resource::<SimulationTick>().0;
            for event in self
                .events
                .read(self.app.world().resource::<Messages<SimulationEvent>>())
            {
                match event {
                    SimulationEvent::AbilityCommitted { unit, ability, .. }
                        if *unit == self.hero =>
                    {
                        self.commits.push((*ability, tick))
                    }
                    SimulationEvent::AttackReleased { source, .. } if *source == self.hero => {
                        self.releases.push(tick)
                    }
                    SimulationEvent::CommandRejected { unit, reason } if *unit == self.hero => {
                        self.rejections.push(reason.clone())
                    }
                    _ => {}
                }
            }
            if tick.is_multiple_of(60) {
                self.app
                    .world_mut()
                    .resource_mut::<Messages<SimulationEvent>>()
                    .update();
            }
        }
    }
    fn state(&self, unit: Entity) -> &UnitState {
        self.app.world().get::<UnitState>(unit).unwrap()
    }
    fn point(&self, unit: Entity) -> Vec2 {
        self.app.world().get::<UnitPose>(unit).unwrap().position()
    }
    fn command(&mut self, unit: Entity, command: UnitCommand) {
        self.app
            .world_mut()
            .resource_mut::<CommandQueue>()
            .submit(unit, command);
    }
}

fn stats() -> UnitStats {
    UnitStats {
        max_health: 480.0,
        max_mana: 500.0,
        mana_regen: 0.0,
        base_attributes: BaseAttributes {
            attack_speed: 0.0,
            move_speed: 5.2,
            attack_range: 6.0,
            ..default()
        },
        ..default()
    }
}
fn target(app: &mut App, position: Vec2, abilities: AbilityLoadout, role: UnitRole) -> Entity {
    let entity = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Visible target".into(),
            team: Team::Ally,
            position,
            stats: UnitStats {
                max_health: 10000.0,
                max_mana: 100.0,
                mana_regen: 0.0,
                base_attributes: BaseAttributes {
                    attack_speed: 0.0,
                    move_speed: 5.0,
                    ..default()
                },
                ..default()
            },
            abilities,
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut()
        .entity_mut(entity)
        .insert((role, Controller::Human));
    entity
}
fn hit(targeting: Targeting, delivery: Delivery, effects: Vec<Effect>) -> AbilityDefinition {
    AbilityDefinition {
        name: "Semantic spell".into(),
        targeting,
        range: 12.0,
        windup: 0.1,
        recovery: 0.0,
        cooldown: 1.0,
        mana_cost: 10.0,
        execution: AbilityExecution::Impact(ImpactDefinition { delivery, effects }),
    }
}
fn damage(value: f32) -> Effect {
    Effect::Damage(DamageSpec {
        kind: DamageType::True,
        base: value,
        ad_ratio: 0.0,
        ap_ratio: 0.0,
    })
}

#[test]
fn reordered_actual_loadout_uses_enemy_direction_area_and_channel_without_slot_special_cases() {
    let duration = StatusDuration::from_ticks(30).unwrap();
    let abilities = vec![
        hit(
            Targeting::Enemy,
            Delivery::Direct,
            vec![
                damage(30.0),
                Effect::ApplyStatus(StatusSpec::Stun { duration }),
            ],
        ),
        hit(
            Targeting::Direction,
            Delivery::Projectile {
                speed: 22.0,
                radius: 0.25,
            },
            vec![damage(20.0)],
        ),
        hit(
            Targeting::Point,
            Delivery::Area {
                radius: 2.0,
                delay: 0.2,
            },
            vec![damage(25.0)],
        ),
        AbilityDefinition {
            targeting: Targeting::SelfCast,
            range: 5.0,
            execution: AbilityExecution::Channel {
                duration: 0.3,
                interval: 0.1,
                pulse: ImpactDefinition {
                    delivery: Delivery::Area {
                        radius: 5.0,
                        delay: 0.0,
                    },
                    effects: vec![damage(10.0)],
                },
            },
            ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(1.0)])
        },
    ];
    let mut a = Arena::new(
        abilities,
        [
            Some(AbilityId(102)),
            Some(AbilityId(103)),
            Some(AbilityId(101)),
            Some(AbilityId(100)),
        ],
        stats(),
    );
    a.ticks(360);
    for id in [
        AbilityId(100),
        AbilityId(101),
        AbilityId(102),
        AbilityId(103),
    ] {
        let ticks = a
            .commits
            .iter()
            .filter(|(ability, _)| *ability == id)
            .map(|(_, tick)| *tick)
            .collect::<Vec<_>>();
        assert!(!ticks.is_empty(), "actual ability {id:?} was not used");
        assert!(
            ticks.windows(2).all(|pair| pair[1] - pair[0] >= 60),
            "AI cannot bypass the shared cooldown"
        );
    }
    assert!(a.state(a.target).health() < 10000.0);
    assert!(
        a.rejections.is_empty(),
        "preflight should filter known unavailable choices: {:?}",
        a.rejections
    );
}

#[test]
fn equipment_active_uses_the_same_semantic_candidate_and_instance_cooldown() {
    let ability = hit(Targeting::Enemy, Delivery::Direct, vec![damage(50.0)]);
    let mut a = Arena::new(vec![ability], [None; 4], stats());
    a.app
        .world_mut()
        .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
            catalog
                .register(
                    EquipmentId(100),
                    EquipmentDefinition {
                        name: "AI active".into(),
                        description: String::new(),
                        flat: default(),
                        additive_percent: default(),
                        passives: vec![],
                        active: Some(AbilityId(100)),
                    },
                    world.resource::<AbilityCatalog>(),
                )
                .unwrap();
        });
    equip_equipment(a.app.world_mut(), a.hero, 0, EquipmentId(100)).unwrap();
    a.ticks(160);
    assert!(a.commits.len() >= 2);
    assert!(a.commits.windows(2).all(|pair| pair[1].1 - pair[0].1 >= 60));
    assert!(a.state(a.target).health() < 10000.0);
    assert!(a.rejections.is_empty());
}

#[test]
fn no_mana_and_hard_control_never_create_free_ai_casts() {
    let spell = hit(Targeting::Enemy, Delivery::Direct, vec![damage(50.0)]);
    let mut dry = Arena::new(
        vec![spell.clone()],
        [Some(AbilityId(100)), None, None, None],
        UnitStats {
            max_mana: 0.0,
            ..stats()
        },
    );
    dry.ticks(240);
    assert!(dry.commits.is_empty());
    assert_eq!(dry.state(dry.hero).mana(), 0.0);
    assert_eq!(dry.state(dry.target).health(), 10000.0);
    let mut controlled = Arena::new(
        vec![spell],
        [Some(AbilityId(100)), None, None, None],
        stats(),
    );
    let stun = AbilityId(200);
    controlled
        .app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            stun,
            AbilityDefinition {
                windup: 0.0,
                mana_cost: 0.0,
                execution: AbilityExecution::Impact(ImpactDefinition {
                    delivery: Delivery::Direct,
                    effects: vec![Effect::ApplyStatus(StatusSpec::Stun {
                        duration: StatusDuration::from_ticks(120).unwrap(),
                    })],
                }),
                ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(1.0)])
            },
        )
        .unwrap();
    let caster_point = controlled.point(controlled.hero) + Vec2::X;
    let caster = target(
        &mut controlled.app,
        caster_point,
        [Some(stun), None, None, None],
        UnitRole::Hero,
    );
    controlled.command(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(controlled.hero),
        },
    );
    let initial = controlled.point(controlled.hero);
    controlled.ticks(100);
    assert!(controlled.commits.is_empty());
    assert_eq!(controlled.point(controlled.hero), initial);
    controlled.ticks(100);
    assert!(
        !controlled.commits.is_empty(),
        "AI must reconsider after real control expires"
    );
}

#[test]
fn ai_preserves_a_long_basic_attack_preparation_instead_of_resetting_it_each_decision() {
    let mut a = Arena::new(
        vec![],
        [None; 4],
        UnitStats {
            attack_windup_fraction: 0.8,
            base_attributes: BaseAttributes {
                attack_speed: 0.5,
                ..stats().base_attributes
            },
            ..stats()
        },
    );
    a.ticks(300);
    assert!(
        !a.releases.is_empty(),
        "96-tick preparation must survive repeated12-tick decisions"
    );
    assert!(a.state(a.target).health() < 10000.0);
    assert!(a.rejections.is_empty());
}

#[test]
fn wounded_hero_retreats_through_real_recovery_then_rejoins_the_lane() {
    let mut a = Arena::new(vec![], [None; 4], stats());
    let hurt = AbilityId(200);
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            hurt,
            AbilityDefinition {
                windup: 0.0,
                mana_cost: 0.0,
                ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(384.0)])
            },
        )
        .unwrap();
    let caster_point = a.point(a.hero) + Vec2::X;
    let caster = target(
        &mut a.app,
        caster_point,
        [Some(hurt), None, None, None],
        UnitRole::Hero,
    );
    a.command(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(a.hero),
        },
    );
    a.ticks(1);
    assert!((a.state(a.hero).health() - 96.0).abs() < 0.001);
    a.app.world_mut().despawn(caster);
    a.app.world_mut().despawn(a.target);
    let map = a.app.world().resource::<MapDefinition>().clone();
    a.ticks(60);
    assert!(a.point(a.hero).distance(map.base_position(Team::Enemy)) < 1.5);
    assert_eq!(
        a.state(a.hero).health(),
        96.0,
        "AI cannot invent recovery before the out-of-combat window"
    );
    a.ticks(900);
    assert!(a.state(a.hero).health() >= 480.0 * 0.75);
    let progress =
        (a.point(a.hero) - map.base_position(Team::Enemy)).dot(map.lane_direction(Team::Enemy));
    assert!(
        progress > 4.0,
        "healthy AI should leave recovery and rejoin the lane"
    );
}

#[test]
fn base_threat_can_override_a_closer_hero_duel() {
    let mut a = Arena::new(vec![], [None; 4], stats());
    let map = a.app.world().resource::<MapDefinition>().clone();
    let danger = map.base_position(Team::Enemy) + map.lane_lateral(Team::Enemy) * 0.8;
    let attacker = target(&mut a.app, danger, [None; 4], UnitRole::MeleeMinion);
    a.ticks(14);
    assert!(
        matches!(a.app.world().get::<UnitOrder>(a.hero),Some(UnitOrder::AttackTarget(life))if life.entity()==attacker)
    );
}

#[test]
fn directional_aim_uses_observed_motion_but_a_later_stop_can_dodge_it() {
    let ability = AbilityDefinition {
        windup: 0.25,
        execution: AbilityExecution::Impact(ImpactDefinition {
            delivery: Delivery::Projectile {
                speed: 6.0,
                radius: 0.1,
            },
            effects: vec![damage(100.0)],
        }),
        ..hit(
            Targeting::Direction,
            Delivery::Projectile {
                speed: 6.0,
                radius: 0.1,
            },
            vec![damage(100.0)],
        )
    };
    let mut a = Arena::new(
        vec![ability],
        [Some(AbilityId(100)), None, None, None],
        stats(),
    );
    let lateral = a
        .app
        .world()
        .resource::<MapDefinition>()
        .lane_lateral(Team::Enemy);
    a.command(
        a.target,
        UnitCommand::MoveTo(a.point(a.target) + lateral * 3.0),
    );
    a.ticks(14);
    let aim = match a.app.world().get::<UnitAction>(a.hero).unwrap() {
        UnitAction::Casting {
            target: AbilityTarget::Direction(direction),
            ..
        } => *direction,
        other => panic!("expected actual directional preparation, got {other:?}"),
    };
    let direct = (a.point(a.target) - a.point(a.hero)).normalize();
    assert!(
        aim.dot(lateral) > direct.dot(lateral) + 0.15,
        "visible movement should lead the current target position"
    );
    a.command(a.target, UnitCommand::Stop);
    a.ticks(60);
    assert_eq!(a.commits.len(), 1);
    assert_eq!(
        a.state(a.target).health(),
        10000.0,
        "finite prediction must not read the player's future stop command"
    );
}

#[test]
fn a_directional_cast_still_hits_the_real_minion_screen_before_the_hero() {
    let mut a = Arena::new(
        vec![hit(
            Targeting::Direction,
            Delivery::Projectile {
                speed: 22.0,
                radius: 0.2,
            },
            vec![damage(100.0)],
        )],
        [Some(AbilityId(100)), None, None, None],
        stats(),
    );
    let point = a.point(a.hero).lerp(a.point(a.target), 0.5);
    let screen = target(&mut a.app, point, [None; 4], UnitRole::MeleeMinion);
    a.ticks(60);
    assert!(!a.commits.is_empty());
    assert!(a.state(screen).health() < 10000.0);
    assert_eq!(
        a.state(a.target).health(),
        10000.0,
        "AI direction scoring cannot bypass real projectile collision"
    );
}

#[test]
fn death_respawn_discards_old_tactical_target_and_restarts_with_a_reaction_interval() {
    let mut a = Arena::new(
        vec![hit(Targeting::Enemy, Delivery::Direct, vec![damage(25.0)])],
        [Some(AbilityId(100)), None, None, None],
        stats(),
    );
    a.ticks(30);
    assert!(!a.commits.is_empty());
    let old = a.state(a.hero).life_ref(a.hero);
    let kill = AbilityId(200);
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            kill,
            AbilityDefinition {
                windup: 0.0,
                mana_cost: 0.0,
                ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(1000.0)])
            },
        )
        .unwrap();
    let caster_point = a.point(a.hero) + Vec2::X;
    let caster = target(
        &mut a.app,
        caster_point,
        [Some(kill), None, None, None],
        UnitRole::Hero,
    );
    a.command(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(a.hero),
        },
    );
    a.ticks(1);
    assert!(!a.state(a.hero).alive());
    a.app.world_mut().despawn(caster);
    a.app.world_mut().despawn(a.target);
    let point = a
        .app
        .world()
        .resource::<MapDefinition>()
        .hero_spawn(Team::Enemy)
        + a.app
            .world()
            .resource::<MapDefinition>()
            .lane_direction(Team::Enemy)
            * 3.0;
    let fresh = target(&mut a.app, point, [None; 4], UnitRole::Hero);
    let previous = a.commits.len();
    a.ticks(180);
    assert!(a.state(a.hero).alive());
    assert_ne!(a.state(a.hero).life_ref(a.hero), old);
    assert_eq!(
        a.commits.len(),
        previous,
        "new life must observe before reacting"
    );
    a.ticks(100);
    assert!(a.commits.len() > previous);
    assert!(a.state(fresh).health() < 10000.0);
}

#[test]
fn a_continuously_fleeing_hero_is_abandoned_before_it_can_drag_ai_across_the_map() {
    let mut a = Arena::new(
        vec![],
        [None; 4],
        UnitStats {
            base_attributes: BaseAttributes {
                attack_range: 0.5,
                ..stats().base_attributes
            },
            ..stats()
        },
    );
    let map = a.app.world().resource::<MapDefinition>().clone();
    let start = a.point(a.hero);
    a.command(
        a.target,
        UnitCommand::MoveTo(a.point(a.target) + map.lane_direction(Team::Enemy) * 15.0),
    );
    a.ticks(30);
    assert!(
        matches!(a.app.world().get::<UnitOrder>(a.hero), Some(UnitOrder::AttackTarget(life)) if life.entity() == a.target)
    );
    a.ticks(70);
    assert!(
        a.point(a.hero).distance(start) < 8.0,
        "chase leash must apply before the far base"
    );
    assert!(
        !matches!(a.app.world().get::<UnitOrder>(a.hero), Some(UnitOrder::AttackTarget(life)) if life.entity() == a.target)
    );
    let offset = (a.point(a.target) - a.point(a.hero)).length();
    assert!(
        offset < 12.0,
        "target remains perceived when the pursuit is abandoned"
    );
    a.command(a.target, UnitCommand::Stop);
    a.ticks(100);
    assert!(
        !matches!(a.app.world().get::<UnitOrder>(a.hero), Some(UnitOrder::AttackTarget(life)) if life.entity() == a.target),
        "return-to-lane interval must prevent immediate re-acquisition"
    );
    let lateral =
        (a.point(a.hero) - map.base_position(Team::Enemy)).dot(map.lane_lateral(Team::Enemy));
    assert!(
        (lateral - 1.0).abs() < 0.2,
        "AI returns to its lane formation"
    );
}

#[test]
fn offensive_dash_cannot_independently_select_an_out_of_lane_hero() {
    let dash = AbilityDefinition {
        targeting: Targeting::Point,
        range: 6.0,
        execution: AbilityExecution::Dash { speed: 24.0 },
        ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(1.0)])
    };
    let mut a = Arena::new(
        vec![dash],
        [Some(AbilityId(100)), None, None, None],
        UnitStats {
            base_attributes: BaseAttributes {
                attack_range: 0.5,
                ..stats().base_attributes
            },
            ..stats()
        },
    );
    let map = a.app.world().resource::<MapDefinition>().clone();
    a.command(
        a.target,
        UnitCommand::MoveTo(
            map.base_position(Team::Enemy)
                + map.lane_direction(Team::Enemy) * 6.0
                + map.lane_lateral(Team::Enemy) * 7.0,
        ),
    );
    a.ticks(120);
    // Make the visible hero attractive to chase through an actual damage command.
    let hurt = AbilityId(200);
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            hurt,
            AbilityDefinition {
                windup: 0.0,
                mana_cost: 0.0,
                ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(8000.0)])
            },
        )
        .unwrap();
    let caster_point = a.point(a.target) + Vec2::X;
    let caster = spawn_unit(
        a.app.world_mut(),
        UnitSpec {
            name: "Wound own enemy".into(),
            team: Team::Enemy,
            position: caster_point,
            stats: stats(),
            abilities: [Some(hurt), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    a.command(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(a.target),
        },
    );
    a.ticks(1);
    assert!(a.state(a.target).health() < 3000.0);
    a.app.world_mut().despawn(caster);
    a.ticks(120);
    assert!(
        a.commits.is_empty(),
        "offensive Dash shares the lane/pursuit eligibility of attacks"
    );
}

fn inflict(a: &mut Arena, victim: Entity, id: AbilityId, amount: f32) {
    a.app
        .world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(
            id,
            AbilityDefinition {
                windup: 0.0,
                mana_cost: 0.0,
                ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(amount)])
            },
        )
        .unwrap();
    let team = opponent(a.app.world().get::<Unit>(victim).unwrap().team());
    let position = a.point(victim) + Vec2::X;
    let caster = spawn_unit(
        a.app.world_mut(),
        UnitSpec {
            name: "Damage fixture".into(),
            team,
            position,
            stats: stats(),
            abilities: [Some(id), None, None, None],
            respawn_delay: None,
        },
    )
    .unwrap();
    a.command(
        caster,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(victim),
        },
    );
    a.ticks(1);
    a.app.world_mut().despawn(caster);
}

#[test]
fn a_semantic_dash_can_close_a_safe_advantage_and_then_escape_after_real_damage() {
    let mut a = Arena::new(
        vec![AbilityDefinition {
            targeting: Targeting::Point,
            range: 6.0,
            cooldown: 0.2,
            execution: AbilityExecution::Dash { speed: 24.0 },
            ..hit(Targeting::Enemy, Delivery::Direct, vec![damage(1.0)])
        }],
        [Some(AbilityId(100)), None, None, None],
        UnitStats {
            base_attributes: BaseAttributes {
                attack_range: 0.5,
                ..stats().base_attributes
            },
            ..stats()
        },
    );
    let initial = a.point(a.hero);
    let victim = a.target;
    inflict(&mut a, victim, AbilityId(300), 8000.0);
    a.ticks(30);
    assert!(
        !a.commits.is_empty(),
        "a safe low-health target permits an actual offensive Dash"
    );
    assert!(a.point(a.hero).distance(initial) > 1.5);
    let before = a.commits.len();
    let hero = a.hero;
    inflict(&mut a, hero, AbilityId(301), 384.0);
    a.ticks(60);
    assert!(
        a.commits.len() > before,
        "retreat evaluates the same equipped Dash through real preflight"
    );
    let home = a
        .app
        .world()
        .resource::<MapDefinition>()
        .base_position(Team::Enemy);
    assert!(a.point(a.hero).distance(home) < 1.5);
    assert!(a.rejections.is_empty());
}
