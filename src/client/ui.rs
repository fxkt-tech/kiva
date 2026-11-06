//! Read-only projections for the browser HUD and native debug overlay.
use super::{
    camera::{CameraRig, GameCamera},
    controls::{Aim, ClientFeedback, PlayerController},
    web,
};
use crate::simulation::*;
use bevy::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct UnitView {
    id: String,
    name: String,
    team: &'static str,
    position: [f32; 2],
    screen: Option<[f32; 2]>,
    health: f32,
    max_health: f32,
    mana: f32,
    max_mana: f32,
    alive: bool,
    action: &'static str,
    order: &'static str,
}
#[derive(Serialize)]
struct AbilityView {
    slot: usize,
    name: String,
    cooldown: f32,
    total_cooldown: f32,
    cost: f32,
    range: f32,
}
#[derive(Serialize)]
struct HudView {
    tick: u64,
    paused: bool,
    player_id: String,
    selected: Option<String>,
    aim: Option<usize>,
    attack_move: bool,
    units: Vec<UnitView>,
    abilities: Vec<AbilityView>,
    notice: String,
    camera_locked: bool,
    camera_center: [f32; 2],
    view_projection: [f32; 16],
    viewport: [f32; 2],
    obstacles: Vec<[f32; 4]>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy system parameters declare independent ECS access"
)]
pub(super) fn publish(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    mut next_publish: Local<f32>,
    player: Res<PlayerController>,
    control: Res<SimulationControl>,
    tick: Res<SimulationTick>,
    rig: Res<CameraRig>,
    feedback: Res<ClientFeedback>,
    catalog: Res<AbilityCatalog>,
    map: Res<NavigationMap>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    units: Query<(
        Entity,
        &Unit,
        &UnitState,
        &UnitPose,
        &UnitAction,
        &UnitOrder,
        &AbilitySlots,
    )>,
) {
    if time.elapsed_secs() < *next_publish {
        return;
    }
    *next_publish = time.elapsed_secs() + 0.08;
    let (camera, transform) = *camera;
    let views = units
        .iter()
        .map(|(id, unit, state, pose, action, order, _)| {
            let point = pose.position();
            let visual = pose.interpolated(if control.paused {
                1.0
            } else {
                fixed.overstep_fraction()
            });
            UnitView {
                id: id.to_bits().to_string(),
                name: unit.name().into(),
                team: if unit.team() == Team::Ally {
                    "ally"
                } else {
                    "enemy"
                },
                position: point.to_array(),
                screen: camera
                    .world_to_viewport(transform, Vec3::new(visual.x, 2.2, visual.y))
                    .ok()
                    .map(|point| point.to_array()),
                health: state.health(),
                max_health: unit.stats().max_health,
                mana: state.mana(),
                max_mana: unit.stats().max_mana,
                alive: state.alive(),
                action: action.label(),
                order: match order {
                    UnitOrder::Idle => "Idle",
                    UnitOrder::MoveTo(_) => "Moving",
                    UnitOrder::AttackTarget(_) => "Attacking",
                    UnitOrder::AttackMove { .. } => "Attack move",
                },
            }
        })
        .collect();
    let abilities = units
        .get(player.player)
        .map(|(_, _, _, _, _, _, slots)| {
            (0..4)
                .filter_map(|slot| {
                    let definition = catalog.get(slots.ability(slot)?)?;
                    Some(AbilityView {
                        slot,
                        name: definition.name.clone(),
                        cooldown: slots.cooldown(slot).unwrap_or(0.0),
                        total_cooldown: definition.cooldown,
                        cost: definition.mana_cost,
                        range: definition.range,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let view = HudView {
        tick: tick.0,
        paused: control.paused,
        player_id: player.player.to_bits().to_string(),
        selected: player.selected.map(|id| id.to_bits().to_string()),
        aim: match player.aim {
            Some(Aim::Ability(slot)) => Some(slot),
            _ => None,
        },
        attack_move: matches!(player.aim, Some(Aim::AttackMove)),
        units: views,
        abilities,
        notice: if time.elapsed_secs() < feedback.expires {
            feedback.message.clone()
        } else {
            String::new()
        },
        camera_locked: rig.locked,
        camera_center: rig.center.to_array(),
        view_projection: (camera.clip_from_view() * transform.to_matrix().inverse())
            .to_cols_array(),
        viewport: [window.width(), window.height()],
        obstacles: map
            .obstacles()
            .iter()
            .map(|o| [o.center.x, o.center.y, o.half_size.x, o.half_size.y])
            .collect(),
    };
    if let Ok(json) = serde_json::to_string(&view) {
        web::publish(&json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub(super) struct NativeHud;
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn native_hud(mut commands: Commands) {
    commands.spawn((
        NativeHud,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(17.0),
            ..default()
        },
        TextColor(Color::srgb_u8(213, 228, 218)),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(18),
            ..default()
        },
    ));
}
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn update_native_hud(
    player: Res<PlayerController>,
    control: Res<SimulationControl>,
    units: Query<(&UnitState, &AbilitySlots)>,
    mut hud: Single<&mut Text, With<NativeHud>>,
) {
    if let Ok((state, slots)) = units.get(player.player) {
        hud.0 = format!(
            "KIVA / THE PROVING GROUNDS\nRight click: move / attack · A + click: attack move · S: stop\nQ: bolt · W: area · E: dash · R: channel · Left click confirms\nY: camera lock · Space: center · Arrows / edges: pan · Scroll: zoom · Esc: pause\nHP {:.0}   Energy {:.0}   Q {:.1}  W {:.1}  E {:.1}  R {:.1}{}",
            state.health(),
            state.mana(),
            slots.cooldown(0).unwrap_or(0.0),
            slots.cooldown(1).unwrap_or(0.0),
            slots.cooldown(2).unwrap_or(0.0),
            slots.cooldown(3).unwrap_or(0.0),
            if control.paused {
                "\nPAUSED · Press Escape to resume"
            } else {
                ""
            }
        );
    }
}
