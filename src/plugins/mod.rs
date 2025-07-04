pub mod game;
// pub mod game2d;
pub mod menu;

pub use game::core::GamePlugin;
// pub use game2d::core::Game2dPlugin;
pub use menu::MenuPlugin;

use bevy::prelude::*;

/// KivaPlugins 包含了游戏所需的所有核心插件
#[derive(Default)]
pub struct KivaPlugins;

impl PluginGroup for KivaPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(MenuPlugin)
            .add(GamePlugin)
        // .add(Game2dPlugin)
    }
}
