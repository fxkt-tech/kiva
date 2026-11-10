//! First entry waits for a submitted scene, not just Startup entity assembly.
use std::sync::{Arc, Mutex, PoisonError};

use bevy::{
    pbr::SpecializedMaterialPipelineCache,
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        camera::ExtractedCamera,
        error_handler::{RenderError, RenderErrorHandler, RenderErrorPolicy},
        render_resource::{CachedPipelineState, PipelineCache},
        sync_world::MainEntity,
        view::{ExtractedView, RenderVisibleEntities, RetainedViewEntity, ViewTarget},
    },
    shader::ShaderCacheError,
};

const RENDER_FAILURE: &str = "画面准备失败，请重新加载后重试";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Readiness {
    #[default]
    Preparing,
    Failed,
    Ready,
}

// RenderApp can run on another thread. Only the main world calls the DOM bridge.
#[derive(Resource, Clone, Default)]
pub(super) struct StartupSignal(Arc<Mutex<Readiness>>);

impl StartupSignal {
    fn get(&self) -> Readiness {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn report(&self, next: Readiness) {
        let mut state = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        // Keep failures visible while preparing, but let genuine ready recover.
        if *state != Readiness::Ready && next != Readiness::Preparing {
            *state = next;
        }
    }
}

/// A selection has its own render acknowledgement; global startup stays sticky.
#[derive(Resource, Clone, Default)]
struct ModelSignal(Arc<Mutex<(u64, bool, bool)>>);

impl ModelSignal {
    fn report(&self, revision: u64, next: Readiness) -> bool {
        let mut state = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if state.0 == revision && next != Readiness::Preparing {
            state.1 = next == Readiness::Ready;
            state.2 = next == Readiness::Failed;
            true
        } else {
            false
        }
    }
}

#[derive(Resource, Clone, Default)]
struct ModelRequirement {
    revision: u64,
    meshes: Vec<MainEntity>,
}

pub(super) fn require_model(world: &mut World, revision: u64, meshes: Vec<Entity>) {
    let signal = world.resource::<ModelSignal>();
    *signal.0.lock().unwrap_or_else(PoisonError::into_inner) = (revision, false, false);
    world.insert_resource(ModelRequirement {
        revision,
        meshes: meshes.into_iter().map(MainEntity::from).collect(),
    });
}

pub(super) fn model_ready(world: &World, revision: u64) -> bool {
    world.get_resource::<ModelSignal>().is_some_and(|signal| {
        *signal.0.lock().unwrap_or_else(PoisonError::into_inner) == (revision, true, false)
    })
}

pub(super) fn model_failed(world: &World, revision: u64) -> bool {
    world.get_resource::<ModelSignal>().is_some_and(|signal| {
        let state = signal.0.lock().unwrap_or_else(PoisonError::into_inner);
        state.0 == revision && state.2
    })
}

#[derive(Resource, Default)]
struct Notifications {
    failed: bool,
    ready: bool,
}

#[derive(Resource)]
struct PreviousErrorHandler(RenderErrorHandler);

#[derive(Resource)]
pub(super) struct RendererAvailable(bool);

#[derive(Resource, Default)]
struct StartupCamera(Option<MainEntity>);

pub(super) struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        let signal = StartupSignal::default();
        let model_signal = ModelSignal::default();
        app.insert_resource(signal.clone())
            .insert_resource(model_signal.clone())
            .init_resource::<ModelRequirement>()
            .init_resource::<Notifications>()
            .add_systems(First, notify_browser);

        let has_renderer = if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(signal)
                .insert_resource(model_signal)
                .init_resource::<ModelRequirement>()
                .init_resource::<StartupCamera>()
                .add_systems(
                    ExtractSchedule,
                    (extract_startup_camera, extract_model_requirement),
                )
                .add_systems(
                    Render,
                    observe_rendered_scene
                        .after(RenderSystems::Render)
                        .before(RenderSystems::Cleanup),
                );
            true
        } else {
            false
        };
        app.insert_resource(RendererAvailable(has_renderer));

        // Preserve Bevy's existing stop/recovery policy, including AppExit. Its
        // handler runs during extraction with both worlds on the main thread.
        if let Some(previous) = app.world_mut().remove_resource::<RenderErrorHandler>() {
            app.insert_resource(PreviousErrorHandler(previous))
                .insert_resource(RenderErrorHandler(render_error));
        }
    }
}

pub(super) fn begin() {
    super::web::progress("initialize", 0, 0);
}

pub(super) fn scene_ready(renderer: Res<RendererAvailable>, signal: Res<StartupSignal>) {
    super::web::progress("render", 0, 0);
    info!("Initial scene assembled; waiting for rendering");
    if cfg!(target_arch = "wasm32") && !renderer.0 {
        signal.report(Readiness::Failed);
    }
}

fn notify_browser(world: &mut World) {
    let state = world.resource::<StartupSignal>().get();
    let mut notifications = world.resource_mut::<Notifications>();
    match state {
        Readiness::Ready if !notifications.ready => {
            notifications.ready = true;
            info!("Client ready after initial scene render");
            super::web::ready();
        }
        Readiness::Failed if !notifications.ready && !notifications.failed => {
            notifications.failed = true;
            super::web::fail(RENDER_FAILURE);
        }
        _ => {}
    }
}

fn render_error(
    error: &RenderError,
    main_world: &mut World,
    render_world: &mut World,
) -> RenderErrorPolicy {
    main_world
        .resource::<StartupSignal>()
        .report(Readiness::Failed);
    {
        let signal = main_world.resource::<ModelSignal>();
        let mut model = signal.0.lock().unwrap_or_else(PoisonError::into_inner);
        model.1 = false;
        model.2 = true;
    }
    if super::web::viewer_mode() {
        super::web::viewer_fail(RENDER_FAILURE);
    }
    // The default handler exits; deliver the failure before delegating to it.
    let mut notifications = main_world.resource_mut::<Notifications>();
    if !notifications.ready && !notifications.failed {
        notifications.failed = true;
        super::web::fail(RENDER_FAILURE);
    }
    let previous = main_world.resource::<PreviousErrorHandler>().0.0;
    previous(error, main_world, render_world)
}

fn extract_startup_camera(
    cameras: Extract<Query<Entity, With<super::rendering::MainView>>>,
    mut startup_camera: ResMut<StartupCamera>,
) {
    startup_camera.0 = cameras.iter().next().map(MainEntity::from);
}

fn extract_model_requirement(
    requirement: Extract<Res<ModelRequirement>>,
    mut target: ResMut<ModelRequirement>,
) {
    *target = requirement.clone();
}

fn required_model_visible(visible: &RenderVisibleEntities, required: &ModelRequirement) -> bool {
    if required.revision == 0 {
        return true;
    }
    visible.get::<Mesh3d>().is_some_and(|visible| {
        visible
            .iter_visible()
            .any(|(_, entity)| required.meshes.contains(entity))
    })
}

// PipelineCache is empty before scene specialization. An empty iterator is
// deliberately not completion. Bevy retries missing shaders/imports itself;
// processing/module errors are terminal and otherwise disappear from its queue.
fn pipelines_ready<'a>(states: impl Iterator<Item = &'a CachedPipelineState>) -> Readiness {
    let mut any = false;
    let mut pending = false;
    for state in states {
        any = true;
        match state {
            CachedPipelineState::Ok(_) => {}
            CachedPipelineState::Err(
                ShaderCacheError::ProcessShaderError(_) | ShaderCacheError::CreateShaderModule(_),
            ) => return Readiness::Failed,
            _ => pending = true,
        }
    }
    if any && !pending {
        Readiness::Ready
    } else {
        Readiness::Preparing
    }
}

fn visible_meshes_prepared(
    visible: &RenderVisibleEntities,
    view: RetainedViewEntity,
    specialized: &SpecializedMaterialPipelineCache,
) -> Option<usize> {
    let visible = visible.get::<Mesh3d>()?;
    let pipelines = specialized.get(&view)?;
    let mut count = 0;
    for (_, entity) in visible.iter_visible() {
        // Bevy only specializes after both GPU mesh and material exist. This
        // also excludes hidden companion bodies and future ability variants.
        if !pipelines.contains_key(entity) {
            return None;
        }
        count += 1;
    }
    (count > 0).then_some(count)
}

fn observe_rendered_scene(
    signal: Res<StartupSignal>,
    startup_camera: Res<StartupCamera>,
    model_signal: Res<ModelSignal>,
    model_requirement: Res<ModelRequirement>,
    cache: Option<Res<PipelineCache>>,
    specialized: Option<Res<SpecializedMaterialPipelineCache>>,
    views: Query<(&ExtractedView, &ViewTarget, &RenderVisibleEntities), With<ExtractedCamera>>,
) {
    if signal.get() == Readiness::Ready
        && (model_requirement.revision == 0
            || *model_signal
                .0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                == (model_requirement.revision, true, false))
    {
        return;
    }
    let (Some(cache), Some(specialized)) = (cache, specialized) else {
        // Render-disabled/headless apps need neither a device nor PBR assets.
        return;
    };
    match pipelines_ready(cache.pipelines().map(|pipeline| &pipeline.state)) {
        Readiness::Failed => {
            if model_signal.report(model_requirement.revision, Readiness::Failed) {
                signal.report(Readiness::Failed);
            }
            return;
        }
        Readiness::Preparing => return,
        Readiness::Ready => {}
    }
    for (view, target, visible) in &views {
        if Some(view.retained_view_entity.main_entity) != startup_camera.0
            // The output attachment marks this only when a render pass uses
            // it; an allocated target or an empty GPU submission isn't enough.
            || !target.needs_present()
            || !required_model_visible(visible, &model_requirement)
        {
            continue;
        }
        if let Some(meshes) =
            visible_meshes_prepared(visible, view.retained_view_entity, &specialized)
        {
            info!(
                "Initial scene rendered: {meshes} visible meshes, {} prepared pipelines",
                cache.pipelines().count()
            );
            if model_signal.report(model_requirement.revision, Readiness::Ready) {
                signal.report(Readiness::Ready);
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        app::SubApp,
        ecs::schedule::ScheduleLabel,
        pbr::SpecializedMaterialViewPipelineCache,
        render::{
            error_handler::ErrorType, render_resource::CachedRenderPipelineId,
            view::RenderVisibleEntitiesClass,
        },
    };
    use std::any::TypeId;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(StartupPlugin)
            .add_systems(Startup, (begin, scene_ready).chain());
        super::super::web::take_startup_notifications();
        app
    }

    #[test]
    fn startup_assembly_and_empty_render_state_cannot_unlock_entry() {
        let mut app = app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((super::super::camera::GameCamera, Camera3d::default()));
        });
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<StartupSignal>().get(),
            Readiness::Preparing
        );
        assert_eq!(super::super::web::take_startup_notifications(), (0, 0));
        assert_eq!(pipelines_ready(std::iter::empty()), Readiness::Preparing);
    }

    #[test]
    fn render_schedule_without_a_device_remains_pending_without_panicking() {
        let mut app = App::new();
        let mut render_app = SubApp::new();
        render_app.add_schedule(Render::base_schedule());
        render_app.update_schedule = Some(Render.intern());
        app.insert_sub_app(RenderApp, render_app);
        app.add_plugins(StartupPlugin)
            .add_systems(Startup, (begin, scene_ready).chain());
        super::super::web::take_startup_notifications();
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<StartupSignal>().get(),
            Readiness::Preparing
        );
        assert_eq!(super::super::web::take_startup_notifications(), (0, 0));
    }

    #[test]
    fn actual_bevy_pipeline_states_keep_transient_errors_pending_and_report_terminal_errors() {
        let states = [
            CachedPipelineState::Queued,
            CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(
                Handle::<Shader>::default().id(),
            )),
            CachedPipelineState::Err(ShaderCacheError::ShaderImportNotYetAvailable),
        ];
        assert_eq!(pipelines_ready(states.iter()), Readiness::Preparing);
        for failure in [
            ShaderCacheError::ProcessShaderError("invalid shader".into()),
            ShaderCacheError::CreateShaderModule("GPU validation failed".into()),
        ] {
            let states = [
                CachedPipelineState::Queued,
                CachedPipelineState::Err(failure),
            ];
            assert_eq!(pipelines_ready(states.iter()), Readiness::Failed);
        }
    }

    #[test]
    fn every_visible_standard_and_forest_mesh_needs_its_real_specialization_entry() {
        let mut world = World::new();
        let camera = MainEntity::from(world.spawn_empty().id());
        let standard = MainEntity::from(
            world
                .spawn((
                    Mesh3d::default(),
                    MeshMaterial3d::<StandardMaterial>::default(),
                ))
                .id(),
        );
        let forest = MainEntity::from(
            world
                .spawn((
                    Mesh3d::default(),
                    MeshMaterial3d::<super::super::forest_material::ForestMaterial>::default(),
                ))
                .id(),
        );
        let hidden = MainEntity::from(world.spawn((Mesh3d::default(), Visibility::Hidden)).id());
        let view = RetainedViewEntity::new(camera, None, 0);
        let mut visible = RenderVisibleEntities::default();
        let mut specialized = SpecializedMaterialPipelineCache::default();
        assert_eq!(visible_meshes_prepared(&visible, view, &specialized), None);
        visible.classes.insert(
            TypeId::of::<Mesh3d>(),
            RenderVisibleEntitiesClass {
                entities_cpu_culling: vec![
                    (Entity::PLACEHOLDER, standard),
                    (Entity::PLACEHOLDER, forest),
                ],
                ..default()
            },
        );
        let mut entries = SpecializedMaterialViewPipelineCache::default();
        entries.insert(standard, CachedRenderPipelineId::new(0));
        specialized.insert(view, entries);
        assert_eq!(visible_meshes_prepared(&visible, view, &specialized), None);
        specialized
            .get_mut(&view)
            .unwrap()
            .insert(forest, CachedRenderPipelineId::new(1));
        assert_eq!(
            visible_meshes_prepared(&visible, view, &specialized),
            Some(2)
        );
        assert!(!specialized[&view].contains_key(&hidden));
    }

    #[test]
    fn render_thread_results_notify_main_once_and_true_ready_recovers_failure() {
        let mut app = app();
        app.update();
        let signal = app.world().resource::<StartupSignal>().clone();
        let render_signal = signal.clone();
        std::thread::spawn(move || {
            render_signal.report(Readiness::Failed);
            assert_eq!(super::super::web::take_startup_notifications(), (0, 0));
        })
        .join()
        .unwrap();
        app.update();
        app.update();
        assert_eq!(super::super::web::take_startup_notifications(), (0, 1));
        signal.report(Readiness::Preparing);
        assert_eq!(signal.get(), Readiness::Failed);
        signal.report(Readiness::Ready);
        app.update();
        app.update();
        assert_eq!(super::super::web::take_startup_notifications(), (1, 0));
        signal.report(Readiness::Failed);
        app.update();
        assert_eq!(signal.get(), Readiness::Ready);
        assert_eq!(super::super::web::take_startup_notifications(), (0, 0));
    }

    #[test]
    fn device_error_notifies_before_preserving_the_existing_render_error_policy() {
        #[derive(Resource, Default)]
        struct PreviousCalled(bool);
        let mut app = App::new();
        app.init_resource::<PreviousCalled>()
            .insert_resource(RenderErrorHandler(|_, main, _| {
                main.resource_mut::<PreviousCalled>().0 = true;
                RenderErrorPolicy::StopRendering
            }))
            .add_plugins(StartupPlugin);
        super::super::web::take_startup_notifications();
        let mut render_world = World::new();
        let handler = app.world().resource::<RenderErrorHandler>().0;
        let result = handler(
            &RenderError {
                ty: ErrorType::Validation,
                description: "invalid render pipeline".into(),
                source: None,
            },
            app.world_mut(),
            &mut render_world,
        );
        assert!(matches!(result, RenderErrorPolicy::StopRendering));
        assert!(app.world().resource::<PreviousCalled>().0);
        assert_eq!(
            app.world().resource::<StartupSignal>().get(),
            Readiness::Failed
        );
        assert_eq!(super::super::web::take_startup_notifications(), (0, 1));
        app.update();
        assert_eq!(super::super::web::take_startup_notifications(), (0, 0));
    }

    #[test]
    fn floor_or_previous_body_cannot_acknowledge_selected_model() {
        let mut world = World::new();
        let floor = MainEntity::from(world.spawn_empty().id());
        let old_body = MainEntity::from(world.spawn_empty().id());
        let selected_body = MainEntity::from(world.spawn_empty().id());
        let mut visible = RenderVisibleEntities::default();
        visible.classes.insert(
            TypeId::of::<Mesh3d>(),
            RenderVisibleEntitiesClass {
                entities_cpu_culling: vec![
                    (Entity::PLACEHOLDER, floor),
                    (Entity::PLACEHOLDER, old_body),
                ],
                ..default()
            },
        );
        let required = ModelRequirement {
            revision: 2,
            meshes: vec![selected_body],
        };
        assert!(!required_model_visible(&visible, &required));
        visible
            .classes
            .get_mut(&TypeId::of::<Mesh3d>())
            .unwrap()
            .entities_cpu_culling
            .push((Entity::PLACEHOLDER, selected_body));
        assert!(required_model_visible(&visible, &required));
        assert!(required_model_visible(
            &visible,
            &ModelRequirement::default()
        ));
        assert!(!required_model_visible(
            &visible,
            &ModelRequirement {
                revision: 3,
                meshes: vec![]
            }
        ));
    }

    #[test]
    fn selection_ready_and_failure_are_versioned_without_resetting_global_ready() {
        let mut app = app();
        let first = app.world_mut().spawn_empty().id();
        require_model(app.world_mut(), 1, vec![first]);
        let signal = app.world().resource::<ModelSignal>().clone();
        signal.report(1, Readiness::Ready);
        assert!(model_ready(app.world(), 1));
        app.world()
            .resource::<StartupSignal>()
            .report(Readiness::Ready);
        let second = app.world_mut().spawn_empty().id();
        require_model(app.world_mut(), 2, vec![second]);
        assert!(!signal.report(1, Readiness::Ready));
        assert!(!signal.report(1, Readiness::Failed));
        assert!(!model_ready(app.world(), 2));
        assert!(!model_failed(app.world(), 2));
        assert_eq!(
            app.world().resource::<StartupSignal>().get(),
            Readiness::Ready
        );
        signal.report(2, Readiness::Failed);
        assert!(model_failed(app.world(), 2));
        signal.report(2, Readiness::Ready);
        assert!(model_ready(app.world(), 2));
        assert!(!model_failed(app.world(), 2));
    }

    #[test]
    fn device_error_after_startup_stays_visible_for_current_selection_and_preserves_policy() {
        let mut app = App::new();
        app.insert_resource(RenderErrorHandler(|_, _, _| {
            RenderErrorPolicy::StopRendering
        }))
        .add_plugins(StartupPlugin);
        let body = app.world_mut().spawn_empty().id();
        require_model(app.world_mut(), 5, vec![body]);
        app.world()
            .resource::<StartupSignal>()
            .report(Readiness::Ready);
        app.world()
            .resource::<ModelSignal>()
            .report(5, Readiness::Ready);
        let mut render_world = World::new();
        let handler = app.world().resource::<RenderErrorHandler>().0;
        let policy = handler(
            &RenderError {
                ty: ErrorType::Validation,
                description: "device lost after first ready".into(),
                source: None,
            },
            app.world_mut(),
            &mut render_world,
        );
        assert!(matches!(policy, RenderErrorPolicy::StopRendering));
        assert_eq!(
            app.world().resource::<StartupSignal>().get(),
            Readiness::Ready
        );
        assert!(model_failed(app.world(), 5));
        assert!(!model_ready(app.world(), 5));
    }
}
