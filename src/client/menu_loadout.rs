//! The armory submits an absolute choice. The live combatant confirms it.
use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{CombatIntent, Combatant, SimulationActive, WeaponPair},
};

#[derive(Resource, Default)]
pub(super) struct MenuLoadout {
    requested: Option<WeaponPair>,
    equipped: WeaponPair,
}

impl MenuLoadout {
    pub(super) fn select(&mut self, pair: WeaponPair) {
        self.requested = Some(pair);
    }

    pub(super) fn displayed(&self) -> WeaponPair {
        self.requested.unwrap_or(self.equipped)
    }
}

pub(super) fn configure(app: &mut App) {
    app.init_resource::<MenuLoadout>()
        .add_systems(
            FixedPreUpdate,
            request.run_if(resource_exists::<super::Assembled>),
        )
        .add_systems(
            FixedPostUpdate,
            acknowledge.run_if(resource_exists::<super::Assembled>),
        );
    #[cfg(target_arch = "wasm32")]
    app.add_systems(PreUpdate, read_armory)
        .add_systems(Update, publish);
}

fn request(
    active: Res<SimulationActive>,
    mut menu: ResMut<MenuLoadout>,
    mut player: Single<(&Combatant, &mut CombatIntent), With<Player>>,
) {
    let Some(pair) = menu.requested else { return };
    if player.0.weapon_pair() == pair {
        // A selection already equipped must leave combo windows and Z alone.
        menu.requested = None;
    } else if active.0 {
        player.1.request_pair(pair);
    }
}

fn acknowledge(mut menu: ResMut<MenuLoadout>, player: Single<&Combatant, With<Player>>) {
    let equipped = player.weapon_pair();
    if menu.equipped != equipped {
        menu.equipped = equipped;
    }
    // A busy or airborne player may have declined this step's intent.
    if menu.requested == Some(equipped) {
        menu.requested = None;
    }
}

#[cfg(target_arch = "wasm32")]
fn read_armory(mut menu: ResMut<MenuLoadout>) {
    let code = super::web::take_pair();
    if let Some(pair) = WeaponPair::ALL
        .into_iter()
        .find(|&pair| super::loadout::of(pair).hud_index == code)
    {
        menu.select(pair);
    }
}

#[cfg(target_arch = "wasm32")]
fn publish(menu: Res<MenuLoadout>) {
    if menu.is_changed() && menu.requested.is_none() {
        super::web::render_pair(super::loadout::of(menu.displayed()).hud_index);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{Assembled, controls},
        gameplay::{self, GameLevel},
        simulation::{ActionKind, CombatStatus, SimulationPlugin, TICK_RATE},
    };
    use bevy::{
        app::RunFixedMainLoopSystems,
        input::{
            ButtonState, InputPlugin,
            keyboard::{Key, KeyboardInput, NativeKey},
            mouse::MouseButtonInput,
        },
        time::TimeUpdateStrategy,
        window::{CursorOptions, PrimaryWindow, WindowFocused},
    };

    fn game(level: GameLevel) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin, SimulationPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                Time::<Fixed>::from_hz(TICK_RATE).timestep(),
            ))
            .insert_resource(level)
            .insert_resource(SimulationActive(false))
            .add_message::<WindowFocused>()
            .init_resource::<controls::CaptureState>()
            .init_resource::<controls::ScrollCapture>()
            .init_resource::<controls::ViewYaw>()
            .add_systems(
                RunFixedMainLoop,
                controls::sample.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            );
        configure(&mut app);
        app.finish();
        gameplay::setup(app.world_mut());
        app.insert_resource(Assembled);
        let world = app.world_mut();
        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        let window = world
            .spawn((Window::default(), CursorOptions::default(), PrimaryWindow))
            .id();
        app.update();
        (app, player, window)
    }

    fn combat(app: &App, player: Entity) -> &Combatant {
        app.world().get::<Combatant>(player).unwrap()
    }

    fn select(app: &mut App, pair: WeaponPair) {
        app.world_mut().resource_mut::<MenuLoadout>().select(pair);
    }

    fn click(app: &mut App, window: Entity) {
        app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window,
        });
        app.update();
    }

    fn capture(app: &mut App, window: Entity) {
        app.world_mut()
            .resource_mut::<controls::ScrollCapture>()
            .request();
        click(app, window);
        assert!(app.world().resource::<SimulationActive>().0);
    }

    fn press(app: &mut App, window: Entity, key_code: KeyCode) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }

    #[test]
    fn first_capture_equips_the_armory_choice_and_z_changes_the_live_loadout() {
        let (mut app, player, window) = game(GameLevel::Station);
        let [hammer, axe] = WeaponPair::ALL;
        select(&mut app, axe);
        click(&mut app, window);
        assert!(!app.world().resource::<SimulationActive>().0);
        assert_eq!(combat(&app, player).weapon_pair(), hammer);
        assert_eq!(app.world().resource::<MenuLoadout>().displayed(), axe);
        capture(&mut app, window);
        assert_eq!(combat(&app, player).weapon_pair(), axe);
        press(&mut app, window, KeyCode::KeyE);
        let action = combat(&app, player).active_action().unwrap();
        assert_eq!((action.kind, action.pair), (ActionKind::ChainSweep, axe));
        for _ in 0..90 {
            app.update();
        }
        press(&mut app, window, KeyCode::KeyZ);
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(combat(&app, player).weapon_pair(), hammer);
        assert_eq!(app.world().resource::<MenuLoadout>().displayed(), hammer);
    }

    #[test]
    fn same_pair_is_a_noop_and_a_paused_choice_waits_for_the_committed_skill() {
        let (mut app, player, window) = game(GameLevel::BambooForest);
        let [hammer, axe] = WeaponPair::ALL;
        select(&mut app, hammer);
        capture(&mut app, window);
        assert_eq!(combat(&app, player).status(), CombatStatus::Ready);
        press(&mut app, window, KeyCode::KeyQ);
        let action = combat(&app, player).active_action().unwrap();
        assert_eq!(action.kind, ActionKind::MusketShot);
        let cooldown = combat(&app, player).ranged_cooldown();

        press(&mut app, window, KeyCode::Escape);
        select(&mut app, axe);
        for _ in 0..6 {
            app.update();
        }
        assert!(!app.world().resource::<SimulationActive>().0);
        assert_eq!(combat(&app, player).ranged_cooldown(), cooldown);
        capture(&mut app, window);
        assert_eq!(
            combat(&app, player).active_action().unwrap().action_id,
            action.action_id
        );
        assert_eq!(combat(&app, player).weapon_pair(), hammer);
        assert_eq!(app.world().resource::<MenuLoadout>().displayed(), axe);

        for _ in 0..90 {
            app.update();
            if combat(&app, player).weapon_pair() == axe {
                break;
            }
        }
        assert_eq!(combat(&app, player).weapon_pair(), axe);
        assert!(combat(&app, player).ranged_cooldown() > 0.0);
        assert_eq!(app.world().resource::<MenuLoadout>().requested, None);
    }
}
