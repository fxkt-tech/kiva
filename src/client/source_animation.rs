//! Recovered Messiah hierarchy and explicitly named source channels.
//! This module only decodes/samples immutable display data. Actor position,
//! combat timing and motion authority remain in simulation.
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

#[derive(Clone)]
struct Joint {
    parent: Option<usize>,
    rest: Transform,
}

struct Keys<T> {
    times: Vec<u16>,
    values: Vec<T>,
}

struct Track {
    joint: usize,
    translation: Keys<Vec3>,
    rotation: Keys<Quat>,
    scale: Keys<Vec3>,
}

struct Clip {
    duration: f32,
    tracks: Vec<Track>,
    root_heading: Quat,
}

#[derive(Resource)]
pub(super) struct SourceAnimationLibrary {
    joints: Vec<Joint>,
    names: HashMap<String, usize>,
    clips: Vec<Clip>,
    clip_names: HashMap<String, usize>,
    root: usize,
    pub(super) rest_model: Vec<Mat4>,
}

struct Reader<'a>(&'a [u8]);
type DecodeResult<T> = Result<T, &'static str>;
impl Reader<'_> {
    fn bytes(&mut self, count: usize) -> DecodeResult<&[u8]> {
        if count > self.0.len() {
            return Err("truncated source animation");
        }
        let (value, remaining) = self.0.split_at(count);
        self.0 = remaining;
        Ok(value)
    }
    fn u16(&mut self) -> DecodeResult<usize> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()) as usize)
    }
    fn f32(&mut self) -> DecodeResult<f32> {
        let value = f32::from_le_bytes(self.bytes(4)?.try_into().unwrap());
        value
            .is_finite()
            .then_some(value)
            .ok_or("non-finite source value")
    }
    fn string(&mut self) -> DecodeResult<String> {
        let length = self.u16()?;
        if !(1..=1024).contains(&length) {
            return Err("invalid source name length");
        }
        let value = std::str::from_utf8(self.bytes(length)?).map_err(|_| "invalid source UTF-8")?;
        if value.contains('\0') {
            return Err("NUL in source name");
        }
        Ok(value.to_owned())
    }
    fn vec3(&mut self) -> DecodeResult<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    fn quat(&mut self) -> DecodeResult<Quat> {
        let value = Quat::from_xyzw(self.f32()?, self.f32()?, self.f32()?, self.f32()?);
        if (value.length_squared() - 1.0).abs() > 0.001 {
            return Err("non-unit source quaternion");
        }
        Ok(value.normalize())
    }
    fn keys<T>(
        &mut self,
        duration: usize,
        mut read: impl FnMut(&mut Self) -> DecodeResult<T>,
    ) -> DecodeResult<Keys<T>> {
        let count = self.u16()?;
        let mut times = Vec::with_capacity(count);
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            let time = self.u16()?;
            if time > duration
                || times
                    .last()
                    .is_some_and(|&previous| previous >= time as u16)
            {
                return Err("invalid source key time");
            }
            times.push(time as u16);
            values.push(read(self)?);
        }
        Ok(Keys { times, values })
    }
}

impl<T: Copy> Keys<T> {
    fn sample(&self, milliseconds: f32, rest: T, interpolate: impl Fn(T, T, f32) -> T) -> T {
        let end = self
            .times
            .partition_point(|&time| (time as f32) <= milliseconds);
        if end == 0 {
            return self.values.first().copied().unwrap_or(rest);
        }
        if end == self.times.len() {
            return self.values[end - 1];
        }
        let start = end - 1;
        let weight = (milliseconds - self.times[start] as f32)
            / (self.times[end] - self.times[start]) as f32;
        interpolate(self.values[start], self.values[end], weight)
    }
}

impl Default for SourceAnimationLibrary {
    fn default() -> Self {
        Self::decode(include_bytes!("../../assets/choulingke/animations.bin"))
            .expect("verified bundled original animation asset")
    }
}

impl SourceAnimationLibrary {
    fn decode(bytes: &[u8]) -> DecodeResult<Self> {
        if bytes.len() > 64 * 1024 * 1024 {
            return Err("oversized source animation asset");
        }
        let mut reader = Reader(bytes);
        if reader.bytes(8)? != b"KIVASR01" {
            return Err("unsupported source animation format");
        }
        let joint_count = reader.u16()?;
        let clip_count = reader.u16()?;
        if !(1..=4096).contains(&joint_count) || !(1..=4096).contains(&clip_count) {
            return Err("invalid source joint/clip count");
        }
        let mut names = HashMap::new();
        let mut joints = Vec::with_capacity(joint_count);
        for index in 0..joint_count {
            if names.insert(reader.string()?, index).is_some() {
                return Err("duplicate source joint name");
            }
            let parent = reader.u16()?;
            if parent != 65535 && parent >= index || index != 0 && parent == 65535 {
                return Err("non-ancestral source parent");
            }
            let translation = reader.vec3()?;
            let rotation = reader.quat()?;
            let scale = reader.vec3()?;
            if scale.abs().min_element() < 0.00000001 {
                return Err("singular source rest scale");
            }
            joints.push(Joint {
                parent: (parent != 65535).then_some(parent),
                rest: Transform {
                    translation,
                    rotation,
                    scale,
                },
            });
        }
        let root = *names.get("Bip001").ok_or("missing source motion root")?;
        let rest_heading = heading(joints[root].rest.rotation);
        let mut clips = Vec::with_capacity(clip_count);
        let mut clip_names = HashMap::new();
        for index in 0..clip_count {
            if clip_names.insert(reader.string()?, index).is_some() {
                return Err("duplicate source clip name");
            }
            let duration = reader.u16()?;
            let track_count = reader.u16()?;
            if duration == 0 || track_count == 0 || track_count > joint_count {
                return Err("invalid source clip duration/track count");
            }
            let mut tracks = Vec::with_capacity(track_count);
            let mut seen = HashSet::new();
            let mut root_heading = None;
            for _ in 0..track_count {
                let joint = reader.u16()?;
                if joint >= joint_count || !seen.insert(joint) {
                    return Err("invalid or duplicate source track joint");
                }
                let translation = reader.keys(duration, Reader::vec3)?;
                let rotation = reader.keys(duration, Reader::quat)?;
                let scale = reader.keys(duration, Reader::vec3)?;
                if scale.values.iter().any(|value| value.min_element() <= 0.0) {
                    return Err("non-positive source key scale");
                }
                if joint == root {
                    root_heading = Some(
                        rest_heading
                            * heading(
                                rotation
                                    .values
                                    .first()
                                    .copied()
                                    .unwrap_or(joints[root].rest.rotation),
                            )
                            .inverse(),
                    );
                }
                tracks.push(Track {
                    joint,
                    translation,
                    rotation,
                    scale,
                });
            }
            clips.push(Clip {
                duration: duration as f32 / 1000.0,
                tracks,
                root_heading: root_heading.ok_or("clip lacks original motion root")?,
            });
        }
        if !reader.0.is_empty() {
            return Err("trailing source animation data");
        }
        let mut result = Self {
            joints,
            names,
            clips,
            clip_names,
            root,
            rest_model: Vec::new(),
        };
        result.rest_model = result.model_pose(
            &result
                .joints
                .iter()
                .map(|joint| joint.rest)
                .collect::<Vec<_>>(),
        );
        Ok(result)
    }

    pub(super) fn bone(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub(super) fn clip(&self, name: &str) -> Option<usize> {
        self.clip_names.get(name).copied()
    }

    pub(super) fn clips(&self) -> Vec<(&str, f32)> {
        let mut clips: Vec<_> = self
            .clip_names
            .iter()
            .map(|(name, &index)| (name.as_str(), self.clips[index].duration))
            .collect();
        clips.sort_by(|a, b| a.0.cmp(b.0));
        clips
    }

    pub(super) fn clip_name(&self, index: usize) -> Option<&str> {
        self.clip_names
            .iter()
            .find_map(|(name, &value)| (value == index).then_some(name.as_str()))
    }

    /// Sparse missing channels use actual local rest TRS, never a flat identity.
    /// Strip only planar root travel. Source vertical bends remain visible;
    /// airborne simulation supplies the world lift exactly once.
    pub(super) fn sample(
        &self,
        clip: usize,
        seconds: f32,
        looping: bool,
        airborne: bool,
    ) -> Vec<Transform> {
        let time = self.sample_time(clip, seconds, looping) * 1000.0;
        let clip = &self.clips[clip];
        let mut local: Vec<_> = self.joints.iter().map(|joint| joint.rest).collect();
        for track in &clip.tracks {
            let transform = &mut local[track.joint];
            transform.translation =
                track
                    .translation
                    .sample(time, transform.translation, Vec3::lerp);
            transform.rotation = track
                .rotation
                .sample(time, transform.rotation, |a, b, weight| {
                    a.slerp(b, weight).normalize()
                });
            transform.scale = track.scale.sample(time, transform.scale, Vec3::lerp);
        }
        let root = &mut local[self.root];
        root.translation.x = self.joints[self.root].rest.translation.x;
        root.translation.z = self.joints[self.root].rest.translation.z;
        if airborne {
            root.translation.y = self.joints[self.root].rest.translation.y;
        }
        root.rotation = clip.root_heading * root.rotation;
        local
    }

    pub(super) fn sample_time(&self, clip: usize, seconds: f32, looping: bool) -> f32 {
        let clip = &self.clips[clip];
        if !seconds.is_finite() {
            0.0
        } else if looping {
            seconds.rem_euclid(clip.duration)
        } else {
            seconds.clamp(0.0, clip.duration)
        }
    }

    pub(super) fn model_pose(&self, local: &[Transform]) -> Vec<Mat4> {
        assert_eq!(local.len(), self.joints.len());
        let mut model: Vec<Mat4> = Vec::with_capacity(local.len());
        for (joint, local) in self.joints.iter().zip(local) {
            let matrix = local.to_matrix();
            model.push(joint.parent.map_or(matrix, |parent| model[parent] * matrix));
        }
        model
    }
}

fn heading(rotation: Quat) -> Quat {
    Quat::from_rotation_y(rotation.to_euler(EulerRot::YXZ).0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_asset_validates_all_clips_and_exact_rest_hierarchy() {
        let library = SourceAnimationLibrary::default();
        assert_eq!(library.joints.len(), 986);
        assert_eq!(library.clips.len(), 35);
        assert_eq!(
            library.joints[library.bone("Bip001 Pelvis").unwrap()].parent,
            library.bone("Bip001")
        );
        for (index, clip) in library.clips.iter().enumerate() {
            for time in [0.0, clip.duration * 0.5, clip.duration] {
                let local = library.sample(index, time, false, false);
                assert!(
                    local
                        .iter()
                        .all(|t| t.is_finite() && (t.rotation.length() - 1.0).abs() < 0.0001)
                );
                assert!(library.model_pose(&local).iter().all(|m| m.is_finite()));
            }
        }
    }

    #[test]
    fn sparse_channels_rest_root_travel_removed_and_real_waist_motion_preserved() {
        let library = SourceAnimationLibrary::default();
        let clip = library.clip("skill_ylk_1").unwrap();
        let a = library.sample(clip, 0.0, false, false);
        let b = library.sample(clip, 2.7, false, false);
        let root = library.root;
        assert_eq!(a[root].translation.xz(), b[root].translation.xz());
        assert!((a[root].translation.y - b[root].translation.y).abs() > 0.5);
        let air = library.sample(clip, 2.7, false, true);
        assert_eq!(air[root].translation, library.joints[root].rest.translation);
        let hand = library.bone("Bip001 R Hand").unwrap();
        assert!(
            library.model_pose(&a)[hand]
                .w_axis
                .xyz()
                .distance(library.model_pose(&b)[hand].w_axis.xyz())
                > 0.3
        );
        let missing = (0..library.joints.len())
            .find(|&joint| {
                !library.clips[clip]
                    .tracks
                    .iter()
                    .any(|track| track.joint == joint)
            })
            .unwrap();
        assert_eq!(b[missing], library.joints[missing].rest);
        let end = library.sample(clip, 999.0, false, false);
        assert_eq!(
            end,
            library.sample(clip, library.clips[clip].duration, false, false)
        );
        assert_eq!(
            a,
            library.sample(clip, library.clips[clip].duration, true, false)
        );
    }

    #[test]
    fn bounds_bad_version_and_trailing_bytes_are_rejected() {
        let bytes = include_bytes!("../../assets/choulingke/animations.bin");
        for end in [0, 7, 11, 20, bytes.len() / 2, bytes.len() - 1] {
            assert!(SourceAnimationLibrary::decode(&bytes[..end]).is_err());
        }
        let mut bad = bytes.to_vec();
        bad[7] = b'9';
        assert!(SourceAnimationLibrary::decode(&bad).is_err());
        let mut extra = bytes.to_vec();
        extra.push(0);
        assert!(SourceAnimationLibrary::decode(&extra).is_err());
    }

    #[test]
    fn sparse_sampler_uses_slerp_and_clamps_key_boundaries() {
        let keys = Keys {
            times: vec![10, 30],
            values: vec![Quat::IDENTITY, Quat::from_rotation_y(1.0)],
        };
        let sample = |time| keys.sample(time, Quat::IDENTITY, Quat::slerp);
        assert!(sample(20.0).angle_between(Quat::from_rotation_y(0.5)) < 0.001);
        assert_eq!(sample(-1.0), keys.values[0]);
        assert_eq!(sample(40.0), keys.values[1]);
        let absent = Keys::<Vec3> {
            times: vec![],
            values: vec![],
        };
        assert_eq!(absent.sample(10.0, Vec3::Y, Vec3::lerp), Vec3::Y);
    }
}
