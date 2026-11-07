//! An explicit trial reset clears runtime state through the public interface.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

const CHARACTER: CharacterId = CharacterId(1);
const CONTROL: AbilityId = AbilityId(1);
const PROJECTILE: AbilityId = AbilityId(2);
const AREA: AbilityId = AbilityId(3);
const CHANNEL: AbilityId = AbilityId(4);

fn arena() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(NavigationMap::new(
        20.0,
        1.0,
        vec![Obstacle {
            center: Vec2::splat(10.0),
            half_size: Vec2::ONE,
        }],
    ));
    let duration = StatusDuration::from_ticks(120).unwrap();
    for (id, targeting, execution) in [
        (
            CONTROL,
            Targeting::Enemy,
            AbilityExecution::Impact(ImpactDefinition {
                delivery: Delivery::Direct,
                effects: vec![
                    Effect::Damage(DamageSpec::magical(10.0)),
                    Effect::ApplyStatus(StatusSpec::Stun { duration }),
                    Effect::ApplyStatus(StatusSpec::Slow {
                        duration,
                        fraction: 0.5,
                    }),
                    Effect::Displace(DisplacementSpec {
                        distance: 4.0,
                        speed: 6.0,
                        direction: DisplacementDirection::AlongImpact,
                    }),
                ],
            }),
        ),
        (
            PROJECTILE,
            Targeting::Direction,
            AbilityExecution::Impact(ImpactDefinition {
                delivery: Delivery::Projectile {
                    speed: 1.0,
                    radius: 0.2,
                },
                effects: vec![Effect::Damage(DamageSpec::magical(100.0))],
            }),
        ),
        (
            AREA,
            Targeting::Point,
            AbilityExecution::Impact(ImpactDefinition {
                delivery: Delivery::Area {
                    radius: 3.0,
                    delay: 2.0,
                },
                effects: vec![Effect::Damage(DamageSpec::magical(100.0))],
            }),
        ),
        (
            CHANNEL,
            Targeting::SelfCast,
            AbilityExecution::Channel {
                duration: 5.0,
                interval: 0.5,
                pulse: ImpactDefinition {
                    delivery: Delivery::Area {
                        radius: 12.0,
                        delay: 0.0,
                    },
                    effects: vec![Effect::Damage(DamageSpec::magical(5.0))],
                },
            },
        ),
    ] {
        app.world_mut()
            .resource_mut::<AbilityCatalog>()
            .register(
                id,
                AbilityDefinition {
                    name: format!("Reset test {}", id.0),
                    targeting,
                    range: 20.0,
                    windup: 0.0,
                    recovery: 0.0,
                    cooldown: 5.0,
                    mana_cost: 20.0,
                    execution,
                },
            )
            .unwrap();
    }
    app.world_mut()
        .resource_scope(|world, mut catalog: Mut<CharacterCatalog>| {
            catalog
                .register(
                    CHARACTER,
                    CharacterDefinition {
                        name: "Reset character".into(),
                        description: "Retained content".into(),
                        stats: UnitStats {
                            mana_regen: 0.0,
                            ..default()
                        },
                        default_loadout: [
                            Some(CONTROL),
                            Some(PROJECTILE),
                            Some(AREA),
                            Some(CHANNEL),
                        ],
                    },
                    world.resource::<AbilityCatalog>(),
                )
                .unwrap();
        });
    app
}

fn character(world: &mut World, position: Vec2) -> Entity {
    spawn_character(
        world,
        CharacterSpawnSpec {
            character: CHARACTER,
            team: Team::Ally,
            position,
            loadout: LoadoutSelection::CharacterDefault,
            respawn_delay: Some(0.1),
        },
    )
    .unwrap()
}

fn npc(world: &mut World) -> Entity {
    spawn_unit(
        world,
        UnitSpec {
            name: "Training NPC".into(),
            team: Team::Enemy,
            position: Vec2::new(4.0, 0.0),
            stats: UnitStats::default(),
            abilities: [None; 4],
            respawn_delay: Some(0.1),
        },
    )
    .unwrap()
}

fn cast(world: &mut World, unit: Entity, slot: usize, target: AbilityTarget) {
    world
        .resource_mut::<CommandQueue>()
        .submit(unit, UnitCommand::CastAbility { slot, target });
}

fn entities_with<T: Component>(world: &mut World) -> Vec<Entity> {
    world
        .query_filtered::<Entity, With<T>>()
        .iter(world)
        .collect()
}

#[test]
fn reset_removes_the_whole_trial_and_linked_visuals_while_retaining_static_content() {
    let mut app = arena();
    let world = app.world_mut();
    let static_root = world.spawn(Name::new("Static scene")).id();
    let static_child = world
        .spawn((Name::new("Static decoration"), ChildOf(static_root)))
        .id();
    let actor = character(world, Vec2::new(-4.0, 0.0));
    let area_caster = character(world, Vec2::new(-4.0, 1.0));
    let channel_caster = character(world, Vec2::new(-4.0, -1.0));
    let target = npc(world);
    cast(world, actor, 0, AbilityTarget::Unit(target));
    world.run_schedule(FixedUpdate);
    assert!(!world.get::<UnitStatuses>(target).unwrap().is_empty());
    assert!(world.get::<ControlState>(target).unwrap().forced_motion());
    assert_eq!(world.get::<UnitState>(target).unwrap().health(), 590.0);
    cast(world, actor, 1, AbilityTarget::Direction(Vec2::X));
    cast(
        world,
        area_caster,
        2,
        AbilityTarget::Point(Vec2::new(4.0, 0.0)),
    );
    cast(world, channel_caster, 3, AbilityTarget::SelfCast);
    world.run_schedule(FixedUpdate);
    let projectiles = entities_with::<Projectile>(world);
    let areas = entities_with::<AreaEffect>(world);
    assert_eq!(projectiles.len(), 1);
    assert_eq!(areas.len(), 1);
    assert!(matches!(
        world.get::<UnitAction>(channel_caster),
        Some(UnitAction::Channeling { .. })
    ));
    assert!(
        world
            .get::<AbilitySlots>(actor)
            .unwrap()
            .cooldown(1)
            .unwrap()
            > 0.0
    );
    let mut trial_entities = entities_with::<Unit>(world);
    trial_entities.extend(projectiles);
    trial_entities.extend(areas);
    for root in trial_entities.clone() {
        let child = world
            .spawn((Name::new("Linked visual"), ChildOf(root)))
            .id();
        let grandchild = world
            .spawn((Name::new("Nested model"), ChildOf(child)))
            .id();
        trial_entities.extend([child, grandchild]);
    }
    let source = world
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .unwrap();
    world.resource_mut::<AttributeChanges>().upsert(
        actor,
        source,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 100.0,
                ..default()
            },
            ..default()
        },
    );
    world
        .resource_mut::<CommandQueue>()
        .submit(actor, UnitCommand::MoveTo(Vec2::splat(8.0)));
    assert!(world.resource::<SimulationTick>().0 > 0);
    let mut messages = MessageCursor::<SimulationEvent>::default();
    assert!(!world.resource::<Messages<SimulationEvent>>().is_empty());

    reset_simulation(world);

    assert!(world.resource::<SimulationControl>().paused);
    assert_eq!(world.resource::<SimulationTick>().0, 0);
    assert!(
        trial_entities
            .iter()
            .all(|entity| world.get_entity(*entity).is_err())
    );
    assert!(world.get_entity(static_root).is_ok());
    assert!(world.get_entity(static_child).is_ok());
    assert!(entities_with::<Unit>(world).is_empty());
    assert!(entities_with::<Projectile>(world).is_empty());
    assert!(entities_with::<AreaEffect>(world).is_empty());
    assert!(entities_with::<CharacterIdentity>(world).is_empty());
    assert_eq!(
        messages
            .read(world.resource::<Messages<SimulationEvent>>())
            .count(),
        0
    );
    let map = world.resource::<NavigationMap>();
    assert!(map.walkable(Vec2::ZERO, 0.45));
    assert!(!map.walkable(Vec2::splat(10.0), 0.45));
    assert_eq!(
        world
            .resource::<CharacterCatalog>()
            .get(CHARACTER)
            .unwrap()
            .description,
        "Retained content"
    );
    for id in [CONTROL, PROJECTILE, AREA, CHANNEL] {
        assert!(world.resource::<AbilityCatalog>().get(id).is_some());
    }
    world.run_schedule(FixedUpdate);
    assert_eq!(world.resource::<SimulationTick>().0, 0);

    let new_actor = character(world, Vec2::new(-4.0, 0.0));
    let new_target = npc(world);
    assert_ne!(new_actor, actor);
    assert_ne!(new_target, target);
    world.resource_mut::<SimulationControl>().paused = false;
    for _ in 0..180 {
        world.run_schedule(FixedUpdate);
    }
    assert_eq!(world.resource::<SimulationTick>().0, 180);
    assert_eq!(world.get::<UnitState>(new_target).unwrap().health(), 600.0);
    assert_eq!(
        world
            .get::<Unit>(new_actor)
            .unwrap()
            .attributes()
            .attack_damage(),
        40.0
    );
    assert_eq!(
        world.get::<UnitPose>(new_actor).unwrap().position(),
        Vec2::new(-4.0, 0.0)
    );
    assert!(world.get::<UnitStatuses>(new_target).unwrap().is_empty());
    assert!(world.get::<ControlState>(new_target).unwrap().can_move());
    assert_eq!(
        messages
            .read(world.resource::<Messages<SimulationEvent>>())
            .count(),
        0
    );
    cast(world, new_actor, 0, AbilityTarget::Unit(new_target));
    world.run_schedule(FixedUpdate);
    assert_eq!(world.get::<UnitState>(new_target).unwrap().health(), 590.0);
    assert_eq!(world.get::<UnitState>(new_actor).unwrap().mana(), 280.0);
}

#[test]
fn reset_is_repeatable_without_recycling_source_or_status_identity() {
    let mut app = arena();
    let world = app.world_mut();
    let actor = character(world, Vec2::ZERO);
    let target = npc(world);
    let source_before = world
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .unwrap();
    cast(world, actor, 0, AbilityTarget::Unit(target));
    world.run_schedule(FixedUpdate);
    let last_status = world
        .get::<UnitStatuses>(target)
        .unwrap()
        .iter()
        .map(StatusView::id)
        .max()
        .unwrap();

    reset_simulation(world);
    reset_simulation(world);

    let source_after = world
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .unwrap();
    assert!(source_after > source_before);
    let actor = character(world, Vec2::ZERO);
    let target = npc(world);
    world.resource_mut::<AttributeChanges>().upsert(
        actor,
        source_after,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 10.0,
                ..default()
            },
            ..default()
        },
    );
    world.resource_mut::<SimulationControl>().paused = false;
    cast(world, actor, 0, AbilityTarget::Unit(target));
    world.run_schedule(FixedUpdate);
    assert_eq!(world.resource::<SimulationTick>().0, 1);
    assert_eq!(
        world
            .get::<Unit>(actor)
            .unwrap()
            .attributes()
            .attack_damage(),
        50.0
    );
    assert!(
        world
            .get::<UnitStatuses>(target)
            .unwrap()
            .iter()
            .all(|status| status.id() > last_status)
    );
    assert!(!world.get::<UnitStatuses>(target).unwrap().is_empty());
}
