use super::{
    combat::{CombatStatus, Fighter, Side},
    input::Controls,
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct FighterLabel(Side);
#[derive(Component)]
pub(super) struct HealthBar(Side);
#[derive(Component)]
pub(super) struct StatusLabel;

fn label(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

pub(super) fn spawn_hud(mut commands: Commands) {
    let ink = Color::srgb_u8(232, 231, 218);
    let gold = Color::srgb_u8(224, 185, 111);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(26),
            left: px(30),
            ..default()
        },
        children![(label("KIVA / SWORD TRAINING", 22.0, ink))],
    ));
    for (side, left, title, color) in [
        (
            Side::Player,
            true,
            "SWORDSMAN",
            Color::srgb_u8(90, 208, 194),
        ),
        (
            Side::Dummy,
            false,
            "TRAINING DUMMY",
            Color::srgb_u8(228, 159, 95),
        ),
    ] {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(68),
                left: if left { px(30) } else { Val::Auto },
                right: if left { Val::Auto } else { px(30) },
                width: px(270),
                flex_direction: FlexDirection::Column,
                row_gap: px(9),
                padding: UiRect::all(px(13)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.04, 0.055, 0.82)),
            children![
                label(title, 14.0, color),
                (
                    Node {
                        width: percent(100),
                        height: px(7),
                        ..default()
                    },
                    BackgroundColor(Color::srgb_u8(45, 49, 51)),
                    children![(
                        HealthBar(side),
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(color)
                    )]
                ),
                (FighterLabel(side), label("100 / 100  READY", 12.0, ink)),
            ],
        ));
    }
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(66),
            left: px(30),
            right: px(30),
            padding: UiRect::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.04, 0.055, 0.78)),
        children![(StatusLabel, label("Click to enter training", 14.0, gold))],
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(24),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![label(
            "WASD move | SPACE x2 jump | L-SHIFT dodge | LMB combo | RMB guard | T strike | R reset | M sound | Esc",
            13.0,
            ink
        )],
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(4),
            height: px(4),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.93, 0.7, 0.7)),
    ));
}

pub(super) fn update_hud(
    controls: Res<Controls>,
    status: Res<CombatStatus>,
    fighters: Query<&Fighter>,
    mut labels: Query<(&FighterLabel, &mut Text), Without<StatusLabel>>,
    mut bars: Query<(&HealthBar, &mut Node)>,
    mut status_label: Single<&mut Text, (With<StatusLabel>, Without<FighterLabel>)>,
    #[cfg(target_arch = "wasm32")] mut accessible: Local<String>,
) {
    // Give the canvas a readable state summary for assistive technology.
    #[cfg(target_arch = "wasm32")]
    {
        let summary = fighters
            .iter()
            .map(|fighter| {
                format!(
                    "{}: {:.0} HP; {}",
                    if fighter.side == Side::Player {
                        "Player"
                    } else {
                        "Dummy"
                    },
                    fighter.health,
                    fighter.label()
                )
            })
            .collect::<Vec<_>>()
            .join(". ");
        if *accessible != summary {
            if let Some(canvas) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.get_element_by_id("game"))
            {
                let _ = canvas.set_attribute(
                    "aria-label",
                    &format!("Sword training. {summary}. Esc releases the mouse."),
                );
            }
            *accessible = summary;
        }
    }
    for (label, mut text) in &mut labels {
        if let Some(fighter) = fighters.iter().find(|fighter| fighter.side == label.0) {
            text.0 = format!("{:.0} / 100  {}", fighter.health, fighter.label());
        }
    }
    for (bar, mut node) in &mut bars {
        if let Some(fighter) = fighters.iter().find(|fighter| fighter.side == bar.0) {
            node.width = percent(fighter.health);
        }
    }
    status_label.0 = if controls.active {
        format!(
            "{}   |   HITS {}  BLOCKS {}",
            status.last, status.hits, status.blocks
        )
    } else {
        "PAUSED / Click to enter training. Mouse moves the camera; Esc releases it.".into()
    };
}
