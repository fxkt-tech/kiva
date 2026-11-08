use super::catalog::{PROFILES, Profile, profile};
use crate::{
    client::{
        character::RigPose,
        picking::PickGeometry,
        presentation::PresentationFrame,
        scene::{UnitGroundBase, UnitVisual},
    },
    gameplay::UnitRole,
    simulation::*,
};
use bevy::{
    app::AnimationSystems,
    asset::LoadState,
    ecs::system::SystemParam,
    gltf::{Gltf, GltfLoaderSettings, GltfSkinnedMeshBoundsPolicy},
    prelude::*,
};
use serde::Serialize;
use std::collections::HashMap;

/// Only included by HUD inside its already observed unit projection.
#[derive(Component, Clone, Serialize)]
pub(in crate::client) struct ImportedModelView {
    pub name: &'static str,
    pub skin: &'static str,
    pub ready: bool,
    pub animation: &'static str,
    pub error: Option<String>,
}

pub(in crate::client) struct ImportedModelsPlugin;
impl Plugin for ImportedModelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load)
            .add_systems(Startup, crate::client::world_scene::load_texture)
            .add_systems(Update, crate::client::world_scene::apply_texture)
            .add_systems(
                Update,
                (prepare, attach, complete, record_events, sample)
                    .chain()
                    .after(crate::client::scene::attach_unit_visuals)
                    .after(crate::client::character::sync_poses),
            )
            .add_systems(
                PostUpdate,
                keep_committed_motion_root
                    .after(AnimationSystems)
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

#[derive(Resource)]
struct ModelCatalog(Vec<ModelAsset>);
struct ModelAsset {
    gltf: Handle<Gltf>,
    graph: Option<Handle<AnimationGraph>>,
    clips: HashMap<&'static str, Clip>,
    error: Option<String>,
}
#[derive(Clone, Copy)]
struct Clip {
    node: AnimationNodeIndex,
    duration: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Idle,
    Run,
    Windup,
    Backswing,
    CastWindup(AbilityId),
    Cast(AbilityId),
    Dead,
}

#[derive(Component)]
struct Instance {
    root: Entity,
    profile: usize,
    players: Vec<Entity>,
    life: UnitLifeRef,
    phase: Phase,
    phase_total: f32,
    last_remaining: f32,
    clock: f32,
    last_time: f64,
    last_cast: Option<(AbilityId, f64)>,
    bind_pose: Vec<(Entity, Transform)>,
    root_joint: Option<(Entity, f32)>,
}

fn load(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ModelCatalog(
        PROFILES
            .iter()
            .map(|profile| ModelAsset {
                gltf: server
                    .load_builder()
                    .with_settings(|settings: &mut GltfLoaderSettings| {
                        settings.skinned_mesh_bounds_policy =
                            Some(GltfSkinnedMeshBoundsPolicy::Dynamic);
                    })
                    .load(profile.path),
                graph: None,
                clips: HashMap::new(),
                error: None,
            })
            .collect(),
    ));
}

fn prepare(
    server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut catalog: ResMut<ModelCatalog>,
) {
    for (index, asset) in catalog.0.iter_mut().enumerate() {
        if asset.graph.is_some() || asset.error.is_some() {
            continue;
        }
        if let Some(LoadState::Failed(error)) = server.get_load_state(asset.gltf.id()) {
            let error = format!("{}: {error}", PROFILES[index].path);
            warn!("Imported model unavailable; keeping procedural model: {error}");
            asset.error = Some(error);
            continue;
        }
        let Some(gltf) = gltfs.get(&asset.gltf) else {
            continue;
        };
        if !gltf.animations.iter().all(|clip| clips.get(clip).is_some()) {
            continue;
        }
        let (graph, indices) = AnimationGraph::from_clips(gltf.animations.iter().cloned());
        let mut bindings = HashMap::new();
        for name in PROFILES[index].clips() {
            let Some(handle) = gltf.named_animations.get(name) else {
                asset.error = Some(format!("{} has no clip {name}", PROFILES[index].path));
                break;
            };
            let Some(position) = gltf.animations.iter().position(|clip| clip == handle) else {
                continue;
            };
            bindings.insert(
                name,
                Clip {
                    node: indices[position],
                    duration: clips.get(handle).unwrap().duration(),
                },
            );
        }
        if asset.error.is_none() {
            asset.clips = bindings;
            asset.graph = Some(graphs.add(graph));
        } else {
            warn!(
                "Imported model invalid; keeping procedural model: {:?}",
                asset.error
            );
        }
    }
}

type Unattached<'a> = (
    Entity,
    &'a Unit,
    &'a UnitState,
    Option<&'a UnitRole>,
    Option<&'a CharacterIdentity>,
);

fn attach(
    mut commands: Commands,
    server: Res<AssetServer>,
    tick: Res<SimulationTick>,
    units: Query<Unattached<'_>, (With<UnitVisual>, Without<ImportedModelView>)>,
) {
    for (id, unit, state, role, identity) in &units {
        let Some(index) = profile(
            role.copied(),
            identity.map(CharacterIdentity::id),
            unit.team(),
        ) else {
            continue;
        };
        let definition = &PROFILES[index];
        let root = commands
            .spawn((
                Name::new(format!("{} {} model", definition.skin, definition.name)),
                WorldAssetRoot(server.load(GltfAssetLabel::Scene(0).from_asset(definition.path))),
                // Authored glTF is +Z forward; UnitVisual uses Bevy's -Z forward.
                Transform::from_scale(Vec3::splat(definition.scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                Visibility::Hidden,
                ChildOf(id),
            ))
            .id();
        commands.entity(id).insert((
            ImportedModelView {
                name: definition.name,
                skin: definition.skin,
                ready: false,
                animation: definition.idle,
                error: None,
            },
            Instance {
                root,
                profile: index,
                players: Vec::new(),
                life: state.life_ref(id),
                phase: Phase::Idle,
                phase_total: 0.0,
                last_remaining: 0.0,
                clock: 0.0,
                last_time: tick.0 as f64 / TICK_RATE,
                last_cast: None,
                bind_pose: Vec::new(),
                root_joint: None,
            },
        ));
    }
}

#[derive(SystemParam)]
struct ModelHierarchy<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    ground: Query<'w, 's, (), With<UnitGroundBase>>,
    geometry: Query<'w, 's, (), With<PickGeometry>>,
    players: Query<'w, 's, (), With<AnimationPlayer>>,
    transforms: Query<'w, 's, (&'static Transform, Option<&'static Name>)>,
}

fn complete(
    mut commands: Commands,
    catalog: Res<ModelCatalog>,
    frame: Res<PresentationFrame>,
    hierarchy: ModelHierarchy,
    mut instances: Query<(Entity, &mut Instance, &mut ImportedModelView)>,
) {
    let ModelHierarchy {
        children,
        ground,
        geometry,
        players,
        transforms,
    } = hierarchy;
    for (owner, mut instance, mut view) in &mut instances {
        if view.ready || view.error.is_some() {
            continue;
        }
        let asset = &catalog.0[instance.profile];
        if let Some(error) = &asset.error {
            view.error = Some(error.clone());
            continue;
        }
        let Some(graph) = &asset.graph else {
            continue;
        };
        if !frame.should_sync() {
            continue;
        }
        let found: Vec<_> = children
            .iter_descendants(instance.root)
            .filter(|id| players.contains(*id))
            .collect();
        if found.is_empty() {
            continue;
        }
        for player in &found {
            commands
                .entity(*player)
                .insert(AnimationGraphHandle(graph.clone()));
        }
        instance.players = found;
        for child in children.iter_descendants(instance.root) {
            if let Ok((transform, name)) = transforms.get(child) {
                instance.bind_pose.push((child, *transform));
                if name.is_some_and(|name| name.as_str() == "Root") {
                    instance.root_joint = Some((child, transform.translation.y));
                }
            }
        }
        // Swap only once the complete authored scene and animation graph exist.
        // Ground markers belong to UnitVisual, not either model implementation.
        if let Ok(roots) = children.get(owner) {
            for child in roots.iter() {
                if child != instance.root && !ground.contains(child) && geometry.contains(child) {
                    commands.entity(child).despawn();
                }
            }
        }
        commands.entity(owner).remove::<RigPose>();
        commands
            .entity(instance.root)
            .insert((Visibility::Inherited, PickGeometry));
        view.ready = true;
    }
}

fn record_events(
    mut events: MessageReader<SimulationEvent>,
    tick: Res<SimulationTick>,
    mut instances: Query<&mut Instance>,
) {
    for event in events.read() {
        match event {
            SimulationEvent::AbilityCommitted { unit, ability, .. } => {
                if let Ok(mut instance) = instances.get_mut(*unit) {
                    instance.last_cast = Some((*ability, tick.0 as f64 / TICK_RATE));
                }
            }
            SimulationEvent::Died { unit }
            | SimulationEvent::Respawned { unit }
            | SimulationEvent::MoveOrdered { unit, .. } => {
                if let Ok(mut instance) = instances.get_mut(*unit) {
                    instance.last_cast = None;
                }
            }
            SimulationEvent::AttackReleased { source, .. } => {
                if let Ok(mut instance) = instances.get_mut(*source) {
                    // The entire attack can finish between display frames.
                    // Clear the older cast without replaying a skipped attack.
                    instance.last_cast = None;
                }
            }
            _ => {}
        }
    }
}

type Animated<'a> = (
    Entity,
    &'a UnitState,
    &'a UnitPose,
    &'a UnitAction,
    &'a UnitStatuses,
    &'a mut Instance,
    &'a mut ImportedModelView,
);

fn sample(
    frame: Res<PresentationFrame>,
    tick: Res<SimulationTick>,
    catalog: Res<ModelCatalog>,
    abilities: Res<AbilityCatalog>,
    mut units: Query<Animated<'_>>,
    mut players: Query<&mut AnimationPlayer>,
    mut transforms: Query<&mut Transform>,
) {
    if !frame.should_sync() {
        return;
    }
    let now = (tick.0 as f64 + f64::from(frame.fraction())) / TICK_RATE;
    for (id, state, pose, action, statuses, mut instance, mut view) in &mut units {
        if !view.ready {
            continue;
        }
        let new_life = instance.life != state.life_ref(id);
        if new_life {
            instance.life = state.life_ref(id);
            instance.last_cast = None;
            instance.phase = Phase::Idle;
            instance.clock = 0.0;
            instance.last_time = now;
        }
        let definition = &PROFILES[instance.profile];
        let asset = &catalog.0[instance.profile];
        let dt = (now - instance.last_time).max(0.0) as f32;
        instance.last_time = now;
        let motion = pose.position().distance(pose.interpolated(0.0));
        if matches!(
            action,
            UnitAction::AttackWindup { .. } | UnitAction::AttackBackswing { .. }
        ) {
            instance.last_cast = None;
        }
        let input = PhaseInput {
            state,
            action,
            statuses,
            now,
            motion,
            fraction: frame.fraction(),
            abilities: &abilities,
        };
        let (phase, remaining, total) = choose_phase(&instance, input);
        if new_life || phase != instance.phase || remaining > instance.last_remaining + 0.0001 {
            instance.phase = phase;
            instance.phase_total = total;
            instance.clock = 0.0;
            // Some authored clips omit tracks. Reset missing tracks at a phase
            // boundary so a prior death/cast posture cannot leak into idle.
            for (joint, bind) in &instance.bind_pose {
                if let Ok(mut transform) = transforms.get_mut(*joint) {
                    *transform = *bind;
                }
            }
        }
        instance.last_remaining = remaining;
        instance.clock += if phase == Phase::Run {
            dt * motion / TICK_SECONDS / 4.2
        } else {
            dt
        };
        let name = phase_clip(definition, phase);
        let Some(clip) = asset.clips.get(name) else {
            continue;
        };
        let time = sample_time(phase, clip.duration, &instance, state, now);
        for player_id in &instance.players {
            if let Ok(mut player) = players.get_mut(*player_id) {
                player.stop_all();
                player.play(clip.node).pause().set_seek_time(time);
            }
        }
        view.animation = name;
    }
}

fn keep_committed_motion_root(
    motion: Res<crate::client::motion_view::MotionView>,
    units: Query<(Entity, &Instance)>,
    mut joints: Query<&mut Transform>,
) {
    // Paused AnimationPlayers still evaluate their frozen curves each frame.
    // Reapply this correction after evaluation using the frozen MotionView;
    // otherwise pausing a leap would restore its authored second jump height.
    for (id, instance) in &units {
        if motion.height(id) > 0.0
            && let Some((joint, bind_height)) = instance.root_joint
            && let Ok(mut transform) = joints.get_mut(joint)
        {
            // Leap height was already applied to UnitVisual. The authored
            // pelvis trajectory cannot add a second world-space jump.
            transform.translation.y = bind_height;
        }
    }
}

struct PhaseInput<'a> {
    state: &'a UnitState,
    action: &'a UnitAction,
    statuses: &'a UnitStatuses,
    now: f64,
    motion: f32,
    fraction: f32,
    abilities: &'a AbilityCatalog,
}

fn choose_phase(instance: &Instance, input: PhaseInput<'_>) -> (Phase, f32, f32) {
    let PhaseInput {
        state,
        action,
        statuses,
        now,
        motion,
        fraction,
        abilities,
    } = input;
    if !state.alive() {
        return (Phase::Dead, 0.0, 0.0);
    }
    if statuses
        .iter()
        .any(|status| matches!(status.kind(), StatusKind::Stun | StatusKind::Airborne))
    {
        return (Phase::Idle, 0.0, 0.0);
    }
    match action {
        UnitAction::AttackWindup { remaining, .. } => (
            Phase::Windup,
            (*remaining as f32 - fraction).max(0.0),
            *remaining as f32,
        ),
        UnitAction::AttackBackswing {
            remaining,
            duration,
        } => (
            Phase::Backswing,
            (*remaining as f32 - fraction).max(0.0),
            *duration as f32,
        ),
        UnitAction::Casting {
            ability, remaining, ..
        } => (
            Phase::CastWindup(*ability),
            (*remaining - fraction * TICK_SECONDS).max(0.0),
            abilities
                .get(*ability)
                .map_or(*remaining, |definition| definition.windup),
        ),
        _ => {
            if let Some((ability, start)) = instance.last_cast
                && now - start < 0.45
                && PROFILES[instance.profile].skill(ability).is_some()
            {
                return (
                    Phase::Cast(ability),
                    (0.45 - (now - start) as f32).max(0.0),
                    0.45,
                );
            }
            if motion > 0.00001 {
                (Phase::Run, 0.0, 0.0)
            } else {
                (Phase::Idle, 0.0, 0.0)
            }
        }
    }
}

fn phase_clip(profile: &Profile, phase: Phase) -> &'static str {
    match phase {
        Phase::Idle => profile.idle,
        Phase::Run => profile.run,
        Phase::Windup | Phase::Backswing => profile.attack,
        Phase::CastWindup(ability) | Phase::Cast(ability) => {
            profile.skill(ability).unwrap_or(profile.idle)
        }
        Phase::Dead => profile.death,
    }
}

fn sample_time(
    phase: Phase,
    duration: f32,
    instance: &Instance,
    state: &UnitState,
    now: f64,
) -> f32 {
    let progress = if instance.phase_total > 0.0 {
        (1.0 - instance.last_remaining / instance.phase_total).clamp(0.0, 1.0)
    } else {
        1.0
    };
    match phase {
        Phase::Windup => duration * progress * 0.45,
        Phase::Backswing => duration * (0.45 + progress * 0.55),
        Phase::CastWindup(_) => duration * progress * 0.45,
        Phase::Cast(_) => duration * (0.45 + progress * 0.55),
        Phase::Dead => {
            let death = state.death_tick().unwrap_or(0) as f64 / TICK_RATE;
            duration * (((now - death).max(0.0) / 0.65) as f32).clamp(0.0, 1.0)
        }
        _ => instance.clock.rem_euclid(duration.max(0.001)),
    }
}

#[cfg(test)]
mod tests;
