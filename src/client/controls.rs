//! Human and demo AI adapters both submit the public simulation commands.
use super::{abilities, camera::GameCamera, pointer::GamePointer, session::ClientSession, web};
use crate::simulation::*;
use bevy::{
    prelude::*,
    shape::{Aabb3d, RayCast3d},
};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) enum Aim {
    AttackMove,
    Ability(usize),
}
#[derive(Resource, Default)]
pub(super) struct PlayerController {
    pub selected: Option<Entity>,
    pub aim: Option<Aim>,
}
#[derive(Resource, Default)]
pub(super) struct PointerState {
    pub position: Option<Vec2>,
    pub ground: Option<Vec2>,
    pub hovered: Option<Entity>,
    pub over_ui: bool,
}
#[derive(Component)]
pub(super) struct Pickable {
    pub offset: Vec3,
    pub half_size: Vec3,
}
#[derive(Resource, Default)]
pub(super) struct ClientFeedback {
    pub message: String,
    pub expires: f32,
    pub marker: Option<(Vec2, bool, f32)>,
}
#[derive(Component)]
pub(super) struct Patrol {
    pub points: [Vec2; 2],
    pub next: usize,
}

fn submit(world: &mut World, player: Entity, command: UnitCommand) {
    world.resource_mut::<CommandQueue>().submit(player, command);
}

fn set_paused(world: &mut World, paused: bool) {
    let player = world.resource::<ClientSession>().player();
    let paused = paused || player.is_none();
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
    let hovered = (!over_ui)
        .then(|| ray.and_then(|ray| hovered_unit(world, ray)))
        .flatten();
    *world.resource_mut::<PointerState>() = PointerState {
        position: cursor,
        ground,
        hovered,
        over_ui,
    };
}

/// Pick the body that was actually displayed, including airborne height and
/// the same root transform used for its label. Simulation still validates hits.
fn hovered_unit(world: &mut World, ray: Ray3d) -> Option<Entity> {
    let cast = RayCast3d::from_ray(ray, 200.0);
    world
        .query::<(Entity, &UnitState, &GlobalTransform, &Pickable)>()
        .iter(world)
        .filter(|(_, state, _, _)| state.alive())
        .filter_map(|(id, _, transform, bounds)| {
            let center = transform.transform_point(bounds.offset);
            let half_size = transform.affine().matrix3.abs() * bounds.half_size;
            cast.aabb_intersection_at(&Aabb3d::new(center, half_size))
                .map(|distance| (id, distance))
        })
        .min_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
        })
        .map(|(id, _)| id)
}

fn skill_unavailable(world: &World, player: Entity, slot: usize) -> Option<&'static str> {
    let Some(state) = world.get::<UnitState>(player) else {
        return Some("无法操控当前单位");
    };
    let (Some(control), Some(slots)) = (
        world.get::<ControlState>(player),
        world.get::<AbilitySlots>(player),
    ) else {
        return Some("当前单位尚未准备就绪");
    };
    let Some(definition) = slots
        .ability(slot)
        .and_then(|id| world.resource::<AbilityCatalog>().get(id))
    else {
        return Some("该槽位没有装配技能");
    };
    abilities::unavailable_reason(state, control, slots, slot, definition)
}

fn select_ability(world: &mut World, slot: usize) {
    let Some(player) = world.resource::<ClientSession>().player() else {
        return;
    };
    if let Some(reason) = skill_unavailable(world, player, slot) {
        let expires = world.resource::<Time>().elapsed_secs() + 2.0;
        let mut feedback = world.resource_mut::<ClientFeedback>();
        feedback.message = reason.into();
        feedback.expires = expires;
        return;
    }
    let definition = world
        .get::<AbilitySlots>(player)
        .and_then(|slots| slots.ability(slot))
        .and_then(|id| world.resource::<AbilityCatalog>().get(id));
    if let Some(definition) = definition {
        if definition.targeting == Targeting::SelfCast {
            world.resource_mut::<PlayerController>().aim = None;
            submit(
                world,
                player,
                UnitCommand::CastAbility {
                    slot,
                    target: AbilityTarget::SelfCast,
                },
            );
        } else {
            world.resource_mut::<PlayerController>().aim = Some(Aim::Ability(slot));
        }
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
    let controller = world.resource::<PlayerController>();
    let aim = controller.aim;
    let unavailable = match aim {
        Some(Aim::Ability(slot)) => skill_unavailable(world, unit, slot).is_some(),
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
            if let Some(slot) = [KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR]
                .iter()
                .position(|code| *code == key)
            {
                select_ability(world, slot);
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
        }
        return;
    }
    let Some(player) = world.resource::<ClientSession>().player() else {
        return;
    };
    match button {
        MouseButton::Right => {
            world.resource_mut::<PlayerController>().aim = None;
            if let Some(target) = hovered.filter(|target| valid_enemy_pick(world, player, *target))
            {
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
                    if let Some(point) = ground {
                        submit(world, player, UnitCommand::AttackMove(point));
                    }
                }
                Some(Aim::Ability(slot)) => {
                    let definition = world
                        .get::<AbilitySlots>(player)
                        .and_then(|slots| slots.ability(slot))
                        .and_then(|id| world.resource::<AbilityCatalog>().get(id));
                    let target = definition.and_then(|definition| match definition.targeting {
                        Targeting::Direction => ground.and_then(|point| {
                            let direction = point
                                - world
                                    .get::<UnitPose>(player)
                                    .expect("player pose")
                                    .position();
                            (direction.length_squared() > 1e-6)
                                .then_some(AbilityTarget::Direction(direction))
                        }),
                        Targeting::Point => ground.map(AbilityTarget::Point),
                        Targeting::Enemy => hovered
                            .filter(|target| valid_enemy_pick(world, player, *target))
                            .map(AbilityTarget::Unit),
                        Targeting::SelfCast => Some(AbilityTarget::SelfCast),
                    });
                    if let Some(target) = target {
                        submit(world, player, UnitCommand::CastAbility { slot, target });
                    } else {
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
    let (focused, current_cursor, size, scale) = {
        let mut windows = world.query::<&Window>();
        let Ok(window) = windows.single(world) else {
            return;
        };
        (
            window.focused,
            window.cursor_position(),
            Vec2::new(window.width(), window.height()),
            window.scale_factor(),
        )
    };
    let size = web::canvas_size().unwrap_or(size);
    let capture = web::pointer_capture();
    let focused = web::document_focused().unwrap_or(focused);
    let paused = world.resource::<ClientSession>().player().is_none()
        || !focused
        || capture.map(web::PointerCapture::confirmed) == Some(false)
        || web::paused().unwrap_or(world.resource::<SimulationControl>().paused);
    set_paused(world, paused);
    let (captured, just_captured) = sync_pointer(world, capture, paused, size, current_cursor);
    let mut events: Vec<_> = cursor
        .read(world.resource::<Messages<WindowEvent>>())
        .cloned()
        .collect();
    // Motion used to reach Enter/resume belongs to the unlocked session.
    if just_captured || world.resource::<ClientSession>().player().is_none() {
        events.clear();
    }
    for event in events {
        match event {
            WindowEvent::CursorMoved(event) => {
                world
                    .resource_mut::<GamePointer>()
                    .absolute(Some(event.position));
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
            }
            WindowEvent::CursorLeft(_) => {
                world.resource_mut::<GamePointer>().absolute(None);
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
            }
            WindowEvent::MouseMotion(event) if captured == Some(true) => {
                world
                    .resource_mut::<GamePointer>()
                    .relative(event.delta / scale);
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
            }
            WindowEvent::KeyboardInput(event)
                if focused && event.state.is_pressed() && !event.repeat =>
            {
                key_pressed(world, event.key_code)
            }
            WindowEvent::MouseButtonInput(event) if event.state.is_pressed() => {
                let point = world.resource::<GamePointer>().position();
                update_pointer(world, point);
                mouse_pressed(world, event.button);
            }
            _ => {}
        }
    }
    let skill_request = web::take_skill_request();
    queued_skill_request(world, skill_request, just_captured);
    if !world.resource::<SimulationControl>().paused {
        let point = world.resource::<GamePointer>().position();
        update_pointer(world, point);
        web::draw_pointer(point);
    } else {
        *world.resource_mut::<PointerState>() = PointerState::default();
        web::draw_pointer(None);
    }
}

pub(super) fn patrol_units(
    control: Res<SimulationControl>,
    mut queue: ResMut<CommandQueue>,
    tick: Res<SimulationTick>,
    mut last_tick: Local<Option<u64>>,
    mut units: Query<(Entity, &UnitState, &UnitOrder, &mut Patrol)>,
) {
    if control.paused {
        *last_tick = None;
        return;
    }
    if *last_tick == Some(tick.0) {
        return;
    }
    *last_tick = Some(tick.0);
    for (entity, state, order, mut patrol) in &mut units {
        if state.alive() && matches!(order, UnitOrder::Idle) {
            queue.submit(entity, UnitCommand::MoveTo(patrol.points[patrol.next]));
            patrol.next = 1 - patrol.next;
        }
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
            SimulationEvent::MoveOrdered { unit, destination }
                if Some(*unit) == session.player() =>
            {
                feedback.marker = Some((*destination, true, time.elapsed_secs()));
            }
            SimulationEvent::CommandRejected { unit, reason }
                if Some(*unit) == session.player() =>
            {
                feedback.message = match reason {
                    Rejection::NoPath => "No route to that location",
                    Rejection::OutOfRange => "Outside cast range",
                    Rejection::OnCooldown => "Ability is cooling down",
                    Rejection::InsufficientMana => "Not enough energy",
                    Rejection::Dead => "Waiting to respawn",
                    Rejection::Busy => "Action already in progress",
                    Rejection::InvalidDamage => "Ability damage cannot be resolved",
                    Rejection::Controlled => "受控制限制，当前操作不可用",
                    _ => "Choose a valid target",
                }
                .into();
                feedback.expires = time.elapsed_secs() + 2.0;
                if let Some(point) = pointer.ground {
                    feedback.marker = Some((point, false, time.elapsed_secs()));
                }
            }
            _ => {}
        }
    }
}
