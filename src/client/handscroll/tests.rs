use super::*;
use crate::{
    client::{controls, menu_loadout},
    gameplay::{self, GameLevel, Player},
    simulation::{Combatant, SimulationActive, SimulationPlugin, TICK_RATE},
};
use bevy::{
    input::{
        ButtonState, InputPlugin,
        keyboard::{Key, KeyboardInput},
        mouse::MouseButtonInput,
    },
    window::{CursorOptions, WindowFocused},
};

// Use real device messages and the game's capture sampler. Only replace drawing
// and startup (no GPU in this fixture); input sees the same page as the renderer.
fn app(direct: bool) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, SimulationPlugin))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Time::<Fixed>::from_hz(TICK_RATE).timestep(),
        ))
        .insert_resource(GameLevel::Station)
        .add_message::<WindowFocused>()
        .init_resource::<CaptureState>()
        .init_resource::<controls::ViewYaw>()
        .init_resource::<Navigation>()
        .init_resource::<Page>()
        .insert_resource(ScrollCapture::new(direct))
        .insert_resource(NativeHandscroll {
            open: !direct,
            ..default()
        })
        .insert_resource(SimulationActive(false))
        .insert_resource(EntryStatus::Ready)
        .add_systems(PreUpdate, input.after(InputSystems))
        .add_systems(
            RunFixedMainLoop,
            (
                controls::sample.in_set(bevy::app::RunFixedMainLoopSystems::BeforeFixedMainLoop),
                observe_capture.in_set(bevy::app::RunFixedMainLoopSystems::AfterFixedMainLoop),
            ),
        )
        .add_systems(
            Update,
            |s: Res<NativeHandscroll>,
             loadout: Res<MenuLoadout>,
             status: Res<EntryStatus>,
             mut p: ResMut<Page>| {
                *p = pages::compose(&s, loadout.displayed(), &status);
            },
        );
    menu_loadout::configure(&mut app);
    app.finish();
    gameplay::setup(app.world_mut());
    app.insert_resource(Assembled);
    let window = app
        .world_mut()
        .spawn((
            Window {
                resolution: (1280, 900).into(),
                ..default()
            },
            CursorOptions::default(),
            PrimaryWindow,
        ))
        .id();
    app.update();
    (app, window)
}

fn mouse(app: &mut App, window: Entity, state: ButtonState) {
    app.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state,
        window,
    });
}

fn click(app: &mut App, window: Entity, position: Vec2) {
    let mut w = app.world_mut().get_mut::<Window>(window).unwrap();
    let (offset, scale) = fit(&w);
    w.set_cursor_position(Some(offset + scale * position));
    mouse(app, window, ButtonState::Pressed);
    app.update();
    mouse(app, window, ButtonState::Released);
    app.update();
}

fn choose(app: &mut App, window: Entity, action: Action) {
    let center = app.world().resource::<Page>().center(action).unwrap();
    click(app, window, center);
}

fn key(app: &mut App, window: Entity, key_code: KeyCode) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Escape,
            state,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }
}

fn active(app: &App) -> bool {
    app.world().resource::<SimulationActive>().0
}

fn combatant(app: &mut App) -> &Combatant {
    app.world_mut()
        .query_filtered::<&Combatant, With<Player>>()
        .single(app.world())
        .unwrap()
}

#[test]
fn direct_station_first_click_captures_once_then_escape_opens_the_scroll() {
    let (mut app, window) = app(true);
    assert!(!app.world().resource::<NativeHandscroll>().is_open());
    click(&mut app, window, Vec2::new(600., 450.));
    assert!(active(&app));
    assert!(combatant(&mut app).active_action().is_none());
    assert!(!app.world().resource::<NativeHandscroll>().is_open());
    mouse(&mut app, window, ButtonState::Pressed);
    key(&mut app, window, KeyCode::Escape);
    assert!(!active(&app));
    assert!(app.world().resource::<NativeHandscroll>().is_open());
    mouse(&mut app, window, ButtonState::Released);
    app.update();
    click(&mut app, window, Vec2::new(600., 450.));
    assert!(!active(&app));
    choose(&mut app, window, Action::Enter);
    assert!(active(&app));
    assert!(combatant(&mut app).active_action().is_none());
}

#[test]
fn default_entry_checks_ready_and_stage_and_keeps_browsing_clicks_out_of_combat() {
    let (mut app, window) = app(false);
    *app.world_mut().resource_mut::<EntryStatus>() = EntryStatus::Preparing;
    key(&mut app, window, KeyCode::ArrowLeft);
    assert_eq!(app.world().resource::<NativeHandscroll>().leaf, Leaf::Map);
    assert!(
        app.world()
            .resource::<Page>()
            .center(Action::Enter)
            .is_none()
    );
    choose(&mut app, window, Action::Stage(1));
    *app.world_mut().resource_mut::<EntryStatus>() = EntryStatus::Ready;
    app.update();
    assert!(
        app.world()
            .resource::<Page>()
            .center(Action::Enter)
            .is_none()
    );
    choose(&mut app, window, Action::Stage(2));
    assert!(
        app.world()
            .resource::<Page>()
            .center(Action::Enter)
            .is_none()
    );
    choose(&mut app, window, Action::Stage(0));
    assert!(!active(&app));
    choose(&mut app, window, Action::Enter);
    assert!(active(&app));
    key(&mut app, window, KeyCode::Escape);
    assert!(!active(&app));
    assert_eq!(app.world().resource::<NativeHandscroll>().leaf, Leaf::Home);
}

#[test]
fn illustrated_selections_and_purchases_survive_folding_the_scroll() {
    let (mut app, w) = app(false);
    choose(&mut app, w, Action::Open(Leaf::Armory));
    choose(&mut app, w, Action::Pair(1));
    choose(&mut app, w, Action::Equip);
    let pair = crate::simulation::WeaponPair::ALL[1];
    assert_eq!(app.world().resource::<MenuLoadout>().displayed(), pair);
    key(&mut app, w, KeyCode::Escape);
    choose(&mut app, w, Action::Open(Leaf::Persona));
    choose(&mut app, w, Action::Portrait(1));
    assert_eq!(pages::picture(app.world().resource()), 3);
    key(&mut app, w, KeyCode::Escape);
    choose(&mut app, w, Action::Open(Leaf::Birds));
    for i in 0..3 {
        choose(&mut app, w, Action::Bird(i));
        assert_eq!(pages::picture(app.world().resource()), 7 + i);
        key(&mut app, w, KeyCode::Escape);
        assert_eq!(app.world().resource::<NativeHandscroll>().leaf, Leaf::Birds);
    }
    key(&mut app, w, KeyCode::Escape);
    choose(&mut app, w, Action::Open(Leaf::Supply));
    choose(&mut app, w, Action::Adjust(1));
    choose(&mut app, w, Action::Buy);
    let shop = &app.world().resource::<NativeHandscroll>().shop;
    assert_eq!(
        (shop.feathers, shop.stamina, shop.quantity, shop.remaining()),
        (1260, 240, 0, 0)
    );
    assert!(app.world().resource::<Page>().center(Action::Buy).is_none());
    choose(&mut app, w, Action::Ware(1));
    choose(&mut app, w, Action::Buy);
    choose(&mut app, w, Action::Stall(1));
    choose(&mut app, w, Action::Buy);
    choose(&mut app, w, Action::Stall(2));
    choose(&mut app, w, Action::Ware(8));
    assert!(app.world().resource::<Page>().center(Action::Buy).is_none());
    key(&mut app, w, KeyCode::Escape);
    choose(&mut app, w, Action::Open(Leaf::Supply));
    let shop = &app.world().resource::<NativeHandscroll>().shop;
    assert_eq!((shop.coins, shop.feathers, shop.bought[1]), (45810, 580, 1));
    assert!(!active(&app));
    key(&mut app, w, KeyCode::Escape);
    key(&mut app, w, KeyCode::ArrowLeft);
    choose(&mut app, w, Action::Enter);
    assert_eq!(combatant(&mut app).weapon_pair(), pair);
}

#[test]
fn resuming_the_station_from_home_after_viewing_a_locked_stage() {
    let (mut app, window) = app(true);
    click(&mut app, window, Vec2::new(800., 450.));
    for locked in [1, 2] {
        key(&mut app, window, KeyCode::Escape);
        choose(&mut app, window, Action::Open(Leaf::Map));
        choose(&mut app, window, Action::Stage(locked));
        assert!(
            app.world()
                .resource::<Page>()
                .center(Action::Enter)
                .is_none()
        );
        choose(&mut app, window, Action::Back);
        choose(&mut app, window, Action::Enter);
        assert!(active(&app));
        assert!(combatant(&mut app).active_action().is_none());
    }
}
