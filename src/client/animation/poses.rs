//! The reference's authored clip crops and model-space arm IK, adapted to current weapons.
use super::super::skeleton::{AnimationLibrary, blend};
use crate::simulation::WeaponMode;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Gesture {
    Slash,
    Axe,
    Hammer,
    Shot,
    Cannon,
    Windup,
    Hit,
    Guard,
}

impl Gesture {
    pub(super) fn duration(self) -> f32 {
        match self {
            Self::Slash | Self::Axe => 0.42,
            Self::Hammer => 0.6,
            Self::Shot | Self::Cannon => 0.32,
            Self::Windup => 0.4,
            Self::Hit => 0.24,
            Self::Guard => 0.2,
        }
    }
}

#[derive(Clone, Copy)]
struct Key {
    grip: Vec3,
    yaw: f32,
    pitch: f32,
    twist: f32,
}

impl Key {
    fn new(grip: Vec3, yaw: f32, pitch: f32, twist: f32) -> Self {
        Self {
            grip,
            yaw,
            pitch,
            twist,
        }
    }

    fn mix(self, other: Self, weight: f32) -> Self {
        Self {
            grip: self.grip.lerp(other.grip, weight),
            yaw: self.yaw.lerp(other.yaw, weight),
            pitch: self.pitch.lerp(other.pitch, weight),
            twist: self.twist.lerp(other.twist, weight),
        }
    }

    fn rotation(self) -> Quat {
        Quat::from_euler(
            EulerRot::YXZ,
            self.yaw.to_radians(),
            self.pitch.to_radians(),
            0.0,
        )
    }
}

pub(super) fn smooth(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn guard(mode: WeaponMode) -> Key {
    if mode == WeaponMode::Ranged {
        Key::new(Vec3::new(0.10, 0.30, -0.25), 180.0, 0.0, 0.0)
    } else {
        Key::new(Vec3::new(0.10, 0.25, -0.20), 165.0, -15.0, 0.0)
    }
}

pub(super) fn idle(library: &AnimationLibrary, seconds: f32, mode: WeaponMode) -> Vec<Transform> {
    let mut pose = library.sample("Sword_Idle", seconds, true);
    hold_guard(library, &mut pose, mode);
    pose
}

pub(super) fn hold_guard(library: &AnimationLibrary, pose: &mut [Transform], mode: WeaponMode) {
    weapon_pose(library, pose, guard(mode), mode == WeaponMode::Ranged);
}

/// Ported from act: lower the pelvis, then solve both legs in model space so
/// imported bone axes cannot turn a crouch into lifted feet or shortened limbs.
pub(super) fn crouch_pose(library: &AnimationLibrary, local: &mut [Transform], weight: f32) {
    let standing = library.model_pose(local);
    let rest_root = Quat::from_rotation_y(std::f32::consts::PI) * library.bones[0].rest.rotation;
    let forward = standing[0].rotation * rest_root.inverse() * Vec3::NEG_Z;
    let pelvis = library.bone("pelvis");
    let parent = library.bones[pelvis].parent.expect("pelvis has a parent");
    local[pelvis].translation += standing[parent]
        .compute_affine()
        .inverse()
        .transform_vector3(Vec3::NEG_Y * (0.33 * weight));
    for side in ["l", "r"] {
        let thigh = library.bone(&format!("thigh_{side}"));
        let calf = library.bone(&format!("calf_{side}"));
        let foot = library.bone(&format!("foot_{side}"));
        let model = library.model_pose(local);
        let hip = model[thigh].translation;
        let target = standing[foot].translation;
        let upper = model[calf].translation.distance(hip);
        let lower = model[foot].translation.distance(model[calf].translation);
        let direction = (target - hip).normalize_or(Vec3::NEG_Y);
        let distance = target.distance(hip).clamp(0.0001, upper + lower - 0.0001);
        let along = (upper * upper - lower * lower + distance * distance) / (2.0 * distance);
        let fallback = if direction.y.abs() < 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };
        let bend = (forward - direction * direction.dot(forward))
            .normalize_or((fallback - direction * direction.dot(fallback)).normalize());
        let knee = hip + direction * along + bend * (upper * upper - along * along).max(0.0).sqrt();
        aim_bone(library, local, &model, thigh, model[calf].translation, knee);
        let model = library.model_pose(local);
        aim_bone(
            library,
            local,
            &model,
            calf,
            model[foot].translation,
            target,
        );
        let model = library.model_pose(local);
        // Preserve the authored foot orientation as well as its position.
        // Toes therefore keep following the same ground trajectory.
        local[foot].rotation = model[calf].rotation.inverse() * standing[foot].rotation;
    }
}

pub(super) fn gesture(
    library: &AnimationLibrary,
    kind: Gesture,
    elapsed: f32,
    mode: WeaponMode,
) -> Vec<Transform> {
    if kind == Gesture::Hit {
        return library.sample(
            "Hit_Chest",
            elapsed / kind.duration() * library.duration("Hit_Chest"),
            false,
        );
    }
    if kind == Gesture::Guard {
        return library.sample("Sword_Block", 0.2 + elapsed.min(0.2), false);
    }
    let k = |x, y, z, yaw, pitch, twist| Key::new(Vec3::new(x, y, z), yaw, pitch, twist);
    let (clip, times, loaded, contact, follow) = match kind {
        Gesture::Slash => (
            "Sword_Regular_A",
            [0.0, 0.18, 0.30],
            k(0.35, 0.28, 0.02, 80.0, -5.0, -30.0),
            k(0.06, 0.24, -0.40, 180.0, 0.0, 0.0),
            k(-0.30, 0.18, -0.12, 265.0, 5.0, 35.0),
        ),
        Gesture::Axe => (
            "Sword_Regular_B",
            [0.0, 0.20, 0.33],
            k(-0.30, 0.35, 0.0, 280.0, -15.0, 30.0),
            k(0.0, 0.30, -0.42, 180.0, -5.0, 0.0),
            k(0.35, 0.20, -0.12, 90.0, 10.0, -35.0),
        ),
        Gesture::Hammer => (
            "Sword_Attack",
            [0.10, 0.42, 0.70],
            k(0.02, 0.65, 0.05, 180.0, -100.0, -10.0),
            k(0.02, 0.28, -0.44, 180.0, 10.0, 0.0),
            k(0.02, 0.18, -0.30, 180.0, 75.0, 15.0),
        ),
        Gesture::Windup => (
            "Sword_Attack",
            [0.10, 0.20, 0.30],
            k(0.02, 0.40, 0.0, 180.0, -35.0, 0.0),
            k(0.02, 0.60, 0.0, 180.0, -75.0, 0.0),
            k(0.02, 0.65, 0.05, 180.0, -100.0, -10.0),
        ),
        Gesture::Shot | Gesture::Cannon => (
            "Sword_Dash",
            [0.05, 0.28, 0.60],
            k(0.10, 0.30, -0.25, 180.0, 0.0, 0.0),
            k(0.10, 0.31, -0.17, 180.0, -5.0, 0.0),
            k(0.10, 0.30, -0.25, 180.0, 0.0, 0.0),
        ),
        Gesture::Hit | Gesture::Guard => unreachable!(),
    };
    let progress = (elapsed / kind.duration()).clamp(0.0, 1.0);
    let (key, source_time, recovery) = if progress < 0.35 {
        let weight = smooth(progress / 0.35);
        (
            loaded.mix(contact, weight),
            times[0].lerp(times[1], weight),
            0.0,
        )
    } else if progress < 0.7 {
        let weight = smooth((progress - 0.35) / 0.35);
        (
            contact.mix(follow, weight),
            times[1].lerp(times[2], weight),
            0.0,
        )
    } else {
        (follow, times[2], smooth((progress - 0.7) / 0.3))
    };
    let base = library.sample("Sword_Idle", 0.0, false);
    let source = library.sample(clip, source_time, false);
    let mut pose: Vec<_> = base
        .into_iter()
        .zip(source)
        .map(|(a, b)| blend(a, b, 0.55))
        .collect();
    weapon_pose(library, &mut pose, key, mode == WeaponMode::Ranged);
    let idle = idle(library, 0.0, mode);
    for (pose, idle) in pose.iter_mut().zip(idle) {
        *pose = blend(*pose, idle, recovery);
    }
    pose
}

/// The imported axes vary by bone; corrections therefore operate in model space.
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
        (current_tip - origin).normalize_or(Vec3::Y),
        (target_tip - origin).normalize_or(Vec3::Y),
    );
    let parent = library.bones[bone].parent.expect("limb bone has a parent");
    local[bone].rotation =
        (model[parent].rotation.inverse() * correction * model[bone].rotation).normalize();
}

fn weapon_pose(library: &AnimationLibrary, local: &mut [Transform], key: Key, two_handed: bool) {
    let model = library.model_pose(local);
    let spine = library.bone("spine_01");
    let parent = library.bones[spine].parent.expect("spine has a parent");
    local[spine].rotation = (model[parent].rotation.inverse()
        * Quat::from_rotation_y(key.twist.to_radians())
        * model[spine].rotation)
        .normalize();
    let model = library.model_pose(local);
    let head = library.bone("Head");
    let parent = library.bones[head].parent.expect("head has a parent");
    local[head].rotation = (model[parent].rotation.inverse()
        * Quat::from_rotation_y(std::f32::consts::PI))
    .normalize();
    let wrist = model[library.bone("pelvis")].translation + key.grip;
    let rotation = key.rotation();
    if two_handed {
        place_hands(library, local, wrist, rotation);
    } else {
        arm(library, local, "r", wrist, rotation);
    }
}

fn place_hands(
    library: &AnimationLibrary,
    local: &mut [Transform],
    mut wrist: Vec3,
    rotation: Quat,
) {
    let model = library.model_pose(local);
    let offhand = rotation * Vec3::new(0.0, 0.0605, -0.12);
    for _ in 0..8 {
        for (side, offset) in [("r", Vec3::ZERO), ("l", offhand)] {
            let upper = library.bone(&format!("upperarm_{side}"));
            let lower = library.bone(&format!("lowerarm_{side}"));
            let hand = library.bone(&format!("hand_{side}"));
            let shoulder = model[upper].translation;
            let reach = shoulder.distance(model[lower].translation)
                + model[lower].translation.distance(model[hand].translation)
                - 0.003;
            let center = shoulder - offset;
            wrist = center + (wrist - center).clamp_length_max(reach);
        }
    }
    arm(library, local, "r", wrist, rotation);
    arm(library, local, "l", wrist + offhand, rotation);
}

fn arm(
    library: &AnimationLibrary,
    local: &mut [Transform],
    side: &str,
    target: Vec3,
    rotation: Quat,
) {
    let upper = library.bone(&format!("upperarm_{side}"));
    let lower = library.bone(&format!("lowerarm_{side}"));
    let hand = library.bone(&format!("hand_{side}"));
    let model = library.model_pose(local);
    let shoulder = model[upper].translation;
    let a = shoulder.distance(model[lower].translation);
    let b = model[lower].translation.distance(model[hand].translation);
    let delta = target - shoulder;
    let direction = delta.normalize_or(Vec3::NEG_Z);
    let distance = delta.length().clamp((a - b).abs() + 0.001, a + b - 0.001);
    let along = (a * a - b * b + distance * distance) / (2.0 * distance);
    let rest_root = Quat::from_rotation_y(std::f32::consts::PI) * library.bones[0].rest.rotation;
    let turn = model[0].rotation * rest_root.inverse();
    let pole = turn * Vec3::new(if side == "r" { 1.0 } else { -1.0 }, -0.6, 0.3);
    let bend = (pole - direction * pole.dot(direction)).normalize_or(Vec3::Y);
    let elbow = shoulder + direction * along + bend * (a * a - along * along).max(0.0).sqrt();
    let wrist = shoulder + direction * distance;
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
        wrist,
    );
    let model = library.model_pose(local);
    local[hand].rotation = (model[lower].rotation.inverse() * rotation).normalize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crouch_ik_preserves_feet_rotation_and_leg_lengths_across_authored_ground_poses() {
        let library = AnimationLibrary::default();
        let pelvis = library.bone("pelvis");
        for clip in [
            "Sword_Idle",
            "Walk_Loop",
            "Jog_Fwd_Loop",
            "Sword_Block",
            "Sword_Regular_A",
            "Sword_Regular_B",
            "Sword_Regular_C",
            "Sword_Attack",
            "Sword_Dash",
            "Hit_Chest",
            "Sword_Heavy_Combo",
        ] {
            for frame in 0..=8 {
                let base = library.sample(clip, library.duration(clip) * frame as f32 / 8.0, false);
                let before = library.model_pose(&base);
                for weight in [0.15, 0.5, 1.0] {
                    let mut local = base.clone();
                    crouch_pose(&library, &mut local, weight);
                    let after = library.model_pose(&local);
                    assert!(after[pelvis].translation.y < before[pelvis].translation.y);
                    for bone in &after {
                        assert!(bone.translation.is_finite(), "{clip} frame {frame}");
                        assert!(bone.rotation.is_finite(), "{clip} frame {frame}");
                    }
                    for side in ["l", "r"] {
                        let thigh = library.bone(&format!("thigh_{side}"));
                        let calf = library.bone(&format!("calf_{side}"));
                        let foot = library.bone(&format!("foot_{side}"));
                        assert!(
                            after[foot].translation.distance(before[foot].translation) < 0.002,
                            "foot drift: {clip} frame {frame}, weight {weight}, side {side}"
                        );
                        assert!(
                            after[foot].rotation.dot(before[foot].rotation).abs() > 0.9999,
                            "foot rotation: {clip} frame {frame}, side {side}"
                        );
                        for (a, b) in [(thigh, calf), (calf, foot)] {
                            let expected = before[a].translation.distance(before[b].translation);
                            let actual = after[a].translation.distance(after[b].translation);
                            assert!(
                                (actual - expected).abs() < 0.001,
                                "leg stretch: {clip} frame {frame}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn crouch_moves_the_weapon_grip_with_the_pelvis_without_changing_two_handed_alignment() {
        let library = AnimationLibrary::default();
        for mode in [WeaponMode::Melee, WeaponMode::Ranged] {
            let mut local = idle(&library, 0.0, mode);
            let before = library.model_pose(&local);
            crouch_pose(&library, &mut local, 1.0);
            let after = library.model_pose(&local);
            let right = library.bone("hand_r");
            let left = library.bone("hand_l");
            let pelvis = library.bone("pelvis");
            let right_offset = before[right].translation - before[pelvis].translation;
            assert!(
                (after[right].translation - after[pelvis].translation).distance(right_offset)
                    < 0.0001
            );
            let hands = before[left].translation - before[right].translation;
            assert!((after[left].translation - after[right].translation).distance(hands) < 0.0001);
            assert!(after[right].rotation.dot(before[right].rotation).abs() > 0.9999);
        }
    }
}
