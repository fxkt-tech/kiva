//! Read-only projections for the browser HUD and native debug overlay.
use super::{
    abilities::{AbilityMetadata, AbilityView},
    camera::{CameraRig, GameCamera},
    controls::{Aim, ClientFeedback, Pickable, PlayerController},
    session::{ClientSession, SelectionAck},
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
    character_id: Option<u16>,
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
    player_id: Option<String>,
    session: SessionView,
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

#[derive(Serialize)]
struct SessionView {
    phase: &'static str,
    selection: Option<SelectionView>,
}
#[derive(Serialize)]
struct SelectionView {
    choices: Vec<CharacterSummary>,
    selected: Option<CharacterPreview>,
    selection_revision: Option<u64>,
    selection_ack: SelectionAck,
    can_start: bool,
    error: Option<String>,
}
#[derive(Serialize)]
struct CharacterSummary {
    id: u16,
    name: String,
    description: String,
}
#[derive(Serialize)]
struct CharacterPreview {
    #[serde(flatten)]
    character: CharacterSummary,
    max_health: f32,
    max_mana: f32,
    attributes: AttributesView,
    abilities: Vec<AbilityMetadata>,
}
impl SessionView {
    fn new(
        session: &ClientSession,
        characters: &CharacterCatalog,
        abilities: &AbilityCatalog,
    ) -> Self {
        let Some(selection) = session.selection() else {
            return Self {
                phase: "Active",
                selection: None,
            };
        };
        let summary = |id: CharacterId, definition: &CharacterDefinition| CharacterSummary {
            id: id.0,
            name: definition.name.clone(),
            description: definition.description.clone(),
        };
        let selected = selection.selected.and_then(|choice| {
            let definition = characters.get(choice.character_id)?;
            let attributes = definition.initial_attributes()?;
            Some(CharacterPreview {
                character: summary(choice.character_id, definition),
                max_health: definition.stats.max_health,
                max_mana: definition.stats.max_mana,
                attributes: (&attributes).into(),
                abilities: definition
                    .default_loadout
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, id)| {
                        Some(AbilityMetadata::new(slot, abilities.get((*id)?)?))
                    })
                    .collect(),
            })
        });
        Self {
            phase: "Selecting",
            selection: Some(SelectionView {
                choices: characters
                    .iter()
                    .map(|(id, definition)| summary(id, definition))
                    .collect(),
                selected,
                selection_revision: selection.selected.map(|choice| choice.revision),
                selection_ack: selection.ack,
                can_start: selection.can_start(),
                error: selection.error.clone(),
            }),
        }
    }
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
    Option<&'static CharacterIdentity>,
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
    session: Res<ClientSession>,
    control: Res<SimulationControl>,
    tick: Res<SimulationTick>,
    rig: Res<CameraRig>,
    feedback: Res<ClientFeedback>,
    catalog: Res<AbilityCatalog>,
    characters: Res<CharacterCatalog>,
    map: Res<NavigationMap>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    units: Query<ProjectedUnit>,
) {
    if !session.is_changed() && time.elapsed_secs() < *next_publish {
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
            |(
                id,
                unit,
                state,
                pose,
                action,
                order,
                _,
                display,
                pickable,
                statuses,
                control,
                character,
            )| {
                let point = pose.position();
                UnitView {
                    id: id.to_bits().to_string(),
                    character_id: character.map(|identity| identity.id().0),
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
    let abilities = session
        .player()
        .and_then(|id| units.get(id).ok())
        .map(|(_, _, state, _, _, _, slots, _, _, _, control, _)| {
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
        player_id: session.player().map(|id| id.to_bits().to_string()),
        session: SessionView::new(&session, &characters, &catalog),
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
    session: Res<ClientSession>,
    control: Res<SimulationControl>,
    catalog: Res<AbilityCatalog>,
    units: Query<(&Unit, &UnitState, &AbilitySlots, &UnitStatuses)>,
    mut hud: Single<&mut Text, With<NativeHud>>,
) {
    if let Some(player) = session.player()
        && let Ok((unit, state, slots, statuses)) = units.get(player)
    {
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
    } else {
        hud.0 = "KIVA / 请选择角色".into();
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        content::{self, DemoSettings},
        session,
    };
    use super::*;

    #[test]
    fn both_role_previews_match_constructed_attributes_and_share_flat_runtime_skill_metadata() {
        for id in [CharacterId(1), CharacterId(2)] {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, SimulationPlugin));
            content::setup(app.world_mut());
            app.world_mut()
                .resource_mut::<DemoSettings>()
                .default_character = id;
            session::setup(app.world_mut());
            let view = SessionView::new(
                app.world().resource::<ClientSession>(),
                app.world().resource::<CharacterCatalog>(),
                app.world().resource::<AbilityCatalog>(),
            );
            let json = serde_json::to_value(view).unwrap();
            assert_eq!(json["phase"], "Selecting");
            assert_eq!(json["selection"]["selection_ack"]["request_id"], 0);
            assert_eq!(json["selection"]["selection_ack"]["accepted"], true);
            assert_eq!(json["selection"]["can_start"], true);
            let choices = json["selection"]["choices"].as_array().unwrap();
            assert_eq!(
                choices
                    .iter()
                    .map(|choice| choice["id"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                [1, 2]
            );
            let preview = &json["selection"]["selected"];
            assert_eq!(preview["id"], id.0);
            assert!(
                preview.get("health").is_none(),
                "a preview has no runtime health"
            );
            assert_eq!(
                app.world_mut().query::<&Unit>().iter(app.world()).count(),
                2,
                "previewing cannot create a hidden player"
            );
            assert_eq!(
                app.world_mut()
                    .query::<&CharacterIdentity>()
                    .iter(app.world())
                    .count(),
                0
            );

            session::start_native(app.world_mut());
            let player = app.world().resource::<ClientSession>().player().unwrap();
            let unit = app.world().get::<Unit>(player).unwrap();
            let state = app.world().get::<UnitState>(player).unwrap();
            let control = app.world().get::<ControlState>(player).unwrap();
            let slots = app.world().get::<AbilitySlots>(player).unwrap();
            assert_eq!(
                preview["attributes"],
                serde_json::to_value(AttributesView::from(unit.attributes())).unwrap()
            );
            assert_eq!(
                preview["max_health"].as_f64().unwrap() as f32,
                state.health()
            );
            assert_eq!(preview["max_mana"].as_f64().unwrap() as f32, state.mana());
            for metadata in preview["abilities"].as_array().unwrap() {
                let slot = metadata["slot"].as_u64().unwrap() as usize;
                let definition = app
                    .world()
                    .resource::<AbilityCatalog>()
                    .get(slots.ability(slot).unwrap())
                    .unwrap();
                let runtime =
                    serde_json::to_value(AbilityView::new(slot, definition, state, control, slots))
                        .unwrap();
                for (key, value) in metadata.as_object().unwrap() {
                    assert_eq!(
                        &runtime[key], value,
                        "preview/runtime metadata diverged at {key}"
                    );
                }
                assert!(metadata.get("cooldown").is_none());
                assert!(metadata.get("available").is_none());
                assert_eq!(runtime["cooldown"], 0.0);
                assert_eq!(runtime["available"], true);
                assert!(
                    runtime.get("metadata").is_none(),
                    "existing runtime JSON stays flat"
                );
            }
        }
    }

    #[test]
    fn active_projection_stops_repeating_character_choices_and_previews() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        content::setup(app.world_mut());
        session::setup(app.world_mut());
        session::start_native(app.world_mut());
        let view = SessionView::new(
            app.world().resource::<ClientSession>(),
            app.world().resource::<CharacterCatalog>(),
            app.world().resource::<AbilityCatalog>(),
        );
        let json = serde_json::to_value(view).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"phase":"Active", "selection":null})
        );
    }
}
