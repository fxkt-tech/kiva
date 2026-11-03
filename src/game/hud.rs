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
pub(super) struct StaminaBar(Side);
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
        children![(label("KIVA / LONGSWORD TRAINING", 22.0, ink))],
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
                (
                    Node {
                        width: percent(100),
                        height: px(4),
                        ..default()
                    },
                    BackgroundColor(Color::srgb_u8(45, 49, 51)),
                    children![(
                        StaminaBar(side),
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(gold)
                    )]
                ),
                (
                    FighterLabel(side),
                    label("100 HP / 100 SP  READY", 12.0, ink)
                ),
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
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        children![
            label("LMB horizontal / RMB vertical (hold: charge) | LMB+RMB / G parry | C+RMB uppercut", 12.0, ink),
            label("WASD move | SPACE x2 jump | SHIFT tap dodge/long-press run | C toggle crouch/slide | F5 parry / F6 blue / F7 normal / F8 reset", 11.0, ink),
        ],
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
    mut bars: Query<(&HealthBar, &mut Node), Without<StaminaBar>>,
    mut stamina_bars: Query<(&StaminaBar, &mut Node), Without<HealthBar>>,
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
                    "{}: {:.0} HP; {}; STAMINA {:.0}",
                    if fighter.side == Side::Player {
                        "Player"
                    } else {
                        "Dummy"
                    },
                    fighter.health,
                    fighter.label(),
                    fighter.stamina.value
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
            text.0 = format!(
                "{:.0} HP / {:.0} SP  {}",
                fighter.health,
                fighter.stamina.value,
                fighter.label()
            );
        }
    }
    for (bar, mut node) in &mut bars {
        if let Some(fighter) = fighters.iter().find(|fighter| fighter.side == bar.0) {
            node.width = percent(fighter.health);
        }
    }
    for (bar, mut node) in &mut stamina_bars {
        if let Some(fighter) = fighters.iter().find(|fighter| fighter.side == bar.0) {
            node.width = percent(fighter.stamina.value);
        }
    }
    status_label.0 = if controls.active {
        format!(
            "{}   |   HITS {}  PARRIES {}",
            status.last, status.hits, status.blocks
        )
    } else {
        "PAUSED / Click to enter training. Mouse moves the camera; Esc releases it.".into()
    };
}
