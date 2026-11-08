use super::*;
use crate::gameplay;

fn arena(character: CharacterId) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin))
        .init_resource::<Runtime>()
        .add_systems(FixedUpdate, capture::capture.after(SimulationSet::Step));
    gameplay::prepare_world(app.world_mut());
    let handle = gameplay::start_match(app.world_mut(), character).unwrap();
    app.insert_resource(ClientSession::Active {
        player: handle.player,
        match_id: handle.id,
        selection_revision: 1,
    });
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    (app, handle.player)
}
fn step(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
fn cast(app: &mut App, player: Entity, slot: usize, target: AbilityTarget) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(player, UnitCommand::CastAbility { slot, target });
}

#[test]
fn committed_facts_are_captured_once_and_cancelled_windup_creates_no_cast_effect() {
    let (mut app, player) = arena(CharacterId(1));
    cast(&mut app, player, 0, AbilityTarget::Direction(Vec2::X));
    step(&mut app, 1);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(player, UnitCommand::Stop);
    step(&mut app, 30);
    assert!(!app.world().resource::<Runtime>().pending.iter()
        .any(|fact| fact.ability == AbilityId(11) && matches!(fact.kind, FactKind::Cast { .. })));
    cast(&mut app, player, 1, AbilityTarget::SelfCast);
    step(&mut app, 10);
    assert_eq!(
        app.world()
            .resource::<Runtime>()
            .pending
            .iter()
            .filter(
                |fact| fact.ability == AbilityId(12) && matches!(fact.kind, FactKind::Cast { .. })
            )
            .count(),
        1
    );
}

#[test]
fn unseen_cast_does_not_leak_but_its_real_arrow_can_end_in_our_visible_area() {
    let (mut app, player) = arena(CharacterId(1));
    let enemy = app
        .world()
        .resource::<gameplay::MatchState>()
        .hero(Team::Enemy)
        .unwrap();
    let start = app.world().get::<UnitPose>(enemy).unwrap().position();
    let target = app.world().get::<UnitPose>(player).unwrap().position();
    assert!(!team_can_see(app.world(), Team::Ally, enemy));
    cast(&mut app, enemy, 3, AbilityTarget::Direction(target - start));
    step(&mut app, 200);
    let pending = &app.world().resource::<Runtime>().pending;
    assert!(!pending.iter().any(|fact| fact.ability == AbilityId(24) && matches!(fact.kind, FactKind::Cast { .. })));
    assert!(pending.iter().any(|fact| fact.ability == AbilityId(24)
        && matches!(
            fact.kind,
            FactKind::Impact(ImpactKind::ProjectileEnded { .. })
        )));
}

#[test]
fn scout_arrival_is_retained_across_missing_display_frames_pause_and_reset() {
    let (mut app, player) = arena(CharacterId(2));
    let destination = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X;
    cast(&mut app, player, 2, AbilityTarget::Point(destination));
    step(&mut app, 60);
    let runtime = app.world().resource::<Runtime>();
    assert_eq!(
        runtime
            .pending
            .iter()
            .filter(|fact| matches!(fact.kind, FactKind::ScoutArrived))
            .count(),
        1
    );
    assert!(runtime.arrivals.values().any(|arrival| *arrival));
    let count = runtime.pending.len();
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    step(&mut app, 20);
    assert_eq!(app.world().resource::<Runtime>().pending.len(), count);
    gameplay::reset_match(app.world_mut());
    step(&mut app, 1);
    assert!(app.world().resource::<Runtime>().pending.is_empty());
    assert!(app.world().resource::<Runtime>().arrivals.is_empty());
}

use crate::client::{
    presentation,
    scene::UnitVisual,
    skill_assets::{MeshKey, TextureKey, mesh_spec, texture_spec},
    visibility,
};
use bevy::{asset::AssetId, math::Affine2};

/// Only the display adapter is replaced: commands, carriers, timers, lives,
/// observation and effect systems below are production implementations.
fn install_renderer(app: &mut App) -> usize {
    app.init_resource::<Assets<Image>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<presentation::PresentationFrame>()
        .add_systems(Startup, drawing::setup)
        .add_systems(
            Update,
            (
                visibility::refresh,
                presentation::prepare,
                displayed_transforms,
                render::update,
            )
                .chain(),
        );
    app.world_mut().run_schedule(Startup);
    let assets = SkillAssets::fixture(app.world_mut());
    app.insert_resource(assets);
    app.world().resource::<Assets<StandardMaterial>>().len()
}

fn displayed_transforms(world: &mut World) {
    if !world
        .resource::<presentation::PresentationFrame>()
        .should_sync()
    {
        return;
    }
    let units: Vec<_> = world
        .query::<(Entity, &UnitPose)>()
        .iter(world)
        .map(|(id, pose)| (id, pose.position(), pose.facing()))
        .collect();
    for (id, point, direction) in units {
        let visible = world.resource::<visibility::VisibleWorld>().contains(id);
        world.entity_mut(id).insert((
            UnitVisual,
            Transform::from_xyz(point.x, 0.0, point.y)
                .with_rotation(Quat::from_rotation_y((-direction.x).atan2(-direction.y))),
            if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ));
    }
    let carriers: Vec<_> = world
        .query::<(Entity, &Projectile)>()
        .iter(world)
        .map(|(id, projectile)| (id, projectile.position(), projectile.direction()))
        .collect();
    for (id, point, direction) in carriers {
        let visible = world.resource::<visibility::VisibleWorld>().contains(id);
        world.entity_mut(id).insert((
            Transform::from_xyz(point.x, 0.9, point.y)
                .with_rotation(Quat::from_rotation_y((-direction.x).atan2(-direction.y))),
            if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ));
    }
    let objects: Vec<_> = world
        .query::<(Entity, &BattlefieldObject)>()
        .iter(world)
        .map(|(id, object)| {
            (
                id,
                object.position(),
                object.active(world.resource::<SimulationTick>().0),
            )
        })
        .collect();
    for (id, point, active) in objects {
        let visible = active && world.resource::<visibility::VisibleWorld>().contains(id);
        world.entity_mut(id).insert((
            Transform::from_xyz(point.x, 0.0, point.y),
            if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ));
    }
}

#[derive(Debug, PartialEq)]
struct RenderSnapshot {
    transforms: HashMap<Entity, Transform>,
    materials: HashMap<AssetId<StandardMaterial>, (Color, AlphaMode, Affine2)>,
}
fn snapshot(world: &mut World) -> RenderSnapshot {
    RenderSnapshot {
        transforms: world
            .query_filtered::<(Entity, &Transform), With<ManagedEffect>>()
            .iter(world)
            .map(|(id, transform)| (id, *transform))
            .collect(),
        materials: world
            .resource::<Assets<StandardMaterial>>()
            .iter()
            .map(|(id, material)| {
                (
                    id,
                    (
                        material.base_color,
                        material.alpha_mode,
                        material.uv_transform,
                    ),
                )
            })
            .collect(),
    }
}
fn managed_count(world: &mut World) -> usize {
    world
        .query_filtered::<Entity, With<ManagedEffect>>()
        .iter(world)
        .count()
}
fn target(world: &mut World, point: Vec2) -> Entity {
    spawn_unit(
        world,
        UnitSpec {
            name: "Durable visual target".into(),
            team: Team::Enemy,
            position: point,
            stats: UnitStats {
                kind: UnitKind::Hero,
                max_health: 50_000.0,
                vision_radius: 12.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap()
}
fn champion(world: &mut World, id: CharacterId, point: Vec2) -> Entity {
    spawn_character(
        world,
        CharacterSpawnSpec {
            character: id,
            team: Team::Ally,
            position: point,
            loadout: LoadoutSelection::CharacterDefault,
            respawn_delay: None,
        },
    )
    .unwrap()
}
fn wait_projectile(app: &mut App, source: Entity, arrows: u8) -> Entity {
    for _ in 0..120 {
        step(app, 1);
        let world = app.world_mut();
        if let Some(id) =
            world
                .query::<(Entity, &Projectile)>()
                .iter(world)
                .find_map(|(id, projectile)| {
                    (projectile.source_life().entity() == source && projectile.arrows() == arrows)
                        .then_some(id)
                })
        {
            return id;
        }
    }
    panic!("real attack did not release the expected projectile");
}

#[test]
fn real_shield_visual_freezes_on_pause_and_releases_materials_on_expiry_and_new_session() {
    let (mut app, player) = arena(CharacterId(1));
    let baseline = install_renderer(&mut app);
    cast(&mut app, player, 1, AbilityTarget::SelfCast);
    step(&mut app, 1);
    app.world_mut().run_schedule(Update);
    assert!(
        app.world()
            .resource::<SkillEffectsView>()
            .active
            .iter()
            .any(|effect| effect.phase == "shield")
    );
    assert!(app.world().resource::<Assets<StandardMaterial>>().len() > baseline);
    let frozen = snapshot(app.world_mut());
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    for _ in 0..12 {
        step(&mut app, 1);
        app.world_mut().run_schedule(Update);
    }
    assert_eq!(snapshot(app.world_mut()), frozen);
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 300);
    app.world_mut().run_schedule(Update);
    assert_eq!(managed_count(app.world_mut()), 0);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
    step(&mut app, 250);
    cast(&mut app, player, 1, AbilityTarget::SelfCast);
    step(&mut app, 1);
    app.world_mut().run_schedule(Update);
    assert!(managed_count(app.world_mut()) > 0);
    gameplay::reset_match(app.world_mut());
    app.world_mut().remove_resource::<ClientSession>();
    app.world_mut().run_schedule(Update);
    assert_eq!(managed_count(app.world_mut()), 0);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
    assert!(app.world().resource::<Runtime>().pending.is_empty());
    assert!(app.world().resource::<SkillEffectsView>().active.is_empty());
}

#[test]
fn ordinary_arrow_keeps_observed_art_after_source_removal_and_cleans_it_when_carrier_ends() {
    let (mut app, player) = arena(CharacterId(1));
    let start = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X * 2.0;
    let source = champion(app.world_mut(), CharacterId(2), start);
    let victim = target(app.world_mut(), start + Vec2::X * 5.0);
    let baseline = install_renderer(&mut app);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(source, UnitCommand::AttackTarget(victim));
    let carrier = wait_projectile(&mut app, source, 1);
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world()
            .resource::<Runtime>()
            .projectile_art
            .get(&carrier),
        Some(&AbilityId(0))
    );
    let materials = app.world().resource::<Assets<StandardMaterial>>().len();
    assert!(materials > baseline);
    app.world_mut().despawn(source);
    app.world_mut().run_schedule(Update);
    assert!(
        app.world()
            .resource::<Runtime>()
            .live
            .contains_key(&render::LiveKey::Carrier(carrier))
    );
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        materials
    );
    step(&mut app, 100);
    app.world_mut().run_schedule(Update);
    assert!(app.world().get::<UnitState>(victim).unwrap().health() < 50_000.0);
    assert!(app.world().get::<Projectile>(carrier).is_none());
    assert!(app.world().resource::<Runtime>().projectile_art.is_empty());
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
}

#[test]
fn real_empowered_attack_uses_its_frozen_origin_after_source_is_gone() {
    let (mut app, player) = arena(CharacterId(1));
    let start = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X * 2.0;
    let source = champion(app.world_mut(), CharacterId(2), start);
    let victim = target(app.world_mut(), start + Vec2::X * 5.0);
    let baseline = install_renderer(&mut app);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(source, UnitCommand::AttackTarget(victim));
    for _ in 0..600 {
        step(&mut app, 1);
        if app.world().get::<UnitMechanics>(source).unwrap().focus() >= 4 {
            break;
        }
    }
    assert_eq!(app.world().get::<UnitMechanics>(source).unwrap().focus(), 4);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(source, UnitCommand::Stop);
    step(&mut app, 1);
    cast(&mut app, source, 0, AbilityTarget::SelfCast);
    step(&mut app, 1);
    assert!(
        app.world()
            .get::<UnitMechanics>(source)
            .unwrap()
            .empowered()
    );
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(source, UnitCommand::AttackTarget(victim));
    let carrier = wait_projectile(&mut app, source, 5);
    assert_eq!(
        app.world().get::<Projectile>(carrier).unwrap().origin(),
        DamageOrigin::BasicAttack
    );
    assert_eq!(
        app.world()
            .get::<Projectile>(carrier)
            .unwrap()
            .empower_origin(),
        Some(DamageOrigin::Ability(AbilityId(21)))
    );
    // First display happens after deletion: a live source cannot supply Q state.
    app.world_mut().despawn(source);
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world()
            .resource::<Runtime>()
            .projectile_art
            .get(&carrier),
        Some(&AbilityId(21))
    );
    let visual = app
        .world()
        .resource::<SkillEffectsView>()
        .active
        .iter()
        .find(|effect| effect.phase == "projectile" && effect.ability_id == 21)
        .unwrap();
    assert_eq!(visual.layers, 10);
    step(&mut app, 100);
    app.world_mut().run_schedule(Update);
    assert!(app.world().get::<Projectile>(carrier).is_none());
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
}

#[test]
fn source_materials_are_immutable_and_fading_meshes_use_transparent_shadow_free_instances() {
    let (mut app, player) = arena(CharacterId(1));
    install_renderer(&mut app);
    let shared = app
        .world()
        .resource::<SkillAssets>()
        .material(mesh_spec(MeshKey::JarvanImpactGround).texture)
        .unwrap()
        .clone();
    assert_eq!(
        app.world()
            .resource::<Assets<StandardMaterial>>()
            .get(&shared)
            .unwrap()
            .alpha_mode,
        AlphaMode::Opaque
    );
    let group = drawing::EffectGroup::spawn(
        app.world_mut(),
        None,
        layers::burst_layers(recipes::Shape::GroundImpact { radius: 3.5 }),
    )
    .unwrap();
    group.fade(app.world_mut(), 0.4);
    let root = group.root();
    let children: Vec<_> = app.world().get::<Children>(root).unwrap().iter().collect();
    for child in children {
        let material = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(child)
            .unwrap();
        let material = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(&material.0)
            .unwrap();
        assert_ne!(material.alpha_mode, AlphaMode::Opaque);
        assert_eq!(material.base_color.alpha(), 0.4);
        assert!(
            app.world()
                .get::<bevy::light::NotShadowCaster>(child)
                .is_some()
        );
        assert!(
            app.world()
                .get::<bevy::light::NotShadowReceiver>(child)
                .is_some()
        );
    }
    assert_eq!(
        app.world()
            .resource::<Assets<StandardMaterial>>()
            .get(&shared)
            .unwrap()
            .alpha_mode,
        AlphaMode::Opaque
    );
    group.dispose(app.world_mut());
    assert_eq!(managed_count(app.world_mut()), 0);
    // The original flag remains opaque until the real Standard activates.
    let group = drawing::EffectGroup::spawn(
        app.world_mut(),
        None,
        layers::burst_layers(recipes::Shape::StandardDrop),
    )
    .unwrap();
    let child = app.world().get::<Children>(group.root()).unwrap()[0];
    let handle = &app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(child)
        .unwrap()
        .0;
    assert_eq!(
        app.world()
            .resource::<Assets<StandardMaterial>>()
            .get(handle)
            .unwrap()
            .alpha_mode,
        texture_spec(TextureKey::JarvanStandardAtlas).blend.into()
    );
    group.dispose(app.world_mut());
    assert!(app.world().get::<Unit>(player).is_some());
}

#[test]
fn real_leap_recast_collapses_the_existing_rocks_without_repeating_the_landing() {
    let (mut app, player) = arena(CharacterId(1));
    let start = app.world().get::<UnitPose>(player).unwrap().position();
    let victim = target(app.world_mut(), start + Vec2::X * 5.0);
    let baseline = install_renderer(&mut app);
    cast(&mut app, player, 3, AbilityTarget::Unit(victim));
    step(&mut app, 1);
    app.world_mut().run_schedule(Update);
    step(&mut app, 25);
    app.world_mut().run_schedule(Update);
    assert!(
        app.world()
            .resource::<SkillEffectsView>()
            .active
            .iter()
            .any(|effect| effect.ability_id == 14
                && effect.phase == "terrain"
                && effect.layers == 24)
    );
    step(&mut app, 60);
    app.world_mut().run_schedule(Update);
    assert!(
        app.world()
            .get::<UnitMechanics>(player)
            .unwrap()
            .recast_ready(3, app.world().resource::<SimulationTick>().0)
    );
    let before = app
        .world()
        .resource::<Runtime>()
        .emitted
        .get(&14)
        .copied()
        .unwrap();
    cast(&mut app, player, 3, AbilityTarget::SelfCast);
    step(&mut app, 1);
    let pending = &app.world().resource::<Runtime>().pending;
    assert!(
        pending
            .iter()
            .any(|fact| matches!(fact.kind, FactKind::Cast { recast: true, .. }))
    );
    assert!(!pending.iter().any(|fact| matches!(
        fact.kind,
        FactKind::Impact(ImpactKind::AreaActivated { .. })
    )));
    app.world_mut().run_schedule(Update);
    let view = app.world().resource::<SkillEffectsView>();
    assert_eq!(view.emitted.get(&14), Some(&(before + 1)));
    let remaining: Vec<_> = view
        .active
        .iter()
        .filter(|effect| effect.ability_id == 14)
        .collect();
    assert_eq!(remaining.len(), 1);
    assert_eq!((remaining[0].phase, remaining[0].layers), ("burst", 24));
    assert!(leap_progress(app.world(), player, 0.0).is_none());
    step(&mut app, 30);
    app.world_mut().run_schedule(Update);
    assert_eq!(managed_count(app.world_mut()), 0);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
}

#[test]
fn many_real_commits_bound_pending_and_burst_storage_without_growing_materials_per_frame() {
    let (mut app, player) = arena(CharacterId(1));
    let point = app.world().get::<UnitPose>(player).unwrap().position();
    let baseline = install_renderer(&mut app);
    for _ in 0..110 {
        let caster = spawn_unit(
            app.world_mut(),
            UnitSpec {
                name: "Concurrent W fixture".into(),
                team: Team::Ally,
                position: point,
                stats: UnitStats {
                    kind: UnitKind::Hero,
                    ..default()
                },
                abilities: [Some(AbilityId(12)), None, None, None],
                respawn_delay: None,
            },
        )
        .unwrap();
        cast(&mut app, caster, 0, AbilityTarget::SelfCast);
    }
    step(&mut app, 1);
    assert_eq!(app.world().resource::<Runtime>().pending.len(), 192);
    app.world_mut().run_schedule(Update);
    assert_eq!(app.world().resource::<Runtime>().bursts.len(), 96);
    assert_eq!(app.world().resource::<Runtime>().live.len(), 110);
    let materials = app.world().resource::<Assets<StandardMaterial>>().len();
    assert_eq!(materials, baseline + 96 + 110);
    for _ in 0..20 {
        app.world_mut().run_schedule(Update);
    }
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        materials
    );
    gameplay::reset_match(app.world_mut());
    app.world_mut().remove_resource::<ClientSession>();
    app.world_mut().run_schedule(Update);
    assert_eq!(managed_count(app.world_mut()), 0);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
}

#[test]
fn a_real_terrain_removal_that_expired_between_display_frames_is_not_replayed() {
    let (mut app, player) = arena(CharacterId(1));
    let start = app.world().get::<UnitPose>(player).unwrap().position();
    let victim = target(app.world_mut(), start + Vec2::X * 5.0);
    let baseline = install_renderer(&mut app);
    cast(&mut app, player, 3, AbilityTarget::Unit(victim));
    step(&mut app, 26);
    app.world_mut().run_schedule(Update);
    assert!(
        app.world()
            .resource::<SkillEffectsView>()
            .active
            .iter()
            .any(|effect| effect.ability_id == 14 && effect.phase == "terrain")
    );
    step(&mut app, 60);
    app.world_mut().run_schedule(Update);
    let before = app
        .world()
        .resource::<Runtime>()
        .emitted
        .get(&14)
        .copied()
        .unwrap();
    cast(&mut app, player, 3, AbilityTarget::SelfCast);
    step(&mut app, 1);
    let has_terrain = {
        let world = app.world_mut();
        world
            .query::<&BattlefieldObject>()
            .iter(world)
            .any(|object| object.kind() == ObjectKind::Terrain)
    };
    assert!(!has_terrain);
    // Thirty authoritative ticks pass without any presentation update. The
    // removal's 0.35s flash is already over when drawing next catches up.
    step(&mut app, 30);
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world().resource::<SkillEffectsView>().emitted.get(&14),
        Some(&before)
    );
    assert!(app.world().resource::<SkillEffectsView>().active.is_empty());
    assert_eq!(managed_count(app.world_mut()), 0);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
}

#[test]
fn original_shield_ownership_requires_a_successful_group_and_cleanup_survives_carrier_despawn() {
    let (mut app, player) = arena(CharacterId(1));
    let baseline = install_renderer(&mut app);
    let handle = app
        .world()
        .resource::<SkillAssets>()
        .material(TextureKey::JarvanShield)
        .unwrap()
        .clone();
    // A rendering resource failure must not claim ownership of the real shield.
    let template = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .remove(handle.id())
        .unwrap();
    cast(&mut app, player, 1, AbilityTarget::SelfCast);
    step(&mut app, 1);
    app.world_mut().run_schedule(Update);
    assert!(app.world().get::<UnitMechanics>(player).unwrap().shield() > 0.0);
    let claims = {
        let world = app.world_mut();
        world.query::<&OriginalShield>().iter(world).count()
    };
    assert_eq!(claims, 0);
    assert!(
        !app.world()
            .resource::<SkillEffectsView>()
            .active
            .iter()
            .any(|effect| effect.phase == "shield")
    );
    app.world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .insert(handle.id(), template)
        .unwrap();
    app.world_mut().run_schedule(Update);
    let owners: Vec<_> = {
        let world = app.world_mut();
        world
            .query::<&OriginalShield>()
            .iter(world)
            .map(|owner| owner.0)
            .collect()
    };
    assert_eq!(owners, [player]);
    assert!(
        app.world()
            .resource::<SkillEffectsView>()
            .active
            .iter()
            .any(|effect| effect.phase == "shield")
    );
    // The authoritative carrier removes its linked visual before the runtime
    // gets its next update. The disposer must still free the owned materials.
    app.world_mut().despawn(player);
    app.world_mut().run_schedule(Update);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
    assert_eq!(managed_count(app.world_mut()), 0);
}

#[test]
fn real_nova_uses_source_radial_uv_rotation_and_alpha_in_its_own_material() {
    let (mut app, player) = arena(CharacterId(1));
    install_renderer(&mut app);
    let mesh = app
        .world()
        .resource::<SkillAssets>()
        .mesh(MeshKey::JarvanNovaRing)
        .unwrap()
        .clone();
    let template = app
        .world()
        .resource::<SkillAssets>()
        .material(TextureKey::JarvanNovaRing)
        .unwrap()
        .clone();
    let original = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(&template)
        .unwrap()
        .clone();
    cast(&mut app, player, 1, AbilityTarget::SelfCast);
    step(&mut app, 1);
    app.world_mut().run_schedule(Update);
    let handle = {
        let world = app.world_mut();
        world
            .query::<(&Mesh3d, &MeshMaterial3d<StandardMaterial>)>()
            .iter(world)
            .find_map(|(drawn, material)| (drawn.0 == mesh).then(|| material.0.clone()))
            .unwrap()
    };
    assert_ne!(handle, template);
    let material = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(&handle)
        .unwrap();
    assert_eq!(material.alpha_mode, AlphaMode::Add);
    assert!(
        material
            .uv_transform
            .transform_point2(Vec2::new(0.0, 0.25))
            .abs_diff_eq(Vec2::new(0.75, 0.0), 1e-6)
    );
    assert!(
        material
            .uv_transform
            .transform_point2(Vec2::new(1.0, 0.75))
            .abs_diff_eq(Vec2::new(0.25, 1.0), 1e-6)
    );
    assert!((material.base_color.alpha() - 0.1569 * 0.9).abs() < 1e-6);
    let unchanged = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(&template)
        .unwrap();
    assert_eq!(unchanged.uv_transform, original.uv_transform);
    assert_eq!(unchanged.base_color, original.base_color);
}

#[test]
fn real_shield_keeps_its_source_anchor_and_puts_the_surface_in_front_of_the_body() {
    use crate::client::camera;
    let (mut app, player) = arena(CharacterId(1));
    let start = app.world().get::<UnitPose>(player).unwrap().position();
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(player, UnitCommand::MoveTo(start + Vec2::X * 2.0));
    step(&mut app, 1);
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(player, UnitCommand::Stop);
    step(&mut app, 1);
    app.add_plugins(TransformPlugin)
        .init_resource::<camera::CameraRig>()
        .add_systems(Startup, camera::setup);
    install_renderer(&mut app);
    cast(&mut app, player, 1, AbilityTarget::SelfCast);
    step(&mut app, 1);
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(PostUpdate);
    let root = {
        let world = app.world_mut();
        world
            .query::<(Entity, &OriginalShield)>()
            .iter(world)
            .find_map(|(id, owner)| (owner.0 == player).then_some(id))
            .unwrap()
    };
    let child = app.world().get::<Children>(root).unwrap()[0];
    let owner = app.world().get::<GlobalTransform>(player).unwrap();
    let anchor = app.world().get::<GlobalTransform>(root).unwrap();
    let plane = app.world().get::<GlobalTransform>(child).unwrap();
    assert!((anchor.translation() - owner.translation()).abs_diff_eq(Vec3::Y * 0.8, 1e-5));
    let surface_offset = plane.translation() - anchor.translation();
    let normal = plane.affine().transform_vector3(Vec3::Z).normalize();
    let material = &app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(child)
        .unwrap()
        .0;
    assert_eq!(
        app.world()
            .resource::<Assets<StandardMaterial>>()
            .get(material)
            .unwrap()
            .depth_bias,
        0.0
    );
    let camera_normal = {
        let world = app.world_mut();
        world
            .query_filtered::<&Transform, With<camera::GameCamera>>()
            .single(world)
            .unwrap()
            .rotation
            * Vec3::Z
    };
    assert!(normal.abs_diff_eq(camera_normal, 1e-5));
    assert!(surface_offset.abs_diff_eq(camera_normal * 0.5, 1e-5));
}

#[test]
fn real_scout_phase_replacement_keeps_the_drone_until_scan_material_is_available() {
    let (mut app, player) = arena(CharacterId(2));
    let baseline = install_renderer(&mut app);
    let destination = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X * 2.0;
    cast(&mut app, player, 2, AbilityTarget::Point(destination));
    let mut carrier = None;
    for _ in 0..60 {
        step(&mut app, 1);
        let world = app.world_mut();
        carrier = world
            .query::<(Entity, &BattlefieldObject)>()
            .iter(world)
            .find_map(|(id, object)| {
                (object.owner().entity() == player && object.kind() == ObjectKind::Scout)
                    .then_some(id)
            });
        if carrier.is_some() {
            break;
        }
    }
    let carrier = carrier.expect("the real E cast must launch its Scout");
    app.world_mut().run_schedule(Update);
    let old_root = app.world().get::<Children>(carrier).unwrap()[0];
    let old_part = app.world().get::<Children>(old_root).unwrap()[0];
    let old_material = app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(old_part)
        .unwrap()
        .0
        .clone();
    assert!(
        app.world()
            .resource::<Runtime>()
            .live
            .contains_key(&render::LiveKey::Carrier(carrier))
    );
    let scan = app
        .world()
        .resource::<SkillAssets>()
        .material(TextureKey::AsheScanRing)
        .unwrap()
        .clone();
    let template = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .remove(scan.id())
        .unwrap();
    for _ in 0..60 {
        if app
            .world()
            .get::<BattlefieldObject>(carrier)
            .unwrap()
            .kind()
            == ObjectKind::Reveal
        {
            break;
        }
        step(&mut app, 1);
    }
    assert_eq!(
        app.world()
            .get::<BattlefieldObject>(carrier)
            .unwrap()
            .kind(),
        ObjectKind::Reveal
    );
    let materials = app.world().resource::<Assets<StandardMaterial>>().len();
    for _ in 0..4 {
        app.world_mut().run_schedule(Update);
        assert!(
            app.world().entities().contains(old_root),
            "failed replacement must preserve the existing drone"
        );
        assert!(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&old_material)
                .is_some()
        );
        assert_eq!(
            app.world().resource::<Assets<StandardMaterial>>().len(),
            materials,
            "failed group creation cannot leak materials"
        );
        assert!(
            app.world()
                .resource::<SkillEffectsView>()
                .active
                .iter()
                .any(|effect| effect.ability_id == 23 && effect.phase == "scout")
        );
    }
    app.world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .insert(scan.id(), template)
        .unwrap();
    app.world_mut().run_schedule(Update);
    assert!(!app.world().entities().contains(old_root));
    assert!(
        app.world()
            .resource::<Assets<StandardMaterial>>()
            .get(&old_material)
            .is_none()
    );
    let effects = &app.world().resource::<SkillEffectsView>().active;
    assert!(
        effects
            .iter()
            .any(|effect| effect.ability_id == 23 && effect.phase == "reveal")
    );
    assert!(!effects.iter().any(|effect| effect.phase == "scout"));
    let layers: usize = effects.iter().map(|effect| effect.layers).sum();
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline + layers
    );
    step(&mut app, 400);
    app.world_mut().run_schedule(Update);
    assert!(app.world().get::<BattlefieldObject>(carrier).is_none());
    assert_eq!(managed_count(app.world_mut()), 0);
    assert_eq!(
        app.world().resource::<Assets<StandardMaterial>>().len(),
        baseline
    );
}
