//! Local bone poses imported from GLB. Rigid segments use the source skeleton,
//! avoiding skin weights and retargeting while retaining authored animation.
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
            assert!(parent < index as i16);
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
            let frames = (0..reader.u16())
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
    fn clips_are_finite_and_have_no_root_translation() {
        let library = AnimationLibrary::default();
        for name in library.clips.keys() {
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
    fn sword_hand_moves_and_loop_wraps() {
        let library = AnimationLibrary::default();
        let hand = library.bone("hand_r");
        let start = library.model_pose(&library.sample("Sword_Regular_A", 0.0, false));
        let end = library.model_pose(&library.sample("Sword_Regular_A", 0.4, false));
        assert!(start[hand].translation.distance(end[hand].translation) > 0.2);
        let a = library.sample("Jog_Fwd_Loop", 0.2, true);
        let b = library.sample("Jog_Fwd_Loop", library.duration("Jog_Fwd_Loop") + 0.2, true);
        for (a, b) in a.iter().zip(b) {
            assert!(a.translation.distance(b.translation) < 0.0001);
            assert!(a.rotation.dot(b.rotation).abs() > 0.9999);
        }
    }
}
