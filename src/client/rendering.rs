//! Camera render passes for local, non-metallic wet-ground reflections.
//! This changes no camera pose, projection or authoritative scene state.
use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Hdr,
    core_pipeline::prepass::{DeferredPrepass, DepthPrepass, NormalPrepass},
    pbr::{DefaultOpaqueRendererMethod, ScreenSpaceReflections},
    prelude::*,
};

use super::render_style::{
    LevelStyles, RenderStyle,
    yancai_post::{YancaiPaper, YancaiShade},
};

/// Actual rendered view, shared by normal gameplay and the inspection App.
#[derive(Component)]
pub(super) struct MainView;

fn setup_base(world: &mut World, camera: Entity) {
    world.entity_mut(camera).insert((
        MainView,
        Hdr,
        Msaa::Off,
        Fxaa::default(),
        DepthPrepass,
        DeferredPrepass,
    ));
}

pub(super) fn setup_viewer(world: &mut World) {
    world.insert_resource(DefaultOpaqueRendererMethod::deferred());
    let cameras: Vec<_> = world
        .query_filtered::<Entity, With<MainView>>()
        .iter(world)
        .collect();
    for camera in cameras {
        setup_base(world, camera);
    }
}

pub(super) fn setup(world: &mut World) {
    // SSR consumes the deferred gbuffer. Standard/extended materials remain
    // responsible for their GPU skinning, alpha masks and normal maps.
    world.insert_resource(DefaultOpaqueRendererMethod::deferred());
    let cameras: Vec<_> = world
        .query_filtered::<Entity, With<super::camera::GameCamera>>()
        .iter(world)
        .collect();
    let style = world
        .get_resource::<RenderStyle>()
        .copied()
        .unwrap_or_default();
    // Single-style levels (the station) keep their shipped camera exactly.
    let switchable = world
        .get_resource::<LevelStyles>()
        .is_some_and(|styles| styles.count() > 1);
    for camera in cameras {
        setup_base(world, camera);
        // Geometric normals feed stylised outlines. Resident in every style
        // of a switchable level so switching never recompiles pipelines.
        if switchable {
            world.entity_mut(camera).insert(NormalPrepass);
        }
        if let Some(ssr) = reflections(style) {
            world.entity_mut(camera).insert(ssr);
        }
        if let Some(post) = yancai(style) {
            world.entity_mut(camera).insert(post);
        }
    }
}

/// The yancai post chain, normalised to the clearing's key light.
fn yancai(style: RenderStyle) -> Option<(YancaiShade, YancaiPaper)> {
    let forest = super::environment::forest_style(style);
    (style == RenderStyle::Yancai).then(|| {
        (
            YancaiShade {
                light_over_pi: forest.y,
                bands: forest.z,
            },
            YancaiPaper::default(),
        )
    })
}

/// Wet-ground SSR belongs to the photoreal look only.
fn reflections(style: RenderStyle) -> Option<ScreenSpaceReflections> {
    (style == RenderStyle::Pbr).then(|| ScreenSpaceReflections {
        min_perceptual_roughness: 0.08..0.12,
        max_perceptual_roughness: 0.18..0.24,
        linear_steps: 24,
        bisection_steps: 4,
        thickness: 0.18,
        edge_fadeout: 0.02..0.10,
        ..default()
    })
}

/// Adds or removes the style-owned camera passes on a switch.
pub(super) fn apply_style(
    mut commands: Commands,
    style: Res<RenderStyle>,
    cameras: Query<Entity, With<super::camera::GameCamera>>,
) {
    for camera in &cameras {
        let mut camera = commands.entity(camera);
        match reflections(*style) {
            Some(ssr) => camera.insert(ssr),
            None => camera.remove::<ScreenSpaceReflections>(),
        };
        match yancai(*style) {
            Some(post) => camera.insert(post),
            None => camera.remove::<(YancaiShade, YancaiPaper)>(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_switchable_levels_add_the_outline_normal_prepass() {
        use crate::gameplay::GameLevel;
        for (level, expected) in [(GameLevel::BambooForest, true), (GameLevel::Station, false)] {
            let mut world = World::new();
            world.insert_resource(LevelStyles::for_level(level));
            let camera = world
                .spawn((super::super::camera::GameCamera, Camera3d::default()))
                .id();
            setup(&mut world);
            assert_eq!(world.get::<NormalPrepass>(camera).is_some(), expected);
            assert!(world.get::<ScreenSpaceReflections>(camera).is_some());
            assert!(world.get::<YancaiShade>(camera).is_none());
        }
    }

    #[test]
    fn wet_ground_reflections_install_real_passes_without_moving_the_camera() {
        let mut world = World::new();
        let transform = Transform::from_xyz(4.2, 6.9, 3.0).looking_at(Vec3::ZERO, Vec3::Y);
        let projection = Projection::Perspective(PerspectiveProjection {
            fov: 50.0_f32.to_radians(),
            ..default()
        });
        let camera = world
            .spawn((
                super::super::camera::GameCamera,
                Camera3d::default(),
                transform,
                projection,
            ))
            .id();
        setup(&mut world);
        assert_eq!(*world.get::<Transform>(camera).unwrap(), transform);
        let Projection::Perspective(projection) = world.get::<Projection>(camera).unwrap() else {
            panic!("camera projection changed");
        };
        assert_eq!(projection.fov, 50.0_f32.to_radians());
        assert_eq!(*world.get::<Msaa>(camera).unwrap(), Msaa::Off);
        assert!(world.get::<DepthPrepass>(camera).is_some());
        assert!(world.get::<DeferredPrepass>(camera).is_some());
        assert!(world.get::<Hdr>(camera).is_some());
        assert!(world.get::<Fxaa>(camera).is_some());
        let ssr = world.get::<ScreenSpaceReflections>(camera).unwrap();
        // Rough damp soil stays outside the stochastic reflection pass.
        assert!(ssr.linear_steps > 0 && ssr.max_perceptual_roughness.end <= 0.25);
    }

    #[test]
    fn inspection_view_shares_pbr_setup_without_wetland_ssr_or_pose_changes() {
        let mut world = World::new();
        let transform = Transform::from_xyz(0.0, 1.5, 3.0).looking_at(Vec3::Y, Vec3::Y);
        let camera = world.spawn((MainView, Camera3d::default(), transform)).id();
        setup_viewer(&mut world);
        assert_eq!(*world.get::<Transform>(camera).unwrap(), transform);
        assert!(world.get::<Hdr>(camera).is_some());
        assert!(world.get::<Fxaa>(camera).is_some());
        assert!(world.get::<DepthPrepass>(camera).is_some());
        assert!(world.get::<DeferredPrepass>(camera).is_some());
        assert!(world.get::<ScreenSpaceReflections>(camera).is_none());
    }
}
