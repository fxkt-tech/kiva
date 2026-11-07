//! Read-only projections for the browser HUD and native debug overlay.
use super::{
    abilities::AbilityView,
    camera::{CameraRig, GameCamera},
    controls::{Aim, ClientFeedback, Pickable, PlayerController},
    web,
};
use crate::simulation::*;
use bevy::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct AttributesView {
    attack_damage: f32,
    ability_power: f32,
    armor: f32,
    magic_resistance: f32,
    move_speed: f32,
    attack_range: f32,
    attack_speed: f32,
}
impl From<&EffectiveAttributes> for AttributesView {
    fn from(attributes: &EffectiveAttributes) -> Self {
        Self {
            attack_damage: attributes.attack_damage(),
            ability_power: attributes.ability_power(),
            armor: attributes.armor(),
            magic_resistance: attributes.magic_resistance(),
            move_speed: attributes.move_speed(),
            attack_range: attributes.attack_range(),
            attack_speed: attributes.attack_speed(),
        }
    }
}

#[derive(Serialize)]
struct StatusInfo {
    kind: &'static str,
    remaining: f32,
    duration: f32,
    slow_fraction: Option<f32>,
}
impl From<&StatusView> for StatusInfo {
    fn from(status: &StatusView) -> Self {
        Self {
            kind: match status.kind() {
                StatusKind::Stun => "Stun",
                StatusKind::Root => "Root",
                StatusKind::Slow => "Slow",
                StatusKind::Airborne => "Airborne",
            },
            remaining: status.remaining_seconds(),
            duration: status.duration_seconds(),
            slow_fraction: status.fraction(),
        }
    }
}

#[derive(Serialize)]
struct ControlView {
    can_move: bool,
    can_attack: bool,
    can_cast: bool,
    can_dash: bool,
    is_displaced: bool,
}
impl From<&ControlState> for ControlView {
    fn from(control: &ControlState) -> Self {
        Self {
            can_move: control.can_move(),
            can_attack: control.can_attack(),
            can_cast: control.can_cast(),
            can_dash: control.can_dash(),
            is_displaced: control.forced_motion(),
        }
    }
}

#[derive(Serialize)]
struct UnitView {
    id: String,
    name: String,
    team: &'static str,
    position: [f32; 2],
    screen: Option<[f32; 2]>,
    body_screen: Option<[f32; 2]>,
    visual_height: f32,
    health: f32,
    max_health: f32,
    mana: f32,
    max_mana: f32,
    attributes: AttributesView,
    statuses: Vec<StatusInfo>,
    control: ControlView,
    alive: bool,
    action: &'static str,
    order: &'static str,
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

type ProjectedUnit = (
    Entity,
    &'static Unit,
    &'static UnitState,
    &'static UnitPose,
    &'static UnitAction,
    &'static UnitOrder,
    &'static AbilitySlots,
    &'static GlobalTransform,
    &'static Pickable,
    &'static UnitStatuses,
    &'static ControlState,
);

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct LabelPosition {
    id: String,
    screen: Option<[f32; 2]>,
    body_screen: Option<[f32; 2]>,
    alive: bool,
}

/// Only lightweight screen positions cross this bridge every rendered frame.
/// Use the actual display transforms, including interpolation and camera motion.
#[cfg(target_arch = "wasm32")]
pub(super) fn publish_labels(
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    units: Query<(Entity, &GlobalTransform, &Pickable, &UnitState), With<super::scene::UnitVisual>>,
) {
    let (camera, camera_transform) = *camera;
    let Some(viewport) = camera.logical_viewport_size() else {
        return;
    };
    let positions: Vec<_> = units
        .iter()
        .map(|(id, transform, pickable, state)| LabelPosition {
            id: id.to_bits().to_string(),
            screen: project_display(camera, camera_transform, transform, Vec3::Y * 2.2, viewport),
            body_screen: project_display(
                camera,
                camera_transform,
                transform,
                pickable.offset,
                viewport,
            ),
            alive: state.alive(),
        })
        .collect();
    if let Ok(json) = serde_json::to_string(&positions) {
        web::publish_labels(&json);
    }
}

fn project_display(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    unit_transform: &GlobalTransform,
    offset: Vec3,
    viewport: Vec2,
) -> Option<[f32; 2]> {
    camera
        .world_to_viewport(camera_transform, unit_transform.transform_point(offset))
        .ok()
        .map(|point| web::viewport_to_css(point, viewport).to_array())
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy system parameters declare independent ECS access"
)]
pub(super) fn publish(
    time: Res<Time>,
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
    units: Query<ProjectedUnit>,
) {
    if time.elapsed_secs() < *next_publish {
        return;
    }
    *next_publish = time.elapsed_secs() + 0.08;
    let (camera, transform) = *camera;
    let viewport = camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(window.width(), window.height()));
    let views = units
        .iter()
        .map(
            |(id, unit, state, pose, action, order, _, display, pickable, statuses, control)| {
                let point = pose.position();
                UnitView {
                    id: id.to_bits().to_string(),
                    name: unit.name().into(),
                    team: if unit.team() == Team::Ally {
                        "ally"
                    } else {
                        "enemy"
                    },
                    position: point.to_array(),
                    screen: project_display(camera, transform, display, Vec3::Y * 2.2, viewport),
                    body_screen: project_display(
                        camera,
                        transform,
                        display,
                        pickable.offset,
                        viewport,
                    ),
                    visual_height: display.translation().y,
                    health: state.health(),
                    max_health: unit.definition().max_health,
                    mana: state.mana(),
                    max_mana: unit.definition().max_mana,
                    attributes: unit.attributes().into(),
                    statuses: statuses.iter().map(Into::into).collect(),
                    control: control.into(),
                    alive: state.alive(),
                    action: action.label(),
                    order: match order {
                        UnitOrder::Idle => "Idle",
                        UnitOrder::MoveTo(_) => "Moving",
                        UnitOrder::AttackTarget(_) => "Attacking",
                        UnitOrder::AttackMove { .. } => "Attack move",
                    },
                }
            },
        )
        .collect();
    let abilities = units
        .get(player.player)
        .map(|(_, _, state, _, _, _, slots, _, _, _, control)| {
            (0..4)
                .filter_map(|slot| {
                    let definition = catalog.get(slots.ability(slot)?)?;
                    Some(AbilityView::new(slot, definition, state, control, slots))
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
        viewport: web::canvas_size().unwrap_or(viewport).to_array(),
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
    catalog: Res<AbilityCatalog>,
    units: Query<(&Unit, &UnitState, &AbilitySlots, &UnitStatuses)>,
    mut hud: Single<&mut Text, With<NativeHud>>,
) {
    if let Ok((unit, state, slots, statuses)) = units.get(player.player) {
        let attributes = unit.attributes();
        let loadout = ["Q", "W", "E", "R"]
            .into_iter()
            .enumerate()
            .filter_map(|(slot, key)| {
                slots
                    .ability(slot)
                    .and_then(|id| catalog.get(id))
                    .map(|definition| format!("{key}: {}", definition.name))
            })
            .collect::<Vec<_>>()
            .join(" · ");
        let statuses = statuses
            .iter()
            .map(|status| format!("{:?} {:.1}s", status.kind(), status.remaining_seconds()))
            .collect::<Vec<_>>()
            .join(" · ");
        hud.0 = format!(
            "KIVA / THE PROVING GROUNDS\nRight click: move / attack · A + click: attack move · S: stop\n{loadout}\nLeft click confirms target; self-cast skills release immediately\nY: camera lock · Space: center · Arrows / edges: pan · Scroll: zoom · Esc: pause\nHP {:.0}   Energy {:.0}   Q {:.1}  W {:.1}  E {:.1}  R {:.1}\nAD {:.1}   AP {:.1}   Armor {:.1}   MR {:.1}\nMove {:.2} units/s   Range {:.2} units   Attack speed {:.2}/s\n{statuses}{}",
            state.health(),
            state.mana(),
            slots.cooldown(0).unwrap_or(0.0),
            slots.cooldown(1).unwrap_or(0.0),
            slots.cooldown(2).unwrap_or(0.0),
            slots.cooldown(3).unwrap_or(0.0),
            attributes.attack_damage(),
            attributes.ability_power(),
            attributes.armor(),
            attributes.magic_resistance(),
            attributes.move_speed(),
            attributes.attack_range(),
            attributes.attack_speed(),
            if control.paused {
                "\nPAUSED · Press Escape to resume"
            } else {
                ""
            }
        );
    }
}
