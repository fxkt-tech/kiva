//! One-shot sniper presentation. Its clock runs through hit stop, but not Esc.
use bevy::prelude::*;

use crate::simulation::{CombatEffect, CombatFeedback, SimulationActive};

const RECOIL_DURATION: f32 = 0.12;

#[derive(Resource, Default)]
pub(super) struct ImpactState {
    cursor: super::FeedbackCursor,
    shot: Option<(Entity, f32)>,
}

impl ImpactState {
    /// Local camera displacement and rotation; no persistent camera offset.
    pub(super) fn recoil(&self, actor: Entity) -> Option<(Vec3, Quat)> {
        let (shooter, age) = self.shot?;
        if shooter != actor || age >= RECOIL_DURATION {
            return None;
        }
        let progress = age / RECOIL_DURATION;
        let strength = (1.0 - progress).powi(2);
        let sway = (progress * std::f32::consts::TAU).sin() * strength;
        Some((
            Vec3::new(0.015 * sway, 0.025 * strength, 0.15 * strength),
            Quat::from_euler(
                EulerRot::YXZ,
                0.004 * sway,
                2.0_f32.to_radians() * strength,
                0.006 * sway,
            ),
        ))
    }
}

#[derive(Resource)]
pub(super) struct ImpactAssets {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    ring: Handle<Mesh>,
    white: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParticleKind {
    Muzzle,
    Tracer,
    Ring,
    Spark,
}

#[derive(Component)]
pub(super) struct ImpactParticle {
    kind: ParticleKind,
    age: f32,
    duration: f32,
    velocity: Vec3,
    growth: f32,
    scale: Vec3,
}

pub(super) fn setup(world: &mut World) {
    world.init_resource::<ImpactState>();
    let (cube, sphere, ring) = {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        (
            meshes.add(Cuboid::default()),
            meshes.add(Sphere::new(0.5).mesh().uv(12, 8)),
            meshes.add(Torus::new(0.92, 1.0).mesh().minor_resolution(6)),
        )
    };
    let (white, gold) = {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mut glow = |color: Color, emissive| {
            materials.add(StandardMaterial {
                base_color: color,
                emissive,
                // Bevy's unlit path returns base_color without emissive.
                // Keep PBR enabled so this reads as a flash, even in shadow.
                ..default()
            })
        };
        (
            glow(Color::srgb(1.0, 0.98, 0.87), LinearRgba::rgb(9.0, 8.0, 6.0)),
            glow(Color::srgb(1.0, 0.65, 0.12), LinearRgba::rgb(6.0, 3.0, 0.3)),
        )
    };
    world.insert_resource(ImpactAssets {
        cube,
        sphere,
        ring,
        white,
        gold,
    });
}

fn spawn(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
    kind: ParticleKind,
    duration: f32,
    motion: (Vec3, f32),
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        transform,
        ImpactParticle {
            kind,
            age: 0.0,
            duration,
            velocity: motion.0,
            growth: motion.1,
            scale: transform.scale,
        },
    ));
}

fn shot(commands: &mut Commands, assets: &ImpactAssets, muzzle: Vec3, end: Vec3) {
    let direction = (end - muzzle).normalize_or(Vec3::NEG_Z);
    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction);
    spawn(
        commands,
        &assets.sphere,
        &assets.white,
        Transform::from_translation(muzzle)
            .with_rotation(rotation)
            .with_scale(Vec3::new(0.25, 0.25, 0.55)),
        ParticleKind::Muzzle,
        0.065,
        (direction * 0.5, 0.5),
    );
    for angle in [0.0, std::f32::consts::FRAC_PI_2] {
        spawn(
            commands,
            &assets.cube,
            &assets.gold,
            Transform::from_translation(muzzle + direction * 0.12)
                .with_rotation(rotation * Quat::from_rotation_z(angle))
                .with_scale(Vec3::new(0.65, 0.035, 0.04)),
            ParticleKind::Muzzle,
            0.08,
            (direction * 0.5, 0.3),
        );
    }
    let length = muzzle.distance(end);
    for (width, material, duration) in [(0.075, &assets.gold, 0.075), (0.028, &assets.white, 0.10)]
    {
        spawn(
            commands,
            &assets.cube,
            material,
            Transform::from_translation((muzzle + end) * 0.5)
                .with_rotation(rotation)
                .with_scale(Vec3::new(width, width, length)),
            ParticleKind::Tracer,
            duration,
            (Vec3::ZERO, 0.0),
        );
    }
}

fn hit(commands: &mut Commands, assets: &ImpactAssets, position: Vec3, origin: Vec3) {
    let incoming = (position - origin).normalize_or(Vec3::NEG_Z);
    let plane = Quat::from_rotation_arc(Vec3::Y, incoming);
    spawn(
        commands,
        &assets.ring,
        &assets.gold,
        Transform::from_translation(position)
            .with_rotation(plane)
            .with_scale(Vec3::splat(0.18)),
        ParticleKind::Ring,
        0.20,
        (Vec3::ZERO, 5.0),
    );
    for index in 0..10 {
        let angle = index as f32 * std::f32::consts::TAU / 10.0;
        let radial = plane * Vec3::new(angle.cos(), 0.0, angle.sin());
        let direction = (radial + incoming * 0.65).normalize();
        spawn(
            commands,
            &assets.cube,
            if index % 2 == 0 {
                &assets.white
            } else {
                &assets.gold
            },
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction))
                .with_scale(Vec3::new(0.025, 0.025, 0.28)),
            ParticleKind::Spark,
            0.15 + (index % 3) as f32 * 0.025,
            (direction * (3.2 + (index % 3) as f32 * 0.7), 0.0),
        );
    }
}

pub(super) fn update(
    mut commands: Commands,
    timing: (Res<Time>, Res<SimulationActive>),
    feedback: (Res<CombatFeedback>, Res<ImpactAssets>),
    mut state: ResMut<ImpactState>,
    mut particles: Query<(Entity, &mut ImpactParticle, &mut Transform)>,
) {
    let (time, active) = timing;
    let (feedback, assets) = feedback;
    if active.0 {
        if let Some((_, age)) = &mut state.shot {
            *age += time.delta_secs();
            if *age >= RECOIL_DURATION {
                state.shot = None;
            }
        }
        for (entity, mut particle, mut transform) in &mut particles {
            particle.age += time.delta_secs();
            if particle.age >= particle.duration {
                commands.entity(entity).despawn();
                continue;
            }
            transform.translation += particle.velocity * time.delta_secs();
            let progress = particle.age / particle.duration;
            let size = if particle.kind == ParticleKind::Ring {
                1.0 + particle.growth * progress
            } else {
                (1.0 - progress) * (1.0 + particle.growth * progress)
            };
            transform.scale = if particle.kind == ParticleKind::Tracer {
                particle.scale * Vec3::new(size, size, 1.0)
            } else {
                particle.scale * size
            };
        }
    }
    let mut cursor = state.cursor;
    for event in cursor.take(&feedback, active.0) {
        match event.kind {
            CombatEffect::SniperShot => {
                state.shot = Some((event.source_owner, 0.0));
                shot(&mut commands, &assets, event.position, event.end);
            }
            CombatEffect::SniperImpact => hit(&mut commands, &assets, event.position, event.end),
            _ => {}
        }
    }
    state.cursor = cursor;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{animation, combat, scene, skeleton::AnimationLibrary, stickman},
        gameplay::{self, Player},
        simulation::{Body, CombatIntent, Combatant, Health, SimulationPlugin},
    };
    use std::time::Duration;

    fn arena() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<super::super::controls::ViewYaw>()
            .init_resource::<AnimationLibrary>()
            .add_systems(
                Startup,
                (
                    gameplay::setup,
                    gameplay::manual_targeting,
                    scene::setup,
                    combat::setup,
                    setup,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    scene::sync_player,
                    combat::sync_enemies,
                    combat::sync_weapons,
                    animation::animate,
                    stickman::sync_attachments,
                    update,
                )
                    .chain(),
            );
        app.finish();
        app.update();
        tick(&mut app, 12);
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        request(&mut app, player, CombatIntent::request_toggle_target);
        let target = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        (app, player, target)
    }

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn render(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
        app.world_mut().run_schedule(Update);
    }

    fn request(app: &mut App, player: Entity, command: impl FnOnce(&mut CombatIntent)) {
        command(&mut app.world_mut().get_mut::<CombatIntent>(player).unwrap());
        tick(app, 1);
    }

    fn prepare(app: &mut App, player: Entity) {
        request(app, player, CombatIntent::request_melee_skill);
        tick(app, 13);
        request(app, player, CombatIntent::request_combo);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .is_combo_active()
        );
    }

    fn discharge(app: &mut App, player: Entity) {
        for _ in 0..30 {
            if app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .sequence()
                .is_some_and(|sequence| sequence.has_fired())
            {
                render(app, 0.0);
                return;
            }
            tick(app, 1);
        }
        panic!("special sniper never fired");
    }

    fn particle_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ImpactParticle>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn real_discharge_uses_the_live_muzzle_and_airborne_hit_once_without_physics_writes() {
        let (mut app, player, target) = arena();
        prepare(&mut app, player);
        tick(&mut app, 14);
        render(&mut app, 0.0);
        assert_eq!(particle_count(&mut app), 0);
        assert!(
            app.world()
                .resource::<ImpactState>()
                .recoil(player)
                .is_none()
        );
        discharge(&mut app, player);
        let muzzle = app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .find(|e| e.kind == CombatEffect::SniperShot)
            .unwrap()
            .position;
        let performer = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .companion()
            .unwrap();
        assert!(
            app.world()
                .get::<crate::simulation::Companion>(performer)
                .is_some()
        );
        assert!(app.world().get::<Body>(performer).is_none());
        // The bullet is in flight; its impact is shown where and when it lands.
        let landed = |app: &App| {
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| event.kind == CombatEffect::SniperImpact)
        };
        assert!(!landed(&app));
        for _ in 0..4 {
            if landed(&app) {
                break;
            }
            tick(&mut app, 1);
        }
        render(&mut app, 0.0);
        let assets = app.world().resource::<ImpactAssets>();
        let sphere = assets.sphere.clone();
        let hit = *app
            .world()
            .resource::<CombatFeedback>()
            .events()
            .find(|event| event.kind == CombatEffect::SniperImpact)
            .unwrap();
        assert!(
            hit.position.y > 1.5,
            "impact must retain airborne elevation"
        );
        let mut saw_muzzle = false;
        let mut saw_ring = false;
        for (entity, particle, transform, mesh) in app
            .world_mut()
            .query::<(Entity, &ImpactParticle, &Transform, &Mesh3d)>()
            .iter(app.world())
        {
            assert!(app.world().get::<Body>(entity).is_none());
            assert!(
                app.world()
                    .get::<avian3d::prelude::Collider>(entity)
                    .is_none()
            );
            if particle.kind == ParticleKind::Muzzle && mesh.0 == sphere {
                assert!(transform.translation.distance(muzzle) < 0.0001);
                saw_muzzle = true;
            }
            if particle.kind == ParticleKind::Ring {
                assert_eq!(transform.translation, hit.position);
                let incoming = (hit.position - hit.end).normalize();
                assert!((transform.rotation * Vec3::Y).dot(incoming) > 0.9999);
                saw_ring = true;
            }
        }
        assert!(saw_muzzle && saw_ring);
        let count = particle_count(&mut app);
        assert_eq!(count, 16);
        let health = app.world().get::<Health>(target).unwrap().current();
        let position = app.world().get::<Body>(target).unwrap().world_position();
        render(&mut app, 0.0);
        render(&mut app, 0.0);
        assert_eq!(particle_count(&mut app), count);
        assert_eq!(app.world().get::<Health>(target).unwrap().current(), health);
        assert_eq!(
            app.world().get::<Body>(target).unwrap().world_position(),
            position
        );
    }

    #[test]
    fn particles_and_recoil_pause_then_expire_with_shared_assets_across_shots() {
        let (mut app, player, _) = arena();
        let meshes = app.world().resource::<Assets<Mesh>>().len();
        let materials = app.world().resource::<Assets<StandardMaterial>>().len();
        for _ in 0..2 {
            prepare(&mut app, player);
            discharge(&mut app, player);

            render(&mut app, 0.03);
            let count = particle_count(&mut app);
            let kick = app
                .world()
                .resource::<ImpactState>()
                .recoil(player)
                .unwrap();
            assert!(kick.0.z > 0.0 && kick.0.z < 0.15);
            let frozen: Vec<_> = app
                .world_mut()
                .query_filtered::<(Entity, &Transform), With<ImpactParticle>>()
                .iter(app.world())
                .map(|(entity, transform)| (entity, *transform))
                .collect();
            app.world_mut().resource_mut::<SimulationActive>().0 = false;
            render(&mut app, 2.0);
            assert_eq!(particle_count(&mut app), count);
            for (entity, transform) in frozen {
                assert_eq!(*app.world().get::<Transform>(entity).unwrap(), transform);
            }
            assert_eq!(
                app.world().resource::<ImpactState>().recoil(player),
                Some(kick)
            );
            app.world_mut().resource_mut::<SimulationActive>().0 = true;
            render(&mut app, 0.25);
            assert_eq!(particle_count(&mut app), 0);
            assert!(
                app.world()
                    .resource::<ImpactState>()
                    .recoil(player)
                    .is_none()
            );
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
            assert_eq!(
                app.world().resource::<Assets<StandardMaterial>>().len(),
                materials
            );
            tick(&mut app, 420);
        }
    }

    #[test]
    fn ordinary_ranged_attack_never_creates_a_special_cue() {
        let (mut app, player, _) = arena();
        request(&mut app, player, CombatIntent::request_ranged_skill);
        tick(&mut app, 14);
        render(&mut app, 0.0);
        assert!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| event.kind == CombatEffect::Shot)
        );
        assert_eq!(particle_count(&mut app), 0);
        assert!(
            app.world()
                .resource::<ImpactState>()
                .recoil(player)
                .is_none()
        );
    }
}
