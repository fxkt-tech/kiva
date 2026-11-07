//! Procedural presentation. Visual transforms interpolate authoritative poses.
use super::{
    abilities,
    character::{self, AnimationLibrary, ModelAssets, RigPose},
    content::Scenery,
    controls::{Aim, ClientFeedback, Pickable, PlayerController, PointerState},
    session::ClientSession,
};
use crate::simulation::*;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct SceneAssets {
    base: Handle<Mesh>,
    bolt: Handle<Mesh>,
    ally: Handle<StandardMaterial>,
    enemy: Handle<StandardMaterial>,
    dark: Handle<StandardMaterial>,
}
#[derive(Component)]
pub(super) struct UnitVisual;
#[derive(Component)]
pub(super) struct UnitGroundBase;
#[derive(Component)]
pub(super) struct ProjectileVisual;

const AIRBORNE_PEAK_HEIGHT: f32 = 1.8;
type GroundBaseFilter = (With<UnitGroundBase>, Without<UnitVisual>);

pub(super) fn setup(
    mut commands: Commands,
    scenery: Res<Scenery>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb_u8(174, 209, 206),
        brightness: 500.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 6500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-12.0, 25.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let ground = materials.add(Color::srgb_u8(35, 61, 54));
    let path = materials.add(Color::srgb_u8(66, 82, 70));
    let stone = materials.add(Color::srgb_u8(79, 91, 83));
    let stone_top = materials.add(Color::srgb_u8(103, 117, 101));
    let foliage = materials.add(Color::srgb_u8(24, 64, 52));
    let foliage_light = materials.add(Color::srgb_u8(36, 83, 61));
    let gold = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(213, 181, 114),
        metallic: 0.5,
        perceptual_roughness: 0.5,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(ground),
        Transform::default(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(14.0, 0.04, 48.0))),
        MeshMaterial3d(path.clone()),
        Transform::from_xyz(0.0, 0.015, -2.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(45.0, 0.035, 9.0))),
        MeshMaterial3d(path),
        Transform::from_xyz(0.0, 0.02, 3.0),
    ));
    let tile = meshes.add(Cuboid::new(1.85, 0.04, 1.85));
    for z in -5..=5 {
        for x in -3..=3 {
            if (x + z) % 3 != 0 {
                continue;
            }
            commands.spawn((
                Mesh3d(tile.clone()),
                MeshMaterial3d(stone.clone()),
                Transform::from_xyz(x as f32 * 2.0, 0.045, z as f32 * 2.0),
            ));
        }
    }
    for obstacle in &scenery.walls {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                obstacle.half_size.x * 2.0,
                1.1,
                obstacle.half_size.y * 2.0,
            ))),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(obstacle.center.x, 0.55, obstacle.center.y),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                obstacle.half_size.x * 2.0 + 0.12,
                0.16,
                obstacle.half_size.y * 2.0 + 0.12,
            ))),
            MeshMaterial3d(stone_top.clone()),
            Transform::from_xyz(obstacle.center.x, 1.12, obstacle.center.y),
        ));
    }
    let trunk = meshes.add(Cylinder::new(0.22, 1.5));
    let crown = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("valid tree mesh"));
    for (i, &(point, height)) in scenery.trees.iter().enumerate() {
        commands.spawn((
            Mesh3d(trunk.clone()),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(point.x, 0.75, point.y),
        ));
        commands.spawn((
            Mesh3d(crown.clone()),
            MeshMaterial3d(if i % 2 == 0 {
                foliage.clone()
            } else {
                foliage_light.clone()
            }),
            Transform::from_xyz(point.x, height, point.y).with_scale(Vec3::new(
                1.2,
                height * 0.85,
                1.2,
            )),
        ));
    }
    // Pillars frame the training grounds without requiring imported assets.
    for point in &scenery.pillars {
        let (x, z) = (point.x, point.y);
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.5, 1.8))),
            MeshMaterial3d(stone_top.clone()),
            Transform::from_xyz(x, 0.9, z),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.25).mesh().uv(12, 8))),
            MeshMaterial3d(gold.clone()),
            Transform::from_xyz(x, 1.95, z),
        ));
    }
    commands.insert_resource(SceneAssets {
        base: meshes.add(Cylinder::new(0.5, 0.12)),
        bolt: meshes.add(Sphere::new(1.0).mesh().uv(12, 8)),
        ally: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(72, 205, 194),
            emissive: LinearRgba::new(0.05, 0.25, 0.22, 1.0),
            ..default()
        }),
        enemy: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(201, 99, 81),
            emissive: LinearRgba::new(0.3, 0.05, 0.03, 1.0),
            ..default()
        }),
        dark: materials.add(Color::srgb_u8(24, 38, 44)),
    });
}

pub(super) fn attach_unit_visuals(
    mut commands: Commands,
    assets: Res<SceneAssets>,
    library: Res<AnimationLibrary>,
    models: Res<ModelAssets>,
    tick: Res<SimulationTick>,
    units: Query<(Entity, &Unit, &UnitPose, &UnitState, &UnitStatuses), Without<UnitVisual>>,
) {
    for (entity, unit, pose, state, statuses) in &units {
        let point = pose.position();
        let height = if state.alive() {
            statuses.airborne_height(0.0) * AIRBORNE_PEAK_HEIGHT
        } else {
            0.0
        };
        let material = if unit.team() == Team::Ally {
            assets.ally.clone()
        } else {
            assets.enemy.clone()
        };
        commands
            .entity(entity)
            .insert((
                UnitVisual,
                Pickable {
                    offset: Vec3::Y,
                    half_size: Vec3::new(
                        unit.definition().radius + 0.1,
                        1.0,
                        unit.definition().radius + 0.1,
                    ),
                },
                Transform::from_xyz(point.x, height, point.y).with_rotation(Quat::from_rotation_y(
                    (-pose.facing().x).atan2(-pose.facing().y),
                )),
                if state.alive() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(unit.name().to_owned()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    UnitGroundBase,
                    Mesh3d(assets.base.clone()),
                    MeshMaterial3d(assets.dark.clone()),
                    Transform::from_xyz(0.0, 0.12 - height, 0.0),
                ));
            });
        let rig = RigPose::new(&library, state.life_ref(entity), tick.0 as f64 / TICK_RATE);
        character::spawn_model(
            &mut commands,
            entity,
            &models,
            &library,
            rig.model(),
            material,
        );
        commands.entity(entity).insert(rig);
    }
}

pub(super) fn sync_units(
    fixed: Res<Time<Fixed>>,
    control: Res<SimulationControl>,
    mut units: Query<
        (
            &UnitPose,
            &UnitState,
            &UnitStatuses,
            &mut Transform,
            &mut Visibility,
        ),
        With<UnitVisual>,
    >,
) {
    if control.paused {
        return;
    }
    let fraction = fixed.overstep_fraction();
    for (pose, state, statuses, mut transform, mut visibility) in &mut units {
        let point = pose.interpolated(fraction);
        let height = if state.alive() {
            statuses.airborne_height(fraction) * AIRBORNE_PEAK_HEIGHT
        } else {
            0.0
        };
        transform.translation = Vec3::new(point.x, height, point.y);
        transform.rotation = Quat::from_rotation_y((-pose.facing().x).atan2(-pose.facing().y));
        *visibility = if state.alive() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_ground_bases(
    units: Query<&Transform, With<UnitVisual>>,
    mut bases: Query<(&ChildOf, &mut Transform), GroundBaseFilter>,
) {
    for (parent, mut transform) in &mut bases {
        if let Ok(root) = units.get(parent.parent()) {
            transform.translation.y = 0.12 - root.translation.y;
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy system parameters declare independent ECS access"
)]
pub(super) fn sync_effects(
    mut commands: Commands,
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    control: Res<SimulationControl>,
    assets: Res<SceneAssets>,
    player: Res<PlayerController>,
    session: Res<ClientSession>,
    pointer: Res<PointerState>,
    feedback: Res<ClientFeedback>,
    catalog: Res<AbilityCatalog>,
    equipment: Res<EquipmentCatalog>,
    units: Query<(
        Entity,
        &Unit,
        &UnitPose,
        &UnitState,
        &AbilitySlots,
        &EquipmentSlots,
        &UnitAction,
    )>,
    projectiles: Query<(Entity, &Projectile), Without<ProjectileVisual>>,
    mut visuals: Query<(&Projectile, &mut Transform), With<ProjectileVisual>>,
    areas: Query<&AreaEffect>,
    mut gizmos: Gizmos,
) {
    let fraction = if control.paused {
        1.0
    } else {
        fixed.overstep_fraction()
    };
    for (entity, projectile) in &projectiles {
        commands.entity(entity).insert((
            ProjectileVisual,
            Mesh3d(assets.bolt.clone()),
            MeshMaterial3d(if projectile.team() == Team::Ally {
                assets.ally.clone()
            } else {
                assets.enemy.clone()
            }),
            Transform::default(),
        ));
    }
    for (projectile, mut transform) in &mut visuals {
        let point = projectile.interpolated(fraction);
        transform.translation = Vec3::new(point.x, 0.9, point.y);
        transform.scale = Vec3::splat(projectile.radius().max(0.16));
    }
    let ring = |gizmos: &mut Gizmos, point: Vec2, radius: f32, color: Color| {
        gizmos.circle(
            Isometry3d::new(
                Vec3::new(point.x, 0.08, point.y),
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ),
            radius,
            color,
        );
    };
    for (id, unit, pose, state, _, _, action) in &units {
        if !state.alive() {
            continue;
        }
        let point = pose.interpolated(fraction);
        if let UnitAction::Channeling {
            pulse, remaining, ..
        } = action
            && let Delivery::Area { radius, .. } = pulse.delivery()
        {
            ring(&mut gizmos, point, radius, Color::srgb_u8(140, 212, 190));
            ring(
                &mut gizmos,
                point,
                radius * (1.0 - (remaining * 2.0).fract()),
                Color::srgb_u8(189, 227, 201),
            );
        }
        let color = if unit.team() == Team::Ally {
            Color::srgb_u8(112, 223, 204)
        } else {
            Color::srgb_u8(211, 121, 100)
        };
        ring(&mut gizmos, point, unit.definition().radius + 0.16, color);
        if player.selected == Some(id) || pointer.hovered == Some(id) {
            ring(
                &mut gizmos,
                point,
                unit.definition().radius + 0.28,
                Color::srgb_u8(242, 207, 137),
            );
        }
    }
    for area in &areas {
        ring(
            &mut gizmos,
            area.position(),
            area.radius(),
            Color::srgb_u8(225, 174, 95),
        );
        ring(
            &mut gizmos,
            area.position(),
            area.radius() * area.progress(),
            Color::srgb_u8(246, 216, 142),
        );
    }
    if let Some((point, valid, start)) = feedback.marker {
        let age = time.elapsed_secs() - start;
        if age < 0.8 {
            ring(
                &mut gizmos,
                point,
                0.3 + age * 0.6,
                if valid {
                    Color::srgb_u8(115, 232, 198)
                } else {
                    Color::srgb_u8(245, 104, 91)
                },
            );
        }
    }
    if let Some(aim) = player.aim
        && let Some(player) = session.player()
        && let Ok((_, unit, pose, _, slots, items, _)) = units.get(player)
    {
        let origin = pose.position();
        match aim {
            Aim::AttackMove => ring(
                &mut gizmos,
                origin,
                unit.attributes().attack_range() + unit.definition().radius,
                Color::srgb_u8(229, 177, 105),
            ),
            aim @ (Aim::Ability(_) | Aim::Equipment { .. }) => {
                if let Some(info) = abilities::resolve(
                    aim.source().expect("cast aim"),
                    slots,
                    items,
                    &catalog,
                    &equipment,
                ) {
                    let definition = info.definition;
                    ring(
                        &mut gizmos,
                        origin,
                        definition.range,
                        Color::srgb_u8(99, 204, 204),
                    );
                    if let Some(point) = pointer.ground {
                        let valid = definition.targeting == Targeting::Direction
                            || definition.execution.is_dash()
                            || origin.distance(point) <= definition.range;
                        let color = if valid {
                            Color::srgb_u8(153, 236, 214)
                        } else {
                            Color::srgb_u8(240, 117, 101)
                        };
                        match &definition.execution {
                            AbilityExecution::Impact(ImpactDefinition {
                                delivery: Delivery::Area { radius, .. },
                                ..
                            }) => ring(&mut gizmos, point, *radius, color),
                            _ => {
                                let end =
                                    origin + (point - origin).clamp_length_max(definition.range);
                                gizmos.line(
                                    Vec3::new(origin.x, 0.1, origin.y),
                                    Vec3::new(end.x, 0.1, end.y),
                                    color,
                                );
                                ring(&mut gizmos, end, 0.25, color);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn airborne_display_keeps_ground_base_and_freezes_until_landing_or_respawn() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, SimulationPlugin))
            .insert_resource(NavigationMap::new(20.0, 1.0, vec![]))
            .add_systems(Update, (sync_units, sync_ground_bases).chain());
        let airborne = AbilityId(100);
        let kill = AbilityId(101);
        for (id, effect) in [
            (
                airborne,
                Effect::ApplyStatus(StatusSpec::Airborne {
                    duration: StatusDuration::from_millis(800).unwrap(),
                }),
            ),
            (
                kill,
                Effect::Damage(DamageSpec {
                    kind: DamageType::True,
                    base: 1000.0,
                    ad_ratio: 0.0,
                    ap_ratio: 0.0,
                }),
            ),
        ] {
            app.world_mut()
                .resource_mut::<AbilityCatalog>()
                .register(
                    id,
                    AbilityDefinition {
                        name: "Display test".into(),
                        targeting: Targeting::Enemy,
                        range: 10.0,
                        windup: 0.0,
                        recovery: 0.0,
                        cooldown: 0.0,
                        mana_cost: 0.0,
                        execution: AbilityExecution::Impact(ImpactDefinition {
                            delivery: Delivery::Direct,
                            effects: vec![effect],
                        }),
                    },
                )
                .unwrap();
        }
        let caster = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Display source".into(),
                team: Team::Ally,
                position: Vec2::ZERO,
                stats: UnitStats::default(),
                abilities: [Some(airborne), Some(kill), None, None],
                respawn_delay: None,
            },
        )
        .unwrap();
        let target = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Visible target".into(),
                team: Team::Enemy,
                position: Vec2::X * 2.0,
                stats: UnitStats::default(),
                abilities: [None; 4],
                respawn_delay: Some(0.1),
            },
        )
        .unwrap();
        app.world_mut().entity_mut(target).insert((
            UnitVisual,
            Transform::from_xyz(2.0, 0.0, 0.0),
            Visibility::Inherited,
        ));
        let base = app
            .world_mut()
            .spawn((
                UnitGroundBase,
                ChildOf(target),
                Transform::from_xyz(0.0, 0.12, 0.0),
            ))
            .id();
        app.world_mut().resource_mut::<CommandQueue>().submit(
            caster,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Unit(target),
            },
        );
        for _ in 0..12 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().run_schedule(Update);
        app.world_mut().run_schedule(PostUpdate);
        let displayed = *app.world().get::<Transform>(target).unwrap();
        assert!(displayed.translation.y > 0.5);
        assert_eq!(displayed.translation.x, 2.0);
        assert!(
            (app.world()
                .get::<GlobalTransform>(base)
                .unwrap()
                .translation()
                .y
                - 0.12)
                .abs()
                < 1e-5,
            "the body's display root rises while its ground base stays down"
        );
        app.world_mut().resource_mut::<SimulationControl>().paused = true;
        let mut fixed = app.world_mut().resource_mut::<Time<Fixed>>();
        let extra = fixed.timestep().mul_f32(0.9);
        fixed.accumulate_overstep(extra);
        app.world_mut().run_schedule(Update);
        assert_eq!(*app.world().get::<Transform>(target).unwrap(), displayed);
        app.world_mut().resource_mut::<SimulationControl>().paused = false;
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .discard_overstep(extra);
        app.world_mut().resource_mut::<CommandQueue>().submit(
            caster,
            UnitCommand::CastAbility {
                slot: 1,
                target: AbilityTarget::Unit(target),
            },
        );
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut().run_schedule(Update);
        assert!(!app.world().get::<UnitState>(target).unwrap().alive());
        assert_eq!(
            *app.world().get::<Visibility>(target).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            app.world().get::<Transform>(target).unwrap().translation.y,
            0.0
        );
        for _ in 0..10 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().run_schedule(Update);
        assert!(app.world().get::<UnitState>(target).unwrap().alive());
        assert_eq!(
            app.world().get::<Transform>(target).unwrap().translation,
            Vec3::X * 2.0
        );
        assert!(app.world().get::<UnitStatuses>(target).unwrap().is_empty());
    }
}
