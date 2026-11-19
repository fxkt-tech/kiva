//! Each texel looks from the reflected game eye through one point on the
//! water. This off-axis frustum has the water itself as its near clip plane.
use super::{SISTER_LAYER, WATER_LAYER, paint};
use crate::gameplay::station::LAYOUT;
use bevy::{
    asset::{RenderAssetUsages, load_internal_asset, uuid_handle},
    camera::{
        CameraProjection, Hdr, RenderTarget, SubCameraView, primitives::Aabb,
        visibility::RenderLayers,
    },
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    light::NotShadowCaster,
    math::Vec3A,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{AsBindGroup, TextureFormat},
    shader::ShaderRef,
    window::PrimaryWindow,
};

const WATER_SHADER: Handle<Shader> = uuid_handle!("3f262d97-87ba-43ee-9ceb-34e9076b7a78");

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
struct Water {
    #[texture(0)]
    #[sampler(1)]
    reflection: Handle<Image>,
    #[uniform(2)]
    seconds: f32,
}

impl Material for Water {
    fn fragment_shader() -> ShaderRef {
        WATER_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub(super) fn configure_water(app: &mut App) {
    load_internal_asset!(app, WATER_SHADER, "water.wesl", Shader::from_wesl);
    app.add_plugins(MaterialPlugin::<Water>::default())
        .add_systems(
            Update,
            move_ripples.run_if(resource_exists::<crate::client::Assembled>),
        );
}

fn move_ripples(
    time: Res<Time>,
    active: Res<crate::simulation::SimulationActive>,
    water: Single<&MeshMaterial3d<Water>>,
    mut materials: ResMut<Assets<Water>>,
) {
    if active.0
        && let Some(mut surface) = materials.get_mut(&water.0)
    {
        surface.seconds += time.delta_secs();
    }
}

#[derive(Component)]
pub(super) struct ReflectionView {
    observer: Entity,
}

/// Eye-space slopes through the x/z bounds of the wet patch.
#[derive(Clone, Copy, Debug)]
struct Aperture {
    lo: Vec2,
    hi: Vec2,
    near: f32,
}

impl Aperture {
    fn matrix(&self, lo: Vec2, hi: Vec2) -> Mat4 {
        let scale = 2. / (hi - lo);
        let offset = (hi + lo) / (hi - lo);
        // Right-handed, infinite reverse-Z; the plane is depth 1.
        Mat4::from_cols(
            Vec4::X * scale.x,
            Vec4::Y * scale.y,
            Vec4::new(offset.x, offset.y, 0., -1.),
            Vec4::new(0., 0., self.near, 0.),
        )
    }
}

impl CameraProjection for Aperture {
    fn get_clip_from_view(&self) -> Mat4 {
        self.matrix(self.lo, self.hi)
    }
    fn get_clip_from_view_for_sub(&self, s: &SubCameraView) -> Mat4 {
        let span = self.hi - self.lo;
        let top = (s.offset / s.full_size.as_vec2()) * span;
        let size = s.size.as_vec2() / s.full_size.as_vec2() * span;
        let lo = Vec2::new(self.lo.x + top.x, self.hi.y - top.y - size.y);
        self.matrix(lo, lo + size)
    }
    fn update(&mut self, _: f32, _: f32) {}
    fn far(&self) -> f32 {
        180.
    }
    fn get_frustum_corners(&self, near: f32, far: f32) -> [Vec3A; 8] {
        let corners = [
            Vec2::new(self.hi.x, self.lo.y),
            self.hi,
            Vec2::new(self.lo.x, self.hi.y),
            self.lo,
        ];
        std::array::from_fn(|i| {
            let z = if i < 4 { near } else { far };
            ((-z * corners[i % 4]).extend(z)).into()
        })
    }
}

fn reflected_view(eye: Vec3) -> (Transform, Aperture) {
    let height = (eye.y - LAYOUT.puddle.y).max(0.001);
    let reflected_eye = eye.with_y(LAYOUT.puddle.y - height);
    // View right = +X, view up = +Z. Mirroring the eye and reversing the
    // texture's V axis gives proper handedness without reversing mesh winding.
    let pose = Transform::from_translation(reflected_eye).looking_to(Vec3::Y, Vec3::Z);
    let center = LAYOUT.puddle.xz() - reflected_eye.xz();
    (
        pose,
        Aperture {
            lo: (center - LAYOUT.puddle_half_size) / height,
            hi: (center + LAYOUT.puddle_half_size) / height,
            near: height,
        },
    )
}

fn sees_water(pose: &Transform, projection: &Projection) -> bool {
    let half = Vec3::new(LAYOUT.puddle_half_size.x, 0.001, LAYOUT.puddle_half_size.y);
    let bounds = Aabb::from_min_max(LAYOUT.puddle - half, LAYOUT.puddle + half);
    pose.translation.y > LAYOUT.puddle.y
        && projection
            .compute_frustum(&GlobalTransform::from(*pose))
            .intersects_obb_identity(&bounds)
}

pub(super) fn update(
    observers: Query<(&Camera, &Transform, &Projection), Without<ReflectionView>>,
    view: Single<(
        &ReflectionView,
        &mut Camera,
        &mut Transform,
        &mut Projection,
    )>,
) {
    let (reflection, mut camera, mut pose, mut projection) = view.into_inner();
    let (game_camera, main, lens) = observers
        .get(reflection.observer)
        .expect("station game camera");
    let visible = game_camera.is_active && sees_water(main, lens);
    if camera.is_active != visible {
        camera.is_active = visible;
    }
    if visible {
        let (new_pose, aperture) = reflected_view(main.translation);
        if *pose != new_pose {
            *pose = new_pose;
            // Marks the projection changed so Bevy refreshes its GPU matrix.
            let Projection::Custom(custom) = &mut *projection else {
                unreachable!("station water view requires an aperture");
            };
            *custom.get_mut::<Aperture>().unwrap() = aperture;
        }
    }
}

fn water_mesh() -> Mesh {
    const SEGMENTS: usize = 128;
    let mut positions = vec![[0., 0., 0.]];
    let mut colors = vec![[0.83, 0.94, 0.97, 0.81]];
    let mut uvs = vec![[0.5, 0.5]];
    let mut indices = Vec::new();
    // Reflected light dominates in the centre. The last few centimetres
    // reveal the platform stone through the ragged waterline.
    for (ring, (radius, alpha)) in [(0.74, 0.81), (0.91, 0.73), (0.98, 0.51), (1.0, 0.0)]
        .into_iter()
        .enumerate()
    {
        for i in 0..SEGMENTS {
            let t = std::f32::consts::TAU * i as f32 / SEGMENTS as f32;
            let shore = 0.92
                + 0.034 * (3. * t + 0.6).sin()
                + 0.022 * (7. * t).cos()
                + 0.013 * (13. * t + 1.).sin()
                + 0.009 * (31. * t).sin();
            let local = Vec2::new(t.cos(), t.sin()) * (shore * radius);
            let point = local * LAYOUT.puddle_half_size;
            positions.push([point.x, 0., point.y]);
            colors.push([0.83, 0.94, 0.97, alpha]);
            uvs.push([0.5 + 0.5 * local.x, 0.5 - 0.5 * local.y]);
            let here = (1 + ring * SEGMENTS + i) as u32;
            let next = (1 + ring * SEGMENTS + (i + 1) % SEGMENTS) as u32;
            if ring == 0 {
                indices.extend([0, next, here]);
            } else {
                let inner = here - SEGMENTS as u32;
                let inner_next = next - SEGMENTS as u32;
                indices.extend([inner, next, here, inner, inner_next, next]);
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; positions.len()])
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn target_size(viewport: UVec2) -> UVec2 {
    // Choose once at assembly so a moving shoreline never reallocates the RTT.
    // Small canvases still have a little sampling headroom at the bank.
    match viewport.min_element() {
        ..=480 => UVec2::new(384, 400),
        481..=680 => UVec2::new(512, 544),
        681..=900 => UVec2::new(704, 736),
        _ => UVec2::new(896, 960),
    }
}

pub(super) fn spawn(world: &mut World, observer: Entity, skylight: EnvironmentMapLight) {
    let eye = world.get::<Transform>(observer).unwrap().translation;
    let viewport = world
        .get::<Camera>(observer)
        .and_then(Camera::physical_viewport_size)
        .or_else(|| {
            world
                .query_filtered::<&Window, With<PrimaryWindow>>()
                .single(world)
                .ok()
                .map(|window| window.physical_size())
        })
        .unwrap_or(UVec2::new(960, 600));
    let size = target_size(viewport);
    // Linear HDR survives the round trip. Only the final game camera applies
    // the output tonemap, including to characters' emissive materials.
    let target = world
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba16Float,
            None,
        ));
    let (pose, aperture) = reflected_view(eye);
    world.spawn((
        ReflectionView { observer },
        Camera3d::default(),
        Camera {
            order: -1,
            ..default()
        },
        Hdr,
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        skylight,
        RenderTarget::Image(target.clone().into()),
        pose,
        Projection::custom(aperture),
        RenderLayers::layer(0).with(SISTER_LAYER),
    ));
    let water = world.resource_mut::<Assets<Water>>().add(Water {
        reflection: target,
        seconds: 0.,
    });
    let surface = paint::finish(
        world,
        water_mesh(),
        &water,
        Transform::from_translation(LAYOUT.puddle),
    );
    world
        .entity_mut(surface)
        .insert((RenderLayers::layer(WATER_LAYER), NotShadowCaster));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembled_game_and_reflection_views_share_actors_light_and_the_water_target() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<Assets<Water>>();
        world.spawn((
            PrimaryWindow,
            Window {
                resolution: (960, 600).into(),
                ..default()
            },
        ));
        let pose = Transform::from_xyz(-0.45, 6.9, 6.7).looking_at(LAYOUT.puddle, Vec3::Y);
        let game = world
            .spawn((crate::client::camera::GameCamera, Camera3d::default(), pose))
            .id();
        crate::client::rendering::setup(&mut world);
        crate::client::station::setup(&mut world);
        assert_eq!(*world.get::<Transform>(game).unwrap(), pose);
        assert!(
            world
                .get::<bevy::core_pipeline::prepass::DeferredPrepass>(game)
                .is_none()
        );
        assert_eq!(
            world.get::<RenderLayers>(game).unwrap(),
            &RenderLayers::layer(0).with(WATER_LAYER)
        );
        let (layers, target, light) = world.query_filtered::<(&RenderLayers, &RenderTarget, &EnvironmentMapLight), With<ReflectionView>>()
            .single(&world).unwrap();
        assert_eq!(layers, &RenderLayers::layer(0).with(SISTER_LAYER));
        assert_eq!(
            light.specular_map,
            world.get::<EnvironmentMapLight>(game).unwrap().specular_map
        );
        let RenderTarget::Image(image) = target else {
            panic!("water requires an offscreen view")
        };
        let handle = image.handle.clone();
        assert_eq!(
            world
                .resource::<Assets<Image>>()
                .get(&handle)
                .unwrap()
                .size(),
            UVec2::new(512, 544)
        );
        let (material, layers, _) = world
            .query::<(&MeshMaterial3d<Water>, &RenderLayers, &NotShadowCaster)>()
            .single(&world)
            .unwrap();
        assert_eq!(layers, &RenderLayers::layer(WATER_LAYER));
        assert_eq!(
            world
                .resource::<Assets<Water>>()
                .get(&material.0)
                .unwrap()
                .reflection,
            handle
        );
        // The full-screen handscroll can cover a game camera still facing water.
        world.get_mut::<Camera>(game).unwrap().is_active = false;
        world.run_system_once(update).unwrap();
        let reflected = world
            .query_filtered::<&Camera, With<ReflectionView>>()
            .single(&world)
            .unwrap();
        assert!(!reflected.is_active);
        world.get_mut::<Camera>(game).unwrap().is_active = true;
        world.run_system_once(update).unwrap();
        let reflected = world
            .query_filtered::<&Camera, With<ReflectionView>>()
            .single(&world)
            .unwrap();
        assert!(reflected.is_active);
        // An unrelated auxiliary 3D camera may appear after assembly.
        world.spawn(Camera3d::default());
        world.get_mut::<Transform>(game).unwrap().translation.x += 2.;
        world.run_system_once(update).unwrap();
        let reflected = world
            .query_filtered::<&Transform, With<ReflectionView>>()
            .single(&world)
            .unwrap();
        assert_eq!(reflected.translation.x, pose.translation.x + 2.);
    }

    #[test]
    fn reflected_eye_registers_plane_texels_and_actor_rays_at_every_orbit_angle() {
        for heading in [0., 0.7, 1.9, 3.1, 4.5] {
            let eye = LAYOUT.spawn + Vec3::Y * 6.9 + Quat::from_rotation_y(heading) * Vec3::Z * 4.2;
            let (view, aperture) = reflected_view(eye);
            let clip = aperture.get_clip_from_view() * view.to_matrix().inverse();
            assert!(view.to_matrix().determinant() > 0.);
            for point in [
                LAYOUT.sister + Vec3::Y * 1.6,
                LAYOUT.spawn + Vec3::Y,
                LAYOUT.pillar + Vec3::Y * 2.,
            ] {
                let image_point = point.with_y(2. * LAYOUT.puddle.y - point.y);
                let hit = eye.lerp(
                    image_point,
                    (LAYOUT.puddle.y - eye.y) / (image_point.y - eye.y),
                );
                let expected = (hit.xz() - LAYOUT.puddle.xz()) / LAYOUT.puddle_half_size;
                let projected = clip.project_point3(point);
                assert!(projected.truncate().distance(expected) < 0.0001);
                assert!((0.0..1.).contains(&projected.z));
            }
            assert!(clip.project_point3(LAYOUT.puddle - Vec3::Y * 0.01).z > 1.);
            assert!(clip.project_point3(LAYOUT.puddle + Vec3::Y * 0.01).z < 1.);
        }
    }

    #[test]
    fn water_can_show_scenery_outside_the_game_frustum_and_skip_hidden_passes() {
        let eye = LAYOUT.spawn + Vec3::new(0., 6.9, 4.2);
        let pose = Transform::from_translation(eye).looking_at(LAYOUT.puddle, Vec3::Y);
        let projection = Projection::Perspective(PerspectiveProjection {
            fov: 50_f32.to_radians(),
            aspect_ratio: 16. / 9.,
            ..default()
        });
        assert!(sees_water(&pose, &projection));
        assert!(!sees_water(
            &Transform::from_translation(eye).looking_to(Vec3::Y, Vec3::Z),
            &projection
        ));
        let (view, aperture) = reflected_view(eye);
        let scenery = view.translation.lerp(LAYOUT.puddle, 2.);
        let direct = projection.get_clip_from_view() * pose.to_matrix().inverse();
        assert!(direct.project_point3(scenery).y > 1.);
        let reflected = aperture.get_clip_from_view() * view.to_matrix().inverse();
        assert!(reflected.project_point3(scenery).truncate().length() < 0.0001);
    }
}
