use bevy::prelude::*;

pub mod ai;
pub mod app_state;
pub mod domain;
pub mod phase;
pub mod rules;
pub mod session;
pub mod ui;

use app_state::{
    ActiveInfoTab, AppScreen, FlowState, NeedsGameRedraw, PendingInput, PlayerAction, PlayerMarks,
    SelectedPlayer, SessionResource,
};

pub struct WerewolfGamePlugin;

impl Plugin for WerewolfGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScreen>()
            .init_resource::<SessionResource>()
            .init_resource::<SelectedPlayer>()
            .init_resource::<ActiveInfoTab>()
            .init_resource::<PlayerMarks>()
            .init_resource::<PendingInput>()
            .init_resource::<PlayerAction>()
            .init_resource::<FlowState>()
            .init_resource::<NeedsGameRedraw>()
            .init_resource::<ui::UiAssets>()
            .add_systems(Startup, (ui::spawn_camera, ui::enable_ime))
            .add_systems(OnEnter(AppScreen::Start), ui::spawn_start_screen)
            .add_systems(OnExit(AppScreen::Start), ui::cleanup_screen)
            .add_systems(OnEnter(AppScreen::RoleReveal), ui::spawn_role_reveal_screen)
            .add_systems(OnExit(AppScreen::RoleReveal), ui::cleanup_screen)
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
                    ui::redraw_game_screen,
                ),
            );
    }
}
