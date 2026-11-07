//! Character content and loadouts exercise the public construction/command path.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

const CHARACTER: CharacterId = CharacterId(10);
const DEFAULT: AbilityId = AbilityId(1);
const ALTERNATE: AbilityId = AbilityId(2);
const CONTROL: AbilityId = AbilityId(3);
const KILL: AbilityId = AbilityId(4);

struct Arena {
    app: App,
    cursor: MessageCursor<SimulationEvent>,
}
impl Arena {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(NavigationMap::new(20.0, 1.0, vec![]));
        for (id, effects) in [
            (DEFAULT, vec![Effect::Damage(DamageSpec::magical(10.0))]),
            (ALTERNATE, vec![Effect::Damage(DamageSpec::magical(30.0))]),
            (
                CONTROL,
                vec![
                    Effect::Damage(DamageSpec::magical(5.0)),
                    Effect::ApplyStatus(StatusSpec::Stun {
                        duration: StatusDuration::from_ticks(60).unwrap(),
                    }),
                    Effect::ApplyStatus(StatusSpec::Slow {
                        duration: StatusDuration::from_ticks(60).unwrap(),
                        fraction: 0.5,
                    }),
                    Effect::Displace(DisplacementSpec {
                        distance: 4.0,
                        speed: 6.0,
                        direction: DisplacementDirection::AwayFromCenter,
                    }),
                ],
            ),
            (KILL, vec![Effect::Damage(DamageSpec::magical(10_000.0))]),
        ] {
            app.world_mut()
                .resource_mut::<AbilityCatalog>()
                .register(
                    id,
                    AbilityDefinition {
                        name: format!("Ability {}", id.0),
                        targeting: Targeting::Enemy,
                        range: 30.0,
                        windup: 0.0,
                        recovery: 0.0,
                        cooldown: 3.0,
                        mana_cost: 20.0,
                        execution: AbilityExecution::Impact(ImpactDefinition {
                            delivery: Delivery::Direct,
                            effects,
                        }),
                    },
                )
                .unwrap();
        }
        let mut arena = Self {
            app,
            cursor: MessageCursor::default(),
        };
        arena.register(CHARACTER, definition()).unwrap();
        arena
    }

    fn register(
        &mut self,
        id: CharacterId,
        definition: CharacterDefinition,
    ) -> Result<(), CharacterError> {
        self.app
            .world_mut()
            .resource_scope(|world, mut characters: Mut<CharacterCatalog>| {
                characters.register(id, definition, world.resource::<AbilityCatalog>())
            })
    }

    fn spawn(&mut self, loadout: LoadoutSelection, position: Vec2) -> Entity {
        let mut spec = spawn_spec();
        spec.loadout = loadout;
        spec.position = position;
        spawn_character(self.app.world_mut(), spec).unwrap()
    }

    fn npc(&mut self, abilities: AbilityLoadout) -> Entity {
        spawn_unit(
            self.app.world_mut(),
            UnitSpec {
                name: "NPC".into(),
                team: Team::Enemy,
                position: Vec2::new(-2.0, 0.0),
                stats: UnitStats {
                    mana_regen: 0.0,
                    ..default()
                },
                abilities,
                respawn_delay: None,
            },
        )
        .unwrap()
    }

    fn cast(&mut self, actor: Entity, slot: usize, target: Entity) {
        self.app.world_mut().resource_mut::<CommandQueue>().submit(
            actor,
            UnitCommand::CastAbility {
                slot,
                target: AbilityTarget::Unit(target),
            },
        );
    }

    fn step(&mut self) {
        self.app.world_mut().run_schedule(FixedUpdate);
    }

    fn state(&self, entity: Entity) -> &UnitState {
        self.app.world().get::<UnitState>(entity).unwrap()
    }

    fn entity_count(&mut self) -> usize {
        self.app
            .world_mut()
            .query::<Entity>()
            .iter(self.app.world())
            .count()
    }

    fn slots(&self, entity: Entity) -> &AbilitySlots {
        self.app.world().get::<AbilitySlots>(entity).unwrap()
    }

    fn attributes(&self, entity: Entity) -> EffectiveAttributes {
        *self.app.world().get::<Unit>(entity).unwrap().attributes()
    }

    fn modifier(&mut self, entity: Entity, amount: f32, removal: ModifierRemoval) {
        let mut changes = self.app.world_mut().resource_mut::<AttributeChanges>();
        let source = changes.allocate_source().unwrap();
        changes.upsert(
            entity,
            source,
            AttributeModifier {
                flat: AttributeDelta {
                    attack_damage: amount,
                    ..default()
                },
                removal,
                ..default()
            },
        );
    }

    fn events(&mut self) -> Vec<SimulationEvent> {
        self.cursor
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect()
    }
}

fn definition() -> CharacterDefinition {
    CharacterDefinition {
        name: "Test character".into(),
        description: String::new(),
        stats: UnitStats {
            mana_regen: 0.0,
            ..default()
        },
        default_loadout: [Some(DEFAULT), Some(DEFAULT), Some(DEFAULT), Some(DEFAULT)],
    }
}

fn spawn_spec() -> CharacterSpawnSpec {
    CharacterSpawnSpec {
        character: CHARACTER,
        team: Team::Ally,
        position: Vec2::ZERO,
        loadout: LoadoutSelection::CharacterDefault,
        respawn_delay: Some(0.1),
    }
}

fn loadout(slots: &AbilitySlots) -> AbilityLoadout {
    std::array::from_fn(|slot| slots.ability(slot))
}

#[test]
fn catalog_rejections_preserve_definitions_and_sorted_iteration() {
    let mut arena = Arena::new();
    let mut replacement = definition();
    replacement.name = "Replacement".into();
    assert_eq!(
        arena.register(CHARACTER, replacement),
        Err(CharacterError::DuplicateId(CHARACTER))
    );
    for name in ["", "  "] {
        let mut invalid = definition();
        invalid.name = name.into();
        assert_eq!(
            arena.register(CharacterId(1), invalid),
            Err(CharacterError::InvalidDefinition)
        );
    }
    for invalid_stats in [
        UnitStats {
            max_health: f32::NAN,
            ..default()
        },
        UnitStats {
            max_mana: -1.0,
            ..default()
        },
        UnitStats {
            mana_regen: f32::INFINITY,
            ..default()
        },
        UnitStats {
            radius: 0.0,
            ..default()
        },
        UnitStats {
            attack_windup_fraction: 1.1,
            ..default()
        },
        UnitStats {
            base_attributes: BaseAttributes {
                attack_speed: -1.0,
                ..default()
            },
            ..default()
        },
    ] {
        let mut invalid = definition();
        invalid.stats = invalid_stats;
        assert!(invalid.initial_attributes().is_none());
        assert_eq!(
            arena.register(CharacterId(1), invalid),
            Err(CharacterError::InvalidDefinition)
        );
    }
    let mut invalid = definition();
    invalid.default_loadout[2] = Some(AbilityId(999));
    assert_eq!(
        arena.register(CharacterId(1), invalid),
        Err(CharacterError::UnknownAbility {
            slot: 2,
            ability: AbilityId(999)
        })
    );
    assert_eq!(
        arena
            .app
            .world()
            .resource::<CharacterCatalog>()
            .iter()
            .len(),
        1
    );
    assert_eq!(
        arena
            .app
            .world()
            .resource::<CharacterCatalog>()
            .get(CHARACTER)
            .unwrap()
            .name,
        "Test character"
    );
    for id in [CharacterId(20), CharacterId(1)] {
        let mut empty = definition();
        empty.default_loadout = [None; 4];
        arena.register(id, empty).unwrap();
    }
    assert_eq!(
        arena
            .app
            .world()
            .resource::<CharacterCatalog>()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        [CharacterId(1), CHARACTER, CharacterId(20)]
    );
}

#[test]
fn failed_construction_leaves_no_units_or_partial_character_identity() {
    let mut arena = Arena::new();
    let npc = arena.npc([None; 4]);
    assert!(arena.app.world().get::<CharacterIdentity>(npc).is_none());
    let before = arena.entity_count();
    let mut unknown = spawn_spec();
    unknown.character = CharacterId(999);
    assert_eq!(
        spawn_character(arena.app.world_mut(), unknown),
        Err(CharacterError::UnknownCharacter(CharacterId(999)))
    );
    let mut invalid = spawn_spec();
    invalid.loadout = LoadoutSelection::Explicit([None, Some(AbilityId(999)), None, None]);
    assert_eq!(
        spawn_character(arena.app.world_mut(), invalid),
        Err(CharacterError::UnknownAbility {
            slot: 1,
            ability: AbilityId(999)
        })
    );
    for position in [Vec2::splat(f32::NAN), Vec2::new(21.0, 0.0)] {
        let mut blocked = spawn_spec();
        blocked.position = position;
        assert_eq!(
            spawn_character(arena.app.world_mut(), blocked),
            Err(CharacterError::InvalidSpawn {
                reason: "unit spawn is blocked"
            })
        );
    }
    for delay in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut invalid = spawn_spec();
        invalid.respawn_delay = Some(delay);
        assert_eq!(
            spawn_character(arena.app.world_mut(), invalid),
            Err(CharacterError::InvalidSpawn {
                reason: "invalid unit definition"
            })
        );
    }
    assert_eq!(arena.entity_count(), before);
    assert_eq!(
        arena
            .app
            .world_mut()
            .query::<&Unit>()
            .iter(arena.app.world())
            .count(),
        1
    );
    assert_eq!(
        arena
            .app
            .world_mut()
            .query::<&CharacterIdentity>()
            .iter(arena.app.world())
            .count(),
        0
    );
    arena.spawn(LoadoutSelection::CharacterDefault, Vec2::ZERO);
    assert_eq!(arena.entity_count(), before + 1);
}

#[test]
fn explicit_loadout_fully_replaces_defaults_and_executes_actual_slots() {
    let mut arena = Arena::new();
    let original = arena.spawn(LoadoutSelection::CharacterDefault, Vec2::ZERO);
    assert_eq!(loadout(arena.slots(original)), definition().default_loadout);
    let explicit = [Some(ALTERNATE), None, Some(ALTERNATE), None];
    let actor = arena.spawn(LoadoutSelection::Explicit(explicit), Vec2::new(1.0, 0.0));
    let target = arena.npc([None; 4]);
    assert_eq!(loadout(arena.slots(actor)), explicit);
    assert_eq!(
        arena
            .app
            .world()
            .get::<CharacterIdentity>(actor)
            .unwrap()
            .id(),
        CHARACTER
    );
    arena.cast(actor, 0, target);
    arena.step();
    assert_eq!(arena.state(target).health(), 570.0);
    assert_eq!(arena.state(actor).mana(), 280.0);
    assert!(arena.slots(actor).cooldown(0).unwrap() > 2.9);
    assert_eq!(arena.slots(actor).cooldown(2), Some(0.0));
    arena.cast(actor, 2, target);
    arena.step();
    assert_eq!(arena.state(target).health(), 540.0);
    assert_eq!(arena.state(actor).mana(), 260.0);
    assert!(arena.slots(actor).cooldown(2).unwrap() > 2.9);
    arena.cast(actor, 1, target);
    arena.step();
    assert_eq!(arena.state(actor).mana(), 260.0);
    assert!(arena.events().iter().any(|event| matches!(event,
        SimulationEvent::CommandRejected { unit, reason: Rejection::NoAbility } if *unit == actor
    )));
    assert_eq!(
        arena
            .app
            .world()
            .resource::<CharacterCatalog>()
            .get(CHARACTER)
            .unwrap()
            .default_loadout,
        definition().default_loadout
    );
    assert_eq!(loadout(arena.slots(original)), definition().default_loadout);
    let empty = arena.spawn(LoadoutSelection::Explicit([None; 4]), Vec2::new(2.0, 0.0));
    assert_eq!(loadout(arena.slots(empty)), [None; 4]);
}

#[test]
fn initial_preview_matches_construction_including_attack_speed_boundaries() {
    let mut arena = Arena::new();
    let before = arena.entity_count();
    for (offset, speed) in [0.0, f32::MIN_POSITIVE, 1.25, f32::MAX]
        .into_iter()
        .enumerate()
    {
        let id = CharacterId(100 + offset as u16);
        let mut character = definition();
        character.stats.base_attributes = BaseAttributes {
            attack_damage: 30.0,
            ability_power: 40.0,
            armor: -20.0,
            magic_resistance: -10.0,
            move_speed: 5.2,
            attack_range: 6.0,
            attack_speed: speed,
        };
        let preview = character.initial_attributes().unwrap();
        assert_eq!(arena.entity_count(), before + offset);
        assert_eq!(
            preview.attack_speed(),
            match offset {
                0 => 0.0,
                1 => MIN_ATTACK_SPEED,
                2 => 1.25,
                _ => MAX_ATTACK_SPEED,
            }
        );
        arena.register(id, character).unwrap();
        let mut spec = spawn_spec();
        spec.character = id;
        let entity = spawn_character(arena.app.world_mut(), spec).unwrap();
        assert_eq!(arena.attributes(entity), preview);
    }
}

#[test]
fn instances_share_content_but_keep_health_attributes_statuses_and_cooldowns_independent() {
    let mut arena = Arena::new();
    let first = arena.spawn(LoadoutSelection::CharacterDefault, Vec2::ZERO);
    let second = arena.spawn(LoadoutSelection::CharacterDefault, Vec2::new(1.0, 0.0));
    let enemy = arena.npc([Some(CONTROL), None, None, None]);
    arena.modifier(first, 10.0, ModifierRemoval::Explicit);
    arena.cast(first, 0, enemy);
    arena.cast(enemy, 0, first);
    arena.step();
    assert_eq!(arena.state(first).health(), 595.0);
    assert_eq!(arena.state(second).health(), 600.0);
    assert_eq!(arena.attributes(first).attack_damage(), 50.0);
    assert_eq!(arena.attributes(second).attack_damage(), 40.0);
    assert_eq!(arena.attributes(first).move_speed(), 2.75);
    assert_eq!(arena.attributes(second).move_speed(), 5.5);
    assert!(
        !arena
            .app
            .world()
            .get::<UnitStatuses>(first)
            .unwrap()
            .is_empty()
    );
    assert!(
        arena
            .app
            .world()
            .get::<UnitStatuses>(second)
            .unwrap()
            .is_empty()
    );
    assert!(arena.slots(first).cooldown(0).unwrap() > 0.0);
    assert_eq!(arena.slots(second).cooldown(0), Some(0.0));
    let template = arena
        .app
        .world()
        .resource::<CharacterCatalog>()
        .get(CHARACTER)
        .unwrap();
    assert_eq!(
        template.stats.base_attributes,
        definition().stats.base_attributes
    );
    assert_eq!(template.default_loadout, definition().default_loadout);
    assert_eq!(template.stats.max_health, 600.0);
}

#[test]
fn death_and_respawn_preserve_character_and_explicit_ids_with_existing_lifecycle_rules() {
    let mut arena = Arena::new();
    let explicit = [Some(ALTERNATE), None, Some(ALTERNATE), None];
    let actor = arena.spawn(LoadoutSelection::Explicit(explicit), Vec2::ZERO);
    let enemy = arena.npc([Some(CONTROL), Some(KILL), None, None]);
    let first_life = arena.state(actor).life_ref(actor);
    arena.modifier(actor, 10.0, ModifierRemoval::Explicit);
    arena.modifier(actor, 20.0, ModifierRemoval::OnDeath);
    arena.cast(actor, 2, enemy);
    arena.cast(enemy, 0, actor);
    arena.step();
    assert_eq!(arena.attributes(actor).attack_damage(), 70.0);
    assert!(arena.slots(actor).cooldown(2).unwrap() > 0.0);
    assert!(
        arena
            .app
            .world()
            .get::<ControlState>(actor)
            .unwrap()
            .forced_motion()
    );
    arena.cast(enemy, 1, actor);
    arena.step();
    assert!(!arena.state(actor).alive());
    assert!(
        arena
            .app
            .world()
            .get::<UnitStatuses>(actor)
            .unwrap()
            .is_empty()
    );
    assert!(
        !arena
            .app
            .world()
            .get::<ControlState>(actor)
            .unwrap()
            .forced_motion()
    );
    assert_eq!(arena.attributes(actor).attack_damage(), 50.0);
    for _ in 0..6 {
        arena.step();
    }
    assert!(arena.state(actor).alive());
    assert_eq!(arena.state(actor).health(), 600.0);
    assert_eq!(arena.state(actor).mana(), 300.0);
    assert_eq!(
        arena.state(actor).life_ref(actor).epoch(),
        first_life.epoch() + 1
    );
    assert_eq!(
        arena
            .app
            .world()
            .get::<CharacterIdentity>(actor)
            .unwrap()
            .id(),
        CHARACTER
    );
    assert_eq!(loadout(arena.slots(actor)), explicit);
    assert_eq!(arena.slots(actor).cooldown(2), Some(0.0));
    assert_eq!(arena.attributes(actor).attack_damage(), 50.0);
    assert_eq!(arena.attributes(actor).move_speed(), 5.5);
    assert_eq!(
        arena.app.world().get::<UnitPose>(actor).unwrap().position(),
        Vec2::ZERO
    );
    assert!(
        arena
            .app
            .world()
            .get::<ControlState>(actor)
            .unwrap()
            .can_move()
    );
    assert!(
        arena
            .app
            .world()
            .get::<ControlState>(actor)
            .unwrap()
            .can_cast()
    );
    arena.cast(actor, 2, enemy);
    arena.step();
    assert_eq!(arena.state(enemy).health(), 540.0);
    assert_eq!(arena.state(actor).mana(), 280.0);
    assert!(arena.events().iter().any(|event| matches!(event,
        SimulationEvent::Respawned { unit } if *unit == actor
    )));
}
