use super::*;
use crate::simulation::{
    BaseAttributes, CommandQueue, SimulationPlugin, UnitCommand, UnitSpec, UnitStats,
};
use bevy::camera::{CameraProjection, RenderTargetInfo};

fn app(distance: f32, scale: f32) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    app.insert_resource(crate::simulation::NavigationMap::new(32.0, 1.0, Vec::new()));
    let mut camera = Camera::default();
    camera.computed.target_info = Some(RenderTargetInfo {
        physical_size: (Vec2::new(1280.0, 800.0) * scale).as_uvec2(),
        scale_factor: scale,
    });
    camera.computed.clip_from_view = PerspectiveProjection {
        aspect_ratio: 1.6,
        ..default()
    }
    .get_clip_from_view();
    let transform = Transform::from_xyz(0.0, 1.0, distance).looking_at(Vec3::Y, Vec3::Y);
    app.world_mut()
        .spawn((GameCamera, camera, GlobalTransform::from(transform)));
    app
}

fn actor(app: &mut App, team: crate::simulation::Team) -> Entity {
    let id = crate::simulation::spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Picking actor".into(),
            team,
            position: Vec2::ZERO,
            stats: UnitStats::default(),
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut().entity_mut(id).insert((
        GlobalTransform::IDENTITY,
        Pickable {
            offset: Vec3::Y,
            half_size: Vec3::splat(0.2),
        },
    ));
    id
}

fn part(
    app: &mut App,
    owner: Entity,
    transform: Transform,
    half_size: Vec3,
    selectable: bool,
) -> Entity {
    let id = app
        .world_mut()
        .spawn((
            Mesh3d(Handle::default()),
            Aabb::from_min_max(-half_size, half_size),
            GlobalTransform::from(transform),
            InheritedVisibility::VISIBLE,
        ))
        .id();
    if selectable {
        app.world_mut().entity_mut(id).insert(PickGeometry);
    }
    app.world_mut().entity_mut(owner).add_child(id);
    id
}

fn screen(app: &mut App, point: Vec3) -> Vec2 {
    let mut query = app
        .world_mut()
        .query_filtered::<(&Camera, &GlobalTransform), With<GameCamera>>();
    let (camera, transform) = query.single(app.world()).unwrap();
    camera.world_to_viewport(transform, point).unwrap()
}

#[test]
fn skinned_model_uses_the_displayed_joint_instead_of_its_bind_pose_mesh_box() {
    use bevy::mesh::{
        VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    };
    let mut app = app(10.0, 1.0);
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>();
    let owner = actor(&mut app, crate::simulation::Team::Enemy);
    let joint = app
        .world_mut()
        .spawn(GlobalTransform::from_translation(Vec3::new(2.4, 1.0, 0.0)))
        .id();
    let mut mesh = Mesh::from(Cuboid::new(0.3, 0.3, 0.3));
    let count = mesh.count_vertices();
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[0, 0, 0, 0]; count]),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![[1.0, 0.0, 0.0, 0.0]; count],
    );
    mesh.generate_skinned_mesh_bounds().unwrap();
    let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
    let inverse = app
        .world_mut()
        .resource_mut::<Assets<SkinnedMeshInverseBindposes>>()
        .add(vec![Mat4::IDENTITY]);
    let part = app
        .world_mut()
        .spawn((
            Mesh3d(mesh),
            PickGeometry,
            SkinnedMesh {
                inverse_bindposes: inverse,
                joints: vec![joint],
            },
            Aabb::from_min_max(-Vec3::splat(0.15), Vec3::splat(0.15)),
            GlobalTransform::IDENTITY,
            InheritedVisibility::VISIBLE,
        ))
        .id();
    app.world_mut().entity_mut(owner).add_child(part);
    let actual = screen(&mut app, Vec3::new(2.4, 1.0, 0.0));
    let stale = screen(&mut app, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(candidates(app.world_mut(), actual), vec![owner]);
    assert!(candidates(app.world_mut(), stale).is_empty());
    app.world_mut()
        .entity_mut(joint)
        .insert(GlobalTransform::from_translation(Vec3::new(-2.2, 2.0, 0.0)));
    assert!(candidates(app.world_mut(), actual).is_empty());
    let moved = screen(&mut app, Vec3::new(-2.2, 2.0, 0.0));
    assert_eq!(candidates(app.world_mut(), moved), vec![owner]);
    app.world_mut()
        .entity_mut(joint)
        .insert(GlobalTransform::from(
            Transform::from_xyz(-2.2, 2.0, 0.0).with_scale(Vec3::ZERO),
        ));
    assert!(
        candidates(app.world_mut(), moved).is_empty(),
        "an authored hidden attachment must not leave a clickable point"
    );
}

#[test]
fn real_model_parts_replace_the_root_box_and_follow_the_last_displayed_transform() {
    let mut app = app(10.0, 1.0);
    let id = actor(&mut app, crate::simulation::Team::Enemy);
    let original_radius = app.world().get::<Unit>(id).unwrap().definition().radius;
    let body = part(
        &mut app,
        id,
        Transform::from_xyz(1.8, 1.0, 0.0),
        Vec3::splat(0.15),
        true,
    );
    let actual = screen(&mut app, Vec3::new(1.8, 1.0, 0.0));
    assert_eq!(candidates(app.world_mut(), actual), vec![id]);
    assert!(
        candidates(app.world_mut(), Vec2::new(640.0, 400.0)).is_empty(),
        "a ready model cannot fall back to the empty root profile"
    );
    let moved = Vec3::new(-1.3, 3.5, 0.0);
    app.world_mut()
        .entity_mut(body)
        .insert(GlobalTransform::from_translation(moved));
    assert!(candidates(app.world_mut(), actual).is_empty());
    let current = screen(&mut app, moved);
    assert_eq!(candidates(app.world_mut(), current), vec![id]);
    assert_eq!(
        app.world().get::<Unit>(id).unwrap().definition().radius,
        original_radius
    );
}

#[test]
fn tolerance_is_six_css_pixels_at_near_far_and_high_dpi_views() {
    for distance in [10.0, 24.0] {
        for scale in [1.0, 2.0] {
            let mut app = app(distance, scale);
            let id = actor(&mut app, crate::simulation::Team::Enemy);
            part(
                &mut app,
                id,
                Transform::from_xyz(0.0, 1.0, 0.0),
                Vec3::splat(0.2),
                true,
            );
            let edge = screen(&mut app, Vec3::new(0.2, 1.0, 0.2));
            assert_eq!(
                candidates(app.world_mut(), edge + Vec2::X * 5.5),
                vec![id],
                "distance={distance} scale={scale}"
            );
            assert!(
                candidates(app.world_mut(), edge + Vec2::X * 6.5).is_empty(),
                "distance={distance} scale={scale}"
            );
        }
    }
}

#[test]
fn a_diagonal_weapon_does_not_fill_the_empty_corners_of_its_screen_rectangle() {
    let mut app = app(10.0, 1.0);
    let id = actor(&mut app, crate::simulation::Team::Enemy);
    let transform = Transform::from_xyz(0.0, 1.0, 0.0)
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4));
    part(&mut app, id, transform, Vec3::new(1.8, 0.04, 0.04), true);
    let actual = screen(&mut app, transform.transform_point(Vec3::X * 1.4));
    assert_eq!(candidates(app.world_mut(), actual), vec![id]);
    assert!(candidates(app.world_mut(), Vec2::new(690.0, 450.0)).is_empty());
}

#[test]
fn explicit_model_hits_outrank_nearby_padding_and_the_same_candidates_support_both_teams() {
    let mut app = app(10.0, 1.0);
    let ally = actor(&mut app, crate::simulation::Team::Ally);
    let enemy = actor(&mut app, crate::simulation::Team::Enemy);
    part(
        &mut app,
        ally,
        Transform::from_xyz(0.0, 1.0, 0.0),
        Vec3::splat(0.2),
        true,
    );
    part(
        &mut app,
        enemy,
        Transform::from_xyz(0.5, 1.0, -2.0),
        Vec3::splat(0.3),
        true,
    );
    let edge = screen(&mut app, Vec3::new(0.2, 1.0, 0.2));
    let hits = candidates(app.world_mut(), edge + Vec2::X * 4.0);
    assert_eq!(hits, vec![enemy, ally]);
    assert_eq!(
        hits.iter().find(
            |id| app.world().get::<Unit>(**id).unwrap().team() == crate::simulation::Team::Ally
        ),
        Some(&ally)
    );
}

#[test]
fn markers_effects_and_hidden_geometry_cannot_replace_the_actual_body() {
    let mut app = app(10.0, 1.0);
    let id = actor(&mut app, crate::simulation::Team::Enemy);
    let body = part(
        &mut app,
        id,
        Transform::from_xyz(0.0, 4.0, 0.0),
        Vec3::splat(0.2),
        true,
    );
    part(
        &mut app,
        id,
        Transform::from_xyz(0.0, 1.0, 0.0),
        Vec3::splat(0.8),
        false,
    );
    assert!(
        candidates(app.world_mut(), Vec2::new(640.0, 400.0)).is_empty(),
        "ground marker is not an airborne body"
    );
    let displayed = screen(&mut app, Vec3::new(0.0, 4.0, 0.0));
    assert_eq!(candidates(app.world_mut(), displayed), vec![id]);
    app.world_mut().entity_mut(body).insert(Visibility::Hidden);
    assert!(candidates(app.world_mut(), displayed).is_empty());
    assert!(
        candidates(app.world_mut(), Vec2::new(640.0, 400.0)).is_empty(),
        "hidden meshes cannot reactivate the fallback profile"
    );
}

#[test]
fn dead_and_despawned_actors_are_not_picked_and_invalid_views_do_not_hit() {
    let mut app = app(10.0, 1.0);
    let id = actor(&mut app, crate::simulation::Team::Enemy);
    let attacker = crate::simulation::spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Picking death source".into(),
            team: crate::simulation::Team::Ally,
            position: Vec2::ZERO,
            stats: UnitStats {
                vision_radius: 5.0,
                base_attributes: BaseAttributes {
                    attack_damage: 1000.0,
                    ..default()
                },
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(attacker, UnitCommand::AttackTarget(id));
    for _ in 0..45 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(!app.world().get::<UnitState>(id).unwrap().alive());
    assert!(candidates(app.world_mut(), Vec2::new(640.0, 400.0)).is_empty());
    app.world_mut().entity_mut(id).despawn();
    assert!(candidates(app.world_mut(), Vec2::new(640.0, 400.0)).is_empty());
    assert!(candidates(app.world_mut(), Vec2::splat(f32::NAN)).is_empty());
    let id = actor(&mut app, crate::simulation::Team::Enemy);
    assert_eq!(
        candidates(app.world_mut(), Vec2::new(640.0, 400.0)),
        vec![id]
    );
    let camera_id = app
        .world_mut()
        .query_filtered::<Entity, With<GameCamera>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<Camera>(camera_id)
        .unwrap()
        .computed
        .clip_from_view = Mat4::ZERO;
    assert!(candidates(app.world_mut(), Vec2::new(640.0, 400.0)).is_empty());
}
