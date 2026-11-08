//! Deployed objects own their lifetimes, aura coverage and navigation handles.
use super::{
    mechanics::{self, RecastState},
    *,
};
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct ObjectSource {
    pub owner: UnitLifeRef,
    pub team: Team,
    pub origin: DamageOrigin,
}
impl From<&ImpactSnapshot> for ObjectSource {
    fn from(payload: &ImpactSnapshot) -> Self {
        Self {
            owner: payload.source,
            team: payload.team,
            origin: payload.origin,
        }
    }
}

pub(super) fn object(
    source: ObjectSource,
    position: Vec2,
    kind: ObjectKind,
    radius: f32,
    vision: f32,
    until: u64,
) -> BattlefieldObject {
    BattlefieldObject {
        kind,
        owner: source.owner,
        team: source.team,
        origin: source.origin,
        position,
        previous: position,
        radius,
        vision,
        until,
        aura: 0.0,
        aura_radius: 0.0,
        scout: None,
        active_at: 0,
        trail_distance: 0.0,
    }
}
pub(super) fn spawn_standard(
    world: &mut World,
    source: ObjectSource,
    position: Vec2,
    spec: &StandardSpec,
) {
    let tick = world.resource::<SimulationTick>().0;
    let delay = if let Delivery::Area { delay, .. } = spec.impact.delivery {
        delay
    } else {
        0.0
    };
    let active = tick + mechanics::ticks(delay);
    let mut object = object(
        source,
        position,
        ObjectKind::Standard,
        0.2,
        spec.vision_radius,
        active + mechanics::ticks(spec.duration),
    );
    object.aura = spec.attack_speed;
    object.aura_radius = spec.aura_radius;
    object.active_at = active;
    world.spawn(object);
}
pub(super) fn spawn_scout(
    world: &mut World,
    source: ObjectSource,
    position: Vec2,
    destination: Vec2,
    spec: ScoutSpec,
) {
    let tick = world.resource::<SimulationTick>().0;
    let mut object = object(
        source,
        position,
        ObjectKind::Scout,
        0.2,
        spec.vision_radius,
        tick + mechanics::ticks(
            ((position.as_dvec2() - destination.as_dvec2()).length() / f64::from(spec.speed)
                + f64::from(spec.destination_duration)) as f32,
        ),
    );
    object.scout = Some((destination, spec));
    world.spawn(object);
}
pub(super) fn reveal(
    world: &mut World,
    source: ObjectSource,
    position: Vec2,
    radius: f32,
    seconds: f32,
) {
    let tick = world.resource::<SimulationTick>().0;
    world.spawn(object(
        source,
        position,
        ObjectKind::Reveal,
        radius,
        radius,
        tick + mechanics::ticks(seconds),
    ));
}
pub(super) fn spawn_ring(
    world: &mut World,
    source: ObjectSource,
    position: Vec2,
    spec: RingTerrainSpec,
    slot: Option<usize>,
) {
    let tick = world.resource::<SimulationTick>().0;
    let id = world.spawn_empty().id();
    let mut proposed = world.resource::<NavigationMap>().clone();
    proposed.add_ring(id, position, spec.radius, spec.thickness);
    let Some(relocations) =
        super::movement::plan_ring_overlap(world, &proposed, position, spec.radius, spec.thickness)
    else {
        world.despawn(id);
        world.write_message(SimulationEvent::TerrainPlacementFailed {
            unit: source.owner.entity(),
            position,
        });
        return;
    };
    // Publish the object only after all collision geometry and displacements
    // agree. Failed placement preserves both prior walls and every actor pose;
    // the leap's already committed damage and cost remain authoritative.
    *world.resource_mut::<NavigationMap>() = proposed;
    super::movement::apply_terrain_relocations(world, relocations);
    world.entity_mut(id).insert(object(
        source,
        position,
        ObjectKind::Terrain,
        spec.radius,
        spec.radius + 1.0,
        tick + mechanics::ticks(spec.duration),
    ));
    if let Some(slot) = slot
        && let Some(mut state) = world.get_mut::<UnitMechanics>(source.owner.entity())
    {
        state.recasts[slot] = Some(RecastState {
            object: id,
            allowed_at: tick + mechanics::ticks(spec.recast_delay),
        });
    }
}
pub(super) fn advance(world: &mut World) {
    let tick = world.resource::<SimulationTick>().0;
    let mut objects: Vec<_> = world
        .query::<(Entity, &BattlefieldObject)>()
        .iter(world)
        .map(|(id, object)| (id, object.clone()))
        .collect();
    objects.sort_by_key(|(id, _)| id.to_bits());
    for (id, mut object) in objects {
        if object.until <= tick {
            if object.kind == ObjectKind::Terrain {
                world.resource_mut::<NavigationMap>().remove_ring(id);
            }
            world.despawn(id);
            continue;
        }
        object.previous = object.position;
        if let Some((destination, spec)) = object.scout {
            let delta = destination - object.position;
            let step = delta.clamp_length_max(spec.speed * TICK_SECONDS);
            object.position += step;
            object.trail_distance += step.length();
            if object.trail_distance >= 1.0 {
                object.trail_distance %= 1.0;
                reveal(
                    world,
                    ObjectSource {
                        owner: object.owner,
                        team: object.team,
                        origin: object.origin,
                    },
                    object.position,
                    spec.vision_radius,
                    spec.path_duration,
                );
            }
            if object.position.distance(destination) <= 0.01 {
                object.kind = ObjectKind::Reveal;
                object.radius = spec.vision_radius;
                object.scout = None;
                object.until = tick + mechanics::ticks(spec.destination_duration);
            }
        }
        world.entity_mut(id).insert(object);
    }
    update_auras(world, tick);
}
fn update_auras(world: &mut World, tick: u64) {
    let objects: Vec<_> = world
        .query::<&BattlefieldObject>()
        .iter(world)
        .filter(|object| object.kind == ObjectKind::Standard && object.active_at <= tick)
        .cloned()
        .collect();
    let heroes: Vec<_> = world
        .query::<(Entity, &Unit, &UnitState, &UnitPose, &UnitMechanics)>()
        .iter(world)
        .filter(|(_, unit, _, _, _)| unit.stats.kind == UnitKind::Hero)
        .map(|(id, unit, state, pose, mechanics)| {
            (
                id,
                unit.team,
                state.alive(),
                pose.position,
                mechanics.aura_source,
            )
        })
        .collect();
    for (id, team, alive, position, source) in heroes {
        let amount = if alive {
            objects
                .iter()
                .filter(|object| {
                    object.team == team && position.distance(object.position) <= object.aura_radius
                })
                .map(|object| object.aura)
                .fold(0.0_f32, f32::max)
        } else {
            0.0
        };
        let modifier = (amount > 0.0).then_some(AttributeModifier {
            additive_percent: AttributeDelta {
                attack_speed: amount,
                ..default()
            },
            removal: ModifierRemoval::OnDeath,
            ..default()
        });
        let _ = attributes::set_immediate(world, id, source, modifier);
    }
}
