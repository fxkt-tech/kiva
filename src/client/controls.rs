//! Human and demo AI adapters both submit the public simulation commands.
use super::{
    abilities,
    ability_targeting::{self, TargetState},
    camera::GameCamera,
    picking,
    pointer::GamePointer,
    session::ClientSession,
    web,
};
use crate::simulation::*;
use bevy::prelude::*;
use serde::Deserialize;

mod casting;
use casting::{CastHotkey, CastInput, CastMode};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) enum Aim {
    AttackMove,
    Ability(usize),
    Equipment {
        slot: usize,
        instance: EquipmentInstanceId,
    },
}
impl Aim {
    pub fn source(self) -> Option<CastSource> {
        match self {
            Self::AttackMove => None,
            Self::Ability(slot) => Some(CastSource::AbilitySlot(slot)),
            Self::Equipment { slot, instance } => {
                Some(CastSource::EquipmentSlot { slot, instance })
            }
        }
    }
    fn from_source(source: CastSource) -> Self {
        match source {
            CastSource::AbilitySlot(slot) => Self::Ability(slot),
            CastSource::EquipmentSlot { slot, instance } => Self::Equipment { slot, instance },
        }
    }
}
#[derive(Resource, Default)]
pub(super) struct PlayerController {
    pub selected: Option<Entity>,
    pub aim: Option<Aim>,
    casting: CastInput,
}
#[derive(Resource, Default)]
pub(super) struct PointerState {
    pub position: Option<Vec2>,
    pub ground: Option<Vec2>,
    pub hovered: Option<Entity>,
    pub candidates: Vec<Entity>,
    pub over_ui: bool,
}
#[derive(Resource, Default)]
pub(super) struct ClientFeedback {
    pub message: String,
    pub expires: f32,
    pub marker: Option<(Vec2, bool, f32)>,
}

fn submit(world: &mut World, player: Entity, command: UnitCommand) {
    world.resource_mut::<CommandQueue>().submit(player, command);
}

fn set_paused(world: &mut World, paused: bool) {
    let player = world.resource::<ClientSession>().player();
    let paused = paused
        || player.is_none()
        || !world
            .resource::<crate::gameplay::MatchState>()
            .can_advance();
    let was_paused = world.resource::<SimulationControl>().paused;
    world.resource_mut::<SimulationControl>().paused = paused;
    web::set_paused(paused);
    if paused {
        if !was_paused {
            web::release_pointer();
            world.resource_mut::<CommandQueue>().clear();
            if let Some(player) = player {
                submit(world, player, UnitCommand::Stop);
            }
        }
        if player.is_none() {
            world.resource_mut::<CommandQueue>().clear();
            world.resource_mut::<PlayerController>().selected = None;
            *world.resource_mut::<ClientFeedback>() = ClientFeedback::default();
        }
        world.resource_mut::<PlayerController>().aim = None;
        world
            .resource_mut::<PlayerController>()
            .casting
            .clear_modifiers();
        *world.resource_mut::<PointerState>() = PointerState::default();
    }
}

fn update_pointer(world: &mut World, cursor: Option<Vec2>) {
    let over_ui = cursor.is_some_and(web::pointer_over_ui);
    let ray = cursor.and_then(|cursor| {
        let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<GameCamera>>();
        let (camera, transform) = cameras.single(world).ok()?;
        let viewport = camera.logical_viewport_size()?;
        camera
            .viewport_to_world(transform, web::css_to_viewport(cursor, viewport))
            .ok()
    });
    let ground = ray.and_then(|ray| {
        let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
        let point = ray.get_point(distance);
        Some(Vec2::new(point.x, point.z))
    });
    let candidates = if over_ui {
        Vec::new()
    } else {
        cursor
            .map(|point| picking::candidates(world, point))
            .unwrap_or_default()
    };
    let player = world.resource::<ClientSession>().player();
    let hovered = player
        .and_then(|player| {
            candidates
                .iter()
                .copied()
                .find(|target| valid_enemy_pick(world, player, *target))
        })
        .or_else(|| candidates.first().copied());
    *world.resource_mut::<PointerState>() = PointerState {
        position: cursor,
        ground,
        hovered,
        candidates,
        over_ui,
    };
}

fn cast_info(world: &World, player: Entity, source: CastSource) -> Option<abilities::CastInfo<'_>> {
    abilities::resolve(
        source,
        world.get::<AbilitySlots>(player)?,
        world.get::<EquipmentSlots>(player)?,
        world.resource::<AbilityCatalog>(),
        world.resource::<EquipmentCatalog>(),
    )
}

fn cast_unavailable(world: &World, player: Entity, source: CastSource) -> Option<&'static str> {
    let Some(state) = world.get::<UnitState>(player) else {
        return Some("无法操控当前单位");
    };
    let Some(control) = world.get::<ControlState>(player) else {
        return Some("当前单位尚未准备就绪");
    };
    let Some(info) = cast_info(world, player, source) else {
        return Some(match source {
            CastSource::AbilitySlot(_) => "该槽位没有装配技能",
            _ => "该装备没有可用主动技能",
        });
    };
    match source {
        CastSource::AbilitySlot(slot) => abilities::runtime_unavailable_reason(
            state,
            control,
            info.cooldown,
            info.definition,
            world.get::<UnitMechanics>(player),
            slot,
            world.resource::<SimulationTick>().0,
        ),
        _ => abilities::unavailable_reason(state, control, info.cooldown, info.definition),
    }
}

fn feedback(world: &mut World, reason: &'static str) {
    let expires = world.resource::<Time>().elapsed_secs() + 2.0;
    let mut feedback = world.resource_mut::<ClientFeedback>();
    feedback.message = reason.into();
    feedback.expires = expires;
}

fn submit_cast(world: &mut World, player: Entity, source: CastSource, target: AbilityTarget) {
    submit(
        world,
        player,
        match source {
            CastSource::AbilitySlot(slot) => UnitCommand::CastAbility { slot, target },
            CastSource::EquipmentSlot { slot, instance } => UnitCommand::UseEquipment {
                slot,
                instance,
                target,
            },
        },
    );
}

fn select_cast(world: &mut World, source: CastSource) {
    let Some(player) = world.resource::<ClientSession>().player() else {
        return;
    };
    if let Some(reason) = cast_unavailable(world, player, source) {
        feedback(world, reason);
        return;
    }
    let recast = matches!(source,CastSource::AbilitySlot(slot) if world.get::<UnitMechanics>(player).is_some_and(|state|state.recast(slot)));
    if recast
        || cast_info(world, player, source)
            .is_some_and(|info| info.definition.targeting == Targeting::SelfCast)
    {
        world.resource_mut::<PlayerController>().aim = None;
        submit_cast(world, player, source, AbilityTarget::SelfCast);
    } else {
        world.resource_mut::<PlayerController>().aim = Some(Aim::from_source(source));
    }
}

fn select_ability(world: &mut World, slot: usize) {
    select_cast(world, CastSource::AbilitySlot(slot));
}

fn cast_at_pointer(world: &mut World, player: Entity, source: CastSource) -> bool {
    let pointer = world.resource::<PointerState>();
    if pointer.over_ui {
        return false;
    }
    let intent = ability_targeting::resolve_intent(
        world,
        player,
        source,
        pointer.ground,
        &pointer.candidates,
    );
    if let Some(target) = intent.as_ref().and_then(|intent| intent.submission()) {
        submit_cast(world, player, source, target);
        true
    } else {
        if let Some(ability_targeting::TargetIntent {
            state: TargetState::Invalid(reason),
            ..
        }) = intent
        {
            feedback(world, rejection_message(&reason));
        }
        false
    }
}

fn cast_hotkey(world: &mut World, player: Entity, hotkey: CastHotkey) {
    let mode = world.resource::<PlayerController>().casting.mode();
    if mode == CastMode::Quick {
        world.resource_mut::<PlayerController>().aim = None;
        if world.resource::<PointerState>().over_ui {
            return;
        }
    }
    let source = match hotkey {
        CastHotkey::Ability(slot) => CastSource::AbilitySlot(slot),
        CastHotkey::Equipment(slot) => match equipment_source(world, player, slot, None) {
            Ok(source) => source,
            Err(reason) => {
                feedback(world, reason);
                return;
            }
        },
    };
    select_cast(world, source);
    if mode == CastMode::Quick {
        // Self casts and recasts were already submitted by select_cast.
        if world.resource::<PlayerController>().aim.is_some() {
            cast_at_pointer(world, player, source);
        }
        world.resource_mut::<PlayerController>().aim = None;
    }
}

fn equipment_source(
    world: &World,
    player: Entity,
    slot: usize,
    expected: Option<&str>,
) -> Result<CastSource, &'static str> {
    let item = world
        .get::<EquipmentSlots>(player)
        .and_then(|slots| slots.item(slot))
        .ok_or("该装备格为空")?;
    if expected.is_some_and(|expected| expected != item.instance().to_string()) {
        return Err("装备已经变更，请重新选择");
    }
    Ok(CastSource::EquipmentSlot {
        slot,
        instance: item.instance(),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EquipmentSkillRequest {
    slot: usize,
    instance_id: String,
}

fn queued_equipment_request(world: &mut World, raw: Option<&str>, just_captured: bool) {
    if just_captured || world.resource::<SimulationControl>().paused {
        return;
    }
    let Some(player) = world.resource::<ClientSession>().player() else {
        return;
    };
    let Some(raw) = raw else {
        return;
    };
    let Ok(request) = serde_json::from_str::<EquipmentSkillRequest>(raw) else {
        feedback(world, "装备释放请求格式无效");
        return;
    };
    match equipment_source(world, player, request.slot, Some(&request.instance_id)) {
        Ok(source) => select_cast(world, source),
        Err(reason) => feedback(world, reason),
    }
}

/// A DOM request may have been queued before capture completed. It belongs to
/// that menu session, just like the window events discarded at the boundary.
fn queued_skill_request(world: &mut World, slot: Option<usize>, just_captured: bool) {
    if !just_captured
        && !world.resource::<SimulationControl>().paused
        && let Some(slot) = slot
    {
        select_ability(world, slot);
    }
}

fn sync_pointer(
    world: &mut World,
    capture: Option<web::PointerCapture>,
    paused: bool,
    size: Vec2,
    cursor: Option<Vec2>,
) -> (Option<bool>, bool) {
    let captured = capture.map(web::PointerCapture::confirmed);
    web::sync_game_pointer(captured.unwrap_or(false) && !paused);
    let just_captured = world.resource_mut::<GamePointer>().sync(
        captured.unwrap_or(false) && !paused,
        size,
        cursor,
    );
    (captured, just_captured)
}

fn valid_enemy_pick(world: &World, player: Entity, target: Entity) -> bool {
    world.get::<UnitState>(target).is_some_and(UnitState::alive)
        && world
            .get::<Unit>(player)
            .is_some_and(|player| team_can_see(world, player.team(), target))
        && world
            .get::<Unit>(target)
            .zip(world.get::<Unit>(player))
            .is_some_and(|(target, player)| target.team() != player.team())
}

/// New control must invalidate an existing aim even without a new input event.
pub(super) fn refresh_aim(world: &mut World) {
    let Some(unit) = world.resource::<ClientSession>().player() else {
        world.resource_mut::<PlayerController>().aim = None;
        return;
    };
    let team = world.get::<Unit>(unit).map(Unit::team);
    let selected = world.resource::<PlayerController>().selected;
    if selected.is_some_and(|selected| team.is_none_or(|team| !team_can_see(world, team, selected)))
    {
        world.resource_mut::<PlayerController>().selected = None;
    }
    let controller = world.resource::<PlayerController>();
    let aim = controller.aim;
    let unavailable = match aim {
        Some(aim @ (Aim::Ability(_) | Aim::Equipment { .. })) => {
            cast_unavailable(world, unit, aim.source().expect("cast aim")).is_some()
        }
        Some(Aim::AttackMove) => world
            .get::<ControlState>(unit)
            .is_none_or(|control| !control.can_move()),
        None => false,
    };
    if unavailable {
        world.resource_mut::<PlayerController>().aim = None;
    }
}

fn key_pressed(world: &mut World, key: KeyCode) {
    if key == KeyCode::Escape {
        if let Some(captured) = web::pointer_captured() {
            if captured {
                set_paused(world, true);
            }
            return;
        }
        let mut player = world.resource_mut::<PlayerController>();
        if player.aim.is_some() {
            player.aim = None;
        } else if player.selected.is_some() {
            player.selected = None;
        } else {
            let paused = world.resource::<SimulationControl>().paused;
            set_paused(world, !paused);
        }
        return;
    }
    if world.resource::<SimulationControl>().paused {
        return;
    }
    let Some(player) = world.resource::<ClientSession>().player() else {
        return;
    };
    match key {
        KeyCode::KeyS => {
            submit(world, player, UnitCommand::Stop);
            world.resource_mut::<PlayerController>().aim = None;
        }
        KeyCode::KeyA => world.resource_mut::<PlayerController>().aim = Some(Aim::AttackMove),
        _ => {
            if let Some((hotkey, _)) = CastHotkey::from_key(key) {
                cast_hotkey(world, player, hotkey);
            }
        }
    }
}

fn mouse_pressed(world: &mut World, button: MouseButton) {
    if world.resource::<SimulationControl>().paused {
        return;
    }
    let pointer = world.resource::<PointerState>();
    let (ground, hovered, over_ui) = (pointer.ground, pointer.hovered, pointer.over_ui);
    if over_ui {
        if button == MouseButton::Left {
            if let Some(point) = pointer.position {
                web::click_ui(point);
            }
            if let Some(slot) = web::take_skill_request() {
                select_ability(world, slot);
            }
            let equipment_request = web::take_equipment_skill_request();
            queued_equipment_request(world, equipment_request.as_deref(), false);
        }
        return;
    }
    let Some(player) = world.resource::<ClientSession>().player() else {
        return;
    };
    let enemy = world
        .resource::<PointerState>()
        .candidates
        .iter()
        .copied()
        .find(|target| valid_enemy_pick(world, player, *target));
    match button {
        MouseButton::Right => {
            world.resource_mut::<PlayerController>().aim = None;
            if let Some(target) = enemy {
                submit(world, player, UnitCommand::AttackTarget(target));
                world.resource_mut::<PlayerController>().selected = Some(target);
            } else if let Some(point) = ground {
                submit(world, player, UnitCommand::MoveTo(point));
            }
        }
        MouseButton::Left => {
            let aim = world.resource::<PlayerController>().aim;
            match aim {
                Some(Aim::AttackMove) => {
                    if let Some(target) = enemy {
                        submit(world, player, UnitCommand::AttackTarget(target));
                        world.resource_mut::<PlayerController>().selected = Some(target);
                    } else if let Some(point) = ground {
                        submit(world, player, UnitCommand::AttackMove(point));
                    }
                }
                Some(aim @ (Aim::Ability(_) | Aim::Equipment { .. })) => {
                    let source = aim.source().expect("cast aim");
                    if !cast_at_pointer(world, player, source) {
                        return;
                    }
                }
                None => world.resource_mut::<PlayerController>().selected = hovered,
            }
            world.resource_mut::<PlayerController>().aim = None;
        }
        _ => {}
    }
}

/// Preserve the unified window event order. Aggregated just_pressed flags lose
/// ordering when e.g. Stop followed by Q arrives within one rendered frame.
pub(super) fn collect_input(
    world: &mut World,
    mut cursor: Local<bevy::ecs::message::MessageCursor<bevy::window::WindowEvent>>,
) {
    use bevy::window::WindowEvent;
    let (focused, current_cursor, size) = {
        let mut windows = world.query::<&Window>();
        let Ok(window) = windows.single(world) else {
            return;
        };
        (
            window.focused,
            window.cursor_position(),
            Vec2::new(window.width(), window.height()),
        )
    };
    let size = web::canvas_size().unwrap_or(size);
    let capture = web::pointer_capture();
    if let Some(mode) = web::cast_mode().as_deref().and_then(CastMode::parse) {
        world.resource_mut::<PlayerController>().casting.mode = mode;
    }
    let focused = web::document_focused().unwrap_or(focused);
    let paused = world.resource::<ClientSession>().player().is_none()
        || !focused
        || capture.map(web::PointerCapture::confirmed) == Some(false)
        || web::paused().unwrap_or(world.resource::<SimulationControl>().paused);
    set_paused(world, paused);
    let (_, just_captured) = sync_pointer(world, capture, paused, size, current_cursor);
    let mut events: Vec<_> = cursor
        .read(world.resource::<Messages<WindowEvent>>())
        .cloned()
        .collect();
    // Motion used to reach Enter/resume belongs to the unlocked session.
    if just_captured || world.resource::<ClientSession>().player().is_none() {
        events.clear();
        web::clear_pointer_presses();
        world
            .resource_mut::<PlayerController>()
            .casting
            .clear_modifiers();
    }
    for event in events {
        match event {
            WindowEvent::CursorMoved(event) if capture.is_none() => {
                world
                    .resource_mut::<GamePointer>()
                    .absolute(Some(event.position));
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
            }
            WindowEvent::CursorLeft(_) if capture.is_none() => {
                world.resource_mut::<GamePointer>().absolute(None);
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
            }
            WindowEvent::KeyboardInput(event) if focused && !event.repeat => {
                world
                    .resource_mut::<PlayerController>()
                    .casting
                    .key_changed(event.key_code, event.state.is_pressed());
                if !event.state.is_pressed() {
                    continue;
                }
                if let Some((_, code)) = CastHotkey::from_key(event.key_code) {
                    if capture.is_some() {
                        let Some(press) = web::take_key_press(code) else {
                            continue;
                        };
                        world
                            .resource_mut::<GamePointer>()
                            .observe(press.pointer.position);
                        world
                            .resource_mut::<PlayerController>()
                            .casting
                            .sync_shift(press.shift);
                        if press.pointer.captured_by_ui {
                            continue;
                        }
                    }
                    let point = world.resource::<GamePointer>().position();
                    update_pointer(world, point);
                }
                key_pressed(world, event.key_code);
            }
            WindowEvent::MouseButtonInput(event) if event.state.is_pressed() => {
                if capture.is_some() {
                    let Some(press) = web::take_pointer_press(event.button) else {
                        continue;
                    };
                    world.resource_mut::<GamePointer>().observe(press.position);
                    if press.captured_by_ui {
                        continue;
                    }
                }
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
                mouse_pressed(world, event.button);
            }
            _ => {}
        }
    }
    let skill_request = web::take_skill_request();
    queued_skill_request(world, skill_request, just_captured);
    let equipment_request = web::take_equipment_skill_request();
    queued_equipment_request(world, equipment_request.as_deref(), just_captured);
    if !world.resource::<SimulationControl>().paused {
        if let Some(point) = web::game_pointer_position() {
            world.resource_mut::<GamePointer>().observe(point);
        }
        let point = world.resource::<GamePointer>().position();
        update_pointer(world, point);
    } else {
        *world.resource_mut::<PointerState>() = PointerState::default();
        web::clear_pointer_presses();
    }
}

fn rejection_message(reason: &Rejection) -> &'static str {
    match reason {
        Rejection::NoPath => "No route to that location",
        Rejection::OutOfRange => "Outside cast range",
        Rejection::OnCooldown => "Ability is cooling down",
        Rejection::NoCharges => "侦察充能不足",
        Rejection::InsufficientStacks => "专注层数不足，请先普攻积攒",
        Rejection::InsufficientMana => "Not enough energy",
        Rejection::Dead => "Waiting to respawn",
        Rejection::Busy => "Action already in progress",
        Rejection::InvalidDamage => "Ability damage cannot be resolved",
        Rejection::Controlled => "受控制限制，当前操作不可用",
        _ => "Choose a valid target",
    }
}

pub(super) fn receive_feedback(
    time: Res<Time>,
    session: Res<ClientSession>,
    pointer: Res<PointerState>,
    mut events: MessageReader<SimulationEvent>,
    mut feedback: ResMut<ClientFeedback>,
) {
    for event in events.read() {
        match event {
            SimulationEvent::TerrainPlacementFailed { unit, .. }
                if Some(*unit) == session.player() =>
            {
                feedback.message = "附近空间不足，围墙未生成".into();
                feedback.expires = time.elapsed_secs() + 2.0;
            }
            SimulationEvent::MoveOrdered { unit, destination }
                if Some(*unit) == session.player() =>
            {
                feedback.marker = Some((*destination, true, time.elapsed_secs()));
            }
            SimulationEvent::CommandRejected { unit, reason }
                if Some(*unit) == session.player() =>
            {
                feedback.message = rejection_message(reason).into();
                feedback.expires = time.elapsed_secs() + 2.0;
                if let Some(point) = pointer.ground {
                    feedback.marker = Some((point, false, time.elapsed_secs()));
                }
            }
            _ => {}
        }
    }
}
