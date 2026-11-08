use super::*;
use crate::client::{ability_targeting::TargetIntent, controls::Aim};
use bevy::camera::{CameraProjection, RenderTargetInfo};

fn actor(character: CharacterId, loadout: LoadoutSelection) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    crate::gameplay::prepare_world(app.world_mut());
    let actor = spawn_character(
        app.world_mut(),
        CharacterSpawnSpec {
            character,
            team: Team::Ally,
            position: Vec2::ZERO,
            loadout,
            respawn_delay: None,
        },
    )
    .unwrap();
    (app, actor)
}

fn intent(app: &App, actor: Entity, slot: usize, point: Vec2) -> TargetIntent {
    ability_targeting::resolve_intent(
        app.world(),
        actor,
        CastSource::AbilitySlot(slot),
        Some(point),
        &[],
    )
    .unwrap()
}

fn derive(app: &App, intent: &TargetIntent) -> geometry::Geometry {
    geometry::derive(
        app.world()
            .resource::<AbilityCatalog>()
            .get(intent.ability)
            .unwrap(),
        intent,
        Vec2::ZERO,
        None,
    )
}

#[test]
fn volley_preview_matches_the_seven_real_projectile_directions() {
    let (mut app, actor) = actor(CharacterId(2), LoadoutSelection::CharacterDefault);
    let intent = intent(&app, actor, 1, Vec2::X);
    let geometry = derive(&app, &intent);
    assert_eq!(geometry.shape, "volley");
    assert_eq!(geometry.rays, 7);
    assert!((geometry.angle.to_degrees() - 27.75).abs() < 0.0001);
    assert_eq!(geometry.width, Some(0.2));
    assert_eq!(
        geometry.target,
        Some(Vec2::X * 12.0),
        "a direction click is not the endpoint"
    );
    app.world_mut().resource_mut::<CommandQueue>().submit(
        actor,
        UnitCommand::CastAbility {
            slot: 1,
            target: intent.submission().unwrap(),
        },
    );
    for _ in 0..16 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let mut actual: Vec<_> = app
        .world_mut()
        .query::<&Projectile>()
        .iter(app.world())
        .map(Projectile::direction)
        .collect();
    let mut preview: Vec<_> = geometry.directions().collect();
    actual.sort_by(|a, b| a.y.total_cmp(&b.y));
    preview.sort_by(|a, b| a.y.total_cmp(&b.y));
    assert_eq!(actual.len(), preview.len());
    for (actual, preview) in actual.iter().zip(preview) {
        assert!(actual.distance(preview) < 0.00001);
    }
}

#[test]
fn crystal_arrow_uses_its_full_width_and_range_without_a_false_impact_circle() {
    let (app, actor) = actor(CharacterId(2), LoadoutSelection::CharacterDefault);
    let geometry = derive(&app, &intent(&app, actor, 3, Vec2::X));
    assert_eq!(geometry.shape, "arrow");
    assert_eq!(geometry.width, Some(2.6));
    assert_eq!(geometry.range, 250.0);
    assert_eq!(geometry.target, Some(Vec2::X * 250.0));
    assert_eq!(
        geometry.radius, None,
        "impact radius is not an aimed landing promise"
    );
}

#[test]
fn line_and_scout_keep_distinct_width_path_and_vision_geometry() {
    let (app, source) = actor(CharacterId(1), LoadoutSelection::CharacterDefault);
    let line = derive(&app, &intent(&app, source, 0, Vec2::X));
    assert_eq!(line.shape, "line");
    assert!((line.width.unwrap() - 1.36).abs() < 0.00001);
    assert!((line.target.unwrap().length() - 7.85).abs() < 0.00001);
    let (app, source) = actor(
        CharacterId(1),
        LoadoutSelection::Explicit([Some(AbilityId(23)), None, None, None]),
    );
    let intent = intent(&app, source, 0, Vec2::X * 1000.0);
    assert_eq!(
        intent.state,
        TargetState::Ready,
        "Scout is a clamped Point execution"
    );
    let scout = derive(&app, &intent);
    assert_eq!(scout.shape, "scout");
    assert_eq!(
        scout.width, None,
        "a vision path does not claim a damage width"
    );
    assert_eq!(scout.radius, Some(10.0));
    assert_eq!(scout.target, Some(Vec2::X * 250.0));
}

#[test]
fn mixed_loadout_standard_keeps_the_original_click_and_previews_the_actual_landing() {
    let (mut app, actor) = actor(
        CharacterId(2),
        LoadoutSelection::Explicit([Some(AbilityId(13)), None, None, None]),
    );
    let original = Vec2::X * 20.0;
    let intent = intent(&app, actor, 0, original);
    assert_eq!(intent.ability, AbilityId(13));
    assert_eq!(intent.original, Some(AbilityTarget::Point(original)));
    let geometry = derive(&app, &intent);
    assert_eq!(geometry.shape, "standard");
    assert_eq!(
        geometry.radius,
        Some(2.0),
        "preview the impact, not the ten-unit aura"
    );
    let landing = geometry.target.unwrap();
    assert!((landing.length() - 8.6).abs() < 0.00001);
    app.world_mut().resource_mut::<CommandQueue>().submit(
        actor,
        UnitCommand::CastAbility {
            slot: 0,
            target: intent.submission().unwrap(),
        },
    );
    app.world_mut().run_schedule(FixedUpdate);
    let object = app
        .world_mut()
        .query::<&BattlefieldObject>()
        .single(app.world())
        .unwrap();
    assert_eq!(object.position(), landing);
    assert!(!object.active(app.world().resource::<SimulationTick>().0));
}

#[test]
fn equipment_preview_uses_actual_binding_and_disappears_when_the_instance_is_removed() {
    let (mut app, actor) = actor(CharacterId(1), LoadoutSelection::CharacterDefault);
    let instance = equip_equipment(app.world_mut(), actor, 0, EquipmentId(8)).unwrap();
    let source = CastSource::EquipmentSlot { slot: 0, instance };
    let intent =
        ability_targeting::resolve_intent(app.world(), actor, source, Some(Vec2::X * 20.0), &[])
            .unwrap();
    assert_eq!(intent.ability, AbilityId(3));
    let geometry = derive(&app, &intent);
    assert_eq!(geometry.shape, "dash");
    assert!((geometry.target.unwrap().length() - 6.0).abs() < 0.00001);
    unequip_equipment(app.world_mut(), actor, 0, instance).unwrap();
    assert!(
        ability_targeting::resolve_intent(app.world(), actor, source, Some(Vec2::X), &[]).is_none()
    );
}

fn camera(point: Vec3) -> GlobalTransform {
    GlobalTransform::from(
        Transform::from_translation(point + Vec3::new(0.0, 12.0, 18.0)).looking_at(point, Vec3::Y),
    )
}

#[test]
fn reticle_reprojects_after_camera_pan_in_the_same_frame_and_clears_without_allocating() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin, SkillIndicatorsPlugin))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<PointerState>()
        .init_resource::<PlayerController>();
    crate::gameplay::prepare_world(app.world_mut());
    let handle = crate::gameplay::start_match(app.world_mut(), CharacterId(1)).unwrap();
    app.insert_resource(ClientSession::Active {
        player: handle.player,
        match_id: handle.id,
        selection_revision: 1,
    });
    let origin = app
        .world()
        .get::<UnitPose>(handle.player)
        .unwrap()
        .position();
    app.world_mut().entity_mut(handle.player).insert((
        UnitVisual,
        GlobalTransform::from_translation(Vec3::new(origin.x, 0.0, origin.y)),
    ));
    let first = Vec3::new(origin.x + 2.0, 0.0, origin.y);
    let mut projection = Camera::default();
    projection.computed.target_info = Some(RenderTargetInfo {
        physical_size: UVec2::new(1280, 800),
        scale_factor: 1.0,
    });
    projection.computed.clip_from_view = PerspectiveProjection {
        aspect_ratio: 1.6,
        ..default()
    }
    .get_clip_from_view();
    let view = app
        .world_mut()
        .spawn((GameCamera, projection.clone(), camera(first)))
        .id();
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    app.update();
    app.world_mut().resource_mut::<PlayerController>().aim = Some(Aim::Ability(2));
    *app.world_mut().resource_mut::<PointerState>() = PointerState {
        position: Some(Vec2::new(680.0, 420.0)),
        ground: Some(Vec2::splat(-999.0)),
        ..default()
    };
    app.world_mut().run_schedule(PostUpdate);
    let initial = app.world().resource::<IndicatorView>().0.clone().unwrap();
    let next_camera = camera(first + Vec3::X * 2.0);
    app.world_mut().entity_mut(view).insert(next_camera);
    app.world_mut().run_schedule(PostUpdate);
    let next = app.world().resource::<IndicatorView>().0.clone().unwrap();
    assert!((next.target.unwrap()[0] - initial.target.unwrap()[0] - 2.0).abs() < 0.0001);
    let target = Vec2::from_array(next.target.unwrap());
    let screen = projection
        .world_to_viewport(&next_camera, Vec3::new(target.x, 0.0, target.y))
        .unwrap();
    assert!(screen.distance(Vec2::new(680.0, 420.0)) < 0.001);
    assert_eq!(
        app.world().resource::<PointerState>().ground,
        Some(Vec2::splat(-999.0))
    );
    let before = app.world().resource::<Assets<Mesh>>().len();
    let custom_angle = 0.9_f32;
    let mut custom = app
        .world()
        .resource::<AbilityCatalog>()
        .get(AbilityId(22))
        .unwrap()
        .clone();
    if let AbilityExecution::Volley(spec) = &mut custom.execution {
        spec.angle = custom_angle;
        spec.count = 4;
    }
    app.world_mut()
        .resource_mut::<AbilityCatalog>()
        .register(AbilityId(501), custom)
        .unwrap();
    app.world_mut().run_schedule(Update);
    let handle = render::cached_fan(app.world(), custom_angle).unwrap();
    let mesh = app.world().resource::<Assets<Mesh>>().get(handle).unwrap();
    let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
    else {
        panic!("a prepared fan must have actual planar vertices");
    };
    let extent = positions
        .iter()
        .map(|point| point[0].abs())
        .fold(0.0_f32, f32::max);
    assert!((extent - (custom_angle * 0.5).sin()).abs() < 0.0001);
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), before + 1);
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world().resource::<Assets<Mesh>>().len(),
        before + 1,
        "a changed catalog must reuse an already prepared angle"
    );
    let meshes = app.world().resource::<Assets<Mesh>>().len();
    let materials = app.world().resource::<Assets<StandardMaterial>>().len();
    for _ in 0..50 {
        app.world_mut().run_schedule(PostUpdate);
    }
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        materials
    );
    let parts: Vec<_> = app
        .world_mut()
        .query_filtered::<(Entity, &Transform, &GlobalTransform), With<render::IndicatorPart>>()
        .iter(app.world())
        .map(|(id, local, global)| {
            assert_eq!(
                GlobalTransform::from(*local),
                *global,
                "late drawing writes its own same-frame global transform"
            );
            id
        })
        .collect();
    assert!(!parts.is_empty());
    app.world_mut().resource_mut::<PointerState>().over_ui = true;
    app.world_mut().run_schedule(PostUpdate);
    assert!(app.world().resource::<IndicatorView>().0.is_none());
    app.world_mut().resource_mut::<PointerState>().over_ui = false;
    app.world_mut().run_schedule(PostUpdate);
    assert!(app.world().resource::<IndicatorView>().0.is_some());
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    app.world_mut().run_schedule(PostUpdate);
    assert!(app.world().resource::<IndicatorView>().0.is_none());
    for id in parts {
        assert_eq!(
            *app.world().get::<Visibility>(id).unwrap(),
            Visibility::Hidden
        );
    }
}
