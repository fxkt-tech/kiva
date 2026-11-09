//! Trusted, bundled CC0 bone animation. Rigid segments retain the authored rig
//! without skin weights or retargeting. Simulation owns all world/root motion.
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub rest: Transform,
}
struct Clip {
    duration: f32,
    frames: Vec<Vec<Transform>>,
}
#[derive(Resource)]
pub(super) struct AnimationLibrary {
    pub bones: Vec<Bone>,
    clips: HashMap<String, Clip>,
}
// Explicit LE decoding of the trusted, bundled asset, on native and WASM.
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let (head, tail) = self.0.split_at(N);
        self.0 = tail;
        head.try_into().unwrap()
    }
    fn u16(&mut self) -> usize {
        u16::from_le_bytes(self.take()) as usize
    }
    fn f32(&mut self) -> f32 {
        f32::from_le_bytes(self.take())
    }
    fn string(&mut self) -> String {
        let length = self.u16();
        let (head, tail) = self.0.split_at(length);
        self.0 = tail;
        String::from_utf8(head.to_vec()).unwrap()
    }
    fn pose(&mut self) -> Transform {
        Transform::from_translation(Vec3::new(self.f32(), self.f32(), self.f32())).with_rotation(
            Quat::from_xyzw(self.f32(), self.f32(), self.f32(), self.f32()).normalize(),
        )
    }
}
impl Default for AnimationLibrary {
    fn default() -> Self {
        let mut reader = Reader(include_bytes!("../../assets/animations/stickman.anim"));
        assert_eq!(&reader.take::<8>(), b"KIVAAN01");
        let count = reader.u16();
        let clip_count = reader.u16();
        let mut bones = Vec::with_capacity(count);
        for index in 0..count {
            let name = reader.string();
            let parent = i16::from_le_bytes(reader.take());
            assert!((-1..index as i16).contains(&parent));
            let rest =
                reader
                    .pose()
                    .with_scale(Vec3::new(reader.f32(), reader.f32(), reader.f32()));
            bones.push(Bone {
                name,
                parent: (parent >= 0).then_some(parent as usize),
                rest,
            });
        }
        let mut clips = HashMap::new();
        for _ in 0..clip_count {
            let name = reader.string();
            let duration = reader.f32();
            let frame_count = reader.u16();
            assert!(duration.is_finite() && duration > 0.0 && frame_count >= 2);
            let frames = (0..frame_count)
                .map(|_| {
                    bones
                        .iter()
                        .map(|bone| reader.pose().with_scale(bone.rest.scale))
                        .collect()
                })
                .collect();
            clips.insert(name, Clip { duration, frames });
        }
        assert!(reader.0.is_empty());
        Self { bones, clips }
    }
}
pub(super) fn blend(a: Transform, b: Transform, weight: f32) -> Transform {
    Transform {
        translation: a.translation.lerp(b.translation, weight),
        rotation: a.rotation.slerp(b.rotation, weight).normalize(),
        scale: a.scale.lerp(b.scale, weight),
    }
}
impl AnimationLibrary {
    pub fn bone(&self, name: &str) -> usize {
        self.bones
            .iter()
            .position(|bone| bone.name == name)
            .expect("bundled bone")
    }
    pub fn duration(&self, name: &str) -> f32 {
        self.clips[name].duration
    }
    pub fn sample(&self, name: &str, seconds: f32, looping: bool) -> Vec<Transform> {
        let clip = &self.clips[name];
        let seconds = if looping {
            seconds.rem_euclid(clip.duration)
        } else {
            seconds.clamp(0.0, clip.duration)
        };
        let frame = seconds / clip.duration * (clip.frames.len() - 1) as f32;
        let lo = frame.floor() as usize;
        let hi = (lo + 1).min(clip.frames.len() - 1);
        clip.frames[lo]
            .iter()
            .zip(&clip.frames[hi])
            .map(|(&a, &b)| blend(a, b, frame.fract()))
            .collect()
    }
    pub fn model_pose(&self, local: &[Transform]) -> Vec<Transform> {
        assert_eq!(local.len(), self.bones.len());
        let mut model: Vec<Transform> = Vec::with_capacity(self.bones.len());
        // Source faces +Z, gameplay faces -Z.
        let basis = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI))
            .with_scale(Vec3::splat(1.1));
        for (bone, &pose) in self.bones.iter().zip(local) {
            model.push(
                bone.parent
                    .map_or(basis, |parent| model[parent])
                    .mul_transform(pose),
            );
        }
        model
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_clips_keep_the_source_hierarchy_required_by_the_rigid_renderer() {
        let library = AnimationLibrary::default();
        assert_eq!(
            include_bytes!("../../assets/animations/stickman.anim").len(),
            578_598
        );
        assert_eq!(library.bones.len(), 23);
        assert_eq!(library.clips.len(), 19);
        assert_eq!(library.bones[library.bone("root")].parent, None);
        for (child, parent) in [
            ("pelvis", "root"),
            ("spine_01", "pelvis"),
            ("spine_02", "spine_01"),
            ("spine_03", "spine_02"),
            ("neck_01", "spine_03"),
            ("Head", "neck_01"),
            ("clavicle_l", "spine_03"),
            ("upperarm_l", "clavicle_l"),
            ("lowerarm_l", "upperarm_l"),
            ("hand_l", "lowerarm_l"),
            ("clavicle_r", "spine_03"),
            ("upperarm_r", "clavicle_r"),
            ("lowerarm_r", "upperarm_r"),
            ("hand_r", "lowerarm_r"),
            ("thigh_l", "pelvis"),
            ("calf_l", "thigh_l"),
            ("foot_l", "calf_l"),
            ("ball_l", "foot_l"),
            ("thigh_r", "pelvis"),
            ("calf_r", "thigh_r"),
            ("foot_r", "calf_r"),
            ("ball_r", "foot_r"),
        ] {
            assert_eq!(
                library.bones[library.bone(child)].parent,
                Some(library.bone(parent))
            );
        }
    }

    #[test]
    fn clips_are_finite_and_have_no_root_translation() {
        let library = AnimationLibrary::default();
        for (name, clip) in &library.clips {
            for frame in &clip.frames {
                assert_eq!(frame.len(), 23);
                assert!(
                    frame[0].translation.length() < 0.0001,
                    "root motion in {name}"
                );
                for transform in frame {
                    assert!(transform.translation.is_finite());
                    assert!(transform.rotation.is_finite());
                    assert!(transform.scale.is_finite());
                    assert!((transform.rotation.length() - 1.0).abs() < 0.001);
                }
            }
            for step in 0..=20 {
                let pose = library.sample(name, library.duration(name) * step as f32 / 20.0, false);
                assert!(
                    pose[0].translation.length() < 0.0001,
                    "root motion in {name}"
                );
                for bone in library.model_pose(&pose) {
                    assert!(bone.translation.is_finite() && bone.rotation.is_finite());
                    assert!((bone.rotation.length() - 1.0).abs() < 0.001);
                }
            }
        }
    }
    #[test]
    fn authored_hand_motion_is_preserved() {
        let library = AnimationLibrary::default();
        let hand = library.bone("hand_r");
        let start = library.model_pose(&library.sample("Sword_Regular_A", 0.0, false));
        let end = library.model_pose(&library.sample("Sword_Regular_A", 0.4, false));
        assert!(start[hand].translation.distance(end[hand].translation) > 0.2);
    }

    #[test]
    fn all_looped_clips_wrap_and_one_shots_clamp_at_both_ends() {
        let library = AnimationLibrary::default();
        for name in library.clips.keys() {
            let duration = library.duration(name);
            let time = duration * 0.3;
            let sample = library.sample(name, time, true);
            for wrapped in [time + duration, time - duration] {
                let other = library.sample(name, wrapped, true);
                assert_same_pose(&sample, &other);
            }
            assert_same_pose(
                &library.sample(name, 0.0, false),
                &library.sample(name, -1.0, false),
            );
            assert_same_pose(
                &library.sample(name, duration, false),
                &library.sample(name, duration + 1.0, false),
            );
        }
    }

    fn assert_same_pose(a: &[Transform], b: &[Transform]) {
        assert_eq!(a.len(), b.len());
        for (a, b) in a.iter().zip(b) {
            assert!(a.translation.distance(b.translation) < 0.0001);
            assert!(a.rotation.dot(b.rotation).abs() > 0.9999);
            assert!(a.scale.distance(b.scale) < 0.0001);
        }
    }

    #[test]
    fn pose_blend_has_exact_endpoints_and_keeps_unit_rotations() {
        let library = AnimationLibrary::default();
        let a = library.sample("Sword_Idle", 0.0, true);
        let b = library.sample("Roll", 0.4, false);
        for (a, b) in a.into_iter().zip(b) {
            assert_same_pose(&[a], &[blend(a, b, 0.0)]);
            assert_same_pose(&[b], &[blend(a, b, 1.0)]);
            let middle = blend(a, b, 0.5);
            assert!(middle.translation.is_finite() && middle.scale.is_finite());
            assert!((middle.rotation.length() - 1.0).abs() < 0.0001);
        }
    }
}
