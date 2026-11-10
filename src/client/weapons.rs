//! Shared melee geometry, source ranged meshes and read-only chain-axe presentation.
use crate::simulation::{WeaponMode, WeaponPair};
use bevy::prelude::*;

#[derive(Resource, Clone)]
pub(super) struct WeaponAssets {
    pub cube: Handle<Mesh>,
    pub steel: Handle<StandardMaterial>,
    pub wood: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
}
impl WeaponAssets {
    pub fn shared(world: &mut World) -> Self {
        if let Some(assets) = world.get_resource::<Self>() {
            return assets.clone();
        }
        let cube = world.resource_mut::<Assets<Mesh>>().add(Cuboid::default());
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mut mat = |color| {
            materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.65,
                ..default()
            })
        };
        let assets = Self {
            cube,
            steel: mat(Color::srgb(0.55, 0.65, 0.70)),
            wood: mat(Color::srgb(0.23, 0.085, 0.035)),
            gold: mat(Color::srgb(0.95, 0.66, 0.18)),
        };
        world.insert_resource(assets.clone());
        assets
    }
}
pub(super) fn part(
    world: &mut World,
    parent: Entity,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    let entity = world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
        ))
        .id();
    world.entity_mut(parent).add_child(entity);
    entity
}
fn bar(
    world: &mut World,
    parent: Entity,
    assets: &WeaponAssets,
    material: &Handle<StandardMaterial>,
    a: Vec3,
    b: Vec3,
    width: f32,
) {
    let delta = b - a;
    part(
        world,
        parent,
        &assets.cube,
        material,
        Transform::from_translation((a + b) * 0.5)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, delta.normalize()))
            .with_scale(Vec3::new(width, delta.length(), width)),
    );
}
/// Melee grip extends +Y; ranged barrel/arrow points -Z, muzzle at local origin.
pub(super) fn spawn(world: &mut World, root: Entity, pair: WeaponPair, mode: WeaponMode) {
    let a = WeaponAssets::shared(world);
    match (pair, mode) {
        (WeaponPair::HammerMusket, WeaponMode::Melee) => {
            bar(
                world,
                root,
                &a,
                &a.wood,
                Vec3::new(0., -0.25, 0.),
                Vec3::new(0., 0.85, 0.),
                0.07,
            );
            part(
                world,
                root,
                &a.cube,
                &a.steel,
                Transform::from_xyz(0., 0.85, 0.).with_scale(Vec3::new(0.70, 0.33, 0.32)),
            );
            for x in [-0.32, 0.32] {
                part(
                    world,
                    root,
                    &a.cube,
                    &a.gold,
                    Transform::from_xyz(x, 0.85, 0.).with_scale(Vec3::new(0.09, 0.37, 0.36)),
                );
            }
        }
        (WeaponPair::ChainAxeBow, WeaponMode::Melee) => {
            bar(
                world,
                root,
                &a,
                &a.wood,
                Vec3::new(0., -0.2, 0.),
                Vec3::new(0., 0.85, 0.),
                0.065,
            );
            for side in [-1., 1.] {
                part(
                    world,
                    root,
                    &a.cube,
                    &a.steel,
                    Transform::from_xyz(side * 0.22, 0.73, 0.)
                        .with_rotation(Quat::from_rotation_z(side * 0.32))
                        .with_scale(Vec3::new(0.35, 0.42, 0.075)),
                );
                part(
                    world,
                    root,
                    &a.cube,
                    &a.gold,
                    Transform::from_xyz(side * 0.36, 0.73, 0.)
                        .with_scale(Vec3::new(0.055, 0.47, 0.09)),
                );
            }
            for i in 0..6 {
                let y = -0.23 - i as f32 * 0.07;
                part(
                    world,
                    root,
                    &a.cube,
                    &a.steel,
                    Transform::from_xyz((i % 2) as f32 * 0.03, y, 0.)
                        .with_scale(Vec3::new(0.07, 0.045, 0.035)),
                );
            }
        }
        (_, WeaponMode::Ranged) => {
            super::ranged_models::spawn(world, root, pair, super::ranged_models::Mount::Muzzle);
        }
    }
}

#[derive(Component)]
pub(super) struct FlyingAxe;
#[derive(Component)]
pub(super) struct Tether;
/// Persistent views; fixed-step snapshots alone determine throw and return progress.
pub(super) fn setup_tether(world: &mut World) {
    let assets = WeaponAssets::shared(world);
    let axe = world
        .spawn((FlyingAxe, Transform::IDENTITY, Visibility::Hidden))
        .id();
    spawn(world, axe, WeaponPair::ChainAxeBow, WeaponMode::Melee);
    world.spawn((
        Tether,
        Mesh3d(assets.cube),
        MeshMaterial3d(assets.gold),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));
}
type TetherData = (
    &'static mut Transform,
    &'static mut Visibility,
    Has<FlyingAxe>,
);
type TetherViews = Or<(With<FlyingAxe>, With<Tether>)>;
pub(super) fn sync_tether(
    time: Res<Time<Fixed>>,
    active: Res<crate::simulation::SimulationActive>,
    hit_stop: Res<crate::simulation::HitStop>,
    player: Single<
        (&crate::simulation::Combatant, &crate::simulation::Body),
        With<crate::gameplay::Player>,
    >,
    mut views: Query<TetherData, TetherViews>,
) {
    use crate::simulation::ActionKind;
    let (combat, body) = *player;
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    let hand = body.interpolated_world(alpha) + Vec3::Y * 1.0;
    let action = combat
        .active_action()
        .filter(|a| a.pair == WeaponPair::ChainAxeBow && combat.axe_away());
    for (mut transform, mut visibility, is_axe) in &mut views {
        let Some(a) = action else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let elapsed = (a.elapsed - time.delta_secs() * (1. - alpha)).max(0.);
        let point = if a.kind == ActionKind::AxeShot {
            if elapsed < 0.30 {
                a.source_position
                    .lerp(a.aim_position, ((elapsed - 0.15) / 0.15).clamp(0., 1.))
            } else {
                a.aim_position
                    .lerp(hand, ((elapsed - 0.30) / (a.duration - 0.30)).clamp(0., 1.))
            }
        } else {
            a.aim_position
                .lerp(hand, ((elapsed - 0.65) / 0.28).clamp(0., 1.))
        };
        *visibility = Visibility::Visible;
        if is_axe {
            *transform = Transform::from_translation(point)
                .with_rotation(Quat::from_rotation_z(elapsed * 20.));
        } else {
            *transform = Transform::from_translation((point + hand) * 0.5)
                .looking_at(point, Vec3::Y)
                .with_scale(Vec3::new(0.018, 0.018, point.distance(hand).max(0.01)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gameplay::Player,
        simulation::{Body, CombatIntent, Combatant, SimulationPlugin},
    };
    use std::time::Duration;
    #[test]
    fn real_chain_throw_returns_its_persistent_axe_and_tether_without_growing_assets() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Startup, (crate::gameplay::setup, setup_tether).chain())
            .add_systems(Update, sync_tether);
        app.finish();
        app.update();
        let tick = |app: &mut App, n| {
            for _ in 0..n {
                app.world_mut()
                    .resource_mut::<Time<Fixed>>()
                    .advance_by(Duration::from_secs_f64(1. / 60.));
                app.world_mut().run_schedule(FixedUpdate);
            }
        };
        tick(&mut app, 12);
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        {
            let mut i = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
            i.request_toggle_target();
            i.request_pair(WeaponPair::ChainAxeBow);
        }
        tick(&mut app, 1);
        let counts = (
            app.world().entities().len(),
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<StandardMaterial>>().len(),
        );
        for _ in 0..2 {
            app.world_mut()
                .get_mut::<CombatIntent>(player)
                .unwrap()
                .request_ranged_skill();
            tick(&mut app, 13);
            assert!(app.world().get::<Combatant>(player).unwrap().axe_away());
            let position = app.world().get::<Body>(player).unwrap().world_position();
            app.world_mut().run_schedule(Update);
            assert_eq!(
                app.world().get::<Body>(player).unwrap().world_position(),
                position
            );
            let axe = app
                .world_mut()
                .query_filtered::<(&Transform, &Visibility), With<FlyingAxe>>()
                .single(app.world())
                .unwrap();
            assert_eq!(*axe.1, Visibility::Visible);
            assert!(axe.0.translation.is_finite());
            assert!(axe.0.translation.y > 0.5);
            let line = app
                .world_mut()
                .query_filtered::<(&Transform, &Visibility), With<Tether>>()
                .single(app.world())
                .unwrap();
            assert_eq!(*line.1, Visibility::Visible);
            assert!(line.0.scale.z > 0.2);
            tick(&mut app, 50);
            app.world_mut().run_schedule(Update);
            assert!(!app.world().get::<Combatant>(player).unwrap().axe_away());
            for visibility in app
                .world_mut()
                .query_filtered::<&Visibility, TetherViews>()
                .iter(app.world())
            {
                assert_eq!(*visibility, Visibility::Hidden);
            }
            assert_eq!(
                (
                    app.world().entities().len(),
                    app.world().resource::<Assets<Mesh>>().len(),
                    app.world().resource::<Assets<StandardMaterial>>().len()
                ),
                counts
            );
            tick(&mut app, 260);
        }
    }
}
