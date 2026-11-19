//! Platform scenery and its single planar reflection in the normal action scene.
mod mirror;
mod paint;
mod platform;
pub(super) mod qingluan;

use crate::gameplay::station::LAYOUT;
use bevy::{
    asset::RenderAssetUsages,
    camera::{CameraUpdateSystems, visibility::RenderLayers},
    core_pipeline::prepass::{DeferredPrepass, DepthPrepass},
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    light::CascadeShadowConfigBuilder,
    pbr::{DefaultOpaqueRendererMethod, ScreenSpaceReflections},
    prelude::*,
    transform::TransformSystems,
};

const SKY: &str = "station/landscape.png";
const PLATFORM: &str = "station/platform.png";
const CARRIAGE: &str = "station/carriage.png";
const PIGMENT: &str = "station/pigment.png";
pub(super) const TEXTURES: [&str; 4] = [SKY, PLATFORM, CARRIAGE, PIGMENT];
const WATER_LAYER: usize = 1;
const SISTER_LAYER: usize = 2;

pub(super) fn configure_effects(app: &mut App) {
    // The game finishes its camera orbit, collision and recoil in Update.
    // Publish this frame's reflected pose before either camera is extracted.
    mirror::configure_water(app);
    app.add_systems(
        PostUpdate,
        mirror::update
            .before(CameraUpdateSystems)
            .before(TransformSystems::Propagate)
            .run_if(resource_exists::<super::Assembled>),
    );
}

fn texture(world: &mut World, path: &'static str, repeat: bool) -> Handle<Image> {
    let bytes = super::bundle::take(world, path);
    let mut sampler = ImageSamplerDescriptor::linear();
    if repeat {
        sampler.set_address_mode(ImageAddressMode::Repeat);
    }
    let image = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Descriptor(sampler),
        RenderAssetUsages::default(),
    )
    .expect("station painting decodes");
    world.resource_mut::<Assets<Image>>().add(image)
}

pub(super) fn setup(world: &mut World) {
    let camera = world
        .query_filtered::<Entity, With<Camera3d>>()
        .single(world)
        .expect("game camera assembled before the station");
    // Both passes draw the normal forward variants of the same actor, weapon
    // and skin materials. The bamboo SSR pass would also reflect the water.
    world.insert_resource(DefaultOpaqueRendererMethod::forward());
    let skylight = EnvironmentMapLight::hemispherical_gradient(
        &mut world.resource_mut::<Assets<Image>>(),
        Color::srgb(0.71, 0.78, 0.85),
        Color::srgb(0.63, 0.60, 0.51),
        Color::srgb(0.29, 0.31, 0.30),
    )
    .with_intensity(410.);
    world
        .entity_mut(camera)
        .remove::<(ScreenSpaceReflections, DeferredPrepass, DepthPrepass)>()
        .insert((RenderLayers::layer(0).with(WATER_LAYER), skylight.clone()));

    world.insert_resource(ClearColor(Color::srgb_u8(175, 194, 193)));
    world.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.81, 0.89, 0.95),
        brightness: 280.,
        ..default()
    });
    world.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.93, 0.82),
            illuminance: 6500.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-9., 16., 8.).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 15.,
            maximum_distance: 40.,
            ..default()
        }
        .build(),
        RenderLayers::layer(0).with(SISTER_LAYER),
    ));
    world.spawn((
        DirectionalLight {
            color: Color::srgb(0.63, 0.81, 1.0),
            illuminance: 1400.,
            ..default()
        },
        Transform::from_xyz(8., 7., -10.).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(0).with(SISTER_LAYER),
    ));

    let sky = texture(world, SKY, true);
    let floor = texture(world, PLATFORM, true);
    let coach = texture(world, CARRIAGE, false);
    let pigment = texture(world, PIGMENT, true);
    let paint = paint::ink(world, Some(pigment));
    platform::spawn(world, &paint, sky, floor, coach);
    mirror::spawn(world, camera, skylight);
    qingluan::setup(world, LAYOUT.sister, LAYOUT.window, SISTER_LAYER);
}
