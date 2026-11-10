//! Camera render passes for local, non-metallic wet-ground reflections.
//! This changes no camera pose, projection or authoritative scene state.
use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Hdr,
    core_pipeline::prepass::{DeferredPrepass, DepthPrepass},
    pbr::{DefaultOpaqueRendererMethod, ScreenSpaceReflections},
    prelude::*,
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
    for camera in cameras {
        setup_base(world, camera);
        world.entity_mut(camera).insert(ScreenSpaceReflections {
            min_perceptual_roughness: 0.08..0.12,
            max_perceptual_roughness: 0.18..0.24,
            linear_steps: 24,
            bisection_steps: 4,
            thickness: 0.18,
            edge_fadeout: 0.02..0.10,
            ..default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
