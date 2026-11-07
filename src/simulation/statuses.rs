//! Status lifetime, aggregation and permissions. New statuses are applied only
//! after the old-status clock advances, so one tick always restricts one step.
use super::{effects::ImpactSnapshot, model::Movement, *};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusDuration(u32);
impl StatusDuration {
    pub fn from_ticks(ticks: u32) -> Option<Self> {
        (ticks > 0).then_some(Self(ticks))
    }
    pub fn from_millis(millis: u64) -> Option<Self> {
        let ticks = (millis as f64 * TICK_RATE / 1000.0).ceil();
        if ticks < 1.0 || ticks > f64::from(u32::MAX) {
            None
        } else {
            Self::from_ticks(ticks as u32)
        }
    }
    pub fn ticks(self) -> u32 {
        self.0
    }
    pub fn seconds(self) -> f32 {
        (f64::from(self.0) / TICK_RATE) as f32
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Stun,
    Root,
    Slow,
    Airborne,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusSpec {
    Stun {
        duration: StatusDuration,
    },
    Root {
        duration: StatusDuration,
    },
    Slow {
        duration: StatusDuration,
        fraction: f32,
    },
    Airborne {
        duration: StatusDuration,
    },
    DecayingSlow {
        duration: StatusDuration,
        fraction: f32,
        to_fraction: f32,
        decay: StatusDuration,
    },
}
impl StatusSpec {
    pub fn kind(self) -> StatusKind {
        match self {
            Self::Stun { .. } => StatusKind::Stun,
            Self::Root { .. } => StatusKind::Root,
            Self::Slow { .. } | Self::DecayingSlow { .. } => StatusKind::Slow,
            Self::Airborne { .. } => StatusKind::Airborne,
        }
    }
    pub fn duration(self) -> StatusDuration {
        match self {
            Self::Stun { duration }
            | Self::Root { duration }
            | Self::Slow { duration, .. }
            | Self::Airborne { duration }
            | Self::DecayingSlow { duration, .. } => duration,
        }
    }
    pub fn fraction(self) -> Option<f32> {
        if let Self::Slow { fraction, .. } | Self::DecayingSlow { fraction, .. } = self {
            Some(fraction)
        } else {
            None
        }
    }
    pub(super) fn valid(self) -> bool {
        (!matches!(self,Self::DecayingSlow {to_fraction,fraction,decay,duration} if !to_fraction.is_finite() || to_fraction<=0.0 || to_fraction>fraction || decay.ticks()>duration.ticks()))
            && self.duration().ticks() > 0
            && self
                .fraction()
                .is_none_or(|v| v.is_finite() && v > 0.0 && v <= 1.0)
    }
}
#[derive(Clone, Debug)]
pub struct StatusView {
    id: u64,
    source: UnitLifeRef,
    origin: DamageOrigin,
    effect_index: usize,
    kind: StatusKind,
    remaining: u32,
    duration: u32,
    fraction: Option<f32>,
    decay: Option<(f32, f32, u32)>,
}
impl StatusView {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn source(&self) -> UnitLifeRef {
        self.source
    }
    pub fn ability(&self) -> Option<AbilityId> {
        match self.origin {
            DamageOrigin::Ability(id) | DamageOrigin::EquipmentActive { ability: id, .. } => {
                Some(id)
            }
            DamageOrigin::BasicAttack
            | DamageOrigin::UnitPassive { .. }
            | DamageOrigin::EquipmentPassive { .. } => None,
        }
    }
    pub fn kind(&self) -> StatusKind {
        self.kind
    }
    pub fn remaining_ticks(&self) -> u32 {
        self.remaining
    }
    pub fn duration_ticks(&self) -> u32 {
        self.duration
    }
    pub fn remaining_seconds(&self) -> f32 {
        (f64::from(self.remaining) / TICK_RATE) as f32
    }
    pub fn duration_seconds(&self) -> f32 {
        (f64::from(self.duration) / TICK_RATE) as f32
    }
    pub fn fraction(&self) -> Option<f32> {
        self.fraction
    }
}
#[derive(Component, Debug)]
pub struct UnitStatuses {
    entries: Vec<StatusView>,
    slow_source: ModifierSourceId,
}
impl UnitStatuses {
    pub(super) fn new(slow_source: ModifierSourceId) -> Self {
        Self {
            entries: Vec::new(),
            slow_source,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &StatusView> {
        self.entries.iter().filter(|v| v.remaining > 0)
    }
    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Normalized continuous cosmetic height; battle coordinates stay planar.
    pub fn airborne_height(&self, overstep: f32) -> f32 {
        self.entries
            .iter()
            .filter(|v| v.kind == StatusKind::Airborne)
            .map(|v| {
                let elapsed = v.duration - v.remaining;
                let progress = ((elapsed as f32 - 1.0 + overstep.clamp(0.0, 1.0)).max(0.0)
                    / v.duration as f32)
                    .clamp(0.0, 1.0);
                4.0 * progress * (1.0 - progress)
            })
            .fold(0.0, f32::max)
    }
}
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlState {
    structure: bool,
    rooted: bool,
    stunned: bool,
    airborne: bool,
    forced: bool,
}
impl ControlState {
    pub(super) fn for_class(class: UnitClass) -> Self {
        Self {
            structure: class == UnitClass::Structure,
            ..default()
        }
    }
    pub fn can_move(&self) -> bool {
        !self.structure && !self.rooted && self.can_attack()
    }
    pub fn can_attack(&self) -> bool {
        !self.stunned && !self.airborne && !self.forced
    }
    pub fn can_cast(&self) -> bool {
        !self.structure && self.can_attack()
    }
    pub fn can_dash(&self) -> bool {
        self.can_move()
    }
    pub fn forced_motion(&self) -> bool {
        self.forced
    }
    pub fn permits(&self, ability: &AbilityDefinition) -> bool {
        self.can_cast() && (!ability.execution.requires_mobility() || self.can_dash())
    }
}
#[derive(Resource, Default)]
pub(super) struct StatusSequence(u64);

pub(super) fn synchronize(world: &mut World, unit: Entity) {
    let Some(statuses) = world.get::<UnitStatuses>(unit) else {
        return;
    };
    let mut control = ControlState {
        forced: world
            .get::<Movement>(unit)
            .is_some_and(|m| m.forced.is_some()),
        ..ControlState::for_class(
            world
                .get::<Unit>(unit)
                .expect("unit statuses owner")
                .class(),
        )
    };
    let mut strongest = 0.0_f32;
    for status in statuses.iter() {
        match status.kind {
            StatusKind::Stun => control.stunned = true,
            StatusKind::Root => control.rooted = true,
            StatusKind::Airborne => control.airborne = true,
            StatusKind::Slow => strongest = strongest.max(status.fraction.expect("slow fraction")),
        }
    }
    let source = statuses.slow_source;
    let modifier = (strongest > 0.0).then_some(AttributeModifier {
        move_speed_multiplier: 1.0 - strongest,
        removal: ModifierRemoval::OnDeath,
        ..default()
    });
    super::attributes::set_immediate(world, unit, source, modifier)
        .expect("validated internal slow source");
    if let Some(mut state) = world.get_mut::<ControlState>(unit)
        && *state != control
    {
        *state = control;
    }
}

pub(super) fn apply(
    world: &mut World,
    target: UnitLifeRef,
    impact: &ImpactSnapshot,
    index: usize,
    spec: StatusSpec,
) {
    if !target.is_current(world)
        || world
            .get::<Unit>(target.entity())
            .is_none_or(|unit| unit.class() == UnitClass::Structure)
    {
        return;
    }
    if spec.kind() == StatusKind::Airborne
        && world
            .get::<super::compound::CompoundMotion>(target.entity())
            .is_some_and(|motion| matches!(motion.kind, super::compound::MotionKind::Leap { .. }))
    {
        return;
    }
    let existing = world.get::<UnitStatuses>(target.entity()).and_then(|s| {
        s.entries.iter().position(|v| {
            v.remaining > 0
                && matches!(spec.kind(), StatusKind::Stun | StatusKind::Root)
                && v.kind == spec.kind()
                && v.source == impact.source
                && v.origin == impact.origin
                && v.effect_index == index
        })
    });
    let id = if let Some(position) = existing {
        let mut statuses = world
            .get_mut::<UnitStatuses>(target.entity())
            .expect("unit statuses");
        let status = &mut statuses.entries[position];
        status.remaining = status.remaining.max(spec.duration().ticks());
        status.duration = status.duration.max(status.remaining);
        status.id
    } else {
        let id = {
            let mut sequence = world.resource_mut::<StatusSequence>();
            sequence.0 = sequence
                .0
                .checked_add(1)
                .expect("status instance space exhausted");
            sequence.0
        };
        world
            .get_mut::<UnitStatuses>(target.entity())
            .expect("unit statuses")
            .entries
            .push(StatusView {
                id,
                source: impact.source,
                origin: impact.origin,
                effect_index: index,
                kind: spec.kind(),
                remaining: spec.duration().ticks(),
                duration: spec.duration().ticks(),
                fraction: spec.fraction(),
                decay: if let StatusSpec::DecayingSlow {
                    fraction,
                    to_fraction,
                    decay,
                    ..
                } = spec
                {
                    Some((fraction, to_fraction, decay.ticks()))
                } else {
                    None
                },
            });
        id
    };
    synchronize(world, target.entity());
    if matches!(
        spec.kind(),
        StatusKind::Root | StatusKind::Stun | StatusKind::Airborne
    ) {
        super::movement::interrupt_autonomous(world, target.entity());
        super::actions::interrupt_control(world, target.entity(), spec.kind());
    }
    world.write_message(SimulationEvent::StatusApplied {
        unit: target.entity(),
        instance: id,
        kind: spec.kind(),
        source: impact.source,
    });
}

pub(super) fn advance_old(world: &mut World) {
    let mut units: Vec<_> = world
        .query_filtered::<Entity, With<UnitStatuses>>()
        .iter(world)
        .collect();
    units.sort_by_key(|id| id.to_bits());
    for unit in units {
        let expired = {
            let mut statuses = world.get_mut::<UnitStatuses>(unit).expect("statuses");
            statuses.entries.retain(|s| s.remaining > 0);
            let mut expired = Vec::new();
            for status in &mut statuses.entries {
                status.remaining -= 1;
                if let Some((initial, end, ticks)) = status.decay {
                    let elapsed = status.duration - status.remaining;
                    status.fraction =
                        Some(initial + (end - initial) * (elapsed as f32 / ticks as f32).min(1.0));
                }
                if status.remaining == 0 {
                    expired.push((status.id, status.kind, status.source));
                }
            }
            // An expired Airborne remains for one render interval so descent
            // interpolates to zero; it is absent from permission/HUD views.
            statuses
                .entries
                .retain(|s| s.remaining > 0 || s.kind == StatusKind::Airborne);
            expired
        };
        synchronize(world, unit);
        for (instance, kind, source) in expired {
            world.write_message(SimulationEvent::StatusExpired {
                unit,
                instance,
                kind,
                source,
            });
        }
    }
}

pub(super) fn clear(world: &mut World, unit: Entity) {
    let expired = {
        let Some(mut statuses) = world.get_mut::<UnitStatuses>(unit) else {
            return;
        };
        std::mem::take(&mut statuses.entries)
    };
    synchronize(world, unit);
    for status in expired.into_iter().filter(|s| s.remaining > 0) {
        world.write_message(SimulationEvent::StatusExpired {
            unit,
            instance: status.id,
            kind: status.kind,
            source: status.source,
        });
    }
}
