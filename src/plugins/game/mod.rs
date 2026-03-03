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
use crate::game::GameLogicSet;
use table::{PlayerEntityMap, setup_table, cleanup_table};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntityMap>()
            .init_resource::<renderer::CardCount>()
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
            // 游戏进行中：所有渲染系统必须在 GameLogicSet（process_game_actions）之后运行，
            // 确保同帧写入的消息能被渲染层读取；
            // on_round_reset 必须在 on_card_dealt 之前，保证新一局先清零计数再生成牌
            .add_systems(
                Update,
                (
                    renderer::on_round_reset.before(renderer::on_card_dealt),
                    renderer::on_card_dealt,
                    hud::update_scores,
                    hud::on_turn_changed,
                    actions::handle_button_input,
                    popup::on_round_over,
                    popup::handle_continue,
                )
                    .after(GameLogicSet)
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
    if room.active_player_count() == 0 {
        room.seat_player(0, Player::new(0, "玩家", false));
        room.seat_player(1, Player::new(1, "AI甲", true));
        room.seat_player(2, Player::new(2, "AI乙", true));
        room.seat_player(3, Player::new(3, "AI丙", true));
    }
    actions.write(GameAction::StartGame);
}
