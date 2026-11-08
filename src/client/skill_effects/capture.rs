use super::*;

const MAX_PENDING: usize = 192;

impl Runtime {
    fn queue(&mut self, fact: Fact) {
        if self.pending.len() == MAX_PENDING {
            self.pending.pop_front();
        }
        self.pending.push_back(fact);
    }
}

/// Read once per atomic tick, before a new life or gameplay cleanup can erase
/// the source. No event, transform or damage is manufactured by the renderer.
pub(super) fn capture(world: &mut World) {
    let current = session(world);
    let tick = world.resource::<SimulationTick>().0;
    world.resource_scope(|world, mut runtime: Mut<Runtime>| {
        if runtime.session != current {
            runtime.session = current;
            runtime.needs_clear = true;
            runtime.pending.clear();
            runtime.previous_objects.clear();
            runtime.arrivals.clear();
            runtime.projectile_art.clear();
            runtime.emitted.clear();
        }
        let events: Vec<_> = runtime
            .reader
            .read(world.resource::<Messages<SimulationEvent>>())
            .filter(|event| {
                matches!(
                    event,
                    SimulationEvent::AbilityCommitted { .. }
                        | SimulationEvent::ImpactResolved { .. }
                )
            })
            .cloned()
            .collect();
        if current.is_none() {
            return;
        }
        for event in events {
            let fact = match event {
                SimulationEvent::AbilityCommitted {
                    ability,
                    source_life,
                    origin,
                    target,
                    recast,
                    position: resolved_position,
                    ..
                } => {
                    if !recipes::known(ability) {
                        continue;
                    }
                    let Some(unit) = world.get::<Unit>(source_life.entity()) else {
                        continue;
                    };
                    let direction = match target {
                        AbilityTarget::Direction(direction) => direction,
                        AbilityTarget::Point(point) => normalize(point - origin),
                        AbilityTarget::Unit(_) => normalize(resolved_position - origin),
                        AbilityTarget::SelfCast => Vec2::NEG_Y,
                    };
                    Fact {
                        tick,
                        ability,
                        source: source_life,
                        team: unit.team(),
                        position: origin,
                        direction,
                        kind: FactKind::Cast { target, recast },
                    }
                }
                SimulationEvent::ImpactResolved {
                    source_life,
                    team,
                    origin,
                    empower_origin,
                    position,
                    direction,
                    kind,
                    ..
                } => {
                    let Some(id) = ability(origin)
                        .or_else(|| empower_origin.and_then(ability))
                        .filter(|id| recipes::known(*id))
                    else {
                        continue;
                    };
                    Fact {
                        tick,
                        ability: id,
                        source: source_life,
                        team,
                        position,
                        direction,
                        kind: FactKind::Impact(kind),
                    }
                }
                _ => unreachable!("filtered facts"),
            };
            let contact = matches!(fact.kind, FactKind::Impact(ImpactKind::Contact { .. }));
            if observable(world, fact.team, fact.position, contact) {
                runtime.queue(fact);
            }
        }
        let objects: Vec<_> = world
            .query::<(Entity, &BattlefieldObject)>()
            .iter(world)
            .map(|(id, object)| (id, object.clone()))
            .collect();
        let mut next = HashMap::new();
        for (id, object) in objects {
            next.insert(id, ObjectObservation::from(&object));
            if object.kind() == ObjectKind::Scout {
                runtime.arrivals.insert(id, false);
            }
            if runtime
                .previous_objects
                .get(&id)
                .is_some_and(|old| old.kind == ObjectKind::Scout)
                && object.kind() == ObjectKind::Reveal
            {
                runtime.arrivals.insert(id, true);
                if let Some(id) = ability(object.origin()).filter(|id| recipes::known(*id))
                    && observable(world, object.team(), object.position(), false)
                {
                    runtime.queue(Fact {
                        tick,
                        ability: id,
                        source: object.owner(),
                        team: object.team(),
                        position: object.position(),
                        direction: Vec2::NEG_Y,
                        kind: FactKind::ScoutArrived,
                    });
                }
            }
        }
        // Observe the disappearance at its actual fixed step. Rendering may
        // resume much later and must not give this old fact a fresh lifetime.
        let removed: Vec<_> = runtime
            .previous_objects
            .iter()
            .filter(|(id, old)| old.kind == ObjectKind::Terrain && !next.contains_key(*id))
            .map(|(_, old)| *old)
            .collect();
        for old in removed {
            if let Some(ability) = old.ability
                && observable(world, old.team, old.position, false)
            {
                runtime.queue(Fact {
                    tick,
                    ability,
                    source: old.source,
                    team: old.team,
                    position: old.position,
                    direction: Vec2::NEG_Y,
                    kind: FactKind::TerrainRemoved { radius: old.radius },
                });
            }
        }
        runtime.arrivals.retain(|id, _| next.contains_key(id));
        runtime.previous_objects = next;
    });
}

pub(super) fn normalize(value: Vec2) -> Vec2 {
    let length = value.as_dvec2().length();
    if length > 1e-9 && length.is_finite() {
        (value.as_dvec2() / length).as_vec2()
    } else {
        Vec2::NEG_Y
    }
}
