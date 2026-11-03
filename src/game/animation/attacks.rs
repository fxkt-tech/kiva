//! Authored sword paths over CC0 body animation. Positions are relative to the
//! pelvis in gameplay model space; this avoids the source bones' differing axes.
use super::{aim_bone, blend};
use crate::game::{
    combat::core::{Attack, AttackAxis, AttackKind, CHARGE_READY, Charge},
    skeleton::AnimationLibrary,
};
use bevy::prelude::*;

#[derive(Clone, Copy)]
struct Key {
    grip: Vec3,
    yaw: f32,
    pitch: f32,
    twist: f32,
}
impl Key {
    fn new(x: f32, y: f32, z: f32, yaw: f32, pitch: f32, twist: f32) -> Self {
        Self {
            grip: Vec3::new(x, y, z),
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
struct Motion {
    clip: &'static str,
    // Crop a single body gesture from the source; never compress an entire combo.
    times: [f32; 3],
    loaded: Key,
    contact: Key,
    follow: Key,
}
fn guard() -> Key {
    Key::new(0.10, 0.25, -0.20, 165.0, -15.0, 0.0)
}
fn motion(kind: AttackKind, dodge: bool) -> Motion {
    use AttackAxis::{Horizontal as H, Vertical as V};
    use AttackKind::*;
    let k = Key::new;
    match kind {
        Light(H, _) if dodge => Motion {
            clip: "Sword_Dash",
            times: [0.08, 0.30, 0.65],
            loaded: k(0.35, 0.18, 0.05, 70.0, 0.0, -25.0),
            contact: k(0.05, 0.20, -0.46, 180.0, 0.0, 5.0),
            follow: k(-0.32, 0.14, -0.15, 275.0, 5.0, 40.0),
        },
        Light(V, _) if dodge => Motion {
            clip: "Sword_Dash",
            times: [0.05, 0.28, 0.60],
            loaded: k(0.20, 0.10, 0.12, 180.0, 0.0, -30.0),
            contact: k(0.02, 0.24, -0.52, 180.0, 0.0, 5.0),
            follow: k(0.00, 0.26, -0.62, 180.0, -5.0, 15.0),
        },
        Light(H, 0) | Training(false) => Motion {
            clip: "Sword_Regular_A",
            times: [0.0, 0.18, 0.30],
            loaded: k(0.35, 0.28, 0.02, 80.0, -5.0, -30.0),
            contact: k(0.06, 0.24, -0.40, 180.0, 0.0, 0.0),
            follow: k(-0.30, 0.18, -0.12, 265.0, 5.0, 35.0),
        },
        Light(H, 1) => Motion {
            clip: "Sword_Regular_B",
            times: [0.0, 0.20, 0.33],
            loaded: k(-0.30, 0.35, 0.0, 280.0, -15.0, 30.0),
            contact: k(0.00, 0.30, -0.42, 180.0, -5.0, 0.0),
            follow: k(0.35, 0.20, -0.12, 90.0, 10.0, -35.0),
        },
        Light(H, _) => Motion {
            clip: "Sword_Heavy_Combo",
            times: [0.15, 0.48, 0.90],
            loaded: k(0.35, 0.30, 0.05, 65.0, -10.0, -55.0),
            contact: k(0.06, 0.22, -0.48, 180.0, 0.0, 0.0),
            follow: k(-0.30, 0.18, 0.0, 310.0, 10.0, 65.0),
        },
        Light(V, 0) => Motion {
            clip: "Sword_Dash",
            times: [0.0, 0.20, 0.40],
            loaded: k(0.18, 0.18, 0.08, 180.0, 0.0, -20.0),
            contact: k(0.06, 0.24, -0.45, 180.0, 0.0, 5.0),
            follow: k(0.02, 0.24, -0.55, 180.0, 5.0, 15.0),
        },
        Light(V, 1) => Motion {
            clip: "Sword_Attack",
            times: [0.15, 0.42, 0.65],
            loaded: k(-0.12, 0.56, 0.02, 220.0, -75.0, 25.0),
            contact: k(0.02, 0.27, -0.42, 180.0, 10.0, 0.0),
            follow: k(0.30, 0.04, -0.20, 120.0, 65.0, -30.0),
        },
        Light(V, _) | Training(true) => Motion {
            clip: "Sword_Attack",
            times: [0.10, 0.42, 0.70],
            loaded: k(0.02, 0.65, 0.05, 180.0, -100.0, -10.0),
            contact: k(0.02, 0.28, -0.44, 180.0, 10.0, 0.0),
            follow: k(0.02, 0.18, -0.30, 180.0, 75.0, 15.0),
        },
        Focus(H) => Motion {
            clip: "Sword_Heavy_Combo",
            times: [1.25, 1.65, 2.20],
            loaded: k(0.32, 0.16, 0.12, 55.0, -5.0, -60.0),
            contact: k(0.02, 0.22, -0.48, 180.0, 0.0, 5.0),
            follow: k(-0.32, 0.28, 0.0, 300.0, -15.0, 60.0),
        },
        Focus(V) => Motion {
            clip: "Sword_Heavy_Combo",
            times: [2.20, 2.60, 3.20],
            loaded: k(0.00, 0.70, 0.10, 180.0, -115.0, -15.0),
            contact: k(0.00, 0.30, -0.48, 180.0, 5.0, 0.0),
            follow: k(0.00, 0.22, -0.32, 180.0, 85.0, 20.0),
        },
        Crouch(H) => Motion {
            clip: "Sword_Regular_A",
            times: [0.02, 0.15, 0.27],
            loaded: k(0.32, 0.15, 0.02, 70.0, 0.0, -40.0),
            contact: k(0.02, 0.12, -0.42, 180.0, 0.0, 0.0),
            follow: k(-0.30, 0.12, -0.10, 280.0, 0.0, 45.0),
        },
        Crouch(V) => Motion {
            clip: "Sword_Regular_C",
            times: [0.0, 0.25, 0.70],
            loaded: k(0.25, 0.18, 0.08, 65.0, 20.0, -30.0),
            contact: k(0.04, 0.25, -0.40, 180.0, -20.0, 0.0),
            follow: k(0.02, 0.65, -0.10, 180.0, -85.0, 15.0),
        },
        Air(H) => Motion {
            clip: "Sword_Regular_B",
            times: [0.0, 0.20, 0.34],
            loaded: k(-0.30, 0.40, 0.0, 285.0, -25.0, 35.0),
            contact: k(0.00, 0.34, -0.42, 180.0, -15.0, 0.0),
            follow: k(0.30, 0.22, -0.15, 85.0, 15.0, -35.0),
        },
        Air(V) => Motion {
            clip: "Sword_Attack",
            times: [0.18, 0.43, 0.70],
            loaded: k(0.00, 0.65, 0.02, 180.0, -95.0, -10.0),
            contact: k(0.02, 0.40, -0.35, 180.0, 65.0, 5.0),
            follow: k(0.02, 0.30, -0.22, 180.0, 85.0, 15.0),
        },
        Riposte => Motion {
            clip: "Sword_Heavy_Combo",
            times: [0.15, 0.50, 0.85],
            loaded: k(0.28, 0.30, 0.10, 140.0, -30.0, -40.0),
            contact: k(0.02, 0.35, -0.50, 180.0, -10.0, 10.0),
            follow: k(-0.30, 0.10, -0.15, 250.0, 35.0, 50.0),
        },
    }
}
fn stroke_keys(kind: AttackKind, motion: &Motion, strike: usize) -> (Key, Key) {
    use AttackAxis::{Horizontal as H, Vertical as V};
    use AttackKind::*;
    let k = Key::new;
    match (kind, strike) {
        (Air(H), 1) => (
            k(0.02, 0.28, -0.42, 180.0, 0.0, 0.0),
            k(-0.30, 0.22, -0.12, 275.0, -10.0, 35.0),
        ),
        (Light(H, 1), 1) => (
            k(0.02, 0.22, -0.42, 180.0, 5.0, 0.0),
            k(-0.32, 0.18, -0.10, 275.0, 10.0, 35.0),
        ),
        (Light(H, 1), 2) => (
            k(0.00, 0.38, -0.42, 180.0, -15.0, 0.0),
            k(0.32, 0.18, -0.16, 75.0, 20.0, -40.0),
        ),
        (Crouch(V), 0) => (motion.contact, k(0.02, 0.45, -0.25, 180.0, -50.0, 8.0)),
        (Crouch(V), 1) => (k(0.02, 0.45, -0.25, 180.0, -50.0, 8.0), motion.follow),
        (Air(V), 0) => (motion.contact, k(0.02, 0.34, -0.30, 180.0, 78.0, 10.0)),
        (Air(V), 1) => (k(0.02, 0.32, -0.26, 180.0, 80.0, 12.0), motion.follow),
        (Riposte, 0) => (motion.contact, k(0.02, 0.36, -0.62, 180.0, -10.0, 15.0)),
        (Riposte, 1) => (k(0.02, 0.45, -0.35, 180.0, -25.0, 0.0), motion.follow),
        _ => (motion.contact, motion.follow),
    }
}

fn strike_key(attack: &Attack, motion: &Motion) -> Key {
    let mut previous_end = attack.spec().windup;
    let mut previous_key = stroke_keys(attack.kind, motion, 0).0;
    for (index, (start, end)) in attack
        .strike_windows()
        .into_iter()
        .take(attack.spec().strikes.len())
        .enumerate()
    {
        let (contact, follow) = stroke_keys(attack.kind, motion, index);
        if attack.elapsed < start {
            return previous_key.mix(
                contact,
                smooth((attack.elapsed - previous_end) / (start - previous_end)),
            );
        }
        if attack.elapsed < end {
            return contact.mix(follow, smooth((attack.elapsed - start) / (end - start)));
        }
        previous_end = end;
        previous_key = follow;
    }
    previous_key
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(super) fn sample(library: &AnimationLibrary, attack: &Attack) -> Vec<Transform> {
    let motion = motion(attack.kind, attack.follows_dodge());
    let spec = attack.spec();
    let t = attack.elapsed;
    let (key, source_time, recovery) = if t < spec.windup {
        let w = t / spec.windup;
        let key = if matches!(attack.kind, AttackKind::Focus(_)) {
            motion.loaded.mix(motion.contact, smooth(w))
        } else if w < 0.65 {
            guard().mix(motion.loaded, smooth(w / 0.65))
        } else {
            motion.loaded.mix(motion.contact, smooth((w - 0.65) / 0.35))
        };
        (key, motion.times[0].lerp(motion.times[1], w), 0.0)
    } else if t < spec.windup + spec.active {
        let w = (t - spec.windup) / spec.active;
        let key = strike_key(attack, &motion);
        (key, motion.times[1].lerp(motion.times[2], w), 0.0)
    } else {
        let w = smooth((t - spec.windup - spec.active) / spec.recovery);
        let follow = stroke_keys(attack.kind, &motion, spec.strikes.len() - 1).1;
        (follow, motion.times[2], w)
    };
    let mut pose = body_pose(library, &motion, source_time, attack.kind, t >= spec.windup);
    let two_handed = attack.kind == AttackKind::Air(AttackAxis::Vertical);
    sword_pose(library, &mut pose, key, two_handed);
    let mut idle = idle(library, 0.0);
    if matches!(
        attack.kind,
        AttackKind::Air(_) | AttackKind::Crouch(AttackAxis::Vertical)
    ) {
        let lower = library.sample("Jump_Loop", 0.3, true);
        idle[library.bone("pelvis")] = lower[library.bone("pelvis")];
        let first_leg = library.bone("thigh_l");
        idle[first_leg..].copy_from_slice(&lower[first_leg..]);
        sword_pose(library, &mut idle, guard(), two_handed);
    }
    // A separate recovery returns the weapon to guard instead of replaying the
    // source clip's next attack or retaining an overhead finishing pose.
    let mut pose: Vec<_> = pose
        .into_iter()
        .zip(idle)
        .map(|(a, b)| blend(a, b, recovery))
        .collect();
    if two_handed {
        grip_handle(library, &mut pose);
    }
    if attack.kind == AttackKind::Crouch(AttackAxis::Vertical) {
        // Spin the whole rig, including the sword, after arm IK and recovery
        // blending. Keep the angle unwrapped so interpolation cannot erase a
        // complete turn. By the earliest follow-up it has passed 180 degrees,
        // so a cancelled recovery blends forward to guard instead of rewinding.
        let start = spec.windup * 0.35;
        let end = spec.windup + spec.active + spec.recovery * 0.4;
        let angle = std::f32::consts::TAU * smooth((t - start) / (end - start));
        pose[0].rotation = (Quat::from_rotation_y(angle) * pose[0].rotation).normalize();
    }
    pose
}

pub(super) fn charge(library: &AnimationLibrary, charge: &Charge) -> Vec<Transform> {
    let motion = motion(AttackKind::Focus(charge.axis), false);
    let weight = smooth(charge.elapsed / CHARGE_READY);
    let mut pose = body_pose(
        library,
        &motion,
        motion.times[0],
        AttackKind::Focus(charge.axis),
        false,
    );
    sword_pose(
        library,
        &mut pose,
        guard().mix(motion.loaded, weight),
        false,
    );
    pose
}

pub(super) fn idle(library: &AnimationLibrary, seconds: f32) -> Vec<Transform> {
    let mut pose = library.sample("Sword_Idle", seconds, true);
    hold_guard(library, &mut pose);
    pose
}

pub(super) fn hold_guard(library: &AnimationLibrary, pose: &mut [Transform]) {
    sword_pose(library, pose, guard(), false);
}

fn body_pose(
    library: &AnimationLibrary,
    motion: &Motion,
    time: f32,
    kind: AttackKind,
    launched: bool,
) -> Vec<Transform> {
    let mut source = library.sample(motion.clip, time, false);
    let idle = library.sample("Sword_Idle", 0.0, false);
    if kind == AttackKind::Crouch(AttackAxis::Vertical) {
        // The source C gesture contains a full turn. Remove its pelvis yaw
        // before blending body poses (quaternion blending would take shortcuts
        // across half turns); the coherent full-rig turn is applied in sample.
        let model = library.model_pose(&source);
        let reference = library.model_pose(&idle);
        let pelvis = library.bone("pelvis");
        let parent = library.bones[pelvis].parent.unwrap();
        let forward = model[pelvis].rotation * Vec3::Z;
        let facing = reference[pelvis].rotation * Vec3::Z;
        let yaw = facing.x.atan2(facing.z) - forward.x.atan2(forward.z);
        source[pelvis].rotation = (model[parent].rotation.inverse()
            * Quat::from_rotation_y(yaw)
            * model[pelvis].rotation)
            .normalize();
    }
    // Keep the authored weight shift, with a stable stance and no source hops.
    let mut pose: Vec<_> = idle
        .into_iter()
        .zip(source)
        .map(|(a, b)| blend(a, b, 0.55))
        .collect();
    let airborne = matches!(kind, AttackKind::Air(_))
        || (kind == AttackKind::Crouch(AttackAxis::Vertical) && launched);
    if airborne {
        let lower = library.sample("Jump_Loop", 0.3, true);
        let pelvis = library.bone("pelvis");
        pose[pelvis] = lower[pelvis];
        let first_leg = library.bone("thigh_l");
        pose[first_leg..].copy_from_slice(&lower[first_leg..]);
    } else {
        let model = library.model_pose(&pose);
        let reference = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
        let floor = |model: &[Transform]| {
            ["foot_l", "ball_l", "foot_r", "ball_r"]
                .into_iter()
                .map(|name| model[library.bone(name)].translation.y)
                .fold(f32::INFINITY, f32::min)
        };
        let pelvis = library.bone("pelvis");
        let parent = library.bones[pelvis].parent.unwrap();
        pose[pelvis].translation += model[parent]
            .compute_affine()
            .inverse()
            .transform_vector3(Vec3::Y * (floor(&reference) - floor(&model)));
    }
    pose
}

fn sword_pose(library: &AnimationLibrary, local: &mut [Transform], key: Key, two_handed: bool) {
    let model = library.model_pose(local);
    let spine = library.bone("spine_01");
    let parent = library.bones[spine].parent.unwrap();
    local[spine].rotation = (model[parent].rotation.inverse()
        * Quat::from_rotation_y(key.twist.to_radians())
        * model[spine].rotation)
        .normalize();
    let model = library.model_pose(local);
    let head = library.bone("Head");
    let parent = library.bones[head].parent.unwrap();
    local[head].rotation = (model[parent].rotation.inverse()
        * Quat::from_rotation_y(std::f32::consts::PI))
    .normalize();
    let wrist = model[library.bone("pelvis")].translation + key.grip;
    let rotation = key.rotation();
    if two_handed {
        place_hands(library, local, wrist, rotation);
    } else {
        // The free arm retains the source animation's balance and follow-through.
        arm(library, local, "r", wrist, rotation);
    }
}

pub(super) fn grip_handle(library: &AnimationLibrary, local: &mut [Transform]) {
    let model = library.model_pose(local);
    let hand = model[library.bone("hand_r")];
    place_hands(library, local, hand.translation, hand.rotation);
}

fn place_hands(
    library: &AnimationLibrary,
    local: &mut [Transform],
    mut wrist: Vec3,
    rotation: Quat,
) {
    let model = library.model_pose(local);
    let offhand = rotation * Vec3::new(0.0, 0.0605, -0.12);
    // Project the shared handle into both arms' reach, instead of clamping
    // each wrist separately and letting the off-hand detach from the hilt.
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
            let delta = wrist - center;
            wrist = center + delta.clamp_length_max(reach);
        }
    }
    // Both hands grip the same handle; the off-hand sits behind the sword hand.
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
    use crate::game::player::{Attachment, attachment_transform};
    use std::io::Write;

    fn attacks() -> Vec<(&'static str, Attack)> {
        use AttackAxis::{Horizontal as H, Vertical as V};
        use AttackKind::*;
        vec![
            ("A", Attack::new(Light(H, 0))),
            ("2A", Attack::new(Light(H, 1))),
            ("3A", Attack::new(Light(H, 2))),
            ("B", Attack::new(Light(V, 0))),
            ("2B", Attack::new(Light(V, 1))),
            ("3B", Attack::new(Light(V, 2))),
            ("Dodge A", Attack::after_dodge(H)),
            ("Dodge B", Attack::after_dodge(V)),
            ("Focus A", Attack::new(Focus(H))),
            ("Focus B", Attack::new(Focus(V))),
            ("Crouch A", Attack::new(Crouch(H))),
            ("Uppercut", Attack::new(Crouch(V))),
            ("Air A", Attack::new(Air(H))),
            ("Plunge", Attack::new(Air(V))),
            ("Riposte", Attack::new(Riposte)),
        ]
    }
    fn tip(library: &AnimationLibrary, pose: &[Transform]) -> Vec3 {
        let model = library.model_pose(pose);
        attachment_transform(
            &Attachment::Sword {
                bone: library.bone("hand_r"),
            },
            &model,
        )
        .transform_point(Vec3::Z * 1.02)
    }

    #[test]
    fn attack_paths_are_finite_and_plunge_keeps_both_hands_on_the_handle() {
        let library = AnimationLibrary::default();
        for (name, mut attack) in attacks() {
            let spec = attack.spec();
            for frame in 0..=80 {
                attack.elapsed = spec.duration() * frame as f32 / 80.0;
                let pose = sample(&library, &attack);
                let model = library.model_pose(&pose);
                assert!(pose[0].translation.length() < 0.0001);
                for bone in &model {
                    assert!(
                        bone.translation.is_finite() && bone.rotation.is_finite(),
                        "{name} frame {frame}"
                    );
                    assert!((bone.rotation.length() - 1.0).abs() < 0.001);
                }
                if attack.kind == AttackKind::Air(AttackAxis::Vertical) {
                    let right = model[library.bone("hand_r")];
                    let left = model[library.bone("hand_l")];
                    let expected =
                        right.translation + right.rotation * Vec3::new(0.0, 0.0605, -0.12);
                    assert!(
                        left.translation.distance(expected) < 0.012,
                        "{name} frame {frame}: hand left the hilt by {}",
                        left.translation.distance(expected)
                    );
                }
            }
        }
    }

    #[test]
    fn sword_paths_distinguish_thrust_uppercut_plunge_and_each_combo_stage() {
        let library = AnimationLibrary::default();
        let mut uppercut = Attack::new(AttackKind::Crouch(AttackAxis::Vertical));
        uppercut.elapsed = uppercut.spec().windup * 0.65;
        let low = tip(&library, &sample(&library, &uppercut));
        uppercut.elapsed = uppercut.spec().windup + uppercut.spec().active;
        let high = tip(&library, &sample(&library, &uppercut));
        assert!(
            high.y - low.y > 0.8,
            "uppercut must lift the blade: {low} -> {high}"
        );
        let mut plunge = Attack::new(AttackKind::Air(AttackAxis::Vertical));
        plunge.elapsed = plunge.spec().windup * 0.65;
        let high = tip(&library, &sample(&library, &plunge));
        plunge.elapsed = plunge.spec().windup + plunge.spec().active;
        let low = tip(&library, &sample(&library, &plunge));
        assert!(
            high.y - low.y > 1.0,
            "plunge must cut down: {high} -> {low}"
        );
        for mut attack in [
            Attack::new(AttackKind::Light(AttackAxis::Vertical, 0)),
            Attack::after_dodge(AttackAxis::Vertical),
        ] {
            attack.elapsed = attack.spec().windup;
            let model = library.model_pose(&sample(&library, &attack));
            assert!((model[library.bone("hand_r")].rotation * Vec3::Z).dot(Vec3::NEG_Z) > 0.95);
        }
        let mut paths: Vec<(&str, Vec<Vec3>)> = Vec::new();
        for (name, mut attack) in attacks() {
            let spec = attack.spec();
            let points: Vec<_> = [spec.windup * 0.65, spec.windup, spec.windup + spec.active]
                .into_iter()
                .map(|t| {
                    attack.elapsed = t;
                    tip(&library, &sample(&library, &attack))
                })
                .collect();
            for (other, previous) in &paths {
                let distance: f32 = points
                    .iter()
                    .zip(previous)
                    .map(|(a, b)| a.distance(*b))
                    .sum();
                assert!(distance > 0.08, "{name} repeats {other}");
            }
            paths.push((name, points));
        }
    }

    #[test]
    fn charged_release_starts_from_the_held_pose() {
        let library = AnimationLibrary::default();
        for axis in [AttackAxis::Horizontal, AttackAxis::Vertical] {
            let held = charge(
                &library,
                &Charge {
                    axis,
                    elapsed: CHARGE_READY,
                    combo: 0,
                    dodge_followup: false,
                },
            );
            let release = sample(&library, &Attack::new(AttackKind::Focus(axis)));
            for (a, b) in library
                .model_pose(&held)
                .iter()
                .zip(library.model_pose(&release))
            {
                assert!(a.translation.distance(b.translation) < 0.0001);
                assert!(a.rotation.dot(b.rotation).abs() > 0.9999);
            }
        }
    }

    #[test]
    fn aerial_left_click_cuts_in_opposite_directions_twice() {
        let library = AnimationLibrary::default();
        let mut attack = Attack::new(AttackKind::Air(AttackAxis::Horizontal));
        let windows = attack.strike_windows();
        let hand = library.bone("hand_r");
        let mut blade_at = |time| {
            attack.elapsed = time;
            library.model_pose(&sample(&library, &attack))[hand].rotation * Vec3::Z
        };
        for (start, _) in windows.into_iter().take(2) {
            assert!(blade_at(start).dot(Vec3::NEG_Z) > 0.95);
        }
        assert!(
            blade_at(windows[0].1).x > 0.85,
            "first cut must sweep right"
        );
        assert!(
            blade_at(windows[1].1).x < -0.85,
            "second cut must sweep back left"
        );
        for boundary in [windows[0].1, windows[1].0, windows[1].1] {
            attack.elapsed = boundary - 0.0001;
            let before = tip(&library, &sample(&library, &attack));
            attack.elapsed = boundary + 0.0001;
            let after = tip(&library, &sample(&library, &attack));
            assert!(before.distance(after) < 0.01, "sword jumped at {boundary}");
        }
    }

    #[test]
    fn multi_hit_choreography_tracks_each_window_without_snapping() {
        let library = AnimationLibrary::default();
        let mut triple = Attack::new(AttackKind::Light(AttackAxis::Horizontal, 1));
        for (index, (_, end)) in triple.strike_windows().into_iter().enumerate() {
            triple.elapsed = end;
            let model = library.model_pose(&sample(&library, &triple));
            let direction = model[library.bone("hand_r")].rotation * Vec3::Z;
            assert!(
                if index == 1 {
                    direction.x < -0.85
                } else {
                    direction.x > 0.85
                },
                "2A must visibly alternate three cuts"
            );
        }
        for kind in [
            triple.kind,
            AttackKind::Crouch(AttackAxis::Vertical),
            AttackKind::Air(AttackAxis::Vertical),
            AttackKind::Riposte,
        ] {
            let mut attack = Attack::new(kind);
            for (index, (start, end)) in attack
                .strike_windows()
                .into_iter()
                .take(attack.spec().strikes.len())
                .enumerate()
            {
                for boundary in [start, end] {
                    if index == 0 && boundary == start {
                        continue;
                    }
                    attack.elapsed = boundary - 0.0001;
                    let before = tip(&library, &sample(&library, &attack));
                    attack.elapsed = boundary + 0.0001;
                    let after = tip(&library, &sample(&library, &attack));
                    assert!(
                        before.distance(after) < 0.01,
                        "{kind:?}: sword jumped at {boundary}"
                    );
                }
            }
        }
    }

    #[test]
    fn attacks_except_plunge_leave_the_offhand_free() {
        let library = AnimationLibrary::default();
        let left = ["upperarm_l", "lowerarm_l", "hand_l"].map(|name| library.bone(name));
        let assert_free = |actual: &[Transform], source: &[Transform]| {
            for bone in left {
                assert!(
                    actual[bone].rotation.dot(source[bone].rotation).abs() > 0.9999,
                    "single-handed motion changed the free arm"
                );
                assert!(actual[bone].translation.distance(source[bone].translation) < 0.0001);
            }
        };
        for (_, mut attack) in attacks() {
            if attack.kind == AttackKind::Air(AttackAxis::Vertical) {
                continue;
            }
            let spec = attack.spec();
            let motion = motion(attack.kind, attack.follows_dodge());
            for (elapsed, source_time) in [
                (
                    spec.windup * 0.65,
                    motion.times[0].lerp(motion.times[1], 0.65),
                ),
                (spec.windup, motion.times[1]),
                (spec.windup + spec.active, motion.times[2]),
            ] {
                attack.elapsed = elapsed;
                let source = body_pose(
                    &library,
                    &motion,
                    source_time,
                    attack.kind,
                    elapsed >= spec.windup,
                );
                assert_free(&sample(&library, &attack), &source);
            }
        }
        assert_free(
            &idle(&library, 0.0),
            &library.sample("Sword_Idle", 0.0, false),
        );
        for axis in [AttackAxis::Horizontal, AttackAxis::Vertical] {
            let charging = Charge {
                axis,
                elapsed: CHARGE_READY,
                combo: 0,
                dodge_followup: false,
            };
            let motion = motion(AttackKind::Focus(axis), false);
            let source = body_pose(
                &library,
                &motion,
                motion.times[0],
                AttackKind::Focus(axis),
                false,
            );
            assert_free(&charge(&library, &charging), &source);
        }
    }

    #[test]
    fn uppercut_turns_a_full_circle_and_followups_continue_forward() {
        let library = AnimationLibrary::default();
        let mut attack = Attack::new(AttackKind::Crouch(AttackAxis::Vertical));
        let spec = attack.spec();
        let base = sample(&library, &attack)[0].rotation;
        let yaw = |root: Transform| {
            let forward = root.rotation * base.inverse() * Vec3::NEG_Z;
            (-forward.x).atan2(-forward.z)
        };
        let delta = |a: f32, b: f32| (b - a).sin().atan2((b - a).cos());
        let mut previous = 0.0;
        let mut turn = 0.0;
        for frame in 0..=120 {
            attack.elapsed = spec.duration() * frame as f32 / 120.0;
            let pose = sample(&library, &attack);
            let current = yaw(pose[0]);
            let step = delta(previous, current);
            assert!(step > -0.0001, "spin reversed at frame {frame}");
            turn += step;
            previous = current;
        }
        assert!((turn - std::f32::consts::TAU).abs() < 0.001);

        attack.elapsed = spec.windup + spec.active;
        let root = sample(&library, &attack)[0];
        for axis in [AttackAxis::Horizontal, AttackAxis::Vertical] {
            let followup = sample(&library, &Attack::new(AttackKind::Air(axis)))[0];
            let mut previous = yaw(root);
            let mut remaining = 0.0;
            for frame in 1..=20 {
                let current = yaw(blend(root, followup, frame as f32 / 20.0));
                let step = delta(previous, current);
                assert!(step > 0.0, "aerial followup rewound the uppercut");
                remaining += step;
                previous = current;
            }
            assert!(remaining < std::f32::consts::PI);
        }
    }

    // Optional model-space export for inspecting the actual Rust animation,
    // including the sword socket. Normal test runs do not write artifacts.
    #[test]
    fn attack_pose_preview() {
        let Ok(path) = std::env::var("KIVA_ATTACK_PREVIEW") else {
            return;
        };
        let library = AnimationLibrary::default();
        let mut file = std::fs::File::create(path).unwrap();
        for (name, mut attack) in attacks() {
            let spec = attack.spec();
            let key_times = vec![
                0.0,
                spec.windup * 0.4,
                spec.windup * 0.65,
                spec.windup,
                spec.windup + spec.active * 0.5,
                spec.windup + spec.active,
                spec.duration(),
            ];
            let times = if let Some(frames) = std::env::var("KIVA_ATTACK_PREVIEW_FRAMES")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
            {
                let frames = frames.clamp(2, 600);
                (0..=frames)
                    .map(|frame| spec.duration() * frame as f32 / frames as f32)
                    .collect()
            } else {
                key_times
            };
            for (frame, time) in times.into_iter().enumerate() {
                attack.elapsed = time;
                let mut pose = sample(&library, &attack);
                if attack.kind == AttackKind::Crouch(AttackAxis::Horizontal) {
                    super::super::crouch_pose(&library, &mut pose, 1.0);
                }
                let model = library.model_pose(&pose);
                for (bone, transform) in library.bones.iter().zip(&model) {
                    let p = transform.translation;
                    writeln!(
                        file,
                        "{name},{frame},{time},{},{},{},{}",
                        bone.name, p.x, p.y, p.z
                    )
                    .unwrap();
                }
                let p = tip(&library, &pose);
                writeln!(
                    file,
                    "{name},{frame},{time},sword_tip,{},{},{}",
                    p.x, p.y, p.z
                )
                .unwrap();
            }
        }
    }
}
