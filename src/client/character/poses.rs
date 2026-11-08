//! Small visual gestures over the source clips; these never create game effects.
use super::{
    rig::{AnimationLibrary, Clip, blend},
    weapon::WeaponStyle,
};
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
    Death,
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

#[cfg(test)]
pub(super) fn sample(
    library: &AnimationLibrary,
    gesture: Gesture,
    progress: f32,
    clock: f32,
) -> Vec<Transform> {
    sample_weapon(library, gesture, progress, clock, WeaponStyle::Sword)
}

pub(super) fn sample_weapon(
    library: &AnimationLibrary,
    gesture: Gesture,
    progress: f32,
    clock: f32,
    weapon: WeaponStyle,
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
        Gesture::Death => (library.bones.iter().map(|bone| bone.rest).collect(), GUARD),
    };
    // A corpse relaxes its arms and legs. Rotating the armed guard pose alone
    // leaves its bent limbs pointing upward, and live grip IK would restore it.
    if gesture != Gesture::Death {
        // Keep eyes facing the unit's gameplay forward even in a twisting gesture.
        let model = library.model_pose(&local);
        let head = library.bone("Head");
        let parent = library.bones[head].parent.expect("head parent");
        local[head].rotation = (model[parent].rotation.inverse()
            * Quat::from_rotation_y(std::f32::consts::PI))
        .normalize();
        let grip = match weapon {
            WeaponStyle::Sword => grip,
            WeaponStyle::Spear => {
                let guard = Grip::new(0.18, 0.32, -0.12, 175.0, -5.0);
                let contact = Grip::new(0.08, 0.32, -0.65, 180.0, 0.0);
                match gesture {
                    Gesture::Windup => {
                        if progress < 0.65 {
                            guard.mix(
                                Grip::new(0.32, 0.3, 0.12, 145.0, 0.0),
                                smooth(progress / 0.65),
                            )
                        } else {
                            Grip::new(0.32, 0.3, 0.12, 145.0, 0.0)
                                .mix(contact, smooth((progress - 0.65) / 0.35))
                        }
                    }
                    Gesture::Backswing => contact.mix(guard, p),
                    Gesture::Cast | Gesture::Channel => {
                        guard.mix(Grip::new(0.18, 0.65, -0.12, 180.0, -40.0), p)
                    }
                    Gesture::Dash => contact,
                    _ => guard,
                }
            }
            WeaponStyle::Bow => {
                let draw = match gesture {
                    Gesture::Windup => p,
                    Gesture::Backswing => 1.0 - p,
                    _ => 0.15,
                };
                place_hand(
                    library,
                    &mut local,
                    Grip::new(-0.12, 0.4, -0.36 + draw * 0.36, 180.0, 0.0),
                    true,
                );
                Grip::new(0.15, 0.4, -0.48, 180.0, 0.0)
            }
        };
        place_hand(library, &mut local, grip, false);
    }
    if gesture == Gesture::Death {
        // The bundled fallback rig has no death clip. Rotate only its visual
        // skeleton; the authoritative/display world root stays at the body.
        local[0].rotation = (Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * p)
            * local[0].rotation)
            .normalize();
        let model = library.model_pose(&local);
        let floor = model
            .iter()
            .map(|bone| bone.translation.y)
            .fold(f32::INFINITY, f32::min);
        let source_scale = model[0].scale.y / local[0].scale.y;
        local[0].translation.y += (0.12 - floor) * p / source_scale;
    }
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
fn place_hand(library: &AnimationLibrary, local: &mut [Transform], grip: Grip, left: bool) {
    let upper = library.bone(if left { "upperarm_l" } else { "upperarm_r" });
    let lower = library.bone(if left { "lowerarm_l" } else { "lowerarm_r" });
    let hand = library.bone(if left { "hand_l" } else { "hand_r" });
    let model = library.model_pose(local);
    let target = model[library.bone("pelvis")].translation + grip.offset;
    let shoulder = model[upper].translation;
    let a = shoulder.distance(model[lower].translation);
    let b = model[lower].translation.distance(model[hand].translation);
    let delta = target - shoulder;
    let direction = delta.normalize_or(Vec3::NEG_Z);
    let distance = delta.length().clamp((a - b).abs() + 0.001, a + b - 0.001);
    let along = (a * a - b * b + distance * distance) / (2.0 * distance);
    let pole = Vec3::new(if left { -1.0 } else { 1.0 }, -0.6, 0.3);
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
