pub mod actions;
pub mod hud;
pub mod popup;
pub mod renderer;
pub mod table;

use bevy::prelude::*;
use crate::states::WorldState;
use crate::game::events::GameAction;
use crate::game::player::Player;
use crate::game::room::Room;
use table::{PlayerEntityMap, setup_table, cleanup_table};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntityMap>()
            // 进入游戏时，依次执行布局、HUD、操作栏、立即开始
            .add_systems(
                OnEnter(WorldState::Game),
                (
                    setup_table,
                    hud::spawn_hud,
                    actions::spawn_action_bar,
                    start_game_immediately,
                )
                    .chain(),
            )
            // 游戏进行中
            .add_systems(
                Update,
                (
                    renderer::on_card_dealt,
                    renderer::on_round_reset,
                    hud::update_scores,
                    hud::on_turn_changed,
                    actions::handle_button_input,
                    popup::on_round_over,
                    popup::handle_continue,
                )
                    .run_if(in_state(WorldState::Game)),
            )
            // 退出游戏时清理
            .add_systems(OnExit(WorldState::Game), cleanup_table);
    }
}

/// 进入游戏时：若房间已有玩家（来自大厅），直接开始；否则填充单机默认配置
fn start_game_immediately(
    mut room: ResMut<Room>,
    mut actions: MessageWriter<GameAction>,
) {
    // 若大厅已配置玩家则直接开始，否则填充单机默认4人
    if room.active_player_count() == 0 {
        room.seat_player(0, Player::new(0, "玩家", false));
        room.seat_player(1, Player::new(1, "AI甲", true));
        room.seat_player(2, Player::new(2, "AI乙", true));
        room.seat_player(3, Player::new(3, "AI丙", true));
    }
    actions.write(GameAction::StartGame);
}
