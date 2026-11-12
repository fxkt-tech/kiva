//! Device input and presentation compose the headless demo.
mod animation;
mod appearance;
mod audio;
mod avatar;
mod bundle;
mod camera;
mod combat;
mod companion;
pub(super) mod controls;
mod enemies;
mod environment;
mod forest_material;
mod hud;
mod impact;
mod loading;
mod model_asset;
mod rendering;
mod scene;
mod skeleton;
mod source_animation;
mod stickman;
mod texture_mips;
mod viewer;
mod weapon_models;
mod weapons;
mod web;

use bevy::{
    asset::{AssetMetaCheck, AssetPlugin},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};

/// Rendering owns these roots; their owners are simulation entities.
#[derive(Component, Clone, Copy)]
pub(super) struct ActorView {
    pub owner: Entity,
}

#[derive(Component)]
pub(super) struct PlayerView;

/// How far one presentation has consumed the simulation's bounded feedback.
///
/// Every consumer shares one rule: an event is presented at most once, and
/// events which arrive while paused are consumed without being presented, so
/// resuming never replays an old hit, shot or sound.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FeedbackCursor(u64);

impl FeedbackCursor {
    /// True the first time `sequence` is seen. Call it on paused frames too.
    pub(super) fn advance(&mut self, sequence: u64) -> bool {
        let fresh = sequence > self.0;
        if fresh {
            self.0 = sequence;
        }
        fresh
    }

    /// Consume everything recorded so far; yields it only while `active`.
    pub(super) fn take<'a>(
        &mut self,
        feedback: &'a crate::simulation::CombatFeedback,
        active: bool,
    ) -> impl Iterator<Item = &'a crate::simulation::CombatEvent> {
        let from = self.0;
        if let Some(last) = feedback.events().last() {
            self.advance(last.sequence);
        }
        feedback
            .events()
            .filter(move |event| active && event.sequence > from)
    }
}

fn init_resource<R: Resource + FromWorld>(world: &mut World) {
    world.init_resource::<R>();
}

fn display_alpha(
    time: &Time<Fixed>,
    active: &crate::simulation::SimulationActive,
    hit_stop: &crate::simulation::HitStop,
) -> f32 {
    if active.0 && !hit_stop.is_active() {
        time.overstep_fraction()
    } else {
        1.0
    }
}

/// The scene is assembled once, in the frame its runtime files have arrived.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Assemble;

/// Present once [`Assemble`] ran; every system reading the scene waits for it.
#[derive(Resource)]
pub(super) struct Assembled;

/// What [`Assemble`] decodes. Later selections request their own files.
#[derive(Resource)]
pub(super) struct Manifest(pub Vec<&'static str>);

const ASSET_FAILURE: &str = "资源加载失败，请重新加载后重试";

fn request_manifest(world: &mut World) {
    let paths = world.resource::<Manifest>().0.clone();
    bundle::request(world, &paths);
}

fn assemble(world: &mut World, mut failed: Local<bool>) {
    if world.contains_resource::<Assembled>() {
        return;
    }
    let status = bundle::status(world, &world.resource::<Manifest>().0);
    match status {
        // Repeated every frame: the page ignores an unchanged count, and the
        // loader's own late "initialize" notification cannot hide this one.
        bundle::Status::Pending { loaded, total } => web::progress("initialize", loaded, total),
        bundle::Status::Failed(path) if !*failed => {
            *failed = true;
            error!("Runtime asset {path} could not be loaded");
            web::fail(ASSET_FAILURE);
        }
        _ => {}
    }
    if status == bundle::Status::Ready {
        world.run_schedule(Assemble);
        world.insert_resource(Assembled);
    }
}

/// Shared by the game and the inspection page: fetch `manifest`, then run
/// [`Assemble`] exactly once.
pub(super) fn assemble_after_loading(app: &mut App, manifest: Vec<&'static str>) {
    app.add_plugins(bundle::BundlePlugin)
        .insert_resource(Manifest(manifest))
        .init_schedule(Assemble)
        .add_systems(Startup, (loading::begin, request_manifest).chain())
        .add_systems(First, assemble);
}

pub(super) fn run() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kiva · 竹林演武".into(),
                    resolution: (1440, 900).into(),
                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#game".into()),
                    #[cfg(target_arch = "wasm32")]
                    fit_canvas_to_parent: true,
                    #[cfg(target_arch = "wasm32")]
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // Trunk copies each asset directory next to the page; native
                // builds read the repository's (or the executable's) assets/.
                file_path: if cfg!(target_arch = "wasm32") {
                    ".".into()
                } else {
                    "assets".into()
                },
                // No .meta files are shipped, so do not request them.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(loading::StartupPlugin);
    if web::viewer_mode() {
        viewer::configure(&mut app);
        app.run();
        return;
    }
    assemble_after_loading(&mut app, bundle::game());
    app.add_plugins(forest_material::ForestMaterialPlugin)
        .add_plugins(crate::simulation::SimulationPlugin)
        // Nothing is simulated before the scene exists; input sampling owns
        // this flag from the first assembled frame.
        .insert_resource(crate::simulation::SimulationActive(false))
        .insert_resource(ClearColor(Color::srgb_u8(15, 22, 29)))
        .init_resource::<controls::ViewYaw>()
        .init_resource::<controls::CaptureState>()
        .init_resource::<skeleton::AnimationLibrary>()
        .init_resource::<appearance::AppearanceSelection>()
        .add_systems(
            Assemble,
            (
                init_resource::<source_animation::SourceAnimationLibrary>,
                crate::gameplay::setup,
                scene::setup,
                combat::setup,
                companion::setup,
                weapons::setup_tether,
                impact::setup,
                camera::setup,
                rendering::setup,
                environment::setup_camera_fog,
                loading::scene_ready,
            )
                .chain(),
        )
        .add_systems(
            RunFixedMainLoop,
            (enemies::sample, controls::sample, appearance::sample)
                .chain()
                .run_if(resource_exists::<Assembled>)
                .in_set(bevy::app::RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(
            Update,
            (
                appearance::reconcile,
                scene::sync_player,
                environment::animate_foliage,
                combat::reconcile_enemies,
                combat::sync_enemies,
                combat::sync_health_bars,
                combat::sync_weapons,
                animation::animate,
                companion::sync,
                avatar::sync,
                stickman::sync_attachments,
                weapons::sync_tether,
                combat::sync_markers,
                combat::effects,
                impact::update,
                audio::sync,
                hud::sync,
                camera::follow,
            )
                .chain()
                .run_if(resource_exists::<Assembled>),
        );
    app.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Runs {
        assembled: u32,
        gated: u32,
    }

    fn app(manifest: Vec<&'static str>) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            },
        ));
        assemble_after_loading(&mut app, manifest);
        app.init_resource::<Runs>()
            .add_systems(Assemble, |mut runs: ResMut<Runs>| runs.assembled += 1)
            .add_systems(
                Update,
                (|mut runs: ResMut<Runs>| runs.gated += 1).run_if(resource_exists::<Assembled>),
            );
        app
    }

    fn settle(app: &mut App) {
        for _ in 0..2_000 {
            app.update();
            let status = bundle::status(app.world(), &app.world().resource::<Manifest>().0);
            if !matches!(status, bundle::Status::Pending { .. }) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[test]
    fn the_scene_is_assembled_once_and_only_after_its_files_arrived() {
        let mut app = app(vec![
            "yizhiying/weapon.bin",
            "choulingke/environment/litter.png",
        ]);
        // Requested in Startup; the files cannot have been delivered yet.
        app.update();
        assert!(!app.world().contains_resource::<Assembled>());
        assert_eq!(app.world().resource::<Runs>().assembled, 0);
        assert_eq!(app.world().resource::<Runs>().gated, 0);
        settle(&mut app);
        for _ in 0..3 {
            app.update();
        }
        assert!(app.world().contains_resource::<Assembled>());
        let runs = app.world().resource::<Runs>();
        assert_eq!(runs.assembled, 1);
        assert!(runs.gated >= 3);
    }

    #[test]
    fn a_missing_file_never_assembles_and_reports_one_failure() {
        web::take_startup_notifications();
        let mut app = app(vec!["yizhiying/weapon.bin", "missing/weapon.bin"]);
        settle(&mut app);
        for _ in 0..3 {
            app.update();
        }
        assert!(!app.world().contains_resource::<Assembled>());
        assert_eq!(app.world().resource::<Runs>().assembled, 0);
        assert_eq!(web::take_startup_notifications(), (0, 1));
    }
}
