//! The bundled CC0 local bone tracks, shared by native and Web builds.
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub(super) struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub rest: Transform,
}

struct Track {
    duration: f32,
    frames: Vec<Vec<Transform>>,
}

#[derive(Clone, Copy)]
pub(super) enum Clip {
    Idle,
    Run,
    Attack,
    Recover,
    Block,
    Air,
    Hit,
    Dash,
}

impl Clip {
    fn name(self) -> &'static str {
        match self {
            Self::Idle => "Sword_Idle",
            Self::Run => "Jog_Fwd_Loop",
            Self::Attack => "Sword_Regular_A",
            Self::Recover => "Sword_Regular_A_Rec",
            Self::Block => "Sword_Block",
            Self::Air => "Jump_Loop",
            Self::Hit => "Hit_Chest",
            Self::Dash => "Sword_Dash",
        }
    }
}

#[derive(Resource)]
pub(in crate::client) struct AnimationLibrary {
    pub(super) bones: Vec<Bone>,
    tracks: HashMap<String, Track>,
}

// This is a trusted embedded build asset, not an external file format/API.
// Reject a broken export at startup instead of displaying corrupt transforms.
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let head = self.0.get(..N).expect("truncated bundled animation");
        self.0 = &self.0[N..];
        head.try_into().expect("fixed-size animation field")
    }
    fn u16(&mut self) -> usize {
        u16::from_le_bytes(self.take()) as usize
    }
    fn f32(&mut self) -> f32 {
        let value = f32::from_le_bytes(self.take());
        assert!(value.is_finite(), "non-finite bundled animation");
        value
    }
    fn string(&mut self) -> String {
        let length = self.u16();
        let head = self.0.get(..length).expect("truncated animation name");
        self.0 = &self.0[length..];
        let name = std::str::from_utf8(head).expect("UTF-8 animation name");
        assert!(!name.is_empty(), "empty animation name");
        name.to_owned()
    }
    fn pose(&mut self) -> Transform {
        let translation = Vec3::new(self.f32(), self.f32(), self.f32());
        let rotation = Quat::from_xyzw(self.f32(), self.f32(), self.f32(), self.f32());
        assert!((rotation.length() - 1.0).abs() < 0.001);
        Transform::from_translation(translation).with_rotation(rotation.normalize())
    }
}

impl Default for AnimationLibrary {
    fn default() -> Self {
        Self::decode(include_bytes!("../../../assets/animations/stickman.anim"))
    }
}

impl AnimationLibrary {
    fn decode(bytes: &[u8]) -> Self {
        let mut reader = Reader(bytes);
        assert_eq!(&reader.take::<8>(), b"KIVAAN01");
        let count = reader.u16();
        let track_count = reader.u16();
        assert_eq!(count, 23, "bundled rigid rig has 23 bones");
        let mut bones = Vec::with_capacity(count);
        let mut names = HashSet::new();
        for index in 0..count {
            let name = reader.string();
            assert!(names.insert(name.clone()), "duplicate bone");
            let parent = i16::from_le_bytes(reader.take());
            assert!(if index == 0 {
                parent == -1
            } else {
                parent >= 0 && (parent as usize) < index
            });
            let rest =
                reader
                    .pose()
                    .with_scale(Vec3::new(reader.f32(), reader.f32(), reader.f32()));
            assert!(rest.scale.cmpgt(Vec3::ZERO).all());
            bones.push(Bone {
                name,
                parent: (parent >= 0).then_some(parent as usize),
                rest,
            });
        }
        let mut tracks = HashMap::new();
        for _ in 0..track_count {
            let name = reader.string();
            let duration = reader.f32();
            let frame_count = reader.u16();
            assert!(duration > 0.0 && frame_count >= 2);
            let mut frames = Vec::with_capacity(frame_count);
            for _ in 0..frame_count {
                let frame: Vec<_> = bones
                    .iter()
                    .map(|bone| reader.pose().with_scale(bone.rest.scale))
                    .collect();
                assert!(frame[0].translation.length() < 0.0001, "root motion");
                frames.push(frame);
            }
            assert!(tracks.insert(name, Track { duration, frames }).is_none());
        }
        assert!(reader.0.is_empty(), "unexpected trailing animation data");
        let library = Self { bones, tracks };
        for clip in [
            Clip::Idle,
            Clip::Run,
            Clip::Attack,
            Clip::Recover,
            Clip::Block,
            Clip::Air,
            Clip::Hit,
            Clip::Dash,
        ] {
            assert!(
                library.tracks.contains_key(clip.name()),
                "missing used clip"
            );
        }
        library
    }

    pub(super) fn bone(&self, name: &str) -> usize {
        self.bones
            .iter()
            .position(|bone| bone.name == name)
            .expect("required bundled bone")
    }

    pub(super) fn duration(&self, clip: Clip) -> f32 {
        self.tracks[clip.name()].duration
    }

    pub(super) fn sample(&self, clip: Clip, seconds: f32, looping: bool) -> Vec<Transform> {
        let track = &self.tracks[clip.name()];
        let seconds = if looping {
            seconds.rem_euclid(track.duration)
        } else {
            seconds.clamp(0.0, track.duration)
        };
        let frame = seconds / track.duration * (track.frames.len() - 1) as f32;
        let lo = (frame.floor() as usize).min(track.frames.len() - 1);
        let hi = (lo + 1).min(track.frames.len() - 1);
        track.frames[lo]
            .iter()
            .zip(&track.frames[hi])
            .map(|(&a, &b)| blend(a, b, frame.fract()))
            .collect()
    }

    pub(super) fn model_pose(&self, local: &[Transform]) -> Vec<Transform> {
        assert_eq!(local.len(), self.bones.len());
        let mut model: Vec<Transform> = Vec::with_capacity(self.bones.len());
        // The source faces +Z; gameplay's local forward is -Z.
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

pub(super) fn blend(a: Transform, b: Transform, weight: f32) -> Transform {
    Transform {
        translation: a.translation.lerp(b.translation, weight),
        rotation: a.rotation.slerp(b.rotation, weight).normalize(),
        scale: a.scale.lerp(b.scale, weight),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_imported_track_is_finite_normalized_and_has_no_root_translation() {
        let library = AnimationLibrary::default();
        assert_eq!(library.tracks.len(), 19);
        for track in library.tracks.values() {
            for frame in &track.frames {
                assert!(frame[0].translation.length() < 0.0001);
                for bone in library.model_pose(frame) {
                    assert!(bone.translation.is_finite() && bone.scale.is_finite());
                    assert!((bone.rotation.length() - 1.0).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn locomotion_wraps_and_authored_hand_motion_is_retained() {
        let library = AnimationLibrary::default();
        let a = library.sample(Clip::Run, 0.2, true);
        let b = library.sample(Clip::Run, library.duration(Clip::Run) + 0.2, true);
        for (a, b) in a.iter().zip(b) {
            assert!(a.translation.distance(b.translation) < 0.0001);
            assert!(a.rotation.dot(b.rotation).abs() > 0.9999);
        }
        let hand = library.bone("hand_r");
        let a = library.model_pose(&library.sample(Clip::Attack, 0.0, false));
        let b = library.model_pose(&library.sample(Clip::Attack, 0.4, false));
        assert!(a[hand].translation.distance(b[hand].translation) > 0.2);
    }
}
