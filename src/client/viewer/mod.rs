//! A single-character inspection App using the game's actual skin and pose pipeline.
mod debug;
mod orbit;
mod session;
mod snapshot;

use super::{
    animation::{self, PreviewRig, RigPose},
    avatar::{self, Appearance, AvatarAssetCache, AvatarAssets},
    companion, loading, rendering,
    skeleton::AnimationLibrary,
    source_animation::SourceAnimationLibrary,
    stickman,
    weapon_models::{self, Mount, WeaponModel},
    web,
};
use bevy::{
    camera::{primitives::Aabb, visibility::VisibilitySystems},
    prelude::*,
    transform::TransformSystems,
};
use debug::Line;
use orbit::Orbit;
use session::{Mode, Session, Weapon};
use std::collections::BTreeMap;

#[derive(Component)]
struct InspectionCamera;
#[derive(Resource)]
struct Scene {
    root: Entity,
    hand: Entity,
    spine: Entity,
    camera: Entity,
    shown_appearance: Appearance,
    shown_weapon: Weapon,
    registered_revision: u64,
    preview: PreviewRig,
    evaluated: Option<(String, f32, Weapon, u64)>,
    parts: Vec<Part>,
    bounds: Option<(Vec3, Vec3)>,
    bounds_dirty: bool,
    skin_dirty: bool,
    bounds_evaluations: u64,
    catalog_sent: bool,
    last_snapshot: String,
}
struct Part {
    id: String,
    label: String,
    entities: Vec<Entity>,
    visible: bool,
}

/// `DefaultPlugins` and `StartupPlugin` are installed by the common entry.
pub(super) fn configure(app: &mut App) {
    app.init_resource::<AnimationLibrary>()
        .init_resource::<SourceAnimationLibrary>()
        .init_resource::<Orbit>()
        .insert_resource(ClearColor(Color::srgb(0.115, 0.135, 0.16)))
        .add_systems(
            Startup,
            (
                loading::begin,
                setup,
                rendering::setup_viewer,
                loading::scene_ready,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update,
                avatar::sync.run_if(skin_dirty),
                stickman::sync_attachments.run_if(skin_dirty),
                attachments.run_if(skin_dirty),
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            finish_view
                .after(TransformSystems::Propagate)
                .after(VisibilitySystems::CalculateBounds)
                .after(VisibilitySystems::VisibilityPropagate)
                .before(VisibilitySystems::UpdateFrusta),
        );
}

fn setup(world: &mut World) {
    let catalog = animation::preview_catalog(
        world.resource::<AnimationLibrary>(),
        world.resource::<SourceAnimationLibrary>(),
    );
    let session = Session::new(catalog);
    let preview = PreviewRig::new(world.resource::<AnimationLibrary>());
    let rig = RigPose::new(world.resource::<AnimationLibrary>());
    let root = world
        .spawn((
            Name::new("Inspection character"),
            Transform::IDENTITY,
            Visibility::Visible,
            rig,
        ))
        .id();
    avatar::spawn_with_appearance(world, root, session.appearance);
    let hand = world
        .spawn((
            Name::new("Inspection hand weapon"),
            Transform::IDENTITY,
            Visibility::Visible,
        ))
        .id();
    let spine = world
        .spawn((
            Name::new("Inspection spine accessories"),
            Transform::IDENTITY,
            Visibility::Visible,
        ))
        .id();
    world.entity_mut(root).add_children(&[hand, spine]);
    spawn_weapon(world, hand, spine, session.weapon);
    world.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.85, 0.9, 1.),
        brightness: 250.,
        ..default()
    });
    world.spawn((
        DirectionalLight {
            color: Color::srgb(1., 0.96, 0.91),
            illuminance: 7000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-3., 6., -4.).looking_at(Vec3::Y, Vec3::Y),
    ));
    world.spawn((
        DirectionalLight {
            color: Color::srgb(0.8, 0.88, 1.),
            illuminance: 2500.,
            ..default()
        },
        Transform::from_xyz(3., 3., 2.).looking_at(Vec3::Y, Vec3::Y),
    ));
    let floor = world
        .resource_mut::<Assets<Mesh>>()
        .add(Plane3d::default().mesh().size(8., 8.));
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.19),
            perceptual_roughness: 0.92,
            ..default()
        });
    world.spawn((
        Name::new("Inspection ground"),
        Mesh3d(floor),
        MeshMaterial3d(material),
        Transform::from_xyz(0., -0.012, 0.),
    ));
    let lighting = EnvironmentMapLight::hemispherical_gradient(
        &mut world.resource_mut::<Assets<Image>>(),
        Color::srgb(0.72, 0.77, 0.83),
        Color::srgb(0.4, 0.43, 0.46),
        Color::srgb(0.22, 0.23, 0.25),
    )
    .with_intensity(250.);
    let orbit = world.resource::<Orbit>();
    let camera = world
        .spawn((
            InspectionCamera,
            rendering::MainView,
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 50f32.to_radians(),
                near: 0.01,
                far: 200.,
                ..default()
            }),
            orbit.transform(),
            lighting,
        ))
        .id();
    let parts = parts(world, root, hand, spine, session.revision);
    loading::require_model(world, session.revision, body_meshes(world, root));
    world.insert_resource(Scene {
        root,
        hand,
        spine,
        camera,
        shown_appearance: session.appearance,
        shown_weapon: session.weapon,
        registered_revision: session.revision,
        preview,
        evaluated: None,
        parts,
        bounds: None,
        bounds_dirty: true,
        skin_dirty: true,
        bounds_evaluations: 0,
        catalog_sent: false,
        last_snapshot: String::new(),
    });
    world.insert_resource(session);
    let draw = debug::DebugDraw::new(world);
    world.insert_resource(draw);
}

fn body_meshes(world: &World, root: Entity) -> Vec<Entity> {
    world
        .get::<Children>(root)
        .into_iter()
        .flatten()
        .copied()
        .filter(|&e| world.get::<bevy::mesh::skinning::SkinnedMesh>(e).is_some())
        .collect()
}
fn descendants(world: &World, root: Entity, result: &mut Vec<Entity>) {
    if let Some(children) = world.get::<Children>(root) {
        for &child in children {
            result.push(child);
            descendants(world, child, result);
        }
    }
}
fn clear_children(world: &mut World, parent: Entity) {
    let children = world
        .get::<Children>(parent)
        .map(|v| v.to_vec())
        .unwrap_or_default();
    for child in children {
        world.despawn(child);
    }
}
fn spawn_weapon(world: &mut World, hand: Entity, spine: Entity, weapon: Weapon) {
    match weapon {
        Weapon::None => {}
        Weapon::Blade => avatar::spawn_blade(world, hand),
        _ => {
            weapon_models::spawn(world, hand, weapon.pair(), weapon.mode(), Mount::Weapon);
            weapon_models::spawn(world, spine, weapon.pair(), weapon.mode(), Mount::Spine);
        }
    }
}
fn parts(world: &World, root: Entity, hand: Entity, spine: Entity, revision: u64) -> Vec<Part> {
    let mut groups: BTreeMap<String, Vec<Entity>> = BTreeMap::new();
    for (parent, prefix) in [(root, "身体"), (hand, "手持"), (spine, "背部")] {
        let mut children = Vec::new();
        if parent == root {
            children.extend(
                world
                    .get::<Children>(root)
                    .into_iter()
                    .flatten()
                    .copied()
                    .filter(|&e| e != hand && e != spine),
            );
        } else {
            descendants(world, parent, &mut children);
        }
        for entity in children {
            if world.get::<Mesh3d>(entity).is_none() {
                continue;
            }
            let name = world.get::<Name>(entity).map_or("长刀", Name::as_str);
            groups
                .entry(format!("{prefix} · {name}"))
                .or_default()
                .push(entity);
        }
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (label, entities))| Part {
            id: format!("r{revision}_part{index}"),
            label,
            entities,
            visible: true,
        })
        .collect()
}
fn cached_assets(world: &World, appearance: Appearance) -> &AvatarAssets {
    AvatarAssets::cached(
        world.get_resource::<AvatarAssets>(),
        world.get_resource::<AvatarAssetCache>(),
        appearance,
    )
    .expect("selected inspection body is cached")
}

fn update(world: &mut World) {
    let Some(mut scene) = world.remove_resource::<Scene>() else {
        return;
    };
    let mut session = world.remove_resource::<Session>().unwrap();
    let mut orbit = world.remove_resource::<Orbit>().unwrap();
    let commands = web::viewer_take_commands();
    let height = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map_or(600., |w| w.height());
    let mut part_changes = Vec::new();
    // Bound the Rust side too; a malformed bridge cannot allocate an unbounded command list.
    if commands.len() <= 16_384 {
        for line in commands.lines().take(128) {
            let effect = session.apply(line);
            if let Some(v) = effect.orbit {
                orbit.rotate(v);
            }
            if let Some(v) = effect.pan {
                orbit.pan(v, height);
            }
            if let Some(v) = effect.zoom {
                orbit.zoom(v);
            }
            if let Some(v) = effect.view {
                orbit.view(&v);
            }
            if effect.fit {
                orbit.fit_requested = true;
            }
            if effect.reset {
                orbit = Orbit::default();
            }
            if let Some(value) = effect.part {
                part_changes.push(value);
            }
        }
    } else {
        session.status = "已忽略过大的检查命令批次".into();
    }
    session.advance(world.resource::<Time>().delta_secs());
    let mut rebuilt = false;
    if scene.shown_appearance != session.appearance {
        avatar::replace(world, scene.root, session.appearance);
        scene.shown_appearance = session.appearance;
        rebuilt = true;
    }
    if scene.shown_weapon != session.weapon {
        clear_children(world, scene.hand);
        clear_children(world, scene.spine);
        spawn_weapon(world, scene.hand, scene.spine, session.weapon);
        scene.shown_weapon = session.weapon;
        rebuilt = true;
    }
    if rebuilt {
        scene.bounds_dirty = true;
        scene.evaluated = None;
    }
    if scene.registered_revision != session.revision {
        // A new configuration must be renderable before it is acknowledged.
        // Rebuild instance controls and restore visibility even when a previous
        // inspection hid every body part; stale part IDs cannot target it.
        scene.parts = parts(world, scene.root, scene.hand, scene.spine, session.revision);
        for part in &scene.parts {
            for &entity in &part.entities {
                // Shown parts still inherit the mode-owned hand/spine roots.
                // `Visible` would override a hidden parent in Bevy.
                world.entity_mut(entity).insert(Visibility::Inherited);
            }
        }
        scene.bounds_dirty = true;
        loading::require_model(world, session.revision, body_meshes(world, scene.root));
        scene.registered_revision = session.revision;
    }
    apply_parts(world, &mut scene, &mut session, part_changes);
    let seconds = session.sample_seconds();
    let key = (
        session.action.clone(),
        seconds,
        session.weapon,
        session.revision,
    );
    if scene.evaluated.as_ref() != Some(&key) {
        let accepted = world.resource_scope(|world, library: Mut<AnimationLibrary>| {
            scene.preview.seek(
                &library,
                world.resource::<SourceAnimationLibrary>(),
                &session.action,
                session.weapon.pair(),
                session.weapon == Weapon::Blade,
                seconds,
                // Session owns looping; absolute endpoint seeks must keep the tail pose.
                false,
            )
        });
        if accepted {
            let mut pose = scene.preview.pose().clone();
            if session.mode == Mode::Game && session.action.contains("companion") {
                let frame = companion_frame(world, session.weapon, session.action.ends_with("aim"));
                pose.sample_companion_hold(
                    world.resource::<AnimationLibrary>(),
                    seconds,
                    frame.pitch,
                    frame.grips,
                );
            }
            world.entity_mut(scene.root).insert(pose);
            scene.evaluated = Some(key);
            scene.bounds_dirty = true;
            scene.skin_dirty = true;
        } else {
            session.status = "当前动作不可用".into();
        }
    }
    *world.get_mut::<Visibility>(scene.hand).unwrap() =
        if session.mode == Mode::Source || session.weapon == Weapon::None {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    *world.get_mut::<Visibility>(scene.spine).unwrap() = if session.weapon == Weapon::Bow {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    world.insert_resource(orbit);
    world.insert_resource(session);
    world.insert_resource(scene);
}

fn apply_parts(
    world: &mut World,
    scene: &mut Scene,
    session: &mut Session,
    changes: Vec<(String, bool)>,
) {
    for (id, visible) in changes {
        if let Some(part) = scene.parts.iter_mut().find(|part| part.id == id) {
            part.visible = visible;
            for &entity in &part.entities {
                world.entity_mut(entity).insert(if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
            scene.bounds_dirty = true;
        } else {
            session.status = "当前模型没有此部件".into();
        }
    }
}

fn companion_frame(world: &mut World, weapon: Weapon, aim: bool) -> companion::CompanionFrame {
    let grips = if weapon == Weapon::None || weapon == Weapon::Blade {
        [Vec3::ZERO; 2]
    } else {
        WeaponModel::shared(world, weapon.pair(), weapon.mode()).grips
    };
    // A recorded flat-ground display fixture: feet at origin, muzzle Y1.2 and
    // 0.55m forward, matching the game's CompanionState source convention.
    let source = Vec3::new(0., 1.2, -0.55);
    let direction = if aim {
        Vec3::new(0., 0.45, -1.).normalize()
    } else {
        Vec3::NEG_Z
    };
    companion::local_frame(Transform::IDENTITY, source, source + direction * 6., grips)
}
fn skin_dirty(scene: Option<Res<Scene>>) -> bool {
    scene.is_some_and(|scene| scene.skin_dirty)
}

fn attachments(world: &mut World) {
    let Some(mut scene) = world.remove_resource::<Scene>() else {
        return;
    };
    let session = world.resource::<Session>();
    let appearance = session.appearance;
    let weapon = session.weapon;
    let companion = session.mode == Mode::Game && session.action.contains("companion");
    let aim = session.action.ends_with("aim");
    let rig = world.get::<RigPose>(scene.root).unwrap();
    let assets = cached_assets(world, appearance);
    let pose = assets.attachment_pose(rig);
    let mount = assets.body_mount(rig);
    let hand = if companion {
        companion_frame(world, weapon, aim).weapon
    } else {
        let hand = world.resource::<AnimationLibrary>().bone("hand_r");
        stickman::attachment_transform(
            &stickman::Attachment::Weapon {
                bone: hand,
                mode: weapon.mode(),
            },
            &pose,
        )
    };
    *world.get_mut::<Transform>(scene.hand).unwrap() = hand;
    *world.get_mut::<Transform>(scene.spine).unwrap() = mount;
    scene.skin_dirty = false;
    world.insert_resource(scene);
}

/// Union the current dynamic Bevy skin AABBs and rigid attachment AABBs. The
/// stage floor and diagnostic helpers are outside the character hierarchy.
fn current_bounds(world: &World, root: Entity) -> Option<(Vec3, Vec3)> {
    let mut entities = Vec::new();
    descendants(world, root, &mut entities);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for entity in entities {
        if world.get::<Mesh3d>(entity).is_none()
            || world
                .get::<InheritedVisibility>(entity)
                .is_some_and(|v| !v.get())
        {
            continue;
        }
        let Some(bounds) = world.get::<Aabb>(entity) else {
            continue;
        };
        let Some(transform) = world.get::<GlobalTransform>(entity) else {
            continue;
        };
        let center = Vec3::from(bounds.center);
        let half = Vec3::from(bounds.half_extents);
        for x in [-1., 1.] {
            for y in [-1., 1.] {
                for z in [-1., 1.] {
                    let p = transform.transform_point(center + half * Vec3::new(x, y, z));
                    if p.is_finite() {
                        min = min.min(p);
                        max = max.max(p);
                    }
                }
            }
        }
    }
    if min.is_finite() && max.is_finite() {
        Some((min, max))
    } else {
        None
    }
}
fn finish_view(world: &mut World) {
    let Some(mut scene) = world.remove_resource::<Scene>() else {
        return;
    };
    let mut orbit = world.remove_resource::<Orbit>().unwrap();
    if (scene.bounds_dirty || orbit.fit_requested)
        && let Some(bounds) = current_bounds(world, scene.root)
    {
        scene.bounds = Some(bounds);
        scene.bounds_dirty = false;
        scene.bounds_evaluations += 1;
    }
    if orbit.fit_requested
        && let Some((min, max)) = scene.bounds
    {
        let aspect = world
            .query::<&Window>()
            .iter(world)
            .next()
            .map_or(1.6, |w| w.width() / w.height().max(1.));
        orbit.fit(min, max, aspect);
    }
    let camera = orbit.transform();
    world
        .entity_mut(scene.camera)
        .insert((camera, GlobalTransform::from(camera)));
    let session = world.resource::<Session>();
    let mut lines = debug::base(session.debug);
    let rig = world.get::<RigPose>(scene.root).unwrap();
    let assets = cached_assets(world, session.appearance);
    if session.debug.bones {
        let bones = assets.structural_bones(rig);
        for (parent, pose) in &bones {
            if let Some(parent) = parent {
                lines.push(Line {
                    from: bones[*parent].1.translation,
                    to: pose.translation,
                    color: 4,
                });
            }
        }
    }
    if session.debug.grips {
        let library = world.resource::<AnimationLibrary>();
        let pose = assets.attachment_pose(rig);
        for name in ["hand_r", "hand_l"] {
            let hand = pose[library.bone(name)];
            debug::axes(&mut lines, hand, 0.12);
            debug::cross(&mut lines, hand.translation, 5);
        }
        let hand = world.get::<Transform>(scene.hand).copied().unwrap();
        debug::axes(&mut lines, hand, 0.18);
        debug::cross(&mut lines, hand.translation, 6);
        let spine = world.get::<Transform>(scene.spine).copied().unwrap();
        debug::axes(&mut lines, spine, 0.12);
        if !matches!(session.weapon, Weapon::None | Weapon::Blade) {
            let weapon = session.weapon;
            let model = WeaponModel::shared(world, weapon.pair(), weapon.mode());
            for grip in model.grips {
                debug::cross(&mut lines, hand.transform_point(grip), 4);
            }
        }
    }
    let session = world.resource::<Session>();
    if session.debug.bounds
        && let Some((min, max)) = scene.bounds
    {
        debug::bounds(&mut lines, min, max);
    }
    let draw = world.remove_resource::<debug::DebugDraw>().unwrap();
    draw.submit(world, &lines);
    world.insert_resource(draw);
    let session = world.resource::<Session>();
    let ready = loading::model_ready(world, session.revision);
    let failed = loading::model_failed(world, session.revision);
    let rendered = snapshot::render(
        session,
        &scene,
        &orbit,
        world.get::<RigPose>(scene.root).unwrap().trace(),
        ready,
        failed,
        !scene.catalog_sent,
    );
    if rendered != scene.last_snapshot {
        web::viewer_render(&rendered);
        scene.last_snapshot = rendered;
        scene.catalog_sent = true;
    }
    world.insert_resource(orbit);
    world.insert_resource(scene);
}

#[cfg(test)]
mod tests;
