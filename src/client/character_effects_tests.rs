use super::*;
use bevy::ecs::system::RunSystemOnce;
use std::time::Duration;

fn fixture() -> (World, Handle<StandardMaterial>, Handle<StandardMaterial>) {
    let mut world = World::new();
    world.init_resource::<Assets<StandardMaterial>>();
    let ember = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            emissive: LinearRgba::new(5.0, 2.0, 0.3, 0.8),
            uv_transform: Affine2::from_scale_angle_translation(
                Vec2::new(0.8, 1.2),
                0.2,
                Vec2::new(0.1, 0.3),
            ),
            ..default()
        });
    let cloth = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    register(&mut world, &ember, SurfaceEffect::Ember);
    register(&mut world, &cloth, SurfaceEffect::None);
    (world, ember, cloth)
}

fn state(world: &World, handle: &Handle<StandardMaterial>) -> (Affine2, LinearRgba) {
    let material = world
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .unwrap();
    (material.uv_transform, material.emissive)
}

#[test]
fn absolute_flow_preserves_baselines_and_seek_is_deterministic() {
    let (mut world, fire, cloth) = fixture();
    let base = state(&world, &fire);
    let opaque = state(&world, &cloth);
    apply_at(&mut world, 0.5);
    let first = state(&world, &fire);
    assert_eq!(
        first.0,
        base.0 * Affine2::from_translation(Vec2::new(0.0, 0.25))
    );
    assert_ne!(first.1, base.1);
    apply_at(&mut world, 2.7);
    assert_ne!(state(&world, &fire), first);
    apply_at(&mut world, 0.5);
    assert_eq!(state(&world, &fire), first);
    apply_at(&mut world, 0.5);
    assert_eq!(state(&world, &fire), first);
    assert_eq!(state(&world, &cloth), opaque);
    assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), 2);
    for invalid in [f64::NAN, f64::INFINITY, -1.0] {
        apply_at(&mut world, invalid);
        assert_eq!(state(&world, &fire), first);
    }
}

#[test]
fn repeated_cached_registration_never_recaptures_animated_values() {
    let (mut world, fire, _) = fixture();
    apply_at(&mut world, 0.5);
    let expected = state(&world, &fire);
    for _ in 0..8 {
        register(&mut world, &fire, SurfaceEffect::Ember);
        apply_at(&mut world, 1.0);
        apply_at(&mut world, 0.5);
        assert_eq!(state(&world, &fire), expected);
    }
    assert_eq!(world.resource::<CharacterEffects>().materials.len(), 1);
    assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), 2);
}

#[test]
fn game_pause_freezes_clock_and_resume_uses_only_current_delta() {
    let (mut world, fire, _) = fixture();
    world.insert_resource(Time::<()>::default());
    world.insert_resource(SimulationActive(true));
    world
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(0.25));
    world.run_system_once(update_game).unwrap();
    let before = state(&world, &fire);
    world.resource_mut::<SimulationActive>().0 = false;
    for _ in 0..3 {
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(2));
        world.run_system_once(update_game).unwrap();
    }
    assert_eq!(world.resource::<CharacterEffects>().game_seconds, 0.25);
    assert_eq!(state(&world, &fire), before);
    world.resource_mut::<SimulationActive>().0 = true;
    world
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(0.125));
    world.run_system_once(update_game).unwrap();
    assert_eq!(world.resource::<CharacterEffects>().game_seconds, 0.375);
    let resumed = state(&world, &fire);
    apply_at(&mut world, 0.375);
    assert_eq!(state(&world, &fire), resumed);
}

#[test]
#[should_panic(expected = "invalid character surface effect 2")]
fn unknown_tags_are_rejected() {
    SurfaceEffect::from_byte(2);
}

#[test]
#[should_panic(expected = "ember surfaces require alpha blending")]
fn opaque_cloth_cannot_be_tagged_as_fire() {
    let (mut world, _, cloth) = fixture();
    register(&mut world, &cloth, SurfaceEffect::Ember);
}

#[test]
fn cached_feedback_variants_keep_their_own_emission_and_share_flow() {
    let (mut world, fire, _) = fixture();
    let base = state(&world, &fire);
    let mut variant = world
        .resource::<Assets<StandardMaterial>>()
        .get(&fire)
        .unwrap()
        .clone();
    variant.emissive = LinearRgba::new(8.0, 4.0, 1.0, 0.6);
    let feedback = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(variant);
    register(&mut world, &feedback, SurfaceEffect::Ember);
    for seconds in [0.125, 2.1, 7.0, 0.125] {
        apply_at(&mut world, seconds);
        let normal = state(&world, &fire);
        let hit = state(&world, &feedback);
        assert_eq!(normal.0, hit.0);
        let gain = normal.1.red / base.1.red;
        assert!((hit.1.red - 8.0 * gain).abs() < 0.00001);
        assert_eq!(hit.1.alpha, 0.6);
    }
    assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), 3);
    assert_eq!(world.resource::<CharacterEffects>().materials.len(), 2);
}

#[test]
fn only_tagged_fire_images_repeat_v_and_preserve_filtering() {
    use bevy::image::ImageFilterMode;
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<StandardMaterial>>();
    let authored = ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        ..ImageSamplerDescriptor::linear()
    };
    let base = world.resource_mut::<Assets<Image>>().add(Image {
        sampler: ImageSampler::Descriptor(authored.clone()),
        ..default()
    });
    let emission = world.resource_mut::<Assets<Image>>().add(Image::default());
    let cloth_image = world.resource_mut::<Assets<Image>>().add(Image::default());
    let fire = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            base_color_texture: Some(base.clone()),
            emissive_texture: Some(emission.clone()),
            ..default()
        });
    let cloth = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: Some(cloth_image.clone()),
            ..default()
        });
    register(&mut world, &fire, SurfaceEffect::Ember);
    register(&mut world, &cloth, SurfaceEffect::None);
    let images = world.resource::<Assets<Image>>();
    for handle in [&base, &emission] {
        let ImageSampler::Descriptor(sampler) = &images.get(handle).unwrap().sampler else {
            panic!("explicit fire sampler")
        };
        assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
        assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
    }
    let ImageSampler::Descriptor(sampler) = &images.get(&base).unwrap().sampler else {
        unreachable!()
    };
    assert_eq!(sampler.mag_filter, authored.mag_filter);
    assert_eq!(sampler.min_filter, authored.min_filter);
    assert_eq!(sampler.mipmap_filter, authored.mipmap_filter);
    assert!(matches!(
        images.get(&cloth_image).unwrap().sampler,
        ImageSampler::Default
    ));
    assert_eq!(images.len(), 3);
}
