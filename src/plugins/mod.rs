pub mod game;
pub mod menu;

use bevy::prelude::*;
use crate::game::GameLogicPlugin;

pub use game::GamePlugin;
pub use menu::MenuPlugin;

/// KivaPlugins 包含了游戏所需的所有核心插件
#[derive(Default)]
pub struct KivaPlugins;

impl PluginGroup for KivaPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(GameLogicPlugin)
            .add(MenuPlugin)
            .add(GamePlugin)
    }
}
