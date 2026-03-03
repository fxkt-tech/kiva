use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum WorldState {
    #[default]
    Menu,
    Lobby, // 局域网房间大厅
    Game,
}
