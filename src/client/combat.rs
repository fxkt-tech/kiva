//! Read-only combat presentation. Meshes and materials are shared, effects are transient.
use avian3d::prelude::{Sensor, SpatialQuery};
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{ActorView, PlayerView};
use crate::{
    gameplay::Player,
    simulation::{
        Body, Boss, BossState, CombatEffect, CombatFeedback, Combatant, DefenseState, Enemy,
        Facing, Health, HitStop, MotionState, SimulationActive, SurfaceHit, WeaponMode, WeaponPair,
        surface_below,
    },
};

#[derive(Resource, Clone)]
pub(super) struct VisualAssets {
    cube: Handle<Mesh>,
    ring: Handle<Mesh>,
    sphere: Handle<Mesh>,
    enemy: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    teal: Handle<StandardMaterial>,
    hit: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(super) struct WeaponVisual(WeaponPair, WeaponMode);

impl WeaponVisual {
    #[cfg(test)]
    pub(super) fn pair(&self) -> WeaponPair {
        self.0
    }
    #[cfg(test)]
    pub(super) fn is_melee(&self) -> bool {
        self.1 == WeaponMode::Melee
    }
}
#[derive(Component)]
pub(super) struct EnemyBody(Entity);
#[derive(Component)]
pub(super) struct EnemyView;
#[derive(Component)]
pub(super) struct EnemyHealthBar(Entity);
#[derive(Component)]
pub(super) enum Marker {
    Target,
    Guard,
}
#[derive(Component)]
pub(super) struct Effect {
    remaining: f32,
    duration: f32,
    expansion: f32,
    scale: Vec3,
}
#[derive(Resource, Default)]
pub(super) struct EffectCursor(u64);

type SurfaceExclusions = Or<(With<Body>, With<Sensor>)>;

#[derive(SystemParam)]
pub(super) struct SurfaceQueries<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    excluded: Query<'w, 's, Entity, SurfaceExclusions>,
}

impl SurfaceQueries<'_, '_> {
    fn effect_transform(&self, position: Vec3) -> Transform {
        surface_below(&self.spatial, position, 32.0, self.excluded.iter()).map_or_else(
            || Transform::from_translation(position),
            |surface| {
                Transform::from_translation(surface.position + surface.normal * 0.06).with_rotation(
                    Quat::from_rotation_arc(Vec3::Y, surface.normal.normalize_or(Vec3::Y)),
                )
            },
        )
    }
}

pub(super) fn setup(world: &mut World) {
    let (cube, ring, sphere) = {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        (
            meshes.add(Cuboid::default()),
            meshes.add(Torus::new(0.94, 1.0).mesh().minor_resolution(6)),
            meshes.add(Sphere::new(0.5).mesh().uv(12, 8)),
        )
    };
    let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
    let mut material = |color, unlit| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit,
            perceptual_roughness: 0.65,
            ..default()
        })
    };
    let assets = VisualAssets {
        cube,
        ring,
        sphere,
        enemy: material(Color::srgb(0.72, 0.30, 0.22), false),
        gold: material(Color::srgb(1.0, 0.70, 0.23), true),
        teal: material(Color::srgb(0.35, 0.89, 0.83), true),
        hit: material(Color::srgb(1.0, 0.93, 0.75), true),
    };
    let enemies: Vec<_> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    for enemy in enemies {
        spawn_enemy_view(world, enemy, &assets);
    }

    let player = world
        .query_filtered::<Entity, With<PlayerView>>()
        .single(world)
        .expect("one player");
    for set in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
        let root = world
            .spawn((
                WeaponVisual(set, WeaponMode::Melee),
                Transform::IDENTITY,
                Visibility::Hidden,
            ))
            .id();
        super::weapons::spawn(world, root, set, WeaponMode::Melee);
        world.entity_mut(player).add_child(root);
        super::stickman::attach_weapon(world, player, root, WeaponMode::Melee);
    }
    for (marker, color, size) in [
        (Marker::Target, &assets.gold, 0.65),
        (Marker::Guard, &assets.teal, 0.7),
    ] {
        let entity = mesh(
            world,
            &assets.ring,
            color,
            Transform::from_scale(Vec3::splat(size)),
        );
        world
            .entity_mut(entity)
            .insert((marker, Visibility::Hidden));
    }
    world.insert_resource(assets);
    world.init_resource::<EffectCursor>();
}

fn spawn_enemy_view(world: &mut World, enemy: Entity, assets: &VisualAssets) {
    let body = world.get::<Body>(enemy).unwrap();
    let position = body.world_position();
    let scale = body.scale();
    let view = world
        .spawn((
            ActorView { owner: enemy },
            EnemyView,
            Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            Visibility::default(),
        ))
        .id();
    for part in super::stickman::spawn(world, view, &assets.enemy, &assets.gold) {
        world.entity_mut(part).insert(EnemyBody(enemy));
    }
    let blade = world
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    super::avatar::spawn_blade(world, blade);
    world.entity_mut(view).add_child(blade);
    super::stickman::attach_weapon(world, view, blade, WeaponMode::Melee);
    let bar = mesh(world, &assets.cube, &assets.teal, Transform::default());
    world.entity_mut(bar).insert(EnemyHealthBar(enemy));
}

/// Roster replacement also removes the old skeleton trees and standalone health bars.
pub(super) fn reconcile_enemies(world: &mut World) {
    let enemies: Vec<_> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    let views: Vec<_> = world
        .query_filtered::<(Entity, &ActorView), With<EnemyView>>()
        .iter(world)
        .map(|(entity, view)| (entity, view.owner))
        .collect();
    let bars: Vec<_> = world
        .query::<(Entity, &EnemyHealthBar)>()
        .iter(world)
        .map(|(entity, bar)| (entity, bar.0))
        .collect();
    for &(entity, owner) in views.iter().chain(&bars) {
        if !enemies.contains(&owner) {
            world.despawn(entity);
        }
    }
    let assets = world.resource::<VisualAssets>().clone();
    for enemy in enemies {
        if !views.iter().any(|&(_, owner)| owner == enemy) {
            spawn_enemy_view(world, enemy, &assets);
        }
    }
}

pub(super) fn sync_enemies(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    hit_stop: Res<HitStop>,
    enemies: Query<(&Body, &Facing, &Health, &Enemy, Option<&Boss>)>,
    mut views: Query<(&ActorView, &mut Transform, &mut Visibility), With<EnemyView>>,
    mut parts: Query<(
        &EnemyBody,
        &super::avatar::AvatarSurface,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    for (view, mut transform, mut visibility) in &mut views {
        let Ok((body, facing, _, _, _)) = enemies.get(view.owner) else {
            continue;
        };
        transform.translation = body.interpolated_world(alpha);
        transform.scale = Vec3::splat(body.scale());
        let direction = facing.direction();
        transform.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.y));
        // Defeated actors remain visible so their imported death pose can play.
        // Health bars and selection still use authoritative health separately.
        *visibility = Visibility::Visible;
    }
    for (part, surface, mut material) in &mut parts {
        if let Ok((_, _, health, enemy, boss)) = enemies.get(part.0) {
            let telegraph = boss.map_or_else(
                || enemy.is_winding_up(),
                |boss| boss.state() == BossState::Telegraph,
            );
            material.0 = surface.material_for(health.hit_remaining() > 0.0, telegraph);
        }
    }
}

pub(super) fn sync_health_bars(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    hit_stop: Res<HitStop>,
    view: Res<super::controls::ViewYaw>,
    enemies: Query<(&Body, &Health), With<Enemy>>,
    mut bars: Query<(&EnemyHealthBar, &mut Transform, &mut Visibility)>,
) {
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    for (bar, mut transform, mut visibility) in &mut bars {
        let Ok((body, health)) = enemies.get(bar.0) else {
            continue;
        };
        transform.translation = body.interpolated_world(alpha) + Vec3::Y * (2.0 * body.scale());
        transform.scale = Vec3::new(0.7 * health.current() / health.maximum(), 0.035, 0.06);
        transform.rotation = Quat::from_rotation_y(view.yaw());
        *visibility = if health.is_alive() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_weapons(
    player: Single<&Combatant, With<Player>>,
    mut weapons: Query<(&WeaponVisual, &mut Visibility)>,
) {
    for (weapon, mut visibility) in &mut weapons {
        *visibility = if weapon.0 == player.weapon_pair()
            && weapon.1 == WeaponMode::Melee
            && !player.axe_away()
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_markers(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    hit_stop: Res<HitStop>,
    surfaces: SurfaceQueries,
    player: Single<(&Body, &MotionState, &Combatant, &Health, &DefenseState), With<Player>>,
    targets: Query<(&Body, &MotionState, &Health), With<Enemy>>,
    mut markers: Query<(&Marker, &mut Transform, &mut Visibility)>,
) {
    let (body, motion, combat, health, defense) = *player;
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    let excluded: Vec<_> = surfaces.excluded.iter().collect();
    for (marker, mut transform, mut visibility) in &mut markers {
        let point = match marker {
            Marker::Target => combat
                .target()
                .and_then(|entity| targets.get(entity).ok())
                .filter(|(_, _, health)| health.is_alive())
                .and_then(|(body, motion, _)| {
                    marker_surface(&surfaces.spatial, &excluded, body, motion, alpha)
                }),
            Marker::Guard if defense.is_active() => {
                marker_surface(&surfaces.spatial, &excluded, body, motion, alpha)
            }
            _ => None,
        };
        if let Some(surface) = point.filter(|_| health.is_alive()) {
            transform.translation = surface.position + surface.normal * 0.04;
            transform.rotation =
                Quat::from_rotation_arc(Vec3::Y, surface.normal.normalize_or(Vec3::Y));
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn marker_surface(
    spatial: &SpatialQuery,
    excluded: &[Entity],
    body: &Body,
    motion: &MotionState,
    alpha: f32,
) -> Option<SurfaceHit> {
    if let Some(mut surface) = motion.surface() {
        surface.position += body.interpolated_world(alpha) - body.world_position();
        Some(surface)
    } else {
        surface_below(
            spatial,
            body.interpolated_world(alpha) + Vec3::Y * 0.05,
            32.0,
            excluded.iter().copied(),
        )
    }
}

pub(super) fn effects(
    mut commands: Commands,
    (time, active): (Res<Time>, Res<SimulationActive>),
    feedback: (Res<CombatFeedback>, Res<VisualAssets>),
    mut cursor: ResMut<EffectCursor>,
    surfaces: SurfaceQueries,
    mut effects: Query<(Entity, &mut Effect, &mut Transform)>,
) {
    let (feedback, assets) = feedback;
    let latest = feedback
        .events()
        .last()
        .map_or(cursor.0, |event| event.sequence);
    if !active.0 {
        // Freeze existing effects and discard unpresented feedback, as audio
        // does, so returning to the game cannot replay an old attack.
        cursor.0 = latest;
        return;
    }
    for (entity, mut effect, mut transform) in &mut effects {
        effect.remaining -= time.delta_secs();
        if effect.remaining <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            let progress = 1.0 - effect.remaining / effect.duration;
            transform.scale = effect.scale * (1.0 + effect.expansion * progress);
        }
    }
    for event in feedback.events().filter(|event| event.sequence > cursor.0) {
        let (mesh, material, transform, duration, expansion) = match event.kind {
            CombatEffect::SniperShot
            | CombatEffect::SniperImpact
            | CombatEffect::AxeThrow
            | CombatEffect::MeleeStart => {
                continue;
            }
            CombatEffect::Shot => {
                let arrow = event.pair == Some(WeaponPair::ChainAxeBow);
                if arrow {
                    let direction = (event.end - event.position).normalize_or(Vec3::NEG_Z);
                    for (offset, scale) in [
                        (0.0, Vec3::new(0.12, 0.06, 0.16)),
                        (0.25, Vec3::new(0.12, 0.07, 0.08)),
                    ] {
                        let transform = Transform::from_translation(event.end - direction * offset)
                            .looking_at(event.end + direction, Vec3::Y)
                            .with_scale(scale);
                        commands.spawn((
                            Mesh3d(assets.cube.clone()),
                            MeshMaterial3d(assets.teal.clone()),
                            transform,
                            Effect {
                                remaining: 0.18,
                                duration: 0.18,
                                expansion: 0.,
                                scale,
                            },
                        ));
                    }
                }
                let length = event.position.distance(event.end).max(0.05);
                let mut transform = Transform::from_translation((event.position + event.end) * 0.5)
                    .with_scale(Vec3::new(0.035, 0.035, length));
                if length > 0.05 {
                    transform = transform.looking_at(event.end, Vec3::Y);
                }
                (
                    &assets.cube,
                    if arrow { &assets.teal } else { &assets.gold },
                    transform,
                    0.16,
                    0.0,
                )
            }
            CombatEffect::Hammer | CombatEffect::ChainSweep | CombatEffect::Melee => (
                &assets.ring,
                &assets.gold,
                if matches!(event.kind, CombatEffect::ChainSweep) {
                    Transform::from_translation(event.position)
                } else {
                    surfaces.effect_transform(event.position)
                }
                .with_scale(Vec3::splat(event.radius.max(0.4))),
                0.35,
                0.25,
            ),
            CombatEffect::EnemyWindup => (
                &assets.ring,
                &assets.gold,
                surfaces
                    .effect_transform(event.position)
                    .with_scale(Vec3::splat(event.radius.max(0.6))),
                0.4,
                0.15,
            ),
            _ => (
                &assets.sphere,
                if matches!(
                    event.kind,
                    CombatEffect::Deflect | CombatEffect::Dodge | CombatEffect::Respawn
                ) {
                    &assets.teal
                } else {
                    &assets.hit
                },
                Transform::from_translation(event.position).with_scale(Vec3::splat(0.25)),
                0.22,
                1.5,
            ),
        };
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
            Effect {
                remaining: duration,
                duration,
                expansion,
                scale: transform.scale,
            },
        ));
    }
    cursor.0 = latest;
}

fn mesh(
    world: &mut World,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{
            animation, camera, controls::ViewYaw, scene, skeleton::AnimationLibrary, stickman,
        },
        simulation::{CombatIntent, SimulationPlugin, teleport_actor},
    };
    use std::time::Duration;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<ViewYaw>()
            .init_resource::<AnimationLibrary>()
            .add_systems(
                Startup,
                (crate::gameplay::setup, scene::setup, setup, camera::setup).chain(),
            )
            .add_systems(
                Update,
                (
                    scene::sync_player,
                    scene::sync_props,
                    reconcile_enemies,
                    sync_enemies,
                    sync_health_bars,
                    sync_weapons,
                    sync_markers,
                    effects,
                    animation::animate,
                    super::super::avatar::sync,
                    stickman::sync_attachments,
                    camera::follow,
                )
                    .chain(),
            );
        app.finish();
        app.update();
        app
    }

    fn physical_player(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap()
    }

    fn fixed_ticks(app: &mut App, count: usize) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        for _ in 0..count {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn surface_at_effect_origin(app: &mut App, origin: Vec3) -> SurfaceHit {
        let excluded: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, SurfaceExclusions>()
            .iter(app.world())
            .collect();
        let mut query = bevy::ecs::system::SystemState::<SpatialQuery>::new(app.world_mut());
        let spatial = query.get_mut(app.world_mut()).unwrap();
        surface_below(&spatial, origin, 32.0, excluded).unwrap()
    }

    #[test]
    fn default_dummy_remains_unselected_and_its_view_projects_ahead_of_the_player() {
        use bevy::camera::CameraProjection;
        use bevy::mesh::skinning::SkinnedMesh;

        let mut app = headless_app();
        let player = physical_player(&mut app);
        fixed_ticks(&mut app, 600);
        app.world_mut().run_schedule(Update);
        let world = app.world_mut();
        let dummy = world
            .query_filtered::<Entity, With<Enemy>>()
            .single(world)
            .unwrap();
        let feet = world.get::<Body>(dummy).unwrap().world_position();
        assert!(feet.distance(Vec3::new(0.0, 0.0, -4.0)) < 0.03);
        assert_eq!(world.get::<Combatant>(player).unwrap().target(), None);
        let (view_entity, _, root, visibility) = world
            .query_filtered::<(Entity, &ActorView, &Transform, &Visibility), With<EnemyView>>()
            .single(world)
            .unwrap();
        assert_eq!(*visibility, Visibility::Visible);
        assert!(root.translation.distance(feet) < 0.001);
        assert_eq!(root.scale, Vec3::splat(1.45));
        let center = root.translation + Vec3::Y * 1.2;
        let camera = world
            .query_filtered::<&Transform, With<camera::GameCamera>>()
            .single(world)
            .unwrap();
        let clip = PerspectiveProjection {
            fov: 50.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            ..default()
        }
        .get_clip_from_view()
            * camera.to_matrix().inverse()
            * center.extend(1.0);
        let ndc = clip.truncate() / clip.w;
        assert!(clip.w > 0.0 && ndc.x.abs() < 1.0 && ndc.y.abs() < 1.0);
        let skins: Vec<_> = world
            .get::<Children>(view_entity)
            .unwrap()
            .iter()
            .filter_map(|entity| world.get::<SkinnedMesh>(entity))
            .collect();
        assert_eq!(skins.len(), 15);
        for skin in skins {
            for &joint in &skin.joints {
                assert_eq!(world.get::<ChildOf>(joint).unwrap().parent(), view_entity);
                assert!(
                    world
                        .get::<Transform>(joint)
                        .unwrap()
                        .translation
                        .is_finite()
                );
            }
        }
    }

    #[test]
    fn switching_enemy_modes_recycles_views_skeletons_and_health_bars() {
        use crate::gameplay::{EnemyMode, set_enemy_mode};

        let mut app = headless_app();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        let baseline = app.world_mut().query::<Entity>().iter(app.world()).count();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        let material_count = app.world().resource::<Assets<StandardMaterial>>().len();
        for mode in [
            EnemyMode::Combat,
            EnemyMode::Target,
            EnemyMode::Combat,
            EnemyMode::Target,
        ] {
            let world = app.world_mut();
            set_enemy_mode(world, mode);
            app.update();
            let world = app.world_mut();
            let expected = 1;
            assert_eq!(
                world
                    .query_filtered::<Entity, With<EnemyView>>()
                    .iter(world)
                    .count(),
                expected
            );
            assert_eq!(
                world
                    .query_filtered::<Entity, With<EnemyHealthBar>>()
                    .iter(world)
                    .count(),
                expected
            );
            for (view_entity, view, transform) in world
                .query_filtered::<(Entity, &ActorView, &Transform), With<EnemyView>>()
                .iter(world)
            {
                assert!(world.get::<Enemy>(view.owner).is_some());
                let scale = world.get::<Body>(view.owner).unwrap().scale();
                assert_eq!(transform.scale, Vec3::splat(scale));
                assert_eq!(scale, if mode == EnemyMode::Target { 1.45 } else { 1.0 });
                assert!(
                    world
                        .get::<Children>(view_entity)
                        .unwrap()
                        .iter()
                        .any(|child| { world.get::<stickman::BoneVisual>(child).is_some() })
                );
            }
            for part in world.query::<&EnemyBody>().iter(world) {
                assert!(world.get::<Enemy>(part.0).is_some());
            }
            assert_eq!(world.resource::<Assets<Mesh>>().len(), mesh_count);
            assert_eq!(
                world.resource::<Assets<StandardMaterial>>().len(),
                material_count
            );
            if mode == EnemyMode::Target {
                assert_eq!(world.query::<Entity>().iter(world).count(), baseline);
            }
        }
    }

    #[test]
    fn view_and_camera_follow_actual_elevation_without_writing_the_physical_actor() {
        use avian3d::prelude::Position;
        let mut app = headless_app();
        let player = physical_player(&mut app);
        let view = app
            .world_mut()
            .query::<(Entity, &ActorView)>()
            .iter(app.world())
            .find(|(_, view)| view.owner == player)
            .unwrap()
            .0;
        assert!(app.world().get::<animation::RigPose>(player).is_none());
        assert!(app.world().get::<Body>(view).is_none());
        assert!(app.world().get::<animation::RigPose>(view).is_some());
        let feet = Vec3::new(16.5, 2.7, 8.5);
        teleport_actor(app.world_mut(), player, feet);
        let physical = app.world().get::<Position>(player).unwrap().0;
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut()
            .get_mut::<Transform>(view)
            .unwrap()
            .translation = Vec3::splat(999.0);
        app.world_mut().run_schedule(Update);
        assert_eq!(app.world().get::<Position>(player).unwrap().0, physical);
        assert!(
            app.world()
                .get::<Transform>(view)
                .unwrap()
                .translation
                .distance(feet)
                < 0.001
        );
        let camera = app
            .world_mut()
            .query_filtered::<&Transform, With<camera::GameCamera>>()
            .single(app.world())
            .unwrap();
        assert!((camera.translation.y - (feet.y + 0.9 + 6.0)).abs() < 0.001);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        fixed_ticks(&mut app, 1);
        assert!(
            app.world()
                .get::<Position>(player)
                .unwrap()
                .0
                .distance(physical)
                < 0.2
        );
    }

    #[test]
    fn target_marker_matches_the_resolved_hill_surface_and_normal() {
        let mut app = headless_app();
        let player = physical_player(&mut app);
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .single(app.world())
            .unwrap();
        teleport_actor(app.world_mut(), player, Vec3::new(11., 0., 8.5));
        teleport_actor(app.world_mut(), enemy, Vec3::new(14., 1.8, 8.5));
        app.world_mut()
            .get_mut::<Facing>(player)
            .unwrap()
            .set(Vec2::X);
        fixed_ticks(&mut app, 12);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        fixed_ticks(&mut app, 1);
        let origin = app.world().get::<Body>(enemy).unwrap().world_position() + Vec3::Y * 0.1;
        let surface = app
            .world()
            .get::<MotionState>(enemy)
            .unwrap()
            .surface()
            .unwrap_or_else(|| surface_at_effect_origin(&mut app, origin));
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut().run_schedule(Update);
        let (_, transform, visible) = app
            .world_mut()
            .query::<(&Marker, &Transform, &Visibility)>()
            .iter(app.world())
            .find(|(m, _, _)| matches!(m, Marker::Target))
            .unwrap();
        assert_eq!(*visible, Visibility::Visible);
        assert!(transform.translation.y > 0.5);
        assert!(
            transform
                .translation
                .distance(surface.position + surface.normal * 0.04)
                < 0.03
        );
        assert!((transform.rotation * Vec3::Y).dot(surface.normal) > 0.99);
    }

    #[test]
    fn attacks_and_enemy_windups_project_effects_onto_the_actual_hill_surface() {
        let mut app = headless_app();
        crate::gameplay::set_enemy_mode(app.world_mut(), crate::gameplay::EnemyMode::Combat);
        let player = physical_player(&mut app);
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        teleport_actor(app.world_mut(), player, Vec3::new(14.0, 1.75, 8.5));
        teleport_actor(app.world_mut(), enemy, Vec3::new(13.0, 1.05, 8.5));
        app.world_mut()
            .get_mut::<Facing>(player)
            .unwrap()
            .set(Vec2::NEG_X);
        for _ in 0..240 {
            fixed_ticks(&mut app, 1);
            if app.world().get::<Boss>(enemy).unwrap().state() == BossState::Telegraph {
                break;
            }
        }
        assert_eq!(
            app.world().get::<Boss>(enemy).unwrap().state(),
            BossState::Telegraph
        );
        let event_origin = app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .find(|event| event.actor == enemy && event.kind == CombatEffect::EnemyWindup)
            .unwrap()
            .position;
        let support = surface_at_effect_origin(&mut app, event_origin);
        assert!(support.position.y > 0.5);
        app.world_mut().run_schedule(Update);
        let windup = app
            .world_mut()
            .query::<(&Effect, &Transform)>()
            .iter(app.world())
            .find(|(effect, _)| effect.duration == 0.4)
            .unwrap()
            .1;
        assert!(windup.translation.y > 0.5);
        assert!(
            windup
                .translation
                .distance(support.position + support.normal * 0.06)
                < 0.001
        );
        assert!((windup.rotation * Vec3::Y).dot(support.normal) > 0.999);

        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        fixed_ticks(&mut app, 12);
        let event_origin = app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .find(|event| event.actor == player && event.kind == CombatEffect::Melee)
            .unwrap()
            .position;
        let support = surface_at_effect_origin(&mut app, event_origin);
        assert!(support.position.y > 1.0);
        app.world_mut().run_schedule(Update);
        let attack = app
            .world_mut()
            .query::<(&Effect, &Transform)>()
            .iter(app.world())
            .find(|(effect, _)| effect.duration == 0.35)
            .unwrap()
            .1;
        assert!(attack.translation.y > 1.0);
        assert!(
            attack
                .translation
                .distance(support.position + support.normal * 0.06)
                < 0.001
        );
        assert!((attack.rotation * Vec3::Y).dot(support.normal) > 0.999);
    }

    #[test]
    fn prop_snapshots_move_display_roots_without_changing_local_children() {
        use crate::{
            client::PropView,
            simulation::{PhysicsPose, spawn_box},
        };
        use avian3d::prelude::{Position, Rotation};
        let mut app = headless_app();
        // The bamboo arena has no decorative crates. Keep this snapshot/child
        // regression independent of whichever props the current level includes.
        let prop = spawn_box(app.world_mut(), Vec3::new(5.0, 0.5, 0.0), Vec3::ONE, true);
        let view = app
            .world_mut()
            .spawn((PropView { owner: prop }, Transform::IDENTITY))
            .id();
        for position in [Vec3::new(0.1, 0.3, 0.0), Vec3::new(-0.4, 0.0, 0.2)] {
            let child = app
                .world_mut()
                .spawn(Transform::from_translation(position))
                .id();
            app.world_mut().entity_mut(view).add_child(child);
        }
        let children: Vec<_> = app.world().get::<Children>(view).unwrap().iter().collect();
        let local: Vec<_> = children
            .iter()
            .map(|&entity| *app.world().get::<Transform>(entity).unwrap())
            .collect();
        app.world_mut().get_mut::<Position>(prop).unwrap().0.y += 1.0;
        app.world_mut().get_mut::<Rotation>(prop).unwrap().0 = Quat::from_rotation_z(0.4);
        fixed_ticks(&mut app, 1);
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut().run_schedule(Update);
        let pose = app.world().get::<PhysicsPose>(prop).unwrap();
        let transform = app.world().get::<Transform>(view).unwrap();
        assert!(transform.translation.distance(pose.position()) < 0.001);
        assert!(transform.rotation.dot(pose.rotation()).abs() > 0.9999);
        for (&child, before) in children.iter().zip(local) {
            assert_eq!(app.world().get::<ChildOf>(child).unwrap().parent(), view);
            assert_eq!(*app.world().get::<Transform>(child).unwrap(), before);
        }
    }

    #[test]
    fn headless_scene_consumes_combat_feedback_and_reclaims_effects_without_new_assets() {
        let mut app = headless_app();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        let material_count = app.world().resource::<Assets<StandardMaterial>>().len();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        fixed_ticks(&mut app, 12);
        app.world_mut().run_schedule(Update);
        assert!(
            app.world_mut()
                .query_filtered::<Entity, With<Effect>>()
                .iter(app.world())
                .count()
                > 0
        );
        let visible_weapons = app
            .world_mut()
            .query::<(&WeaponVisual, &Visibility)>()
            .iter(app.world())
            .filter(|(_, visibility)| **visibility == Visibility::Visible)
            .count();
        assert_eq!(visible_weapons, 1);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<EnemyHealthBar>>()
                .iter(app.world())
                .count(),
            1
        );

        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        let frozen: Vec<_> = app
            .world_mut()
            .query::<(Entity, &Effect, &Transform)>()
            .iter(app.world())
            .map(|(entity, effect, transform)| (entity, effect.remaining, *transform))
            .collect();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut().run_schedule(Update);
        for (entity, remaining, transform) in frozen {
            assert_eq!(
                app.world().get::<Effect>(entity).unwrap().remaining,
                remaining
            );
            assert_eq!(*app.world().get::<Transform>(entity).unwrap(), transform);
        }
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        app.world_mut().run_schedule(Update);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Effect>>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(
            app.world().resource::<Assets<StandardMaterial>>().len(),
            material_count
        );
    }

    #[test]
    fn defeated_enemy_keeps_its_body_for_the_death_clip_and_paused_rigs_stay_frozen() {
        let mut app = headless_app();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().run_schedule(FixedUpdate);
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .single(app.world())
            .unwrap();
        teleport_actor(app.world_mut(), player, Vec3::new(0., 0., -2.));
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        fixed_ticks(&mut app, 1);
        let enemy_view = app
            .world_mut()
            .query::<(Entity, &ActorView)>()
            .iter(app.world())
            .find(|(_, view)| view.owner == enemy)
            .unwrap()
            .0;
        let head = app.world().resource::<AnimationLibrary>().bone("Head");
        let standing = app
            .world()
            .get::<animation::RigPose>(enemy_view)
            .unwrap()
            .model[head]
            .translation
            .y;
        app.world_mut().entity_mut(enemy).insert(Health::new(1.0));
        {
            let mut intent = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
            intent.request_attack();
        }
        fixed_ticks(&mut app, 12);
        assert!(!app.world().get::<Health>(enemy).unwrap().is_alive());
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.world_mut().run_schedule(Update);
        assert_eq!(
            *app.world().get::<Visibility>(enemy_view).unwrap(),
            Visibility::Visible
        );
        let fallen = app
            .world()
            .get::<animation::RigPose>(enemy_view)
            .unwrap()
            .model[head]
            .translation
            .y;
        assert!(
            fallen < standing - 0.3,
            "standing {standing}, fallen {fallen}"
        );
        let frozen: Vec<_> = app
            .world_mut()
            .query::<(Entity, &stickman::BoneVisual, &Transform)>()
            .iter(app.world())
            .map(|(entity, _, transform)| (entity, *transform))
            .collect();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.5));
        app.world_mut().run_schedule(Update);
        for (entity, before) in frozen {
            let after = app.world().get::<Transform>(entity).unwrap();
            assert!(after.translation.distance(before.translation) < 0.0001);
            assert!(after.rotation.dot(before.rotation).abs() > 0.9999);
        }
    }
}
