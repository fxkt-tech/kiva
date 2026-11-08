use super::{controls::PointerState, session::ClientSession, web};
use crate::simulation::{NavigationMap, SimulationControl, UnitPose};
use bevy::{input::mouse::MouseWheel, prelude::*};
use serde::Deserialize;

#[derive(Component)]
pub(super) struct GameCamera;
#[derive(Resource)]
pub(super) struct CameraRig {
    pub center: Vec2,
    pub locked: bool,
    pub zoom: f32,
}

const DEFAULT_ZOOM: f32 = 0.5;
const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 1.5;
const OFFSET: Vec3 = Vec3::new(10.0, 25.0, 20.0);

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            center: Vec2::new(0.0, 3.0),
            locked: false,
            zoom: DEFAULT_ZOOM,
        }
    }
}
impl CameraRig {
    fn transform(&self) -> Transform {
        let target = Vec3::new(self.center.x, 0.0, self.center.y);
        Transform::from_translation(target + OFFSET * self.zoom).looking_at(target, Vec3::Y)
    }

    fn clamp_to_map(&mut self, map: &NavigationMap) {
        let bound = map.half_size() - 5.0;
        self.center = self.center.clamp(Vec2::splat(-bound), Vec2::splat(bound));
    }

    /// Minimap navigation owns only the view. It cannot issue a unit command,
    /// change zoom, or replay a request against a different match's player.
    fn apply_minimap_input(
        &mut self,
        request: Option<&str>,
        dragging: bool,
        player: Option<Entity>,
        paused: bool,
        map: &NavigationMap,
    ) -> bool {
        let Some(player) = player.filter(|_| !paused) else {
            return false;
        };
        let request = request
            .and_then(|raw| serde_json::from_str::<CameraRequest>(raw).ok())
            .filter(|request| request.player_id == player.to_bits().to_string())
            .filter(|request| Vec2::from_array(request.center).is_finite());
        let accepted = request.is_some();
        if let Some(request) = request {
            self.center = Vec2::from_array(request.center);
            self.clamp_to_map(map);
        }
        if accepted || dragging {
            self.locked = false;
        }
        accepted || dragging
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CameraRequest {
    player_id: String,
    center: [f32; 2],
}

pub(super) fn setup(mut commands: Commands, rig: Res<CameraRig>) {
    commands.spawn((GameCamera, Camera3d::default(), rig.transform()));
}

/// The rendered view's actual ground-plane outline, in screen-corner order.
/// This shares Bevy's projection with picking; CSS/DPR never changes world units.
pub(super) fn ground_footprint(
    camera: &Camera,
    transform: &GlobalTransform,
) -> Option<[[f32; 2]; 4]> {
    let projection = camera.clip_from_view();
    let determinant = projection.determinant();
    if !projection.is_finite() || !determinant.is_finite() || determinant == 0.0 {
        return None;
    }
    let viewport = camera.logical_viewport_rect()?;
    if !viewport.size().is_finite() || viewport.size().cmple(Vec2::ZERO).any() {
        return None;
    }
    let corners = [
        viewport.min,
        Vec2::new(viewport.max.x, viewport.min.y),
        viewport.max,
        Vec2::new(viewport.min.x, viewport.max.y),
    ];
    let mut outline = [[0.0; 2]; 4];
    for (index, corner) in corners.into_iter().enumerate() {
        let ray = camera.viewport_to_world(transform, corner).ok()?;
        let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
        let point = ray.get_point(distance);
        if !point.is_finite() {
            return None;
        }
        outline[index] = [point.x, point.z];
    }
    Some(outline)
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy system parameters declare independent ECS access"
)]
pub(super) fn update(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    control: Res<SimulationControl>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    pointer: Res<PointerState>,
    session: Res<ClientSession>,
    poses: Query<&UnitPose>,
    map: Res<NavigationMap>,
    mut rig: ResMut<CameraRig>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
    mut observed_player: Local<Option<Entity>>,
) {
    // Read unconditionally: an input received during pause/selection must not
    // become a deferred camera jump after the next capture or new match.
    let request = web::take_camera_request();
    let dragging = web::minimap_dragging();
    let player = session.player();
    if player.is_none() {
        *rig = CameraRig::default();
        *observed_player = None;
    } else if player != *observed_player
        && let Some(player) = player
        && let Ok(pose) = poses.get(player)
    {
        // A new player means a new match, including Active -> Active restart.
        // Center once; subsequent free-camera movement and ordinary resume
        // keep their existing center, zoom and lock behavior.
        rig.center = pose.position();
        rig.locked = false;
        *observed_player = Some(player);
    }
    let size = web::canvas_size().unwrap_or(Vec2::new(window.width(), window.height()));
    let scroll: f32 = wheel.read().map(|event| event.y).sum();
    let minimap_navigation =
        rig.apply_minimap_input(request.as_deref(), dragging, player, control.paused, &map);
    if !control.paused && session.player().is_some() {
        if !minimap_navigation && keys.just_pressed(KeyCode::KeyY) {
            rig.locked = !rig.locked;
        }
        if !pointer.over_ui && !dragging {
            rig.zoom = (rig.zoom - scroll * 0.045).clamp(MIN_ZOOM, MAX_ZOOM);
        }
        if !minimap_navigation
            && (rig.locked || keys.pressed(KeyCode::Space))
            && let Some(player) = session.player()
            && let Ok(pose) = poses.get(player)
        {
            rig.center = pose.interpolated(fixed.overstep_fraction());
        } else if !minimap_navigation {
            let mut direction = Vec2::new(
                f32::from(keys.pressed(KeyCode::ArrowRight))
                    - f32::from(keys.pressed(KeyCode::ArrowLeft)),
                f32::from(keys.pressed(KeyCode::ArrowDown))
                    - f32::from(keys.pressed(KeyCode::ArrowUp)),
            );
            if !pointer.over_ui
                && let Some(cursor) = pointer.position
            {
                let margin = 14.0;
                if cursor.x >= 0.0 && cursor.x < margin {
                    direction.x -= 1.0;
                }
                if cursor.x <= size.x && cursor.x > size.x - margin {
                    direction.x += 1.0;
                }
                if cursor.y >= 0.0 && cursor.y < margin {
                    direction.y -= 1.0;
                }
                if cursor.y <= size.y && cursor.y > size.y - margin {
                    direction.y += 1.0;
                }
            }
            let right = Vec2::new(OFFSET.z, -OFFSET.x).normalize();
            let down = Vec2::new(OFFSET.x, OFFSET.z).normalize();
            let speed = 18.0 * rig.zoom * time.delta_secs();
            rig.center += (right * direction.x + down * direction.y).clamp_length_max(1.0) * speed;
        }
        rig.clamp_to_map(&map);
    }
    **camera = rig.transform();
    web::set_camera_lock(rig.locked);
}

#[cfg(test)]
mod tests {
    use super::super::{content, session};
    use super::*;
    use crate::simulation::SimulationPlugin;
    use bevy::{
        camera::{CameraProjection, RenderTargetInfo, Viewport},
        input::{mouse::MouseScrollUnit, touch::TouchPhase},
    };

    fn perspective_camera(size: Vec2, scale: f32, offset: UVec2) -> Camera {
        let physical_size = (size * scale).as_uvec2();
        let mut camera = Camera {
            viewport: Some(Viewport {
                physical_position: offset,
                physical_size,
                ..default()
            }),
            ..default()
        };
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: physical_size + offset,
            scale_factor: scale,
        });
        let mut projection = PerspectiveProjection::default();
        projection.update(size.x, size.y);
        camera.computed.clip_from_view = projection.get_clip_from_view();
        camera
    }

    fn footprint_area(points: [[f32; 2]; 4]) -> f32 {
        (0..4)
            .map(|index| {
                let a = Vec2::from_array(points[index]);
                let b = Vec2::from_array(points[(index + 1) % 4]);
                a.perp_dot(b)
            })
            .sum::<f32>()
            .abs()
            * 0.5
    }

    #[test]
    fn minimap_request_moves_only_the_camera_and_releases_follow_lock() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let map = NavigationMap::new(32.0, 1.0, vec![]);
        let mut rig = CameraRig {
            center: Vec2::new(-6.0, 20.0),
            locked: true,
            zoom: 0.83,
        };
        let request = serde_json::json!({
            "player_id": player.to_bits().to_string(),
            "center": [12.5, -16.25],
        })
        .to_string();
        assert!(rig.apply_minimap_input(Some(&request), false, Some(player), false, &map));
        assert_eq!(rig.center, Vec2::new(12.5, -16.25));
        assert!(!rig.locked);
        assert_eq!(rig.zoom, 0.83);
    }

    #[test]
    fn minimap_request_uses_the_same_world_bounds_as_manual_panning() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let map = NavigationMap::new(32.0, 1.0, vec![]);
        let mut rig = CameraRig::default();
        for (point, expected) in [
            ([300.0, -50.0], [27.0, -27.0]),
            ([-80.0, 32.0], [-27.0, 27.0]),
        ] {
            let request = serde_json::json!({
                "player_id": player.to_bits().to_string(), "center": point,
            })
            .to_string();
            assert!(rig.apply_minimap_input(Some(&request), false, Some(player), false, &map));
            assert_eq!(rig.center.to_array(), expected);
            assert_eq!(rig.zoom, DEFAULT_ZOOM);
        }
    }

    #[test]
    fn minimap_rejects_stale_malformed_paused_and_playerless_intents_without_a_jump() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let stale_player = world.spawn_empty().id();
        let map = NavigationMap::new(32.0, 1.0, vec![]);
        let request =
            |id, center| serde_json::json!({ "player_id": id, "center": center }).to_string();
        let valid = request(player.to_bits().to_string(), [10.0, -10.0]);
        let overflow = format!(
            "{{\"player_id\":\"{}\",\"center\":[1e100,0]}}",
            player.to_bits()
        );
        let cases = [
            (valid.clone(), Some(player), true),
            (valid.clone(), None, false),
            (valid.clone(), Some(stale_player), false),
            ("not json".into(), Some(player), false),
            ("null".into(), Some(player), false),
            (overflow, Some(player), false),
            (
                valid.replace("[10.0,-10.0]", "[null,0]"),
                Some(player),
                false,
            ),
            (
                valid.replace("[10.0,-10.0]", "[NaN,0]"),
                Some(player),
                false,
            ),
            (valid.replace("[10.0,-10.0]", "[1]"), Some(player), false),
            (valid.replace("}", ",\"extra\":1}"), Some(player), false),
            ("{\"center\":[10,0]}".into(), Some(player), false),
            (
                "{\"player_id\":1,\"center\":[10,0]}".into(),
                Some(player),
                false,
            ),
        ];
        for (raw, player, paused) in cases {
            let mut rig = CameraRig {
                center: Vec2::new(3.0, 5.0),
                locked: true,
                zoom: 1.1,
            };
            assert!(
                !rig.apply_minimap_input(Some(&raw), false, player, paused, &map),
                "rejected input must not own the camera frame: {raw}"
            );
            assert_eq!(rig.center, Vec2::new(3.0, 5.0), "{raw}");
            assert!(rig.locked, "{raw}");
            assert_eq!(rig.zoom, 1.1, "{raw}");
            assert!(!rig.apply_minimap_input(None, false, player, false, &map));
            assert_eq!(rig.center, Vec2::new(3.0, 5.0));
        }
    }

    #[test]
    fn held_minimap_drag_reserves_camera_navigation_without_a_new_motion_sample() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let map = NavigationMap::new(32.0, 1.0, vec![]);
        let mut rig = CameraRig {
            center: Vec2::new(4.0, -7.0),
            locked: true,
            zoom: 0.7,
        };
        // A hold between mouse-move events must still suppress automatic
        // follow and manual edge/key panning, without making the frame jump.
        assert!(rig.apply_minimap_input(None, true, Some(player), false, &map));
        assert_eq!(rig.center, Vec2::new(4.0, -7.0));
        assert!(!rig.locked);
        assert_eq!(rig.zoom, 0.7);
        assert!(!rig.apply_minimap_input(None, false, Some(player), false, &map));
        assert!(!rig.apply_minimap_input(None, true, Some(player), true, &map));
        assert!(!rig.apply_minimap_input(None, true, None, false, &map));
    }

    #[test]
    fn initial_spawn_and_first_update_use_the_same_close_framing() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<PointerState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<CameraRig>()
            .add_message::<MouseWheel>()
            .add_systems(Startup, setup)
            .add_systems(Update, update);
        content::setup(app.world_mut());
        session::setup(app.world_mut());
        app.world_mut().spawn(Window::default());
        app.world_mut().run_schedule(Startup);
        let initial = *app
            .world_mut()
            .query_filtered::<&Transform, With<GameCamera>>()
            .single(app.world())
            .unwrap();
        app.world_mut().run_schedule(Update);
        let updated = *app
            .world_mut()
            .query_filtered::<&Transform, With<GameCamera>>()
            .single(app.world())
            .unwrap();
        assert_eq!(initial, updated);
        assert_eq!(initial.translation, Vec3::Z * 3.0 + OFFSET * DEFAULT_ZOOM);
    }

    #[test]
    fn default_framing_projects_a_two_meter_body_at_reference_screen_size() {
        let size = Vec2::new(1280.0, 800.0);
        let camera = perspective_camera(size, 1.0, UVec2::ZERO);
        let projected_height = |zoom| {
            let transform = GlobalTransform::from(
                CameraRig {
                    center: Vec2::ZERO,
                    locked: false,
                    zoom,
                }
                .transform(),
            );
            let feet = camera.world_to_viewport(&transform, Vec3::ZERO).unwrap();
            let head = camera.world_to_viewport(&transform, Vec3::Y * 2.0).unwrap();
            feet.y - head.y
        };
        let height = projected_height(CameraRig::default().zoom);
        assert!((0.08..=0.12).contains(&(height / size.y)));
        assert!(height > projected_height(1.0) * 1.9);
    }

    #[test]
    fn ground_outline_tracks_zoom_aspect_and_actual_viewport_corners() {
        let size = Vec2::new(1280.0, 800.0);
        let camera = perspective_camera(size, 1.0, UVec2::ZERO);
        let rig = CameraRig::default();
        let transform = GlobalTransform::from(rig.transform());
        let outline = ground_footprint(&camera, &transform).unwrap();
        for (point, expected) in outline.into_iter().zip([
            Vec2::ZERO,
            Vec2::new(size.x, 0.0),
            size,
            Vec2::new(0.0, size.y),
        ]) {
            let projected = camera
                .world_to_viewport(&transform, Vec3::new(point[0], 0.0, point[1]))
                .unwrap();
            assert!(projected.abs_diff_eq(expected, 0.001));
        }
        let farther = GlobalTransform::from(
            CameraRig {
                zoom: rig.zoom * 2.0,
                ..rig
            }
            .transform(),
        );
        let expanded = ground_footprint(&camera, &farther).unwrap();
        assert!((footprint_area(expanded) / footprint_area(outline) - 4.0).abs() < 0.001);
        let wide = perspective_camera(Vec2::new(size.x * 2.0, size.y), 1.0, UVec2::ZERO);
        let wider = ground_footprint(&wide, &transform).unwrap();
        assert!((footprint_area(wider) / footprint_area(outline) - 2.0).abs() < 0.001);
        let scaled = perspective_camera(size, 2.0, UVec2::new(200, 100));
        let high_dpi = ground_footprint(&scaled, &transform).unwrap();
        for (actual, expected) in high_dpi.into_iter().zip(outline) {
            assert!(Vec2::from_array(actual).abs_diff_eq(Vec2::from_array(expected), 0.001));
        }
    }

    #[test]
    fn ground_outline_is_absent_without_a_valid_complete_ground_projection() {
        let transform = GlobalTransform::from(CameraRig::default().transform());
        assert!(ground_footprint(&Camera::default(), &transform).is_none());
        let mut camera = perspective_camera(Vec2::new(1280.0, 800.0), 1.0, UVec2::ZERO);
        let horizontal = GlobalTransform::from(
            Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::new(0.0, 10.0, -1.0), Vec3::Y),
        );
        assert!(ground_footprint(&camera, &horizontal).is_none());
        camera.computed.clip_from_view = Mat4::ZERO;
        assert!(ground_footprint(&camera, &transform).is_none());
    }

    #[test]
    fn scrolling_preserves_near_far_ui_and_pause_boundaries() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            SimulationPlugin,
            crate::gameplay::GameplayPlugin,
        ))
        .init_resource::<PointerState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<CameraRig>()
        .add_message::<MouseWheel>()
        .add_systems(Update, update);
        content::setup(app.world_mut());
        session::setup(app.world_mut());
        session::start_native(app.world_mut());
        app.world_mut().spawn((GameCamera, Transform::default()));
        let window = app.world_mut().spawn(Window::default()).id();
        for (scroll, over_ui, paused, expected) in [
            (1000.0, false, false, MIN_ZOOM),
            (-1000.0, true, false, MIN_ZOOM),
            (-1000.0, false, false, MAX_ZOOM),
            (1000.0, false, true, MAX_ZOOM),
        ] {
            app.world_mut().resource_mut::<PointerState>().over_ui = over_ui;
            app.world_mut().resource_mut::<SimulationControl>().paused = paused;
            app.world_mut().write_message(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: scroll,
                window,
                phase: TouchPhase::Moved,
            });
            app.world_mut().run_schedule(Update);
            assert_eq!(app.world().resource::<CameraRig>().zoom, expected);
        }
    }

    #[test]
    fn selecting_restores_the_existing_camera_to_its_initial_rig() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<PointerState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<MouseWheel>()
            .insert_resource(CameraRig {
                center: Vec2::new(15.0, -9.0),
                locked: true,
                zoom: 1.4,
            })
            .add_systems(Update, update);
        content::setup(app.world_mut());
        session::setup(app.world_mut());
        let camera = app
            .world_mut()
            .spawn((GameCamera, Transform::default()))
            .id();
        app.world_mut().spawn(Window::default());
        app.world_mut().run_schedule(Update);
        let rig = app.world().resource::<CameraRig>();
        assert_eq!(rig.center, CameraRig::default().center);
        assert!(!rig.locked);
        assert_eq!(rig.zoom, DEFAULT_ZOOM);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<GameCamera>>()
                .iter(app.world())
                .collect::<Vec<_>>(),
            [camera]
        );
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            Vec3::new(0.0, 0.0, 3.0) + OFFSET * DEFAULT_ZOOM
        );
    }

    #[test]
    fn y_and_space_center_the_real_hero_on_initial_and_restarted_rounds() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            SimulationPlugin,
            crate::gameplay::GameplayPlugin,
        ))
        .init_resource::<PointerState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<CameraRig>()
        .add_message::<MouseWheel>()
        .add_systems(Update, update);
        content::setup(app.world_mut());
        app.world_mut().spawn((GameCamera, Transform::default()));
        app.world_mut().spawn(Window::default());
        let mut previous_player = None;
        for _ in 0..2 {
            session::setup(app.world_mut());
            session::start_native(app.world_mut());
            let player = app.world().resource::<ClientSession>().player().unwrap();
            assert_ne!(Some(player), previous_player);
            let point = app.world().get::<UnitPose>(player).unwrap().position();
            for key in [KeyCode::KeyY, KeyCode::Space] {
                *app.world_mut().resource_mut::<CameraRig>() = CameraRig::default();
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .reset_all();
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
                app.world_mut().run_schedule(Update);
                let rig = app.world().resource::<CameraRig>();
                assert_eq!(
                    rig.center, point,
                    "{key:?} must reach the real spawn without clamping"
                );
                assert_eq!(rig.locked, key == KeyCode::KeyY);
            }
            previous_player = Some(player);
            crate::gameplay::reset_match(app.world_mut());
        }
    }
}
