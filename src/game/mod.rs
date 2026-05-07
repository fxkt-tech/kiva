use bevy::prelude::*;
use bevy_input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin};
use bevy_ui_widgets::ScrollbarPlugin;

pub mod ai;
pub mod app_state;
pub mod domain;
pub mod events;
pub mod llm;
pub mod phase;
pub mod player_config;
pub mod player_pool;
pub mod rules;
pub mod session;
pub mod ui;
pub mod visibility;

use app_state::{
    ActiveInfoTab, AppScreen, FlowState, NeedsGameRedraw, PendingInput, PlayerAction, PlayerMarks,
    SelectedPlayer, SessionResource, SpeechPlayback,
};

pub struct WerewolfGamePlugin;

impl Plugin for WerewolfGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ScrollbarPlugin, InputDispatchPlugin, TabNavigationPlugin))
            .init_state::<AppScreen>()
            .init_resource::<SessionResource>()
            .init_resource::<SelectedPlayer>()
            .init_resource::<ActiveInfoTab>()
            .init_resource::<PlayerMarks>()
            .init_resource::<PendingInput>()
            .init_resource::<PlayerAction>()
            .init_resource::<FlowState>()
            .init_resource::<NeedsGameRedraw>()
            .init_resource::<SpeechPlayback>()
            .init_resource::<ui::UiAssets>()
            .add_systems(Startup, (ui::spawn_camera, ui::enable_ime))
            .add_systems(OnEnter(AppScreen::Start), ui::spawn_start_screen)
            .add_systems(OnExit(AppScreen::Start), ui::cleanup_screen)
            .add_systems(OnEnter(AppScreen::Game), ui::spawn_game_placeholder)
            .add_systems(OnExit(AppScreen::Game), ui::cleanup_screen)
            .add_systems(OnEnter(AppScreen::Review), ui::spawn_review_screen)
            .add_systems(OnExit(AppScreen::Review), ui::cleanup_screen)
            .add_systems(
                Update,
                (
                    ui::button_action_system,
                    ui::seat_selection_system,
                    ui::text_input_system,
                    ui::speech_playback_system,
                    ui::send_scroll_events,
                    ui::redraw_game_screen,
                ),
            )
            .add_observer(ui::on_scroll_handler);
    }
}
