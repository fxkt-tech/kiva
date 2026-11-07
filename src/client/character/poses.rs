//! Small visual gestures over the source clips; these never create game effects.
use super::rig::{AnimationLibrary, Clip, blend};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Gesture {
    Idle,
    Run,
    Windup,
    Backswing,
    Cast,
    Channel,
    Recovery,
    Dash,
    Airborne,
    Stunned,
    Forced,
}

#[derive(Clone, Copy)]
struct Grip {
    offset: Vec3,
    yaw: f32,
    pitch: f32,
}
impl Grip {
    const fn new(x: f32, y: f32, z: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            offset: Vec3::new(x, y, z),
            yaw,
            pitch,
        }
    }
    fn mix(self, other: Self, weight: f32) -> Self {
        Self {
            offset: self.offset.lerp(other.offset, weight),
            yaw: self.yaw.lerp(other.yaw, weight),
            pitch: self.pitch.lerp(other.pitch, weight),
        }
    }
}

const GUARD: Grip = Grip::new(0.10, 0.25, -0.20, 165.0, -15.0);
const LOADED: Grip = Grip::new(0.35, 0.28, 0.02, 80.0, -5.0);
const CONTACT: Grip = Grip::new(0.06, 0.24, -0.40, 180.0, 0.0);
const FOLLOW: Grip = Grip::new(-0.30, 0.18, -0.12, 265.0, 5.0);
const CAST: Grip = Grip::new(0.13, 0.52, -0.16, 180.0, -70.0);

pub(super) fn smooth(weight: f32) -> f32 {
    let weight = weight.clamp(0.0, 1.0);
    weight * weight * (3.0 - 2.0 * weight)
}

pub(super) fn sample(
    library: &AnimationLibrary,
    gesture: Gesture,
    progress: f32,
    clock: f32,
) -> Vec<Transform> {
    let p = smooth(progress);
    let (mut local, grip) = match gesture {
        Gesture::Idle => (library.sample(Clip::Idle, clock, true), GUARD),
        Gesture::Run => (library.sample(Clip::Run, clock, true), GUARD),
        Gesture::Windup => (
            body_gesture(library, Clip::Attack, progress * 0.18),
            if progress < 0.65 {
                GUARD.mix(LOADED, smooth(progress / 0.65))
            } else {
                LOADED.mix(CONTACT, smooth((progress - 0.65) / 0.35))
            },
        ),
        Gesture::Backswing => {
            let local = if progress < 0.4 {
                body_gesture(
                    library,
                    Clip::Attack,
                    0.18 + progress / 0.4 * (library.duration(Clip::Attack) - 0.18),
                )
            } else {
                body_gesture(
                    library,
                    Clip::Recover,
                    (progress - 0.4) / 0.6 * library.duration(Clip::Recover),
                )
            };
            let grip = if progress < 0.35 {
                CONTACT.mix(FOLLOW, smooth(progress / 0.35))
            } else {
                FOLLOW.mix(GUARD, smooth((progress - 0.35) / 0.65))
            };
            (local, grip)
        }
        Gesture::Cast => (
            body_gesture(library, Clip::Block, 0.1 + p * 0.2),
            GUARD.mix(CAST, p),
        ),
        Gesture::Channel => (body_gesture(library, Clip::Block, 0.3), CAST),
        Gesture::Recovery => (
            body_gesture(library, Clip::Block, 0.3 * (1.0 - p)),
            CAST.mix(GUARD, p),
        ),
        Gesture::Dash => (
            body_gesture(library, Clip::Dash, 0.25),
            Grip::new(0.22, 0.15, 0.10, 90.0, 15.0),
        ),
        Gesture::Airborne => (library.sample(Clip::Air, clock, true), GUARD),
        Gesture::Stunned | Gesture::Forced => (
            body_gesture(library, Clip::Hit, 0.08 + clock.min(0.18)),
            Grip::new(0.20, 0.14, -0.10, 150.0, 30.0),
        ),
    };
    // Keep eyes facing the unit's gameplay forward even in a twisting gesture.
    let model = library.model_pose(&local);
    let head = library.bone("Head");
    let parent = library.bones[head].parent.expect("head parent");
    local[head].rotation = (model[parent].rotation.inverse()
        * Quat::from_rotation_y(std::f32::consts::PI))
    .normalize();
    place_sword_hand(library, &mut local, grip);
    local
}

fn body_gesture(library: &AnimationLibrary, clip: Clip, seconds: f32) -> Vec<Transform> {
    let mut local: Vec<_> = library
        .sample(Clip::Idle, 0.0, false)
        .into_iter()
        .zip(library.sample(clip, seconds, false))
        .map(|(idle, source)| blend(idle, source, 0.55))
        .collect();
    let model = library.model_pose(&local);
    let reference = library.model_pose(&library.sample(Clip::Idle, 0.0, false));
    let floor = |model: &[Transform]| {
        ["foot_l", "ball_l", "foot_r", "ball_r"]
            .into_iter()
            .map(|name| model[library.bone(name)].translation.y)
            .fold(f32::INFINITY, f32::min)
    };
    let pelvis = library.bone("pelvis");
    let parent = library.bones[pelvis].parent.expect("pelvis parent");
    local[pelvis].translation += model[parent]
        .compute_affine()
        .inverse()
        .transform_vector3(Vec3::Y * (floor(&reference) - floor(&model)));
    local
}

// Two-bone right-arm IK preserves upper/lower arm lengths. The left arm retains
// the source gesture. This is presentation only, not a weapon hitbox.
fn place_sword_hand(library: &AnimationLibrary, local: &mut [Transform], grip: Grip) {
    let upper = library.bone("upperarm_r");
    let lower = library.bone("lowerarm_r");
    let hand = library.bone("hand_r");
    let model = library.model_pose(local);
    let target = model[library.bone("pelvis")].translation + grip.offset;
    let shoulder = model[upper].translation;
    let a = shoulder.distance(model[lower].translation);
    let b = model[lower].translation.distance(model[hand].translation);
    let delta = target - shoulder;
    let direction = delta.normalize_or(Vec3::NEG_Z);
    let distance = delta.length().clamp((a - b).abs() + 0.001, a + b - 0.001);
    let along = (a * a - b * b + distance * distance) / (2.0 * distance);
    let pole = Vec3::new(1.0, -0.6, 0.3);
    let bend = (pole - direction * pole.dot(direction)).normalize_or(Vec3::Y);
    let elbow = shoulder + direction * along + bend * (a * a - along * along).max(0.0).sqrt();
    aim_bone(
        library,
        local,
        &model,
        upper,
        model[lower].translation,
        elbow,
    );
    let model = library.model_pose(local);
    aim_bone(
        library,
        local,
        &model,
        lower,
        model[hand].translation,
        shoulder + direction * distance,
    );
    let model = library.model_pose(local);
    let rotation = Quat::from_euler(
        EulerRot::YXZ,
        grip.yaw.to_radians(),
        grip.pitch.to_radians(),
        0.0,
    );
    local[hand].rotation = (model[lower].rotation.inverse() * rotation).normalize();
}

fn aim_bone(
    library: &AnimationLibrary,
    local: &mut [Transform],
    model: &[Transform],
    bone: usize,
    current_tip: Vec3,
    target_tip: Vec3,
) {
    let origin = model[bone].translation;
    let correction = Quat::from_rotation_arc(
        (current_tip - origin).normalize(),
        (target_tip - origin).normalize(),
    );
    let parent = library.bones[bone].parent.expect("arm parent");
    local[bone].rotation =
        (model[parent].rotation.inverse() * correction * model[bone].rotation).normalize();
}
