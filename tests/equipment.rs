//! Equipment behavior crosses the same public interface as the client.
use bevy::{ecs::message::MessageCursor, prelude::*};
use kiva::simulation::*;

const DIRECT: AbilityId = AbilityId(100);
const SHOT: AbilityId = AbilityId(101);
const AREA: AbilityId = AbilityId(102);
const DASH: AbilityId = AbilityId(103);
const CHANNEL: AbilityId = AbilityId(104);
const KILL: AbilityId = AbilityId(105);
const ROOT: AbilityId = AbilityId(106);
const STUN: AbilityId = AbilityId(107);
const EXPENSIVE: AbilityId = AbilityId(108);
const OVERFLOW: AbilityId = AbilityId(109);
const SWORD: EquipmentId = EquipmentId(1);
const BOW: EquipmentId = EquipmentId(2);
const FROST: EquipmentId = EquipmentId(3);
const ORB: EquipmentId = EquipmentId(4);
const LEAF: EquipmentId = EquipmentId(5);
const PRISM: EquipmentId = EquipmentId(6);
const SHOT_ITEM: EquipmentId = EquipmentId(7);
const AREA_ITEM: EquipmentId = EquipmentId(8);
const EXPENSIVE_ITEM: EquipmentId = EquipmentId(9);
const OVERFLOW_ITEM: EquipmentId = EquipmentId(10);

fn magic(base: f32, ap_ratio: f32) -> Effect {
    Effect::Damage(DamageSpec {
        kind: DamageType::Magical,
        base,
        ad_ratio: 0.0,
        ap_ratio,
    })
}

fn item(name: &str) -> EquipmentDefinition {
    EquipmentDefinition {
        name: name.into(),
        description: String::new(),
        flat: AttributeDelta::default(),
        additive_percent: AttributeDelta::default(),
        passives: vec![],
        active: None,
    }
}

struct Arena {
    app: App,
    events: MessageCursor<SimulationEvent>,
}
impl Arena {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.insert_resource(NavigationMap::new(25.0, 1.0, vec![]));
        let duration = StatusDuration::from_ticks(60).unwrap();
        let direct = |effects| {
            AbilityExecution::Impact(ImpactDefinition {
                delivery: Delivery::Direct,
                effects,
            })
        };
        for (id, targeting, execution, cost, windup) in [
            (
                DIRECT,
                Targeting::Enemy,
                direct(vec![
                    magic(40.0, 0.5),
                    Effect::ApplyStatus(StatusSpec::Root { duration }),
                ]),
                20.0,
                0.1,
            ),
            (
                SHOT,
                Targeting::Direction,
                AbilityExecution::Impact(ImpactDefinition {
                    delivery: Delivery::Projectile {
                        speed: 1.0,
                        radius: 0.2,
                    },
                    effects: vec![magic(100.0, 1.0)],
                }),
                20.0,
                0.0,
            ),
            (
                AREA,
                Targeting::Point,
                AbilityExecution::Impact(ImpactDefinition {
                    delivery: Delivery::Area {
                        radius: 2.0,
                        delay: 1.0,
                    },
                    effects: vec![magic(50.0, 0.5)],
                }),
                20.0,
                0.0,
            ),
            (
                DASH,
                Targeting::Point,
                AbilityExecution::Dash { speed: 6.0 },
                10.0,
                0.1,
            ),
            (
                CHANNEL,
                Targeting::SelfCast,
                AbilityExecution::Channel {
                    duration: 2.0,
                    interval: 0.25,
                    pulse: ImpactDefinition {
                        delivery: Delivery::Area {
                            radius: 4.0,
                            delay: 0.3,
                        },
                        effects: vec![magic(10.0, 1.0)],
                    },
                },
                30.0,
                0.0,
            ),
            (
                KILL,
                Targeting::Enemy,
                direct(vec![Effect::Damage(DamageSpec {
                    kind: DamageType::True,
                    base: 1e6,
                    ad_ratio: 0.0,
                    ap_ratio: 0.0,
                })]),
                0.0,
                0.0,
            ),
            (
                ROOT,
                Targeting::Enemy,
                direct(vec![Effect::ApplyStatus(StatusSpec::Root { duration })]),
                0.0,
                0.0,
            ),
            (
                STUN,
                Targeting::Enemy,
                direct(vec![Effect::ApplyStatus(StatusSpec::Stun { duration })]),
                0.0,
                0.0,
            ),
            (
                EXPENSIVE,
                Targeting::Enemy,
                direct(vec![magic(10.0, 0.0)]),
                1e6,
                0.0,
            ),
            (
                OVERFLOW,
                Targeting::Enemy,
                direct(vec![Effect::Damage(DamageSpec {
                    kind: DamageType::Magical,
                    base: f32::MAX,
                    ad_ratio: f32::MAX,
                    ap_ratio: 0.0,
                })]),
                20.0,
                0.0,
            ),
        ] {
            app.world_mut()
                .resource_mut::<AbilityCatalog>()
                .register(
                    id,
                    AbilityDefinition {
                        name: format!("Ability {}", id.0),
                        targeting,
                        range: 20.0,
                        windup,
                        recovery: 0.0,
                        cooldown: 2.0,
                        mana_cost: cost,
                        execution,
                    },
                )
                .unwrap();
        }
        let mut arena = Self {
            app,
            events: MessageCursor::default(),
        };
        let mut sword = item("Sword");
        sword.flat.attack_damage = 20.0;
        let mut bow = item("Bow");
        bow.flat.attack_range = 0.5;
        bow.additive_percent.attack_speed = 0.25;
        let mut frost = item("Frost");
        frost.flat.ability_power = 10.0;
        frost.passives.push(EquipmentPassive::OnBasicAttackHit {
            name: "Frost hit".into(),
            effects: vec![
                magic(12.0, 0.1),
                Effect::ApplyStatus(StatusSpec::Slow {
                    duration,
                    fraction: 0.2,
                }),
            ],
        });
        for (id, definition) in [(SWORD, sword), (BOW, bow), (FROST, frost)] {
            arena.register(id, definition).unwrap();
        }
        for (id, active, ad, ap) in [
            (ORB, DIRECT, 0.0, 15.0),
            (LEAF, DASH, 10.0, 0.0),
            (PRISM, CHANNEL, 0.0, 20.0),
            (SHOT_ITEM, SHOT, 0.0, 30.0),
            (AREA_ITEM, AREA, 0.0, 0.0),
            (EXPENSIVE_ITEM, EXPENSIVE, 0.0, 0.0),
            (OVERFLOW_ITEM, OVERFLOW, 0.0, 0.0),
        ] {
            let mut definition = item(&format!("Item {}", id.0));
            definition.active = Some(active);
            definition.flat.attack_damage = ad;
            definition.flat.ability_power = ap;
            arena.register(id, definition).unwrap();
        }
        arena
    }
    fn world(&self) -> &World {
        self.app.world()
    }
    fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }
    fn register(
        &mut self,
        id: EquipmentId,
        definition: EquipmentDefinition,
    ) -> Result<(), EquipmentError> {
        self.world_mut()
            .resource_scope(|world, mut catalog: Mut<EquipmentCatalog>| {
                catalog.register(id, definition, world.resource::<AbilityCatalog>())
            })
    }
    fn spawn(
        &mut self,
        team: Team,
        position: Vec2,
        abilities: AbilityLoadout,
        respawn: Option<f32>,
    ) -> Entity {
        spawn_unit(
            self.world_mut(),
            UnitSpec {
                name: "Unit".into(),
                team,
                position,
                stats: UnitStats::default(),
                abilities,
                respawn_delay: respawn,
            },
        )
        .unwrap()
    }
    fn actor(&mut self) -> Entity {
        self.spawn(
            Team::Ally,
            Vec2::new(-5.0, 0.0),
            [Some(DIRECT), Some(SHOT), Some(DASH), Some(CHANNEL)],
            None,
        )
    }
    fn target(&mut self) -> Entity {
        self.spawn(Team::Enemy, Vec2::ZERO, [None; 4], None)
    }
    fn step(&mut self, count: usize) {
        for _ in 0..count {
            self.world_mut().run_schedule(FixedUpdate);
        }
    }
    fn command(&mut self, unit: Entity, command: UnitCommand) {
        self.world_mut()
            .resource_mut::<CommandQueue>()
            .submit(unit, command);
    }
    fn equip(&mut self, unit: Entity, slot: usize, id: EquipmentId) -> EquipmentInstanceId {
        equip_equipment(self.world_mut(), unit, slot, id).unwrap()
    }
    fn use_item(&mut self, unit: Entity, slot: usize, target: AbilityTarget) {
        let instance = self.slots(unit).item(slot).unwrap().instance();
        self.command(
            unit,
            UnitCommand::UseEquipment {
                slot,
                instance,
                target,
            },
        );
    }
    fn slots(&self, unit: Entity) -> &EquipmentSlots {
        self.world().get::<EquipmentSlots>(unit).unwrap()
    }
    fn state(&self, unit: Entity) -> &UnitState {
        self.world().get::<UnitState>(unit).unwrap()
    }
    fn attributes(&self, unit: Entity) -> &EffectiveAttributes {
        self.world().get::<Unit>(unit).unwrap().attributes()
    }
    fn drain(&mut self) -> Vec<SimulationEvent> {
        self.events
            .read(self.app.world().resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect()
    }
    fn attack_once(&mut self, unit: Entity, target: Entity) {
        self.command(unit, UnitCommand::AttackTarget(target));
        for _ in 0..100 {
            self.step(1);
            if self
                .drain()
                .iter()
                .any(|e| matches!(e, SimulationEvent::AttackReleased {source,..} if *source==unit))
            {
                self.command(unit, UnitCommand::Stop);
                self.step(1);
                return;
            }
        }
        panic!("attack did not release");
    }
}

#[test]
fn catalog_rejects_invalid_definitions_without_replacing_existing_content() {
    let mut a = Arena::new();
    assert_eq!(
        a.register(SWORD, item("Replacement")),
        Err(EquipmentError::DuplicateId(SWORD))
    );
    assert_eq!(
        a.world()
            .resource::<EquipmentCatalog>()
            .get(SWORD)
            .unwrap()
            .name,
        "Sword"
    );
    let mut invalid = vec![item(" ")];
    let mut nan = item("Nan");
    nan.flat.armor = f32::NAN;
    invalid.push(nan);
    let mut infinity = item("Infinite");
    infinity.additive_percent.attack_speed = f32::INFINITY;
    invalid.push(infinity);
    for (name, effects) in [
        ("", vec![magic(10.0, 0.0)]),
        ("Empty", vec![]),
        ("Negative", vec![magic(-1.0, 0.0)]),
        (
            "Slow",
            vec![Effect::ApplyStatus(StatusSpec::Slow {
                duration: StatusDuration::from_ticks(1).unwrap(),
                fraction: 0.0,
            })],
        ),
    ] {
        let mut definition = item("Bad passive");
        definition
            .passives
            .push(EquipmentPassive::OnBasicAttackHit {
                name: name.into(),
                effects,
            });
        invalid.push(definition);
    }
    for definition in invalid {
        assert_eq!(
            a.register(EquipmentId(20), definition),
            Err(EquipmentError::InvalidDefinition)
        );
        assert!(
            a.world()
                .resource::<EquipmentCatalog>()
                .get(EquipmentId(20))
                .is_none()
        );
    }
    let mut unknown = item("Unknown active");
    unknown.active = Some(AbilityId(999));
    assert_eq!(
        a.register(EquipmentId(20), unknown),
        Err(EquipmentError::UnknownActive(AbilityId(999)))
    );
    a.register(EquipmentId(30), item("Thirty")).unwrap();
    a.register(EquipmentId(20), item("Twenty")).unwrap();
    let ids: Vec<_> = a
        .world()
        .resource::<EquipmentCatalog>()
        .iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(
        ids.iter().rev().take(2).copied().collect::<Vec<_>>(),
        vec![EquipmentId(30), EquipmentId(20)]
    );
}

#[test]
fn six_slots_are_atomic_source_owned_and_immediate_while_paused() {
    let mut a = Arena::new();
    let actor = a.actor();
    a.world_mut().resource_mut::<SimulationControl>().paused = true;
    assert!((0..EQUIPMENT_SLOTS).all(|slot| a.slots(actor).item(slot).is_none()));
    let first = a.equip(actor, 0, SWORD);
    let second = a.equip(actor, 1, SWORD);
    assert_ne!(first, second);
    assert_ne!(first.to_string(), second.to_string());
    assert_eq!(a.attributes(actor).attack_damage(), 80.0);
    for slot in 2..EQUIPMENT_SLOTS {
        a.equip(actor, slot, BOW);
    }
    assert_eq!(a.attributes(actor).attack_speed(), 2.5);
    assert_eq!(a.attributes(actor).attack_range(), 7.5);
    assert_eq!(
        equip_equipment(a.world_mut(), actor, 0, FROST),
        Err(EquipmentError::SlotOccupied)
    );
    assert_eq!(
        equip_equipment(a.world_mut(), actor, 6, FROST),
        Err(EquipmentError::InvalidSlot)
    );
    let unrelated = a.world_mut().spawn_empty().id();
    assert_eq!(
        equip_equipment(a.world_mut(), unrelated, 0, SWORD),
        Err(EquipmentError::InvalidUnit)
    );
    assert_eq!(
        unequip_equipment(a.world_mut(), actor, 0, second),
        Err(EquipmentError::StaleInstance)
    );
    assert_eq!(a.attributes(actor).attack_damage(), 80.0);
    unequip_equipment(a.world_mut(), actor, 0, first).unwrap();
    assert_eq!(a.attributes(actor).attack_damage(), 60.0);
    assert_eq!(
        equip_equipment(a.world_mut(), actor, 0, EquipmentId(999)),
        Err(EquipmentError::UnknownEquipment(EquipmentId(999)))
    );
    assert!(a.slots(actor).item(0).is_none());
    let third = a.equip(actor, 0, SWORD);
    assert!(third > second);
    assert_eq!(
        unequip_equipment(a.world_mut(), actor, 0, first),
        Err(EquipmentError::StaleInstance)
    );
    assert_eq!(a.slots(actor).item(0).unwrap().instance(), third);
    a.step(20);
    assert_eq!(a.world().resource::<SimulationTick>().0, 0);
    for _ in 0..3 {
        let current = a.slots(actor).item(0).unwrap().instance();
        unequip_equipment(a.world_mut(), actor, 0, current).unwrap();
        a.equip(actor, 0, SWORD);
    }
    assert_eq!(a.attributes(actor).attack_damage(), 80.0);
    for slot in 0..EQUIPMENT_SLOTS {
        let current = a.slots(actor).item(slot).unwrap().instance();
        unequip_equipment(a.world_mut(), actor, slot, current).unwrap();
    }
    assert_eq!(a.attributes(actor).attack_damage(), 40.0);
    assert_eq!(a.attributes(actor).attack_speed(), 1.25);
    assert_eq!(a.attributes(actor).attack_range(), 5.5);
}

#[test]
fn shared_ability_definitions_have_independent_role_and_equipment_cooldowns_and_status_origins() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let first = a.equip(actor, 0, ORB);
    let second = a.equip(actor, 1, ORB);
    a.use_item(actor, 0, AbilityTarget::Unit(target));
    a.step(7);
    assert!(a.slots(actor).item(0).unwrap().cooldown() > 1.9);
    assert_eq!(a.slots(actor).item(1).unwrap().cooldown(), 0.0);
    assert_eq!(
        a.world().get::<AbilitySlots>(actor).unwrap().cooldown(0),
        Some(0.0)
    );
    a.use_item(actor, 1, AbilityTarget::Unit(target));
    a.step(7);
    a.command(
        actor,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(target),
        },
    );
    a.step(7);
    assert!(
        a.world()
            .get::<AbilitySlots>(actor)
            .unwrap()
            .cooldown(0)
            .unwrap()
            > 1.9
    );
    assert!(a.slots(actor).item(0).unwrap().cooldown() > 1.5);
    assert!(a.slots(actor).item(1).unwrap().cooldown() > 1.8);
    assert_eq!(
        a.world()
            .get::<UnitStatuses>(target)
            .unwrap()
            .iter()
            .filter(|status| status.kind() == StatusKind::Root)
            .count(),
        3
    );
    assert!(
        a.world()
            .get::<UnitStatuses>(target)
            .unwrap()
            .iter()
            .all(|status| status.ability() == Some(DIRECT))
    );
    let events = a.drain();
    for instance in [first, second] {
        assert!(events.iter().any(|event| matches!(
            event,
            SimulationEvent::Damaged {
                origin: DamageOrigin::EquipmentActive {
                    equipment: ORB,
                    instance: actual,
                    ability: DIRECT,
                },
                ..
            } if *actual == instance
        )));
    }
    assert!(events.iter().any(|event| matches!(
        event,
        SimulationEvent::Damaged {
            origin: DamageOrigin::Ability(DIRECT),
            ..
        }
    )));
}

#[test]
fn equipment_cast_failures_preserve_existing_orders_resources_and_other_sources() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let first = a.equip(actor, 0, ORB);
    let other = a.equip(actor, 1, SWORD);
    a.equip(actor, 2, EXPENSIVE_ITEM);
    a.equip(actor, 3, OVERFLOW_ITEM);
    a.command(actor, UnitCommand::MoveTo(Vec2::new(-5.0, 5.0)));
    a.step(1);
    for command in [
        UnitCommand::UseEquipment {
            slot: 0,
            instance: other,
            target: AbilityTarget::Unit(target),
        },
        UnitCommand::UseEquipment {
            slot: 6,
            instance: first,
            target: AbilityTarget::Unit(target),
        },
        UnitCommand::UseEquipment {
            slot: 1,
            instance: other,
            target: AbilityTarget::Unit(target),
        },
        UnitCommand::UseEquipment {
            slot: 0,
            instance: first,
            target: AbilityTarget::Point(Vec2::ZERO),
        },
    ] {
        a.command(actor, command);
        a.step(1);
    }
    a.use_item(actor, 2, AbilityTarget::Unit(target));
    a.step(1);
    a.use_item(actor, 3, AbilityTarget::Unit(target));
    a.step(1);
    assert_eq!(a.state(actor).mana(), 300.0);
    assert!(matches!(
        a.world().get::<UnitOrder>(actor),
        Some(UnitOrder::MoveTo(_))
    ));
    for slot in 0..4 {
        assert_eq!(a.slots(actor).item(slot).unwrap().cooldown(), 0.0);
    }
    let events = a.drain();
    for reason in [
        Rejection::NoAbility,
        Rejection::InvalidTarget,
        Rejection::InsufficientMana,
        Rejection::InvalidDamage,
    ] {
        assert!(events.iter().any(|event| matches!(
            event,
            SimulationEvent::CommandRejected { unit, reason: actual }
                if *unit == actor && *actual == reason
        )));
    }
}

#[test]
fn item_casts_share_control_and_busy_rules_with_ordinary_skills() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let controller = a.spawn(
        Team::Enemy,
        Vec2::new(-5.0, 2.0),
        [Some(ROOT), Some(STUN), None, None],
        None,
    );
    a.equip(actor, 0, LEAF);
    a.equip(actor, 1, ORB);
    a.command(
        controller,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(actor),
        },
    );
    a.step(1);
    a.use_item(actor, 0, AbilityTarget::Point(Vec2::new(-3.0, 0.0)));
    a.step(1);
    assert_eq!(a.slots(actor).item(0).unwrap().cooldown(), 0.0);
    a.use_item(actor, 1, AbilityTarget::Unit(target));
    a.step(7);
    assert!(
        a.slots(actor).item(1).unwrap().cooldown() > 0.0,
        "root permits a nonmovement item"
    );
    a.command(
        controller,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::Unit(actor),
        },
    );
    a.step(1);
    a.use_item(actor, 0, AbilityTarget::Point(Vec2::new(-3.0, 0.0)));
    a.step(1);
    assert!(a.drain().iter().any(|event| matches!(
        event,
        SimulationEvent::CommandRejected {
            unit,
            reason: Rejection::Controlled,
        } if *unit == actor
    )));

    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    a.equip(actor, 0, PRISM);
    a.equip(actor, 1, ORB);
    a.use_item(actor, 0, AbilityTarget::SelfCast);
    a.step(1);
    a.use_item(actor, 1, AbilityTarget::Unit(target));
    a.step(1);
    assert!(matches!(
        a.world().get::<UnitAction>(actor),
        Some(UnitAction::Channeling { .. })
    ));
    assert_eq!(a.slots(actor).item(1).unwrap().cooldown(), 0.0);
    assert!(a.drain().iter().any(|event| matches!(
        event,
        SimulationEvent::CommandRejected {
            unit,
            reason: Rejection::Busy,
        } if *unit == actor
    )));
}

#[test]
fn removing_an_instance_cancels_only_its_uncommitted_action_and_stale_requests_fail() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let first = a.equip(actor, 0, ORB);
    a.use_item(actor, 0, AbilityTarget::Unit(target));
    a.step(1);
    assert!(matches!(
        a.world().get::<UnitAction>(actor),
        Some(UnitAction::Casting {
            source: CastSource::EquipmentSlot { instance, .. },
            ..
        }) if *instance == first
    ));
    unequip_equipment(a.world_mut(), actor, 0, first).unwrap();
    assert!(matches!(
        a.world().get::<UnitAction>(actor),
        Some(UnitAction::Ready)
    ));
    let replacement = a.equip(actor, 0, ORB);
    assert_ne!(first, replacement);
    a.command(
        actor,
        UnitCommand::UseEquipment {
            slot: 0,
            instance: first,
            target: AbilityTarget::Unit(target),
        },
    );
    a.step(10);
    assert_eq!(a.state(target).health(), 600.0);
    assert_eq!(a.state(actor).mana(), 300.0);
    assert_eq!(a.slots(actor).item(0).unwrap().cooldown(), 0.0);
    a.command(
        actor,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(target),
        },
    );
    a.step(1);
    unequip_equipment(a.world_mut(), actor, 0, replacement).unwrap();
    assert!(matches!(
        a.world().get::<UnitAction>(actor),
        Some(UnitAction::Casting {
            source: CastSource::AbilitySlot(0),
            ..
        })
    ));
    a.step(6);
    assert_eq!(
        a.state(target).health(),
        560.0,
        "ordinary cast commits after its unrelated equipment is removed"
    );
}

#[test]
fn removal_preserves_committed_projectiles_areas_and_dash_but_stops_future_channel_pulses() {
    {
        let mut a = Arena::new();
        let actor = a.actor();
        let target = a.target();
        let instance = a.equip(actor, 0, SHOT_ITEM);
        a.use_item(actor, 0, AbilityTarget::Direction(Vec2::X));
        a.step(1);
        assert_eq!(
            a.world_mut().query::<&Projectile>().iter(a.world()).count(),
            1
        );
        unequip_equipment(a.world_mut(), actor, 0, instance).unwrap();
        a.step(300);
        assert_eq!(
            a.state(target).health(),
            470.0,
            "committed AP30 remains snapshotted"
        );
    }
    {
        let mut a = Arena::new();
        let actor = a.actor();
        let target = a.target();
        let instance = a.equip(actor, 0, AREA_ITEM);
        a.use_item(actor, 0, AbilityTarget::Point(Vec2::ZERO));
        a.step(1);
        unequip_equipment(a.world_mut(), actor, 0, instance).unwrap();
        a.step(70);
        assert_eq!(a.state(target).health(), 550.0);
    }
    {
        let mut a = Arena::new();
        let actor = a.actor();
        let instance = a.equip(actor, 0, LEAF);
        a.use_item(actor, 0, AbilityTarget::Point(Vec2::new(-1.0, 0.0)));
        a.step(7);
        unequip_equipment(a.world_mut(), actor, 0, instance).unwrap();
        a.step(45);
        assert!(
            a.world()
                .get::<UnitPose>(actor)
                .unwrap()
                .position()
                .distance(Vec2::new(-1.0, 0.0))
                < 0.001
        );
    }
    {
        let mut a = Arena::new();
        let actor = a.spawn(Team::Ally, Vec2::new(-3.0, 0.0), [None; 4], None);
        let target = a.target();
        let instance = a.equip(actor, 0, PRISM);
        a.use_item(actor, 0, AbilityTarget::SelfCast);
        a.step(16);
        assert_eq!(
            a.world_mut().query::<&AreaEffect>().iter(a.world()).count(),
            1
        );
        unequip_equipment(a.world_mut(), actor, 0, instance).unwrap();
        assert!(matches!(
            a.world().get::<UnitAction>(actor),
            Some(UnitAction::Ready)
        ));
        a.step(60);
        assert_eq!(
            a.state(target).health(),
            570.0,
            "one committed pulse survives, future pulses stop"
        );
        assert_eq!(
            a.world_mut().query::<&AreaEffect>().iter(a.world()).count(),
            0
        );
    }
}

#[test]
fn attack_passives_snapshot_at_release_and_do_not_follow_later_inventory_or_source_changes() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let frost = a.equip(actor, 0, FROST);
    let sword = a.equip(actor, 1, SWORD);
    a.attack_once(actor, target);
    unequip_equipment(a.world_mut(), actor, 0, frost).unwrap();
    unequip_equipment(a.world_mut(), actor, 1, sword).unwrap();
    let mut replacement = item("Later passive");
    replacement
        .passives
        .push(EquipmentPassive::OnBasicAttackHit {
            name: "Later".into(),
            effects: vec![magic(100.0, 0.0)],
        });
    a.register(EquipmentId(20), replacement).unwrap();
    a.equip(actor, 0, EquipmentId(20));
    a.world_mut().despawn(actor);
    a.step(25);
    assert_eq!(
        a.state(target).health(),
        527.0,
        "60 physical and13 magic remain committed after source removal"
    );
    assert_eq!(
        a.world()
            .get::<UnitStatuses>(target)
            .unwrap()
            .iter()
            .filter(|s| s.kind() == StatusKind::Slow)
            .count(),
        1
    );
    assert!(a.drain().iter().any(|event| matches!(
        event,
        SimulationEvent::Damaged {
            amount,
            origin: DamageOrigin::EquipmentPassive {
                equipment: FROST,
                instance,
                passive: 0,
            },
            ..
        } if *instance == frost && *amount == 13.0
    )));
}

#[test]
fn duplicate_passives_are_independent_and_skills_do_not_discover_attack_passives() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let mut definition = item("Root passive");
    definition
        .passives
        .push(EquipmentPassive::OnBasicAttackHit {
            name: "Root".into(),
            effects: vec![Effect::ApplyStatus(StatusSpec::Root {
                duration: StatusDuration::from_ticks(60).unwrap(),
            })],
        });
    a.register(EquipmentId(20), definition).unwrap();
    a.equip(actor, 0, FROST);
    a.equip(actor, 1, FROST);
    a.equip(actor, 2, EquipmentId(20));
    a.equip(actor, 3, EquipmentId(20));
    a.attack_once(actor, target);
    a.step(20);
    assert_eq!(a.state(target).health(), 532.0);
    let statuses = a.world().get::<UnitStatuses>(target).unwrap();
    assert!(statuses.iter().all(|status| status.ability().is_none()));
    assert_eq!(
        statuses
            .iter()
            .filter(|s| s.kind() == StatusKind::Root)
            .count(),
        2
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|s| s.kind() == StatusKind::Slow)
            .count(),
        2
    );
    let passives = a
        .drain()
        .into_iter()
        .filter(|e| {
            matches!(
                e,
                SimulationEvent::Damaged {
                    origin: DamageOrigin::EquipmentPassive { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        passives, 2,
        "the passive's own damage must not discover another passive"
    );

    let fresh = a.target();
    a.command(
        actor,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(fresh),
        },
    );
    a.step(7);
    assert_eq!(a.state(fresh).health(), 550.0);
    assert_eq!(
        a.world().get::<UnitStatuses>(fresh).unwrap().len(),
        1,
        "only the role's own root applies"
    );
    assert!(!a.drain().iter().any(|e| matches!(
        e,
        SimulationEvent::Damaged {
            origin: DamageOrigin::EquipmentPassive { .. },
            ..
        }
    )));
}

#[test]
fn passive_order_is_bounded_by_real_damage_and_target_life() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let mut zero = item("Zero attack");
    zero.flat.attack_damage = -40.0;
    zero.passives.push(EquipmentPassive::OnBasicAttackHit {
        name: "Never".into(),
        effects: vec![magic(100.0, 0.0)],
    });
    a.register(EquipmentId(20), zero).unwrap();
    a.equip(actor, 0, EquipmentId(20));
    a.attack_once(actor, target);
    a.step(20);
    assert_eq!(a.state(target).health(), 600.0);
    assert!(
        !a.drain()
            .iter()
            .any(|e| matches!(e, SimulationEvent::Damaged { .. }))
    );

    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let duration = StatusDuration::from_ticks(60).unwrap();
    let mut first = item("First");
    first.passives.push(EquipmentPassive::OnBasicAttackHit {
        name: "Kill then root".into(),
        effects: vec![
            magic(1000.0, 0.0),
            Effect::ApplyStatus(StatusSpec::Root { duration }),
        ],
    });
    let mut second = item("Second");
    second.passives.push(EquipmentPassive::OnBasicAttackHit {
        name: "Late".into(),
        effects: vec![Effect::ApplyStatus(StatusSpec::Stun { duration })],
    });
    a.register(EquipmentId(20), first).unwrap();
    a.register(EquipmentId(21), second).unwrap();
    a.equip(actor, 0, EquipmentId(20));
    a.equip(actor, 1, EquipmentId(21));
    a.attack_once(actor, target);
    a.step(20);
    assert!(!a.state(target).alive());
    let events = a.drain();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SimulationEvent::StatusApplied { .. }))
    );
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::Damaged {
            origin: DamageOrigin::EquipmentPassive {
                equipment: EquipmentId(20),
                ..
            },
            ..
        }
    )));
}

#[test]
fn invalid_passive_snapshot_cannot_partially_release_or_spend_an_attack_interval() {
    let mut a = Arena::new();
    let actor = a.actor();
    let target = a.target();
    let mut broken = item("Overflow");
    broken.passives.push(EquipmentPassive::OnBasicAttackHit {
        name: "Overflow".into(),
        effects: vec![Effect::Damage(DamageSpec {
            kind: DamageType::Magical,
            base: f32::MAX,
            ad_ratio: f32::MAX,
            ap_ratio: 0.0,
        })],
    });
    a.register(EquipmentId(20), broken).unwrap();
    a.equip(actor, 0, EquipmentId(20));
    a.command(actor, UnitCommand::AttackTarget(target));
    a.step(20);
    assert_eq!(a.state(actor).attack_cooldown(), 0.0);
    assert_eq!(a.state(target).health(), 600.0);
    assert_eq!(
        a.world_mut().query::<&Projectile>().iter(a.world()).count(),
        0
    );
    let events = a.drain();
    assert!(events.iter().any(|event| matches!(
        event,
        SimulationEvent::CommandRejected {
            unit,
            reason: Rejection::InvalidDamage,
        } if *unit == actor
    )));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SimulationEvent::AttackReleased { .. }))
    );
}

#[test]
fn death_preserves_equipment_and_ticking_cooldowns_while_pause_and_trial_reset_have_distinct_lifetimes()
 {
    let mut a = Arena::new();
    let actor = a.spawn(
        Team::Ally,
        Vec2::new(-5.0, 0.0),
        [Some(DIRECT), None, None, None],
        Some(0.2),
    );
    let target = a.target();
    let killer = a.spawn(
        Team::Enemy,
        Vec2::new(-5.0, 2.0),
        [Some(KILL), None, None, None],
        None,
    );
    let instance = a.equip(actor, 0, ORB);
    a.use_item(actor, 0, AbilityTarget::Unit(target));
    a.step(7);
    let before = a.slots(actor).item(0).unwrap().cooldown();
    a.command(
        killer,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::Unit(actor),
        },
    );
    a.step(1);
    assert!(!a.state(actor).alive());
    a.step(6);
    let dead = a.slots(actor).item(0).unwrap().cooldown();
    assert!(dead < before && dead > 1.8);
    a.world_mut().resource_mut::<SimulationControl>().paused = true;
    a.step(60);
    assert_eq!(a.slots(actor).item(0).unwrap().cooldown(), dead);
    let sword = a.equip(actor, 1, SWORD);
    assert_eq!(a.attributes(actor).attack_damage(), 60.0);
    a.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.step(10);
    assert!(a.state(actor).alive());
    assert_eq!(a.slots(actor).item(0).unwrap().instance(), instance);
    assert!(a.slots(actor).item(0).unwrap().cooldown() > 1.5);
    assert_eq!(a.attributes(actor).ability_power(), 15.0);
    assert_eq!(a.attributes(actor).attack_damage(), 60.0);
    a.command(
        actor,
        UnitCommand::UseEquipment {
            slot: 0,
            instance,
            target: AbilityTarget::Unit(target),
        },
    );
    reset_simulation(a.world_mut());
    assert!(a.world().resource::<EquipmentCatalog>().get(ORB).is_some());
    let fresh = a.actor();
    assert!((0..EQUIPMENT_SLOTS).all(|slot| a.slots(fresh).item(slot).is_none()));
    assert_eq!(a.attributes(fresh).attack_damage(), 40.0);
    let new = a.equip(fresh, 0, ORB);
    assert!(new > sword);
    a.world_mut().resource_mut::<SimulationControl>().paused = false;
    a.step(1);
    assert_eq!(a.slots(fresh).item(0).unwrap().cooldown(), 0.0);
}
