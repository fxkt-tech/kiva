//! Read-only projections for the browser HUD and native debug overlay.
use super::{
    abilities::{AbilityMetadata, AbilityView},
    camera::{CameraRig, GameCamera},
    controls::{Aim, ClientFeedback, Pickable, PlayerController},
    equipment::{self, EquipmentAdapter, EquipmentDefinitionView, EquipmentView},
    match_view::{self, MapView, MatchProjection, MatchView},
    session::{ClientSession, SelectionAck},
    unit_appearance::UnitAppearance,
    visibility::VisibleWorld,
    web,
};
use crate::{
    gameplay::{UnitRole, WaveMember},
    simulation::*,
};
use bevy::{ecs::system::SystemParam, prelude::*};
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

impl AttributesView {
    fn display(attributes: &EffectiveAttributes) -> Self {
        let mut view = Self::from(attributes);
        view.move_speed = crate::gameplay::units::league_units(view.move_speed);
        view.attack_range = crate::gameplay::units::league_units(view.attack_range);
        view
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
    role: Option<&'static str>,
    wave: Option<u64>,
    formation_slot: Option<u8>,
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
    display_attributes: AttributesView,
    base_attack_speed: f32,
    passives: Vec<String>,
    shield: f32,
    focus: u8,
    empowered: bool,
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
    #[serde(rename = "match")]
    game: MatchView,
    map: MapView,
    selected: Option<String>,
    aim: Option<usize>,
    equipment_aim: Option<usize>,
    equipment: EquipmentView,
    equipment_catalog: Option<Vec<EquipmentDefinitionView>>,
    attack_move: bool,
    units: Vec<UnitView>,
    objects: Vec<ObjectView>,
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
    display_attributes: AttributesView,
    base_attack_speed: f32,
    passives: Vec<String>,
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
            let attributes = definition
                .initial_attributes_with_loadout(abilities, &definition.default_loadout)?;
            Some(CharacterPreview {
                character: summary(choice.character_id, definition),
                max_health: definition.stats.max_health,
                max_mana: definition.stats.max_mana,
                attributes: (&attributes).into(),
                display_attributes: AttributesView::display(&attributes),
                base_attack_speed: definition.stats.base_attributes.attack_speed,
                passives: super::abilities::describe_passives(&definition.stats),
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
    (
        Option<&'static CharacterIdentity>,
        Option<&'static UnitRole>,
        Option<&'static WaveMember>,
        Option<&'static UnitAppearance>,
        Option<&'static UnitMechanics>,
    ),
);

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct LabelPosition {
    id: String,
    unit: LabelUnit,
    screen: Option<[f32; 2]>,
    body_screen: Option<[f32; 2]>,
    alive: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct LabelUnit {
    name: String,
    team: &'static str,
    role: Option<&'static str>,
    health: f32,
    max_health: f32,
}

#[cfg(target_arch = "wasm32")]
type DisplayedLabel<'a> = (
    Entity,
    &'a GlobalTransform,
    &'a Pickable,
    &'a UnitState,
    &'a UnitAppearance,
    &'a Unit,
    Option<&'a UnitRole>,
);

/// Visible positions and minimal label metadata cross this bridge every frame.
/// Newly observed units gain a label immediately; hidden identities stay local.
#[cfg(target_arch = "wasm32")]
pub(super) fn publish_labels(
    visible: Res<VisibleWorld>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    units: Query<DisplayedLabel<'_>, With<super::scene::UnitVisual>>,
) {
    let (camera, camera_transform) = *camera;
    let Some(viewport) = camera.logical_viewport_size() else {
        return;
    };
    let positions: Vec<_> = units
        .iter()
        .filter(|(id, ..)| visible.contains(*id))
        .map(
            |(id, transform, pickable, state, appearance, unit, role)| LabelPosition {
                unit: LabelUnit {
                    name: unit.name().into(),
                    team: if unit.team() == Team::Ally {
                        "ally"
                    } else {
                        "enemy"
                    },
                    role: role.copied().map(match_view::role_name),
                    health: state.health(),
                    max_health: unit.definition().max_health,
                },
                id: id.to_bits().to_string(),
                screen: project_display(
                    camera,
                    camera_transform,
                    transform,
                    appearance.label_offset,
                    viewport,
                ),
                body_screen: project_display(
                    camera,
                    camera_transform,
                    transform,
                    pickable.offset,
                    viewport,
                ),
                alive: state.alive(),
            },
        )
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

#[derive(Serialize)]
struct ObjectView {
    id: String,
    kind: &'static str,
    team: &'static str,
    owner: Option<String>,
    position: [f32; 2],
    radius: f32,
    remaining: f32,
    active: bool,
}

#[derive(SystemParam)]
pub(super) struct ObservedProjection<'w, 's> {
    visible: Res<'w, VisibleWorld>,
    units: Query<'w, 's, ProjectedUnit>,
    objects: Query<'w, 's, (Entity, &'static BattlefieldObject)>,
}
impl ObservedProjection<'_, '_> {
    fn objects(&self, tick: u64) -> Vec<ObjectView> {
        self.objects
            .iter()
            .filter(|(id, _)| self.visible.contains(*id))
            .map(|(id, object)| ObjectView {
                id: id.to_bits().to_string(),
                kind: match object.kind() {
                    ObjectKind::Standard => "Standard",
                    ObjectKind::Terrain => "Terrain",
                    ObjectKind::Scout => "Scout",
                    ObjectKind::Reveal => "Reveal",
                },
                team: if object.team() == Team::Ally {
                    "ally"
                } else {
                    "enemy"
                },
                owner: self
                    .visible
                    .contains(object.owner().entity())
                    .then(|| object.owner().entity().to_bits().to_string()),
                position: object.position().to_array(),
                radius: object.radius(),
                remaining: object.remaining_seconds(tick),
                active: object.active(tick),
            })
            .collect()
    }
}

#[derive(SystemParam)]
pub(super) struct EquipmentProjection<'w, 's> {
    adapter: Res<'w, EquipmentAdapter>,
    catalog: Res<'w, EquipmentCatalog>,
    units: Query<
        'w,
        's,
        (
            &'static EquipmentSlots,
            &'static UnitState,
            &'static ControlState,
        ),
    >,
    sent_catalog: Local<'s, bool>,
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
    mut equipment: EquipmentProjection,
    game: MatchProjection,
    observed: ObservedProjection,
) {
    if !session.is_changed()
        && !game.changed()
        && !equipment.adapter.is_changed()
        && time.elapsed_secs() < *next_publish
    {
        return;
    }
    *next_publish = time.elapsed_secs() + 0.08;
    let (camera, transform) = *camera;
    let viewport = camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(window.width(), window.height()));
    let units = &observed.units;
    let views = units
        .iter()
        .filter(|(id, ..)| observed.visible.contains(*id))
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
                (character, role, wave, appearance, mechanics),
            )| {
                let point = pose.position();
                UnitView {
                    id: id.to_bits().to_string(),
                    character_id: character.map(|identity| identity.id().0),
                    role: role.copied().map(match_view::role_name),
                    wave: wave.map(WaveMember::wave),
                    formation_slot: wave.map(WaveMember::slot),
                    name: unit.name().into(),
                    team: if unit.team() == Team::Ally {
                        "ally"
                    } else {
                        "enemy"
                    },
                    position: point.to_array(),
                    screen: project_display(
                        camera,
                        transform,
                        display,
                        appearance.map_or(Vec3::Y * 2.2, |appearance| appearance.label_offset),
                        viewport,
                    ),
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
                    display_attributes: AttributesView::display(unit.attributes()),
                    base_attack_speed: unit.definition().base_attributes.attack_speed,
                    passives: super::abilities::describe_passives(unit.definition()),
                    shield: mechanics.map_or(0.0, UnitMechanics::shield),
                    focus: mechanics.map_or(0, UnitMechanics::focus),
                    empowered: mechanics.is_some_and(UnitMechanics::empowered),
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
        .map(
            |(_, _, state, _, _, _, slots, _, _, _, control, (_, _, _, _, mechanics))| {
                (0..4)
                    .filter_map(|slot| {
                        let definition = catalog.get(slots.ability(slot)?)?;
                        Some(
                            AbilityView::new(slot, definition, state, control, slots)
                                .with_mechanics(
                                    definition, state, control, mechanics, slot, tick.0,
                                ),
                        )
                    })
                    .collect()
            },
        )
        .unwrap_or_default();
    let view = HudView {
        tick: tick.0,
        paused: control.paused,
        player_id: session.player().map(|id| id.to_bits().to_string()),
        session: SessionView::new(&session, &characters, &catalog),
        game: game.view(tick.0),
        map: game.map_view(),
        selected: player.selected.map(|id| id.to_bits().to_string()),
        aim: match player.aim {
            Some(Aim::Ability(slot)) => Some(slot),
            _ => None,
        },
        equipment_aim: match player.aim {
            Some(Aim::Equipment { slot, .. }) => Some(slot),
            _ => None,
        },
        equipment: EquipmentView::new(
            session.player().and_then(|id| equipment.units.get(id).ok()),
            &equipment.catalog,
            &catalog,
            equipment::paused_unlocked(&control) && game.can_manage(),
            &equipment.adapter.ack,
        ),
        equipment_catalog: (!*equipment.sent_catalog)
            .then(|| equipment::catalog_view(&equipment.catalog, &catalog)),
        attack_move: matches!(player.aim, Some(Aim::AttackMove)),
        units: views,
        objects: observed.objects(tick.0),
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
        *equipment.sent_catalog = true;
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
            "KIVA / THE PROVING GROUNDS\nRight click: move / attack · A + click: attack move · S: stop\n{loadout}\nLeft click confirms target; self-cast skills release immediately\nY: camera lock · Space: center · Arrows / edges: pan · Scroll: zoom · Esc: pause\nHP {:.0}   Energy {:.0}   Q {:.1}  W {:.1}  E {:.1}  R {:.1}\nAD {:.1}   AP {:.1}   Armor {:.1}   MR {:.1}\nMove {:.0}   Range {:.0}   Attack speed {:.2}/s\n{statuses}{}",
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
            crate::gameplay::units::league_units(attributes.move_speed()),
            crate::gameplay::units::league_units(attributes.attack_range()),
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
        content::{self, SelectionSettings},
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
                .resource_mut::<SelectionSettings>()
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
                0,
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
            assert_eq!(
                preview["display_attributes"],
                serde_json::to_value(AttributesView::display(unit.attributes())).unwrap()
            );
            assert_eq!(
                preview["display_attributes"]["move_speed"],
                if id == CharacterId(1) { 340.0 } else { 325.0 }
            );
            assert_eq!(
                preview["display_attributes"]["attack_range"],
                if id == CharacterId(1) { 175.0 } else { 600.0 }
            );
            assert!(!preview["passives"].as_array().unwrap().is_empty());

            for metadata in preview["abilities"].as_array().unwrap() {
                let slot = metadata["slot"].as_u64().unwrap() as usize;
                let definition = app
                    .world()
                    .resource::<AbilityCatalog>()
                    .get(slots.ability(slot).unwrap())
                    .unwrap();
                let runtime = serde_json::to_value(
                    AbilityView::new(slot, definition, state, control, slots).with_mechanics(
                        definition,
                        state,
                        control,
                        app.world().get::<UnitMechanics>(player),
                        slot,
                        0,
                    ),
                )
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
                assert_eq!(
                    runtime["available"],
                    !(id == CharacterId(2) && slot == 0),
                    "zero-focus Q alone starts unavailable"
                );
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
