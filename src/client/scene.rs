//! Procedural presentation. Visual transforms interpolate authoritative poses.
use super::{
    abilities,
    character::{self, AnimationLibrary, ModelAssets, RigPose},
    controls::{Aim, ClientFeedback, PlayerController, PointerState},
    picking::Pickable,
    presentation::PresentationFrame,
    session::ClientSession,
    unit_appearance::{self, ModelKind, UnitAppearance, UnitModelAssets},
    visibility::VisibleWorld,
    world_scene,
};
use crate::{
    gameplay::{MapDefinition, UnitRole},
    simulation::*,
};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct SceneAssets {
    base: Handle<Mesh>,
    bolt: Handle<Mesh>,
    crystal: Handle<Mesh>,
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
type UnattachedUnit<'a> = (
    Entity,
    &'a Unit,
    &'a UnitPose,
    &'a UnitState,
    &'a UnitStatuses,
    Option<&'a UnitRole>,
    Option<&'a CharacterIdentity>,
);
type DisplayedUnit<'a> = (
    Entity,
    &'a UnitPose,
    &'a UnitState,
    &'a UnitStatuses,
    Option<&'a UnitAppearance>,
    &'a mut Transform,
    &'a mut Visibility,
);

pub(super) fn setup(
    mut commands: Commands,
    map: Res<MapDefinition>,
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
    world_scene::spawn_map(&mut commands, &map, &mut meshes, &mut materials);
    commands.insert_resource(SceneAssets {
        base: meshes.add(Cylinder::new(0.5, 0.12)),
        bolt: meshes.add(Cuboid::new(0.12, 0.12, 0.85)),
        crystal: meshes.add(Sphere::new(1.0).mesh().ico(0).expect("crystal arrow")),
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
    simple: Res<UnitModelAssets>,
    units: Query<UnattachedUnit<'_>, Without<UnitVisual>>,
) {
    for (entity, unit, pose, state, statuses, role, identity) in &units {
        let appearance = UnitAppearance::new(role.copied(), unit.definition().radius);
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
                appearance,
                Pickable {
                    offset: appearance.pick_center,
                    half_size: appearance.pick_half_size,
                },
                Transform::from_xyz(point.x, height, point.y).with_rotation(Quat::from_rotation_y(
                    (-pose.facing().x).atan2(-pose.facing().y),
                )),
                appearance.visible(state.alive()),
                Name::new(unit.name().to_owned()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    UnitGroundBase,
                    Mesh3d(assets.base.clone()),
                    MeshMaterial3d(assets.dark.clone()),
                    Transform::from_xyz(0.0, 0.12 - height, 0.0).with_scale(Vec3::new(
                        unit.definition().radius / 0.45,
                        1.0,
                        unit.definition().radius / 0.45,
                    )),
                ));
            });
        if appearance.kind == ModelKind::Hero {
            let weapon = character::WeaponStyle::for_character(identity.map(CharacterIdentity::id));
            let rig = RigPose::new(
                &library,
                state.life_ref(entity),
                tick.0 as f64 / TICK_RATE,
                weapon,
            );
            character::spawn_model(
                &mut commands,
                entity,
                &models,
                &library,
                rig.model(),
                material,
                weapon,
            );
            commands.entity(entity).insert(rig);
        } else {
            unit_appearance::spawn_simple_model(
                &mut commands,
                entity,
                appearance,
                &simple,
                material,
            );
        }
    }
}

pub(super) fn sync_units(
    frame: Res<PresentationFrame>,
    visible: Option<Res<VisibleWorld>>,
    motion: Option<Res<super::motion_view::MotionView>>,
    mut units: Query<DisplayedUnit<'_>, With<UnitVisual>>,
) {
    if !frame.should_sync() {
        return;
    }
    let fraction = frame.fraction();
    for (id, pose, state, statuses, appearance, mut transform, mut visibility) in &mut units {
        let point = pose.interpolated(fraction);
        let height = if state.alive() {
            (statuses.airborne_height(fraction) * AIRBORNE_PEAK_HEIGHT)
                .max(motion.as_ref().map_or(0.0, |motion| motion.height(id)))
        } else {
            0.0
        };
        transform.translation = Vec3::new(point.x, height, point.y);
        transform.rotation = Quat::from_rotation_y((-pose.facing().x).atan2(-pose.facing().y));
        *visibility = if visible
            .as_ref()
            .is_some_and(|visible| !visible.contains(id))
        {
            Visibility::Hidden
        } else {
            appearance.map_or_else(
                || {
                    if state.alive() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                },
                |appearance| appearance.visible(state.alive()),
            )
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

fn projectile_visibility(visible: Option<&VisibleWorld>, id: Entity) -> Visibility {
    if visible.is_none_or(|visible| visible.contains(id)) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}
fn projectile_transform(projectile: &Projectile, fraction: f32) -> Transform {
    let point = projectile.interpolated(fraction);
    let direction = projectile.direction();
    Transform::from_xyz(point.x, 0.9, point.y)
        .with_rotation(if direction.length_squared() > 1e-8 {
            Quat::from_rotation_y((-direction.x).atan2(-direction.y))
        } else {
            Quat::IDENTITY
        })
        .with_scale(if projectile.is_crystal_arrow() {
            Vec3::new(0.45, 0.25, 1.2)
        } else {
            Vec3::ONE
        })
}
pub(super) fn attach_projectiles(
    mut commands: Commands,
    frame: Res<PresentationFrame>,
    assets: Res<SceneAssets>,
    visible: Option<Res<VisibleWorld>>,
    projectiles: Query<(Entity, &Projectile), Without<ProjectileVisual>>,
) {
    for (id, projectile) in &projectiles {
        commands.entity(id).insert((
            ProjectileVisual,
            projectile_visibility(visible.as_deref(), id),
            Mesh3d(if projectile.is_crystal_arrow() {
                assets.crystal.clone()
            } else {
                assets.bolt.clone()
            }),
            MeshMaterial3d(if projectile.team() == Team::Ally {
                assets.ally.clone()
            } else {
                assets.enemy.clone()
            }),
            projectile_transform(projectile, frame.fraction()),
        ));
        if projectile.arrows() > 1 && !projectile.is_crystal_arrow() {
            let count = projectile.arrows();
            commands.entity(id).with_children(|carrier| {
                for index in 1..count {
                    let side = if index % 2 == 0 { 1.0 } else { -1.0 };
                    let offset = side * ((index as f32 + 1.0) * 0.5).floor() * 0.16;
                    carrier.spawn((
                        Mesh3d(assets.bolt.clone()),
                        MeshMaterial3d(if projectile.team() == Team::Ally {
                            assets.ally.clone()
                        } else {
                            assets.enemy.clone()
                        }),
                        Transform::from_xyz(offset, 0.0, 0.0),
                    ));
                }
            });
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy system parameters declare independent ECS access"
)]
pub(super) fn sync_effects(
    time: Res<Time>,
    frame: Res<PresentationFrame>,
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
    mut visuals: Query<
        (Entity, &Projectile, &mut Transform, &mut Visibility),
        With<ProjectileVisual>,
    >,
    areas: Query<(Entity, &AreaEffect)>,
    visible: Option<Res<VisibleWorld>>,
    mut gizmos: Gizmos,
) {
    let fraction = frame.fraction();
    for (id, projectile, mut transform, mut visibility) in &mut visuals {
        *visibility = projectile_visibility(visible.as_deref(), id);
        *transform = projectile_transform(projectile, fraction);
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
        if !state.alive()
            || visible
                .as_ref()
                .is_some_and(|visible| !visible.contains(id))
        {
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
    for (id, area) in &areas {
        if visible
            .as_ref()
            .is_some_and(|visible| !visible.contains(id))
        {
            continue;
        }
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
                            || abilities::clamps_point_target(definition)
                            || origin.distance(point) <= definition.range;
                        let color = if valid {
                            Color::srgb_u8(153, 236, 214)
                        } else if definition.targeting == Targeting::Enemy {
                            Color::srgb_u8(229, 177, 105)
                        } else {
                            Color::srgb_u8(240, 117, 101)
                        };
                        match &definition.execution {
                            AbilityExecution::Impact(ImpactDefinition {
                                delivery: Delivery::Area { radius, .. },
                                ..
                            }) => ring(&mut gizmos, point, *radius, color),
                            AbilityExecution::Standard(spec) => {
                                let end =
                                    origin + (point - origin).clamp_length_max(definition.range);
                                ring(&mut gizmos, end, spec.aura_radius, color);
                            }
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
            .init_resource::<PresentationFrame>()
            .add_systems(
                Update,
                (
                    super::super::presentation::prepare,
                    sync_units,
                    sync_ground_bases,
                )
                    .chain(),
            );
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

    #[test]
    fn a_hidden_enemy_projectile_starts_hidden_with_its_actual_interpolated_transform() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            SimulationPlugin,
            crate::gameplay::GameplayPlugin,
        ))
        .init_resource::<PresentationFrame>()
        .init_resource::<VisibleWorld>()
        .insert_resource(SceneAssets {
            base: default(),
            bolt: default(),
            crystal: default(),
            ally: default(),
            enemy: default(),
            dark: default(),
        })
        .add_systems(
            Update,
            (
                crate::client::visibility::refresh,
                crate::client::presentation::prepare,
                attach_projectiles,
            )
                .chain(),
        );
        crate::client::content::setup(app.world_mut());
        crate::client::session::setup(app.world_mut());
        crate::client::session::start_native(app.world_mut());
        let caster = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Hidden projectile fixture".into(),
                team: Team::Enemy,
                position: Vec2::ZERO,
                stats: UnitStats::default(),
                abilities: [Some(AbilityId(1)), None, None, None],
                respawn_delay: None,
            },
        )
        .unwrap();
        app.world_mut().resource_mut::<CommandQueue>().submit(
            caster,
            UnitCommand::CastAbility {
                slot: 0,
                target: AbilityTarget::Direction(Vec2::X),
            },
        );
        for _ in 0..20 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        let id = app
            .world_mut()
            .query_filtered::<Entity, With<Projectile>>()
            .single(app.world())
            .unwrap();
        assert!(!point_visible(
            app.world(),
            Team::Ally,
            app.world().get::<Projectile>(id).unwrap().position()
        ));
        app.world_mut().run_schedule(Update);
        assert!(app.world().get::<ProjectileVisual>(id).is_some());
        assert_eq!(
            *app.world().get::<Visibility>(id).unwrap(),
            Visibility::Hidden,
            "no first-frame flash before the next synchronization"
        );
        let projectile = app.world().get::<Projectile>(id).unwrap();
        let point = projectile.interpolated(app.world().resource::<PresentationFrame>().fraction());
        let displayed = app.world().get::<Transform>(id).unwrap();
        assert_eq!(
            displayed.translation,
            Vec3::new(point.x, 0.9, point.y),
            "first carrier frame uses the actual displayed sample"
        );
        assert!(
            (displayed.rotation * Vec3::NEG_Z).dot(Vec3::X) > 0.99,
            "arrow geometry starts facing its real travel direction"
        );
        assert_eq!(
            displayed.scale,
            Vec3::ONE,
            "ordinary arrows start at their final presentation scale"
        );
    }
}
