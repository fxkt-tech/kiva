//! Coordinate sampling and visual lifetimes on the shared presentation clock.
pub(super) use super::sampling::LiveKey;
use super::{
    capture::normalize,
    drawing::EffectGroup,
    layers::{burst_layers, live_layers},
    recipes::{self, Shape},
    sampling::{LiveKind, Request, requests},
    *,
};
use crate::client::{camera::GameCamera, presentation::PresentationFrame};

const MAX_BURSTS: usize = 96;
const MAX_LIVE: usize = 256;

pub(super) struct LiveVisual {
    request: Request,
    group: EffectGroup,
    born: f64,
}
pub(super) struct Burst {
    group: EffectGroup,
    shape: Shape,
    born: f64,
    seconds: f64,
    base: Transform,
    team: Team,
    contact: bool,
    ability: u16,
    attachment: Option<UnitLifeRef>,
}
fn heading(direction: Vec2) -> Quat {
    let direction = normalize(direction);
    Quat::from_rotation_y((-direction.x).atan2(-direction.y))
}
fn spawn_burst(world: &mut World, runtime: &mut Runtime, fact: &Fact) {
    let Some(recipe) = recipes::recipe(world, fact) else {
        return;
    };
    let age = runtime.clock - fact.tick as f64 / TICK_RATE;
    if age >= recipe.seconds {
        return;
    }
    let Some(group) = EffectGroup::spawn(world, None, burst_layers(recipe.shape)) else {
        return;
    };
    let billboard = matches!(recipe.shape, Shape::Spark { .. });
    let rotation = if billboard {
        camera_rotation(world)
    } else {
        heading(recipe.direction)
    };
    let base = Transform::from_xyz(recipe.position.x, recipe.height, recipe.position.y)
        .with_rotation(rotation);
    world.entity_mut(group.root()).insert(base);
    if runtime.bursts.len() == MAX_BURSTS {
        let old = runtime.bursts.pop_front().expect("bounded bursts");
        old.group.dispose(world);
    }
    runtime.bursts.push_back(Burst {
        group,
        shape: recipe.shape,
        born: fact.tick as f64 / TICK_RATE,
        seconds: recipe.seconds,
        base,
        team: fact.team,
        contact: matches!(fact.kind, FactKind::Impact(ImpactKind::Contact { .. })),
        ability: fact.ability.0,
        attachment: (billboard && matches!(fact.kind, FactKind::Cast { .. }))
            .then_some(fact.source),
    });
    *runtime.emitted.entry(fact.ability.0).or_default() += 1;
}

fn camera_rotation(world: &mut World) -> Quat {
    world
        .query_filtered::<&Transform, With<GameCamera>>()
        .iter(world)
        .next()
        .map_or(Quat::IDENTITY, |camera| camera.rotation)
}

fn spawn_live(world: &mut World, request: Request, clock: f64) -> Option<LiveVisual> {
    let owner = request.key.owner();
    if request.kind == LiveKind::Area {
        world.entity_mut(owner).insert((
            Transform::from_xyz(request.position.x, 0.0, request.position.y),
            Visibility::Inherited,
        ));
    }
    let group = EffectGroup::spawn(world, Some(owner), live_layers(&request))?;
    if request.kind == LiveKind::Shield {
        world.entity_mut(group.root()).insert(OriginalShield(owner));
    }
    if matches!(request.key, LiveKey::Carrier(_)) {
        // Replace only this carrier's old generic geometry. Unit rigs and
        // shields have separate ownership and are never traversed here.
        let children: Vec<_> = world
            .get::<Children>(owner)
            .map(|c| c.iter().filter(|id| *id != group.root()).collect())
            .unwrap_or_default();
        for child in children {
            let _ = world.despawn(child);
        }
        world
            .entity_mut(owner)
            .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>()
            .insert(ManagedEffect);
    }
    Some(LiveVisual {
        request,
        group,
        born: clock,
    })
}

fn reset(world: &mut World, runtime: &mut Runtime) {
    for (_, visual) in runtime.live.drain() {
        visual.group.dispose(world);
    }
    for burst in runtime.bursts.drain(..) {
        burst.group.dispose(world);
    }
    runtime.clock = 0.0;
    runtime.needs_clear = false;
}

pub(super) fn update(world: &mut World) {
    let current = session(world);
    let (sync, fraction) = world
        .get_resource::<PresentationFrame>()
        .map_or((true, 0.0), |frame| (frame.should_sync(), frame.fraction()));
    let tick = world.resource::<SimulationTick>().0;
    world.resource_scope(|world, mut runtime: Mut<Runtime>| {
        if runtime.session != current {
            runtime.session = current;
            runtime.pending.clear();
            runtime.previous_objects.clear();
            runtime.arrivals.clear();
            runtime.projectile_art.clear();
            runtime.emitted.clear();
            runtime.needs_clear = true;
        }
        if runtime.needs_clear {
            reset(world, &mut runtime);
        }
        if current.is_none() {
            publish(world, &runtime);
            return;
        }
        if !sync {
            publish(world, &runtime);
            return;
        }
        runtime.clock = (tick as f64 + f64::from(fraction)) / TICK_RATE;
        while let Some(fact) = runtime.pending.pop_front() {
            let contact = matches!(fact.kind, FactKind::Impact(ImpactKind::Contact { .. }));
            if observable(world, fact.team, fact.position, contact) {
                spawn_burst(world, &mut runtime, &fact);
            }
        }
        runtime
            .projectile_art
            .retain(|id, _| world.get::<Projectile>(*id).is_some());
        let desired = requests(world, &runtime.arrivals, &runtime.projectile_art);
        let wanted: std::collections::HashSet<_> = desired.iter().map(|r| r.key).collect();
        let removed: Vec<_> = runtime
            .live
            .keys()
            .copied()
            .filter(|key| !wanted.contains(key))
            .collect();
        for key in removed {
            let old = runtime.live.remove(&key).expect("known visual");
            old.group.dispose(world);
        }
        let camera = camera_rotation(world);
        let clock = runtime.clock;
        for request in desired {
            let rebuild = runtime.live.get(&request.key).is_none_or(|old| {
                old.request.kind != request.kind
                    || old.request.ability != request.ability
                    || old.request.source != request.source
                    || old.request.arrows != request.arrows
                    || old.request.radius != request.radius
                    || !world.entities().contains(old.group.root())
            });
            if rebuild {
                let keep_old = runtime.live.get(&request.key).is_some_and(|old| {
                    old.request.source == request.source
                        && old.request.ability == request.ability
                        && world.entities().contains(old.group.root())
                });
                // A new life cannot inherit an old attached effect. A valid
                // carrier changing phase can keep its current art until the
                // replacement is completely ready.
                if !keep_old && let Some(old) = runtime.live.remove(&request.key) {
                    old.group.dispose(world);
                }
                if !keep_old && runtime.live.len() >= MAX_LIVE {
                    continue;
                }
                let Some(visual) = spawn_live(world, request, clock) else {
                    // Keep the old request as well as its group: publishing
                    // the new kind here would falsely mark stale art ready
                    // and prevent another replacement attempt next frame.
                    continue;
                };
                if request.kind == LiveKind::Projectile {
                    runtime
                        .projectile_art
                        .insert(request.key.owner(), AbilityId(request.ability));
                }
                if let Some(old) = runtime.live.insert(request.key, visual) {
                    old.group.dispose(world);
                }
            }
            if let Some(visual) = runtime.live.get_mut(&request.key) {
                visual.request = request;
                let parent = world
                    .get::<Transform>(request.key.owner())
                    .copied()
                    .unwrap_or_default();
                let mut transform = Transform::from_scale(parent.scale.recip());
                if request.kind == LiveKind::Shield {
                    // W_buf outerring's source offset is (0,80,0). Position
                    // the center on world Y before billboarding the plane;
                    // rotating a child offset would move it sideways/down.
                    transform.translation = parent.rotation.inverse() * Vec3::Y * 0.8;
                    transform.rotation = parent.rotation.inverse() * camera;
                }
                if let LiveKind::Reveal(destination) = request.kind {
                    let phase = (clock - visual.born) as f32;
                    transform.rotation = Quat::from_rotation_y(phase * 1.4);
                    if destination {
                        transform.scale *= 0.8 + 0.2 * (phase * 4.0).sin().abs();
                    }
                }
                if request.kind == LiveKind::Area {
                    transform.scale *= 0.85 + request.progress * 0.15;
                }
                world.entity_mut(visual.group.root()).insert(transform);
            }
        }
        let mut retained = VecDeque::new();
        while let Some(mut burst) = runtime.bursts.pop_front() {
            let progress = ((runtime.clock - burst.born) / burst.seconds).clamp(0.0, 1.0) as f32;
            if progress >= 1.0 || !world.entities().contains(burst.group.root()) {
                burst.group.dispose(world);
                continue;
            }
            if let Some(life) = burst.attachment {
                if !world
                    .get::<UnitState>(life.entity())
                    .is_some_and(|state| state.alive() && state.life_ref(life.entity()) == life)
                {
                    burst.group.dispose(world);
                    continue;
                }
                if let Some(transform) = world.get::<Transform>(life.entity()) {
                    burst.base.translation.x = transform.translation.x;
                    burst.base.translation.z = transform.translation.z;
                }
            }
            let point = Vec2::new(burst.base.translation.x, burst.base.translation.z);
            let shown = observable(world, burst.team, point, burst.contact);
            let mut transform = burst.base;
            match burst.shape {
                Shape::Nova { .. } | Shape::CrystalImpact { .. } => {
                    transform.scale *= 0.15 + 0.85 * progress
                }
                Shape::StandardDrop => transform.translation.y += 7.0 * (1.0 - progress).powi(2),
                Shape::Spark { .. } => {
                    transform.rotation = camera;
                    transform.scale *= 0.8 + progress * 0.8;
                }
                Shape::WallBreak { .. } => transform.scale.y *= 1.0 - progress,
                _ => {}
            }
            world.entity_mut(burst.group.root()).insert((
                transform,
                if shown {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ));
            if !matches!(burst.shape, Shape::StandardDrop) {
                burst.group.fade(world, (1.0 - progress).min(0.9));
            }
            retained.push_back(burst);
        }
        runtime.bursts = retained;
        publish(world, &runtime);
    });
}

fn publish(world: &mut World, runtime: &Runtime) {
    let mut view = SkillEffectsView {
        ready: asset_ready(world),
        pending: runtime.pending.len(),
        emitted: runtime.emitted.clone(),
        ..default()
    };
    if let Some(assets) = world.get_resource::<SkillAssets>() {
        view.errors.clone_from(&assets.status().errors);
    }
    for visual in runtime.live.values() {
        if world.get::<Visibility>(visual.request.key.owner()) == Some(&Visibility::Hidden) {
            continue;
        }
        let phase = match visual.request.kind {
            LiveKind::Projectile => "projectile",
            LiveKind::Area => "area",
            LiveKind::Standard => "standard",
            LiveKind::Terrain => "terrain",
            LiveKind::Scout => "scout",
            LiveKind::Reveal(_) => "reveal",
            LiveKind::Shield => "shield",
            LiveKind::Focus(true) => "empowered",
            LiveKind::Focus(false) => "ready",
        };
        view.active.push(ActiveEffectView {
            ability_id: visual.request.ability,
            phase,
            layers: visual.group.layers(),
        });
    }
    for burst in &runtime.bursts {
        if world.get::<Visibility>(burst.group.root()) != Some(&Visibility::Hidden) {
            view.active.push(ActiveEffectView {
                ability_id: burst.ability,
                phase: "burst",
                layers: burst.group.layers(),
            });
        }
    }
    world.insert_resource(view);
}
