//! Lifetimes and read-only views for reusable unit enhancements and field objects.
//! Combat, movement and attributes retain their own authoritative writes.
use super::*;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct ChargeState {
    pub count: u8,
    pub max: u8,
    pub recharge: u64,
    pub next: Option<u64>,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct RecastState {
    pub object: Entity,
    pub allowed_at: u64,
}
#[derive(Clone, Debug)]
pub(super) struct ShieldInstance {
    pub amount: f32,
    pub until: u64,
}
#[derive(Clone, Debug)]
pub(super) struct ArmorDebuff {
    pub source: ModifierSourceId,
    pub until: u64,
    pub caster: UnitLifeRef,
    pub origin: DamageOrigin,
}
#[derive(Component, Debug)]
pub struct UnitMechanics {
    pub(super) shields: Vec<ShieldInstance>,
    pub(super) focus: u8,
    pub(super) focus_until: u64,
    pub(super) empowered_until: u64,
    pub(super) multiplier: f32,
    pub(super) arrows: u8,
    pub(super) charges: [Option<ChargeState>; 4],
    pub(super) recasts: [Option<RecastState>; 4],
    pub(super) aura_source: ModifierSourceId,
    pub(super) empower_source: ModifierSourceId,
    pub(super) armor: Vec<ArmorDebuff>,
}
impl UnitMechanics {
    pub fn shield(&self) -> f32 {
        self.shields
            .iter()
            .map(|s| f64::from(s.amount))
            .sum::<f64>()
            .min(f64::from(f32::MAX)) as f32
    }
    pub fn focus(&self) -> u8 {
        self.focus
    }
    pub fn empowered(&self) -> bool {
        self.empowered_until > 0
    }
    pub fn arrows(&self) -> u8 {
        self.arrows
    }
    pub fn charges(&self, slot: usize) -> Option<u8> {
        self.charges.get(slot)?.as_ref().map(|c| c.count)
    }
    /// Remaining seconds are projected relative to the authoritative simulation tick.
    pub fn recharge(&self, slot: usize, tick: u64) -> Option<f32> {
        self.charges
            .get(slot)?
            .as_ref()?
            .next
            .map(|next| next.saturating_sub(tick) as f32 * TICK_SECONDS)
    }
    pub fn recast(&self, slot: usize) -> bool {
        self.recasts.get(slot).is_some_and(Option::is_some)
    }
    pub fn recast_ready(&self, slot: usize, tick: u64) -> bool {
        self.recasts
            .get(slot)
            .and_then(|v| *v)
            .is_some_and(|r| tick >= r.allowed_at)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectKind {
    Standard,
    Terrain,
    Scout,
    Reveal,
}
#[derive(Component, Clone, Debug)]
pub struct BattlefieldObject {
    pub(super) kind: ObjectKind,
    pub(super) owner: UnitLifeRef,
    pub(super) team: Team,
    pub(super) position: Vec2,
    pub(super) previous: Vec2,
    pub(super) radius: f32,
    pub(super) vision: f32,
    pub(super) until: u64,
    pub(super) aura: f32,
    pub(super) aura_radius: f32,
    pub(super) scout: Option<(Vec2, ScoutSpec)>,
    pub(super) active_at: u64,
    pub(super) trail_distance: f32,
}
impl BattlefieldObject {
    pub fn active(&self, tick: u64) -> bool {
        self.active_at <= tick && self.until > tick
    }
    pub fn kind(&self) -> ObjectKind {
        self.kind
    }
    pub fn owner(&self) -> UnitLifeRef {
        self.owner
    }
    pub fn team(&self) -> Team {
        self.team
    }
    pub fn position(&self) -> Vec2 {
        self.position
    }
    pub fn interpolated(&self, fraction: f32) -> Vec2 {
        self.previous.lerp(self.position, fraction.clamp(0.0, 1.0))
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn aura_radius(&self) -> f32 {
        self.aura_radius
    }
    pub fn attack_speed_aura(&self) -> f32 {
        self.aura
    }
    pub fn vision_radius(&self) -> f32 {
        self.vision
    }
    pub fn remaining_seconds(&self, tick: u64) -> f32 {
        self.until.saturating_sub(tick) as f32 * TICK_SECONDS
    }
}
#[derive(Resource, Default)]
pub(super) struct MechanicMemory {
    pub cooldowns: BTreeMap<(UnitLifeRef, usize, UnitLifeRef), u64>,
    pub groups: BTreeMap<(u64, UnitLifeRef), u64>,
}
#[derive(Clone, Debug, Default)]
pub(super) enum ProjectileBehavior {
    #[default]
    Ordinary,
    Volley {
        group: u64,
        expires: u64,
        hero_slow: Option<StatusSpec>,
    },
    Arrow {
        spec: ArrowSpec,
        traveled: f32,
    },
}

pub(super) fn ticks(seconds: f32) -> u64 {
    if seconds == 0.0 {
        0
    } else {
        (f64::from(seconds) * TICK_RATE - 1e-6).ceil().max(1.0) as u64
    }
}
pub(super) fn duration(seconds: f32) -> StatusDuration {
    StatusDuration::from_ticks(ticks(seconds).clamp(1, u32::MAX as u64) as u32)
        .expect("validated duration")
}
pub(super) fn passive_speed(loadout: &AbilityLoadout, catalog: &AbilityCatalog) -> f32 {
    let mut found = BTreeSet::new();
    loadout
        .iter()
        .flatten()
        .filter(|id| found.insert(**id))
        .filter_map(|id| match &catalog.get(*id)?.execution {
            AbilityExecution::Standard(spec) => Some(f64::from(spec.passive_attack_speed)),
            _ => None,
        })
        .sum::<f64>()
        .min(f64::from(f32::MAX)) as f32
}
pub(super) fn initialize(world: &mut World, entity: Entity) -> Result<(), &'static str> {
    let mut allocate = || {
        world
            .resource_mut::<AttributeChanges>()
            .allocate_source()
            .ok_or("attribute source space exhausted")
    };
    let passive_source = allocate()?;
    let aura_source = allocate()?;
    let empower_source = allocate()?;
    let slots = world.get::<AbilitySlots>(entity).expect("slots").ids;
    let tick = world.resource::<SimulationTick>().0;
    let catalog = world.resource::<AbilityCatalog>();
    let charges = std::array::from_fn(|slot| {
        slots[slot]
            .and_then(|id| catalog.get(id))
            .and_then(|definition| match definition.execution {
                AbilityExecution::Scout(spec) => Some(ChargeState {
                    count: spec.initial_charges,
                    max: spec.charges,
                    recharge: ticks(spec.recharge),
                    next: (spec.initial_charges < spec.charges)
                        .then_some(tick + ticks(spec.recharge)),
                }),
                _ => None,
            })
    });
    let speed = passive_speed(&slots, catalog);
    world.entity_mut(entity).insert(UnitMechanics {
        shields: Vec::new(),
        focus: 0,
        focus_until: 0,
        empowered_until: 0,
        multiplier: 1.0,
        arrows: 1,
        charges,
        recasts: [None; 4],
        aura_source,
        empower_source,
        armor: Vec::new(),
    });
    set_speed(
        world,
        entity,
        passive_source,
        speed,
        ModifierRemoval::Explicit,
    );
    Ok(())
}
fn set_speed(
    world: &mut World,
    entity: Entity,
    source: ModifierSourceId,
    speed: f32,
    removal: ModifierRemoval,
) {
    let modifier = (speed > 0.0).then_some(AttributeModifier {
        additive_percent: AttributeDelta {
            attack_speed: speed,
            ..default()
        },
        removal,
        ..default()
    });
    let _ = super::attributes::set_immediate(world, entity, source, modifier);
}
pub(super) fn before_commands(world: &mut World) {
    let tick = world.resource::<SimulationTick>().0;
    world
        .resource_mut::<MechanicMemory>()
        .cooldowns
        .retain(|_, until| *until > tick);
    world
        .resource_mut::<MechanicMemory>()
        .groups
        .retain(|_, until| *until > tick);
    let mut ids: Vec<_> = world
        .query_filtered::<Entity, With<UnitMechanics>>()
        .iter(world)
        .collect();
    ids.sort_by_key(|id| id.to_bits());
    for id in ids {
        let alive = world.get::<UnitState>(id).is_some_and(UnitState::alive);
        let slots = world.get::<AbilitySlots>(id).expect("slots").ids;
        let focus_spec = slots
            .iter()
            .flatten()
            .filter_map(|id| world.resource::<AbilityCatalog>().get(*id))
            .find_map(|definition| match definition.execution {
                AbilityExecution::Empower(spec) => Some(spec),
                _ => None,
            });
        let (expired_empower, empower_source, expired_armor) = {
            let mut state = world.get_mut::<UnitMechanics>(id).expect("mechanics");
            state
                .shields
                .retain(|shield| shield.until > tick && shield.amount > 0.0 && alive);
            for charges in state.charges.iter_mut().flatten() {
                while charges.next.is_some_and(|next| next <= tick) {
                    charges.count = (charges.count + 1).min(charges.max);
                    charges.next = if charges.count < charges.max {
                        charges.next.map(|next| next + charges.recharge)
                    } else {
                        None
                    };
                }
            }
            let expired = state.empowered_until > 0 && (state.empowered_until <= tick || !alive);
            if expired {
                state.empowered_until = 0;
                state.multiplier = 1.0;
                state.arrows = 1;
            }
            if !alive {
                state.focus = 0;
                state.focus_until = 0;
            }
            if let Some(spec) = focus_spec
                && state.focus > 0
                && state.focus_until <= tick
            {
                state.focus -= 1;
                state.focus_until = tick + ticks(spec.stack_decay);
            }
            let armor: Vec<_> = state
                .armor
                .iter()
                .filter(|debuff| debuff.until <= tick || !alive)
                .map(|debuff| debuff.source)
                .collect();
            state.armor.retain(|debuff| debuff.until > tick && alive);
            (expired, state.empower_source, armor)
        };
        if expired_empower {
            set_speed(world, id, empower_source, 0.0, ModifierRemoval::OnDeath);
        }
        for source in expired_armor {
            let _ = attributes::set_immediate(world, id, source, None);
        }
        let recasts = world.get::<UnitMechanics>(id).expect("mechanics").recasts;
        for (slot, recast) in recasts.into_iter().enumerate() {
            if recast.is_some_and(|r| world.get::<BattlefieldObject>(r.object).is_none()) {
                world
                    .get_mut::<UnitMechanics>(id)
                    .expect("mechanics")
                    .recasts[slot] = None;
            }
        }
    }
}

pub(super) fn clear_life(world: &mut World, entity: Entity) {
    if let Some(mut state) = world.get_mut::<UnitMechanics>(entity) {
        state.shields.clear();
        state.focus = 0;
        state.focus_until = 0;
        state.empowered_until = 0;
        state.multiplier = 1.0;
        state.arrows = 1;
        state.armor.clear();
        state.recasts = [None; 4];
    }
}
pub(super) fn absorb(world: &mut World, target: Entity, mut damage: f64) -> f64 {
    if let Some(mut state) = world.get_mut::<UnitMechanics>(target) {
        for shield in &mut state.shields {
            let used = damage.min(f64::from(shield.amount));
            shield.amount = (f64::from(shield.amount) - used).max(0.0) as f32;
            damage -= used;
            if damage <= 0.0 {
                break;
            }
        }
        state.shields.retain(|s| s.amount > 0.0);
    }
    damage
}
pub(super) fn armor_reduction(
    world: &mut World,
    target: UnitLifeRef,
    payload: &ImpactSnapshot,
    fraction: f32,
    seconds: f32,
) {
    let tick = world.resource::<SimulationTick>().0;
    let existing = world
        .get::<UnitMechanics>(target.entity())
        .and_then(|state| {
            state
                .armor
                .iter()
                .find(|debuff| debuff.caster == payload.source && debuff.origin == payload.origin)
        })
        .map(|debuff| debuff.source);
    let Some(source) =
        existing.or_else(|| world.resource_mut::<AttributeChanges>().allocate_source())
    else {
        return;
    };
    let modifier = AttributeModifier {
        additive_percent: AttributeDelta {
            armor: -fraction,
            ..default()
        },
        removal: ModifierRemoval::OnDeath,
        ..default()
    };
    if attributes::set_immediate(world, target.entity(), source, Some(modifier)).is_err() {
        return;
    }
    let mut state = world
        .get_mut::<UnitMechanics>(target.entity())
        .expect("mechanics");
    state.armor.retain(|debuff| debuff.source != source);
    state.armor.push(ArmorDebuff {
        source,
        until: tick + ticks(seconds),
        caster: payload.source,
        origin: payload.origin,
    });
}

/// Attack stacks are an actor resource shared by identical equipped empower
/// definitions. Different stack semantics must not silently compete for it.
pub(super) fn validate_loadout(
    loadout: &AbilityLoadout,
    catalog: &AbilityCatalog,
) -> Result<(), &'static str> {
    let mut stack = None;
    for id in loadout.iter().flatten() {
        if let Some(AbilityDefinition {
            execution: AbilityExecution::Empower(spec),
            ..
        }) = catalog.get(*id)
        {
            if stack.is_some_and(|current| current != *spec) {
                return Err("distinct attack-stack abilities cannot share one actor");
            }
            stack = Some(*spec);
        }
    }
    Ok(())
}
