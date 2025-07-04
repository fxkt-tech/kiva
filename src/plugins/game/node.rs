use bevy::prelude::*;

pub fn create_card_node() -> Node {
    Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(Val::Px(5.0)),
        width: Val::Percent(12.5),
        height: Val::Percent(90.0),
        ..default()
    }
}

pub fn create_card_font(asset_server: &AssetServer) -> TextFont {
    TextFont {
        font: asset_server.load("fonts/WenCangShuFang-2.ttf"),
        font_size: 80.0,
        ..default()
    }
}
