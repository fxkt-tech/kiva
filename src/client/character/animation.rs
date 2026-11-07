//! Read-only simulation-to-pose adaptation. Animation owns no combat clocks.
use super::{
    model::BoneVisual,
    poses::{self, Gesture},
    rig::{AnimationLibrary, blend},
    weapon::WeaponStyle,
};
use crate::client::presentation::PresentationFrame;
use crate::simulation::*;
use bevy::prelude::*;

struct CastMemory {
    ability: AbilityId,
    recovery: f32,
    dash: Option<Vec2>,
}

#[derive(Component)]
pub(in crate::client) struct RigPose {
    life: UnitLifeRef,
    weapon: WeaponStyle,
    pub(super) local: Vec<Transform>,
    pub(super) model: Vec<Transform>,
    from: Vec<Transform>,
    pub(super) gesture: Gesture,
    phase_ability: Option<AbilityId>,
    phase_total: f32,
    last_remaining: f32,
    pub(super) last_time: f64,
    clock: f32,
    transition: f32,
    cast: Option<CastMemory>,
    reset_pending: bool,
}

impl RigPose {
    pub(in crate::client) fn model(&self) -> &[Transform] {
        &self.model
    }

    pub fn new(
        library: &AnimationLibrary,
        life: UnitLifeRef,
        now: f64,
        weapon: WeaponStyle,
    ) -> Self {
        let local = poses::sample_weapon(library, Gesture::Idle, 0.0, 0.0, weapon);
        Self {
            life,
            weapon,
            model: library.model_pose(&local),
            from: local.clone(),
            local,
            gesture: Gesture::Idle,
            phase_ability: None,
            phase_total: 0.0,
            last_remaining: 0.0,
            last_time: now,
            clock: 0.0,
            transition: 1.0,
            cast: None,
            reset_pending: false,
        }
    }
}

fn presentation_time(tick: &SimulationTick, frame: &PresentationFrame) -> f64 {
    (tick.0 as f64 + f64::from(frame.fraction())) / TICK_RATE
}

type AnimatedUnit<'a> = (
    Entity,
    &'a UnitPose,
    &'a UnitState,
    &'a UnitAction,
    &'a UnitOrder,
    &'a UnitStatuses,
    &'a ControlState,
    &'a mut RigPose,
);

// Visual stride calibration for the imported jog, in world units per clip second.
const RUN_REFERENCE_SPEED: f32 = 4.2;

/// Each reader has its own cursor. These messages describe already committed
/// actions and never trigger damage, movement, costs or cooldowns.
pub(in crate::client) fn record_events(
    mut events: MessageReader<SimulationEvent>,
    catalog: Res<AbilityCatalog>,
    mut rigs: Query<(Entity, &UnitState, &mut RigPose)>,
) {
    for (entity, state, mut rig) in &mut rigs {
        let life = state.life_ref(entity);
        if rig.life != life {
            rig.life = life;
            rig.cast = None;
            rig.reset_pending = true;
        }
    }
    // One display frame may span commit -> death -> respawn. Preserve message
    // order so an earlier life's dash cannot leak into the new character.
    for event in events.read() {
        match event {
            SimulationEvent::AbilityCommitted {
                unit,
                ability,
                position,
            } => {
                if let Ok((_, _, mut rig)) = rigs.get_mut(*unit)
                    && let Some(definition) = catalog.get(*ability)
                {
                    rig.cast = Some(CastMemory {
                        ability: *ability,
                        recovery: definition.recovery,
                        dash: definition.execution.is_dash().then_some(*position),
                    });
                }
            }
            SimulationEvent::Died { unit } | SimulationEvent::Respawned { unit } => {
                if let Ok((_, _, mut rig)) = rigs.get_mut(*unit) {
                    rig.cast = None;
                    rig.reset_pending = true;
                }
            }
            SimulationEvent::MoveOrdered { unit, .. } => {
                if let Ok((_, _, mut rig)) = rigs.get_mut(*unit) {
                    rig.cast = None;
                }
            }
            _ => {}
        }
    }
}

pub(in crate::client) fn sync_poses(
    frame: Res<PresentationFrame>,
    tick: Res<SimulationTick>,
    library: Res<AnimationLibrary>,
    catalog: Res<AbilityCatalog>,
    motion: Option<Res<crate::client::motion_view::MotionView>>,
    mut units: Query<AnimatedUnit>,
) {
    // Retain the exact last local/model pose, blend origin and clocks. In
    // particular do not resample a one-tick backswing with an alpha of zero.
    if !frame.should_sync() {
        return;
    }
    let now = presentation_time(&tick, &frame);
    let fraction = frame.fraction();
    for (id, pose, state, action, order, statuses, permissions, mut rig) in &mut units {
        if !state.alive() {
            continue;
        }
        if rig.reset_pending {
            let cast = rig.cast.take();
            *rig = RigPose::new(&library, rig.life, now, rig.weapon);
            rig.cast = cast;
        }
        let dt = (now - rig.last_time).max(0.0) as f32;
        rig.last_time = now;
        let moving = pose.position().distance(pose.interpolated(0.0)) > 0.00001;
        let airborne = statuses
            .iter()
            .any(|status| status.kind() == StatusKind::Airborne);
        let stunned = statuses
            .iter()
            .any(|status| status.kind() == StatusKind::Stun);
        let passed_dash_goal = rig
            .cast
            .as_ref()
            .and_then(|cast| cast.dash)
            .is_some_and(|goal| pose.interpolated(0.0).distance(goal) < 0.01);
        if (!moving
            || !matches!(order, UnitOrder::Idle)
            || passed_dash_goal
            || permissions.forced_motion()
            || airborne
            || stunned)
            && let Some(cast) = &mut rig.cast
        {
            cast.dash = None;
        }
        let dashing = moving
            && motion.as_ref().map_or_else(
                || rig.cast.as_ref().is_some_and(|cast| cast.dash.is_some()),
                |motion| motion.dashing(id),
            );
        let (gesture, remaining, duration, ability) = if airborne {
            (Gesture::Airborne, 0.0, 0.0, None)
        } else if permissions.forced_motion() {
            (Gesture::Forced, 0.0, 0.0, None)
        } else if stunned {
            (Gesture::Stunned, 0.0, 0.0, None)
        } else if dashing {
            (Gesture::Dash, 0.0, 0.0, None)
        } else {
            match action {
                UnitAction::AttackWindup { remaining, .. } => (
                    Gesture::Windup,
                    (*remaining as f32 - fraction).max(0.0),
                    *remaining as f32,
                    None,
                ),
                UnitAction::AttackBackswing {
                    remaining,
                    duration,
                } => (
                    Gesture::Backswing,
                    (*remaining as f32 - fraction).max(0.0),
                    *duration as f32,
                    None,
                ),
                UnitAction::Casting {
                    ability, remaining, ..
                } => (
                    Gesture::Cast,
                    (*remaining - fraction * TICK_SECONDS).max(0.0),
                    catalog
                        .get(*ability)
                        .map_or(*remaining, |definition| definition.windup),
                    Some(*ability),
                ),
                UnitAction::Channeling { .. } => (Gesture::Channel, 0.0, 0.0, None),
                UnitAction::Recovery { remaining } => (
                    Gesture::Recovery,
                    (*remaining - fraction * TICK_SECONDS).max(0.0),
                    rig.cast.as_ref().map_or(*remaining, |cast| cast.recovery),
                    rig.cast.as_ref().map(|cast| cast.ability),
                ),
                UnitAction::Ready if moving => (Gesture::Run, 0.0, 0.0, None),
                UnitAction::Ready => (Gesture::Idle, 0.0, 0.0, None),
            }
        };
        let restarted = remaining > rig.last_remaining + 0.0001 && duration > 0.0;
        if rig.gesture != gesture || rig.phase_ability != ability || restarted {
            rig.from = rig.local.clone();
            rig.gesture = gesture;
            rig.phase_ability = ability;
            rig.phase_total = duration;
            rig.clock = 0.0;
            rig.transition = 0.0;
        }
        rig.last_remaining = remaining;
        rig.transition += dt;
        // Gait speed follows actual travel, including slows. Merely retaining
        // a MoveTo order while blocked/rooted must not keep the feet walking.
        let speed = pose.position().distance(pose.interpolated(0.0)) / TICK_SECONDS;
        rig.clock += if gesture == Gesture::Run {
            dt * speed / RUN_REFERENCE_SPEED
        } else {
            dt
        };
        let progress = if rig.phase_total > 0.0 {
            (1.0 - remaining / rig.phase_total).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let target = poses::sample_weapon(&library, gesture, progress, rig.clock, rig.weapon);
        // The first release pose follows the authoritative action transition,
        // even if a fast attack has only a single recovery tick.
        let weight = if matches!(gesture, Gesture::Windup | Gesture::Backswing) {
            1.0
        } else {
            poses::smooth(rig.transition / 0.12)
        };
        rig.local = rig
            .from
            .iter()
            .copied()
            .zip(target)
            .map(|(a, b)| blend(a, b, weight))
            .collect();
        rig.model = library.model_pose(&rig.local);
    }
}

pub(in crate::client) fn sync_attachments(
    frame: Res<PresentationFrame>,
    rigs: Query<&RigPose>,
    mut visuals: Query<(&BoneVisual, &mut Transform)>,
) {
    if !frame.should_sync() {
        return;
    }
    for (visual, mut transform) in &mut visuals {
        if let Ok(rig) = rigs.get(visual.owner) {
            *transform = visual.attachment.transform(&rig.model);
        }
    }
}
