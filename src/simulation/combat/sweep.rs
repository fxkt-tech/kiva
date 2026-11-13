//! Melee hit detection shared by the player, ordinary enemies and the Boss:
//! the blade's baked path, swept through each fixed step, against the target's
//! physical capsule. The paths are data ([`super::hit_tracks`], baked from the
//! attack animations); nothing here knows a concrete attack.
use super::hit::CombatWorld;
use crate::simulation::{ActionKind, Body, Boss, MeleeStage, MotionState, TrackId, WeaponPair};
use bevy::prelude::*;
use std::collections::VecDeque;

/// One attack's blade path in the attacker's own frame: feet at the origin,
/// facing -Z, one sample per fixed step from the attack's start.
#[derive(Debug)]
pub struct HitTrack {
    /// The attack timeline this was baked for; a test compares it to the rules.
    pub duration: f32,
    /// Grip then tip of the blade at each tick.
    pub samples: &'static [[f32; 6]],
    /// Inclusive tick ranges in which the moving blade can hit. The sweep of
    /// tick `k` runs from sample `k - 1` to sample `k`.
    pub windows: &'static [(u16, u16)],
    /// Half-thickness of the swept blade.
    pub blade_radius: f32,
    /// How far from the attacker's feet the blade reaches while it can hit.
    pub reach: f32,
}

impl HitTrack {
    /// The fixed-step index of `elapsed`.
    pub fn tick(elapsed: f32) -> u16 {
        (elapsed * crate::simulation::TICK_RATE as f32)
            .round()
            .max(0.0) as u16
    }

    pub fn time(tick: u16) -> f32 {
        f32::from(tick) / crate::simulation::TICK_RATE as f32
    }

    /// Which of [`Self::windows`] `tick` falls in: one swing of the blade.
    pub fn window(&self, tick: u16) -> Option<usize> {
        self.windows
            .iter()
            .position(|&(start, end)| (start..=end).contains(&tick))
    }

    pub fn is_active(&self, tick: u16) -> bool {
        self.windows
            .iter()
            .any(|&(start, end)| (start..=end).contains(&tick))
    }

    /// The first tick whose sweep can hit, and the last.
    pub fn first_active(&self) -> u16 {
        self.windows.first().map_or(u16::MAX, |window| window.0)
    }
    pub fn last_active(&self) -> u16 {
        self.windows.last().map_or(0, |window| window.1)
    }

    /// When the blade first becomes dangerous, in attack time.
    pub fn strike_time(&self) -> f32 {
        Self::time(self.first_active().saturating_sub(1))
    }

    /// When the first swing has landed, in attack time.
    pub fn impact_time(&self) -> f32 {
        Self::time(self.windows.first().map_or(0, |window| window.1))
    }

    /// How far straight ahead of a still attacker of `scale` a target
    /// shaped like `target` can stand, feet to feet, and still be swept by
    /// some active tick; zero if no swing passes in front. A multi-swing
    /// strike may touch its full [`Self::reach`] only off to one side.
    pub(in crate::simulation) fn frontal_reach(&self, scale: f32, target: &Hurtbox) -> f32 {
        const STEP: f32 = 0.05;
        let stance = Stance {
            feet: Vec3::ZERO,
            previous_feet: Vec3::ZERO,
            forward: Vec2::NEG_Y,
            scale,
        };
        let farthest = self.reach * scale + target.radius;
        (0..=(farthest / STEP) as u32)
            .rev()
            .map(|step| step as f32 * STEP)
            .find(|&distance| {
                let at = target.moved_to(Vec3::NEG_Z * distance);
                self.windows
                    .iter()
                    .flat_map(|&(start, end)| start..=end)
                    .any(|tick| sweep(self, tick, &stance, &at, |_| {}).is_some())
            })
            .unwrap_or(0.0)
    }

    fn sample(&self, tick: u16) -> (Vec3, Vec3) {
        let sample = self.samples[usize::from(tick).min(self.samples.len() - 1)];
        (
            Vec3::new(sample[0], sample[1], sample[2]),
            Vec3::new(sample[3], sample[4], sample[5]),
        )
    }
}

impl TrackId {
    /// Every attack which swings a blade, in a stable order.
    pub fn all() -> Vec<Self> {
        let stages = [MeleeStage::One, MeleeStage::Two, MeleeStage::Three];
        let mut tracks = Vec::new();
        for pair in WeaponPair::ALL {
            tracks.extend(stages.map(|stage| Self::PlayerMelee { pair, stage }));
        }
        tracks.extend(
            ActionKind::ALL
                .into_iter()
                .filter(|kind| kind.swings_blade() && *kind != ActionKind::Melee)
                .map(Self::PlayerSkill),
        );
        tracks.extend((0..Boss::STRIKES.len() as u8).map(|strike| Self::Boss { strike }));
        tracks.push(Self::Enemy);
        tracks
    }

    /// The length of this attack's timeline, from the rules which own it.
    pub fn duration(self) -> f32 {
        match self {
            Self::PlayerMelee { pair, stage } => pair.melee_profile(stage).duration,
            Self::PlayerSkill(kind) => kind.spec().duration,
            Self::Boss { strike } => {
                Boss::strike_duration(strike).expect("a Boss track is one of its strikes")
            }
            Self::Enemy => super::enemy::ATTACK_DURATION,
        }
    }

    /// The baked blade path, for attacks which swing one.
    pub fn hit_track(self) -> Option<&'static HitTrack> {
        super::hit_tracks::of(self)
    }
}

/// Where an attacker stands for one sweep.
#[derive(Clone, Copy)]
pub(in crate::simulation) struct Stance {
    pub feet: Vec3,
    /// Where the feet were one step earlier.
    pub previous_feet: Vec3,
    pub forward: Vec2,
    pub scale: f32,
}

impl Stance {
    pub fn of(body: &Body, forward: Vec2) -> Self {
        Self {
            feet: body.world_position(),
            previous_feet: body.world_position() - {
                let step = body.step_displacement();
                Vec3::new(step.x, 0.0, step.y)
            },
            forward,
            scale: body.scale(),
        }
    }

    /// Attacker-local (feet origin, -Z forward) to world, around `feet`.
    fn place(&self, feet: Vec3, local: Vec3) -> Vec3 {
        let forward = self.forward.normalize_or(Vec2::NEG_Y);
        // Local -Z is forward and local +X is the attacker's right.
        let right = Vec2::new(-forward.y, forward.x);
        let planar = right * local.x - forward * local.z;
        feet + Vec3::new(planar.x, local.y, planar.y) * self.scale
    }
}

/// A target's physical capsule.
#[derive(Clone, Copy)]
pub struct Hurtbox {
    pub(in crate::simulation) bottom: Vec3,
    pub(in crate::simulation) top: Vec3,
    pub(in crate::simulation) radius: f32,
}

impl Hurtbox {
    pub(in crate::simulation) fn of(body: &Body, motion: &MotionState) -> Self {
        let radius = body.radius();
        let feet = body.world_position();
        let height = motion.capsule_height.max(radius * 2.0);
        Self {
            bottom: feet + Vec3::Y * radius,
            top: feet + Vec3::Y * (height - radius),
            radius,
        }
    }

    /// The same capsule standing at `feet`.
    fn moved_to(&self, feet: Vec3) -> Self {
        let base = self.bottom - Vec3::Y * self.radius;
        Self {
            bottom: self.bottom - base + feet,
            top: self.top - base + feet,
            radius: self.radius,
        }
    }
}

/// The closest points of segments `a` and `b`, on `a` then on `b`.
fn closest_points(a: (Vec3, Vec3), b: (Vec3, Vec3)) -> (Vec3, Vec3) {
    let (d1, d2, r) = (a.1 - a.0, b.1 - b.0, a.0 - b.0);
    let (len1, len2, f) = (d1.length_squared(), d2.length_squared(), d2.dot(r));
    let (s, t);
    if len1 <= 1e-9 && len2 <= 1e-9 {
        return (a.0, b.0);
    }
    if len1 <= 1e-9 {
        s = 0.0;
        t = (f / len2).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if len2 <= 1e-9 {
            t = 0.0;
            s = (-c / len1).clamp(0.0, 1.0);
        } else {
            let b_dot = d1.dot(d2);
            let denominator = len1 * len2 - b_dot * b_dot;
            let mut s_value = if denominator > 1e-9 {
                ((b_dot * f - c * len2) / denominator).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut t_value = (b_dot * s_value + f) / len2;
            if t_value < 0.0 {
                t_value = 0.0;
                s_value = (-c / len1).clamp(0.0, 1.0);
            } else if t_value > 1.0 {
                t_value = 1.0;
                s_value = ((b_dot - c) / len1).clamp(0.0, 1.0);
            }
            s = s_value;
            t = t_value;
        }
    }
    (a.0 + d1 * s, b.0 + d2 * t)
}

/// The most intermediate blades one step is cut into.
const MAX_SLICES: u32 = 12;

/// Where the blade, moving through tick `tick` of `track`, touches `target`:
/// a point on the target's surface. The blade turns about its grip between
/// the two samples, so a fast swing sweeps an arc rather than a chord.
pub(in crate::simulation) fn sweep(
    track: &HitTrack,
    tick: u16,
    stance: &Stance,
    target: &Hurtbox,
    mut probe: impl FnMut([Vec3; 2]),
) -> Option<Vec3> {
    let (from_grip, from_tip) = track.sample(tick.saturating_sub(1));
    let (to_grip, to_tip) = track.sample(tick);
    let (from_grip, from_tip, to_grip, to_tip) = (
        stance.place(stance.previous_feet, from_grip),
        stance.place(stance.previous_feet, from_tip),
        stance.place(stance.feet, to_grip),
        stance.place(stance.feet, to_tip),
    );
    let radius = track.blade_radius * stance.scale;
    let travel = from_tip.distance(to_tip).max(from_grip.distance(to_grip));
    let slices = ((travel / radius.max(0.01)).ceil() as u32).clamp(1, MAX_SLICES);
    let (from_blade, to_blade) = (from_tip - from_grip, to_tip - to_grip);
    let turn = Quat::from_rotation_arc(
        from_blade.normalize_or(Vec3::Y),
        to_blade.normalize_or(Vec3::Y),
    );
    let reach = radius + target.radius;
    let mut contact = None;
    for slice in 0..=slices {
        let t = slice as f32 / slices as f32;
        let grip = from_grip.lerp(to_grip, t);
        let length = from_blade.length().lerp(to_blade.length(), t);
        let direction = Quat::IDENTITY.slerp(turn, t) * from_blade.normalize_or(Vec3::Y);
        let tip = grip + direction * length;
        probe([grip, tip]);
        if contact.is_some() {
            continue;
        }
        let (on_blade, on_axis) = closest_points((grip, tip), (target.bottom, target.top));
        let gap = on_blade.distance(on_axis);
        if gap <= reach {
            let outward = (on_blade - on_axis).normalize_or_zero();
            contact = Some(on_axis + outward * gap.min(target.radius));
        }
    }
    contact
}

/// A swept contact which no wall separates from the attacker.
pub(in crate::simulation) fn strike(
    track: &HitTrack,
    tick: u16,
    attacker: (Entity, &Body),
    forward: Vec2,
    target: (Entity, &Body, &MotionState),
    world: &CombatWorld,
    probes: &mut HitProbes,
) -> Option<Vec3> {
    let stance = Stance::of(attacker.1, forward);
    let hurtbox = Hurtbox::of(target.1, target.2);
    let radius = track.blade_radius * stance.scale;
    let contact = sweep(track, tick, &stance, &hurtbox, |blade| {
        probes.push(attacker.0, ProbeShape::Blade { blade, radius });
    });
    probes.push(target.0, ProbeShape::Hurtbox(hurtbox));
    let contact = contact
        .filter(|&point| world.visible(attacker.1.aim_position(), point, attacker.0, target.0))?;
    probes.push(target.0, ProbeShape::Contact(contact));
    Some(contact)
}

/// The ticks the action passed through between two elapsed times.
pub(in crate::simulation) fn ticks_between(
    previous: f32,
    elapsed: f32,
) -> impl Iterator<Item = u16> {
    (HitTrack::tick(previous) + 1)..=HitTrack::tick(elapsed)
}

/// What a sweep looked at, kept for the debug overlay only. Rules never read it.
#[derive(Clone, Copy)]
pub enum ProbeShape {
    Blade { blade: [Vec3; 2], radius: f32 },
    Hurtbox(Hurtbox),
    Contact(Vec3),
}

impl Hurtbox {
    pub fn segment(&self) -> [Vec3; 2] {
        [self.bottom, self.top]
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
}

#[derive(Clone, Copy)]
pub struct HitProbe {
    /// The fixed step which produced it.
    pub tick: u64,
    pub actor: Entity,
    pub shape: ProbeShape,
}

/// A short, bounded history of sweep geometry. Recording is off until a
/// display asks for it.
#[derive(Resource, Default)]
pub struct HitProbes {
    pub enabled: bool,
    tick: u64,
    entries: VecDeque<HitProbe>,
}

impl HitProbes {
    const CAPACITY: usize = 512;
    /// Fixed steps an entry stays available.
    pub const LIFETIME: u64 = 30;

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn entries(&self) -> impl Iterator<Item = &HitProbe> {
        self.entries.iter()
    }

    fn push(&mut self, actor: Entity, shape: ProbeShape) {
        if !self.enabled {
            return;
        }
        if self.entries.len() == Self::CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(HitProbe {
            tick: self.tick,
            actor,
            shape,
        });
    }
}

pub(in crate::simulation) fn tick_probes(mut probes: ResMut<HitProbes>) {
    probes.tick += 1;
    let expired = probes.tick.saturating_sub(HitProbes::LIFETIME);
    while probes
        .entries
        .front()
        .is_some_and(|probe| probe.tick < expired || !probes.enabled)
    {
        probes.entries.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACK: HitTrack = HitTrack {
        duration: 0.1,
        // A 1 m blade held at chest height swings from the right to straight
        // ahead to the left: a quarter turn per tick.
        samples: &[
            [0.0, 1.0, 0.0, 1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 1.0, -1.0],
            [0.0, 1.0, 0.0, -1.0, 1.0, 0.0],
        ],
        windows: &[(1, 2)],
        blade_radius: 0.05,
        reach: 1.05,
    };

    fn stance(forward: Vec2) -> Stance {
        Stance {
            feet: Vec3::ZERO,
            previous_feet: Vec3::ZERO,
            forward,
            scale: 1.0,
        }
    }

    fn target(feet: Vec3, height: f32) -> Hurtbox {
        Hurtbox {
            bottom: feet + Vec3::Y * 0.3,
            top: feet + Vec3::Y * (height - 0.3),
            radius: 0.3,
        }
    }

    fn touches(tick: u16, forward: Vec2, feet: Vec3, height: f32) -> bool {
        sweep(
            &TRACK,
            tick,
            &stance(forward),
            &target(feet, height),
            |_| {},
        )
        .is_some()
    }

    #[test]
    fn segments_report_their_closest_points() {
        let (a, b) = closest_points(
            (Vec3::ZERO, Vec3::X * 2.0),
            (Vec3::new(1.0, 1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
        );
        assert!(a.distance(Vec3::X) < 1e-5 && b.distance(Vec3::new(1.0, 1.0, 0.0)) < 1e-5);
        // Parallel, degenerate and end-clamped cases stay finite and correct.
        let (a, b) = closest_points((Vec3::ZERO, Vec3::X), (Vec3::Y, Vec3::Y + Vec3::X));
        assert!((a.distance(b) - 1.0).abs() < 1e-5);
        let (a, b) = closest_points((Vec3::ZERO, Vec3::ZERO), (Vec3::Z, Vec3::Z * 3.0));
        assert!(a == Vec3::ZERO && b == Vec3::Z);
        let (a, b) = closest_points((Vec3::ZERO, Vec3::X), (Vec3::X * 3.0, Vec3::X * 5.0));
        assert!(a.distance(Vec3::X) < 1e-5 && b.distance(Vec3::X * 3.0) < 1e-5);
    }

    #[test]
    fn the_blade_hits_what_it_passes_through_and_nothing_else() {
        let ahead = Vec2::NEG_Y;
        let diagonal = Vec3::new(0.6, 0.0, -0.6);
        // Tick 1 sweeps the right-front quarter; the chord between the two
        // samples would pass 0.71 m from the attacker, the arc reaches 1 m.
        assert!(touches(1, ahead, diagonal.normalize() * 1.25, 1.73));
        assert!(!touches(2, ahead, diagonal.normalize() * 1.25, 1.73));
        // The mirrored target is only reached by the second tick.
        let mirrored = Vec3::new(-diagonal.x, 0.0, diagonal.z).normalize() * 1.25;
        assert!(!touches(1, ahead, mirrored, 1.73));
        assert!(touches(2, ahead, mirrored, 1.73));
        // Straight ahead but beyond the tip, and directly behind: never.
        for tick in [1, 2] {
            assert!(!touches(tick, ahead, Vec3::NEG_Z * 1.4, 1.73));
            assert!(!touches(tick, ahead, Vec3::Z * 0.8, 1.73));
        }
        // A crouched target under a chest-high swing is missed.
        assert!(touches(1, ahead, Vec3::NEG_Z * 1.2, 1.73));
        assert!(!touches(1, ahead, Vec3::NEG_Z * 1.2, 0.6));
    }

    #[test]
    fn the_sweep_follows_the_attackers_facing_scale_and_movement() {
        // Facing +X: the attacker's "ahead" is world +X.
        assert!(touches(1, Vec2::X, Vec3::X * 1.2, 1.73));
        assert!(!touches(1, Vec2::X, Vec3::NEG_Z * 1.2, 1.73));
        let mut scaled = stance(Vec2::NEG_Y);
        scaled.scale = 2.0;
        let far = target(Vec3::NEG_Z * 2.2, 3.0);
        assert!(sweep(&TRACK, 1, &stance(Vec2::NEG_Y), &far, |_| {}).is_none());
        assert!(sweep(&TRACK, 1, &scaled, &far, |_| {}).is_some());
        // The contact lies on the target's surface, toward the blade.
        let near = target(Vec3::NEG_Z * 1.2, 1.73);
        let contact = sweep(&TRACK, 1, &stance(Vec2::NEG_Y), &near, |_| {}).unwrap();
        assert!((contact.xz().distance(near.bottom.xz()) - 0.3).abs() < 0.05);
        assert!(contact.z > -1.2);
    }

    #[test]
    fn frontal_reach_is_where_a_target_straight_ahead_is_still_swept() {
        let body = target(Vec3::ZERO, 1.73);
        let reach = TRACK.frontal_reach(1.0, &body);
        assert!(reach > 0.0);
        let at = |distance: f32| body.moved_to(Vec3::NEG_Z * distance);
        let swept = |target: &Hurtbox| {
            (1..TRACK.samples.len() as u16)
                .filter(|&tick| TRACK.is_active(tick))
                .any(|tick| sweep(&TRACK, tick, &stance(Vec2::NEG_Y), target, |_| {}).is_some())
        };
        assert!(swept(&at(reach)));
        assert!(!swept(&at(reach + 0.06)));
        // A larger attacker reaches farther.
        assert!(TRACK.frontal_reach(1.5, &body) > reach);
    }

    #[test]
    fn every_blade_track_is_baked_for_its_current_timeline() {
        let tracks = TrackId::all();
        assert_eq!(tracks.len(), 9 + Boss::STRIKES.len());
        for track in tracks {
            let baked = track
                .hit_track()
                .unwrap_or_else(|| panic!("{track:?} is not baked"));
            assert_eq!(
                baked.duration,
                track.duration(),
                "{track:?}: run `make bake-hits`"
            );
            assert_eq!(
                baked.samples.len(),
                usize::from(HitTrack::tick(baked.duration)) + 1,
                "{track:?}"
            );
            assert!(!baked.windows.is_empty(), "{track:?} never swings");
            assert!(baked.windows.iter().all(|&(start, end)| 1 <= start
                && start <= end
                && usize::from(end) < baked.samples.len()));
            assert!(baked.blade_radius > 0.0 && baked.reach > baked.blade_radius);
        }
    }
}
