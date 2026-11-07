//! Human and demo AI adapters both submit the public simulation commands.
use super::{camera::GameCamera, pointer::GamePointer, web};
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
#[derive(Resource)]
pub(super) struct PlayerController {
    pub player: Entity,
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
    let was_paused = world.resource::<SimulationControl>().paused;
    world.resource_mut::<SimulationControl>().paused = paused;
    web::set_paused(paused);
    if paused {
        let player = world.resource::<PlayerController>().player;
        if !was_paused {
            web::release_pointer();
            world.resource_mut::<CommandQueue>().clear();
            submit(world, player, UnitCommand::Stop);
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
    let hovered = if over_ui {
        None
    } else {
        ray.and_then(|ray| {
            let cast = RayCast3d::from_ray(ray, 200.0);
            world
                .query::<(Entity, &UnitState, &UnitPose, &Pickable)>()
                .iter(world)
                .filter(|(_, state, _, _)| state.alive())
                .filter_map(|(id, _, pose, bounds)| {
                    let position = pose.position();
                    let center = Vec3::new(position.x, 0.0, position.y) + bounds.offset;
                    cast.aabb_intersection_at(&Aabb3d::new(center, bounds.half_size))
                        .map(|distance| (id, distance))
                })
                .min_by(|a, b| {
                    a.1.total_cmp(&b.1)
                        .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
                })
                .map(|(id, _)| id)
        })
    };
    *world.resource_mut::<PointerState>() = PointerState {
        position: cursor,
        ground,
        hovered,
        over_ui,
    };
}

fn select_ability(world: &mut World, slot: usize) {
    let player = world.resource::<PlayerController>().player;
    let definition = world
        .get::<AbilitySlots>(player)
        .and_then(|slots| slots.ability(slot))
        .and_then(|id| world.resource::<AbilityCatalog>().get(id));
    if let Some(definition) = definition {
        if definition.targeting == Targeting::SelfCast {
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
    let player = world.resource::<PlayerController>().player;
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
    let player = world.resource::<PlayerController>().player;
    match button {
        MouseButton::Right => {
            world.resource_mut::<PlayerController>().aim = None;
            if let Some(target) = hovered.filter(|target| {
                world
                    .get::<Unit>(*target)
                    .is_some_and(|u| u.team() == Team::Enemy)
            }) {
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
                        Targeting::Direction => ground.map(|point| {
                            AbilityTarget::Direction(
                                point
                                    - world
                                        .get::<UnitPose>(player)
                                        .expect("player pose")
                                        .position(),
                            )
                        }),
                        Targeting::Point => ground.map(AbilityTarget::Point),
                        Targeting::Enemy => hovered.map(AbilityTarget::Unit),
                        Targeting::SelfCast => Some(AbilityTarget::SelfCast),
                    });
                    if let Some(target) = target {
                        submit(world, player, UnitCommand::CastAbility { slot, target });
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
    let captured = web::pointer_captured();
    let just_captured =
        world
            .resource_mut::<GamePointer>()
            .sync(captured.unwrap_or(false), size, current_cursor);
    let focused = web::document_focused().unwrap_or(focused);
    let paused = !focused
        || captured == Some(false)
        || web::paused().unwrap_or(world.resource::<SimulationControl>().paused);
    set_paused(world, paused);
    let mut events: Vec<_> = cursor
        .read(world.resource::<Messages<WindowEvent>>())
        .cloned()
        .collect();
    // Motion used to reach Enter/resume belongs to the unlocked session.
    if just_captured {
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
    if !world.resource::<SimulationControl>().paused {
        if let Some(slot) = web::take_skill_request() {
            select_ability(world, slot);
        }
        let point = world.resource::<GamePointer>().position();
        update_pointer(world, point);
        web::draw_pointer(point);
    } else {
        *world.resource_mut::<PointerState>() = PointerState::default();
        let _ = web::take_skill_request();
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
    player: Res<PlayerController>,
    pointer: Res<PointerState>,
    mut events: MessageReader<SimulationEvent>,
    mut feedback: ResMut<ClientFeedback>,
) {
    for event in events.read() {
        match event {
            SimulationEvent::MoveOrdered { unit, destination } if *unit == player.player => {
                feedback.marker = Some((*destination, true, time.elapsed_secs()));
            }
            SimulationEvent::CommandRejected { unit, reason } if *unit == player.player => {
                feedback.message = match reason {
                    Rejection::NoPath => "No route to that location",
                    Rejection::OutOfRange => "Outside cast range",
                    Rejection::OnCooldown => "Ability is cooling down",
                    Rejection::InsufficientMana => "Not enough energy",
                    Rejection::Dead => "Waiting to respawn",
                    Rejection::Busy => "Action already in progress",
                    Rejection::InvalidDamage => "Ability damage cannot be resolved",
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
