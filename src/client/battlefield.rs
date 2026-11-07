//! Read-only field-object and shield presentation; all lifetimes belong to simulation.
use super::{presentation::PresentationFrame, scene::UnitVisual, visibility::VisibleWorld};
use crate::simulation::*;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct FieldAssets {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    crystal: Handle<Mesh>,
    ally: Handle<StandardMaterial>,
    enemy: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    shield: Handle<StandardMaterial>,
}
#[derive(Component)]
pub(super) struct ObjectVisual;
#[derive(Component)]
pub(super) struct HasShield;
#[derive(Component)]
pub(super) struct ShieldVisual(Entity);

pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(FieldAssets {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        sphere: meshes.add(Sphere::new(1.0).mesh().uv(16, 12)),
        crystal: meshes.add(Sphere::new(1.0).mesh().ico(0).expect("field crystal")),
        ally: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(99, 201, 235),
            emissive: LinearRgba::new(0.06, 0.25, 0.4, 1.0),
            ..default()
        }),
        enemy: materials.add(Color::srgb_u8(216, 112, 87)),
        stone: materials.add(Color::srgb_u8(92, 85, 68)),
        gold: materials.add(Color::srgb_u8(228, 191, 88)),
        shield: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.77, 0.3, 0.19),
            alpha_mode: AlphaMode::Blend,
            emissive: LinearRgba::new(0.12, 0.06, 0.005, 1.0),
            unlit: true,
            ..default()
        }),
    });
}

pub(super) fn attach(
    mut commands: Commands,
    assets: Res<FieldAssets>,
    objects: Query<(Entity, &BattlefieldObject), Without<ObjectVisual>>,
    units: Query<Entity, (With<UnitVisual>, Without<HasShield>)>,
) {
    for owner in &units {
        commands
            .entity(owner)
            .insert(HasShield)
            .with_children(|root| {
                root.spawn((
                    ShieldVisual(owner),
                    Mesh3d(assets.sphere.clone()),
                    MeshMaterial3d(assets.shield.clone()),
                    Transform::from_xyz(0.0, 1.0, 0.0).with_scale(Vec3::new(0.85, 1.1, 0.85)),
                    Visibility::Hidden,
                ));
            });
    }
    for (id, object) in &objects {
        let color = if object.team() == Team::Ally {
            assets.ally.clone()
        } else {
            assets.enemy.clone()
        };
        let point = object.position();
        commands
            .entity(id)
            .insert((
                ObjectVisual,
                Name::new(format!("{:?}", object.kind())),
                Transform::from_xyz(point.x, 0.0, point.y),
                Visibility::Inherited,
            ))
            .with_children(|root| match object.kind() {
                ObjectKind::Standard => {
                    root.spawn((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(assets.gold.clone()),
                        Transform::from_xyz(0.0, 1.15, 0.0).with_scale(Vec3::new(0.07, 2.3, 0.07)),
                    ));
                    root.spawn((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(color),
                        Transform::from_xyz(0.4, 1.9, 0.0).with_scale(Vec3::new(0.8, 0.55, 0.055)),
                    ));
                }
                ObjectKind::Terrain => {
                    for index in 0..24 {
                        let angle = index as f32 * std::f32::consts::TAU / 24.0;
                        root.spawn((
                            Mesh3d(assets.cube.clone()),
                            MeshMaterial3d(assets.stone.clone()),
                            Transform::from_xyz(
                                angle.cos() * object.radius(),
                                0.65,
                                angle.sin() * object.radius(),
                            )
                            .with_rotation(Quat::from_rotation_y(-angle))
                            .with_scale(Vec3::new(
                                0.32,
                                1.3,
                                object.radius() * 0.3,
                            )),
                        ));
                    }
                }
                ObjectKind::Scout => {
                    root.spawn((
                        Mesh3d(assets.crystal.clone()),
                        MeshMaterial3d(color.clone()),
                        Transform::from_xyz(0.0, 1.8, 0.0).with_scale(Vec3::new(0.2, 0.2, 0.55)),
                    ));
                    for side in [-1.0, 1.0] {
                        root.spawn((
                            Mesh3d(assets.crystal.clone()),
                            MeshMaterial3d(color.clone()),
                            Transform::from_xyz(side * 0.4, 1.82, 0.05)
                                .with_scale(Vec3::new(0.55, 0.06, 0.26)),
                        ));
                    }
                }
                ObjectKind::Reveal => {
                    root.spawn((
                        Mesh3d(assets.crystal.clone()),
                        MeshMaterial3d(color),
                        Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::new(0.12, 0.45, 0.12)),
                    ));
                }
            });
    }
}

pub(super) fn sync(
    frame: Res<PresentationFrame>,
    tick: Res<SimulationTick>,
    visible: Res<VisibleWorld>,
    mut objects: Query<
        (Entity, &BattlefieldObject, &mut Transform, &mut Visibility),
        With<ObjectVisual>,
    >,
    mechanics: Query<&UnitMechanics>,
    mut shields: Query<(&ShieldVisual, &mut Visibility), Without<ObjectVisual>>,
    mut gizmos: Gizmos,
) {
    for (id, object, mut transform, mut visibility) in &mut objects {
        let shown = visible.contains(id) && object.active(tick.0);
        *visibility = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if frame.should_sync() {
            let point = object.interpolated(frame.fraction());
            transform.translation = Vec3::new(point.x, 0.0, point.y);
            if object.kind() == ObjectKind::Scout {
                let direction = object.position() - object.interpolated(0.0);
                if direction.length_squared() > 1e-8 {
                    transform.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.y));
                }
            }
        }
        if shown {
            let color = match object.kind() {
                ObjectKind::Terrain => Color::srgb_u8(232, 180, 89),
                _ => Color::srgb_u8(107, 205, 231),
            };
            let radius = if matches!(object.kind(), ObjectKind::Scout | ObjectKind::Reveal) {
                object.vision_radius()
            } else if object.kind() == ObjectKind::Standard {
                object.aura_radius()
            } else {
                object.radius()
            };
            gizmos.circle(
                Isometry3d::new(
                    transform.translation + Vec3::Y * 0.07,
                    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                ),
                radius,
                color,
            );
        }
    }
    if frame.should_sync() {
        for (shield, mut visibility) in &mut shields {
            *visibility = if mechanics
                .get(shield.0)
                .is_ok_and(|state| state.shield() > 0.0)
            {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
