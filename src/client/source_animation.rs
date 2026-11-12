//! Immutable, independently indexed source rigs. Rules and actor roots remain in simulation.
mod layers;

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

/// Identifies an actual baked hierarchy, not merely the originating game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SourceRig {
    #[default]
    Yanyun,
    Naraka,
}
impl SourceRig {
    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Yanyun => "yanyun/choulingke",
            Self::Naraka => "naraka/l22_male_skeleton",
        }
    }
    pub(super) fn prefix(self) -> &'static str {
        match self {
            Self::Yanyun => "Bip001",
            Self::Naraka => "gMan",
        }
    }
}

/// Declared target composition. The immutable native matrices stay intact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SourceScope {
    #[default]
    FullBody,
    UpperBody,
    Fingers,
}
impl SourceScope {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::FullBody => "source full body",
            Self::UpperBody => "source upper body / fingers + CC0 lower body",
            Self::Fingers => "source fingers + CC0 body",
        }
    }
}

#[derive(Clone)]
struct Joint {
    parent: Option<usize>,
    rest: Transform,
}
struct Keys<T> {
    times: Vec<f32>,
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

/// Existing methods select Yanyun for compatibility. `for_rig` selects another
/// independently decoded library; no pose arrays or bone indices are shared.
#[derive(Resource)]
pub(super) struct SourceAnimationLibrary {
    rig: SourceRig,
    secondary: Option<Box<Self>>,
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
    fn u32(&mut self) -> DecodeResult<usize> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()) as usize)
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
        duration: f32,
        precise: bool,
        width: usize,
        mut read: impl FnMut(&mut Self) -> DecodeResult<T>,
    ) -> DecodeResult<Keys<T>> {
        let count = if precise { self.u32()? } else { self.u16()? };
        // Bound allocations before reading any attacker-controlled element count.
        let stride = width * 4 + if precise { 4 } else { 2 };
        if count > 1_000_000 || count > self.0.len() / stride {
            return Err("invalid source key count");
        }
        let mut times = Vec::with_capacity(count);
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            let time = if precise {
                self.f32()?
            } else {
                self.u16()? as f32 / 1000.0
            };
            if time < 0.0 || time > duration || times.last().is_some_and(|&v| v >= time) {
                return Err("invalid source key time");
            }
            times.push(time);
            values.push(read(self)?);
        }
        Ok(Keys { times, values })
    }
}
impl<T: Copy> Keys<T> {
    fn sample(&self, seconds: f32, rest: T, interpolate: impl Fn(T, T, f32) -> T) -> T {
        let end = self.times.partition_point(|&time| time <= seconds);
        if end == 0 {
            return self.values.first().copied().unwrap_or(rest);
        }
        if end == self.times.len() {
            return self.values[end - 1];
        }
        let start = end - 1;
        if seconds == self.times[start] {
            return self.values[start];
        }
        // Adjacent f32 timestamps can deliberately bracket a native Humanoid
        // branch cut. Their positive dt must not be widened to EPSILON.
        let weight = (seconds - self.times[start]) / (self.times[end] - self.times[start]);
        interpolate(self.values[start], self.values[end], weight)
    }
}
/// The primary (Yanyun) and secondary (Naraka) source rigs.
pub(super) const PATHS: [&str; 2] = ["animations/yanyun.bin", "animations/naraka.bin"];

impl FromWorld for SourceAnimationLibrary {
    fn from_world(world: &mut World) -> Self {
        let [primary, secondary] = PATHS.map(|path| super::bundle::take(world, path));
        Self {
            secondary: Some(Box::new(
                Self::decode(&secondary).expect("verified native Humanoid animation asset"),
            )),
            ..Self::decode(&primary).expect("verified bundled original animation asset")
        }
    }
}
impl SourceAnimationLibrary {
    /// Headless fixtures decode the shipped files directly.
    #[cfg(test)]
    pub(super) fn bundled() -> Self {
        Self::from_world(&mut World::new())
    }

    fn decode(bytes: &[u8]) -> DecodeResult<Self> {
        if bytes.len() > 64 * 1024 * 1024 {
            return Err("oversized source animation asset");
        }
        let mut reader = Reader(bytes);
        let magic = reader.bytes(8)?;
        let precise = match magic {
            b"KIVASR01" => false,
            b"KIVASR02" => true,
            _ => return Err("unsupported source animation format"),
        };
        let rig = if precise {
            if reader.string()? != SourceRig::Naraka.id() {
                return Err("unsupported source rig identity");
            }
            SourceRig::Naraka
        } else {
            SourceRig::Yanyun
        };
        let joint_count = reader.u16()?;
        let clip_count = reader.u16()?;
        if !(1..=4096).contains(&joint_count) || !(1..=4096).contains(&clip_count) {
            return Err("invalid source joint/clip count");
        }
        let root_index = if precise {
            let index = reader.u16()?;
            if reader.u16()? != 0 {
                return Err("unsupported source flags");
            }
            Some(index)
        } else {
            None
        };
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
            if scale.abs().min_element() < 1e-8 {
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
        let root = match root_index {
            Some(index) if index < joint_count => index,
            Some(_) => return Err("invalid source motion root"),
            None => *names.get("Bip001").ok_or("missing source motion root")?,
        };
        let rest_heading = heading(joints[root].rest.rotation);
        let mut clips = Vec::with_capacity(clip_count);
        let mut clip_names = HashMap::new();
        for index in 0..clip_count {
            if clip_names.insert(reader.string()?, index).is_some() {
                return Err("duplicate source clip name");
            }
            let duration = if precise {
                reader.f32()?
            } else {
                reader.u16()? as f32 / 1000.0
            };
            let track_count = reader.u16()?;
            if duration <= 0.0 || duration > 3600.0 || track_count == 0 || track_count > joint_count
            {
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
                let translation = reader.keys(duration, precise, 3, Reader::vec3)?;
                let rotation = reader.keys(duration, precise, 4, Reader::quat)?;
                let scale = reader.keys(duration, precise, 3, Reader::vec3)?;
                if scale
                    .values
                    .iter()
                    .any(|v| v.abs().min_element() < 1e-8 || !precise && v.min_element() <= 0.0)
                {
                    return Err("singular source key scale");
                }
                if joint == root {
                    root_heading = Some(if precise {
                        Quat::IDENTITY
                    } else {
                        rest_heading
                            * heading(
                                rotation
                                    .values
                                    .first()
                                    .copied()
                                    .unwrap_or(joints[root].rest.rotation),
                            )
                            .inverse()
                    });
                }
                tracks.push(Track {
                    joint,
                    translation,
                    rotation,
                    scale,
                });
            }
            clips.push(Clip {
                duration,
                tracks,
                root_heading: root_heading.ok_or("clip lacks original motion root")?,
            });
        }
        if !reader.0.is_empty() {
            return Err("trailing source animation data");
        }
        let mut result = Self {
            rig,
            secondary: None,
            joints,
            names,
            clips,
            clip_names,
            root,
            rest_model: Vec::new(),
        };
        result.rest_model =
            result.model_pose(&result.joints.iter().map(|j| j.rest).collect::<Vec<_>>());
        Ok(result)
    }
    pub(super) fn for_rig(&self, rig: SourceRig) -> Option<&Self> {
        if self.rig == rig {
            Some(self)
        } else {
            self.secondary
                .as_deref()
                .filter(|library| library.rig == rig)
        }
    }
    pub(super) fn rig(&self) -> SourceRig {
        self.rig
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
            .map(|(name, &i)| (name.as_str(), self.clips[i].duration))
            .collect();
        clips.sort_by(|a, b| a.0.cmp(b.0));
        clips
    }
    pub(super) fn clip_name(&self, index: usize) -> Option<&str> {
        self.clip_names
            .iter()
            .find_map(|(name, &i)| (i == index).then_some(name.as_str()))
    }
    pub(super) fn duration(&self, clip: usize) -> f32 {
        self.clips[clip].duration
    }
    pub(super) fn scope(&self, clip: usize) -> SourceScope {
        if self.rig == SourceRig::Naraka
            && let Some(name) = self.clip_name(clip)
        {
            if name.contains("_finger_") {
                return SourceScope::Fingers;
            }
            // Native root extraction leaves these weapon layers with Hips
            // near/below zero. Declare the same target composition in game and
            // inspection instead of silently presenting them as a full body.
            if name.contains("_sniper_") || name.contains("_fk_") || name == "male_bow_attack_01" {
                return SourceScope::UpperBody;
            }
        }
        SourceScope::FullBody
    }

    pub(super) fn layer_description(&self, clip: usize) -> String {
        if self.rig == SourceRig::Yanyun {
            return "original named source channels".into();
        }
        let Some((left, right)) = self.clip_name(clip).and_then(layers::finger_layers) else {
            return "unregistered composition metadata".into();
        };
        if left.is_empty() && right.is_empty() {
            return "native main; no supplemental finger layers".into();
        }
        format!(
            "native main priority; Kiva missing-chain supplements: L={} / R={} (original controller weights unknown)",
            if left.is_empty() { "none" } else { left },
            if right.is_empty() { "none" } else { right }
        )
    }

    /// Local source channels are already fully composed offline. A missing
    /// baked track keeps its true rest; missing Humanoid fingers are not filled here.
    pub(super) fn sample(
        &self,
        clip: usize,
        seconds: f32,
        looping: bool,
        airborne: bool,
    ) -> Vec<Transform> {
        let time = self.sample_time(clip, seconds, looping);
        let clip = &self.clips[clip];
        let mut local: Vec<_> = self.joints.iter().map(|joint| joint.rest).collect();
        for track in &clip.tracks {
            let transform = &mut local[track.joint];
            transform.translation =
                track
                    .translation
                    .sample(time, transform.translation, Vec3::lerp);
            transform.rotation = track.rotation.sample(time, transform.rotation, |a, b, w| {
                a.slerp(b, w).normalize()
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
        if airborne && self.rig == SourceRig::Naraka {
            // Native applyRootMotion=false leaves MotionRoot stationary but
            // encodes jumping body elevation in Hips. Physics already supplies
            // actor height; keep the actual leg rotations without a second lift.
            if let Some(pelvis) = self.bone("gMan Pelvis") {
                local[pelvis].translation.y = self.joints[pelvis].rest.translation.y;
            }
        }
        local
    }
    pub(super) fn sample_time(&self, clip: usize, seconds: f32, looping: bool) -> f32 {
        let duration = self.clips[clip].duration;
        if !seconds.is_finite() {
            0.0
        } else if looping {
            seconds.rem_euclid(duration)
        } else {
            seconds.clamp(0.0, duration)
        }
    }
    pub(super) fn model_pose(&self, local: &[Transform]) -> Vec<Mat4> {
        assert_eq!(local.len(), self.joints.len());
        let mut model = Vec::with_capacity(local.len());
        for (joint, local) in self.joints.iter().zip(local) {
            let m = local.to_matrix();
            model.push(joint.parent.map_or(m, |parent| model[parent] * m));
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
        let library = SourceAnimationLibrary::bundled();
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
        let library = SourceAnimationLibrary::bundled();
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
        let bytes = crate::client::bundle::take(&mut World::new(), PATHS[0]);
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
            times: vec![10.0, 30.0],
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
    fn precise_fixture(times: &[f32]) -> Vec<u8> {
        fn u16_value(bytes: &mut Vec<u8>, value: u16) {
            bytes.extend(value.to_le_bytes());
        }
        fn string(bytes: &mut Vec<u8>, value: &str) {
            u16_value(bytes, value.len() as u16);
            bytes.extend(value.as_bytes());
        }
        fn float(bytes: &mut Vec<u8>, value: f32) {
            bytes.extend(value.to_le_bytes());
        }
        let mut bytes = b"KIVASR02".to_vec();
        string(&mut bytes, SourceRig::Naraka.id());
        for value in [2, 1, 0, 0] {
            u16_value(&mut bytes, value);
        }
        for (name, parent) in [("MotionRoot", 65535), ("gMan Pelvis", 0)] {
            string(&mut bytes, name);
            u16_value(&mut bytes, parent);
            for value in [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0] {
                float(&mut bytes, value);
            }
        }
        string(&mut bytes, "precision");
        float(&mut bytes, 1.0);
        u16_value(&mut bytes, 2);
        for joint in 0..2 {
            u16_value(&mut bytes, joint);
            bytes.extend((times.len() as u32).to_le_bytes());
            for &time in times {
                for value in [time, 0.0, time, 0.0] {
                    float(&mut bytes, value);
                }
            }
            bytes.extend(0u32.to_le_bytes());
            bytes.extend(0u32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn precise_seconds_and_independent_rig_indices_are_preserved() {
        let library =
            SourceAnimationLibrary::decode(&precise_fixture(&[0.0, 1.0 / 30.0, 1.0])).unwrap();
        assert_eq!(library.rig(), SourceRig::Naraka);
        assert_eq!(library.joints.len(), 2);
        assert!(library.for_rig(SourceRig::Yanyun).is_none());
        let local = library.sample(0, 1.0 / 60.0, false, false);
        assert!((local[1].translation.y - 1.0 / 60.0).abs() < 1e-8);
        assert_eq!(library.clips[0].tracks[0].translation.times[1], 1.0 / 30.0);
        let yanyun = SourceAnimationLibrary {
            secondary: Some(Box::new(library)),
            ..SourceAnimationLibrary::bundled()
        };
        assert_eq!(yanyun.for_rig(SourceRig::Yanyun).unwrap().joints.len(), 986);
        assert_eq!(yanyun.for_rig(SourceRig::Naraka).unwrap().joints.len(), 2);
    }

    #[test]
    fn precise_decoder_bounds_counts_times_identity_and_trailing_data() {
        for times in [
            &[-0.01, 0.5][..],
            &[0.1, 0.1],
            &[0.2, 0.1],
            &[0.0, 1.001],
            &[f32::NAN],
        ] {
            assert!(SourceAnimationLibrary::decode(&precise_fixture(times)).is_err());
        }
        let bytes = precise_fixture(&[0.0, 1.0]);
        for end in [0, 7, 9, bytes.len() - 1] {
            assert!(SourceAnimationLibrary::decode(&bytes[..end]).is_err());
        }
        let mut bad = bytes.clone();
        bad[10] = b'x';
        assert!(SourceAnimationLibrary::decode(&bad).is_err());
        let mut extra = bytes;
        extra.push(0);
        assert!(SourceAnimationLibrary::decode(&extra).is_err());
        let count = u32::MAX.to_le_bytes();
        let mut reader = Reader(&count);
        assert!(reader.keys(1.0, true, 3, Reader::vec3).is_err());
    }
    #[test]
    fn native_airborne_removes_only_the_extra_hips_height_and_preserves_leg_pose() {
        let libraries = SourceAnimationLibrary::bundled();
        let source = libraries
            .for_rig(SourceRig::Naraka)
            .expect("verified native Naraka library");
        assert_eq!(source.joints.len(), 63);
        assert_eq!(source.root, source.bone("MotionRoot").unwrap());
        let pelvis = source.bone("gMan Pelvis").unwrap();
        let foot = source.bone("gMan R Foot").unwrap();
        for name in [
            "male_hengdao_jump_inplace_up_01",
            "male_hengdao_jump_inplace_up_02",
            "male_hengdao_jump_down_loop_01",
        ] {
            let clip = source.clip(name).unwrap();
            for time in [0.0, source.duration(clip) * 0.4, source.duration(clip)] {
                let raw = source.model_pose(&source.sample(clip, time, false, false));
                let display = source.model_pose(&source.sample(clip, time, false, true));
                assert!(
                    (display[pelvis].w_axis.y - source.rest_model[pelvis].w_axis.y).abs() < 0.00002,
                    "{name} {time}: duplicate jumping height"
                );
                let relative = |model: &[Mat4]| model[pelvis].inverse() * model[foot];
                assert!(
                    relative(&raw).abs_diff_eq(relative(&display), 0.00002),
                    "{name}: removing actor-height changed leg pose"
                );
            }
        }
        let clip = source.clip("male_hengdao_jump_inplace_up_01").unwrap();
        let start = source.model_pose(&source.sample(clip, 0.0, false, false));
        let end = source.model_pose(&source.sample(clip, source.duration(clip), false, false));
        assert!(
            (start[pelvis].w_axis.y - end[pelvis].w_axis.y).abs() > 0.5,
            "raw source comparison lost its original body elevation"
        );
    }
    #[test]
    fn every_native_clip_has_explicit_verified_composition_metadata() {
        let libraries = SourceAnimationLibrary::bundled();
        let native = libraries.for_rig(SourceRig::Naraka).unwrap();
        for (name, _) in native.clips() {
            assert!(
                layers::finger_layers(name).is_some(),
                "{name}: update actual native composition metadata"
            );
        }
    }
    #[test]
    fn adjacent_float_native_cuts_keep_exact_keys_and_true_positive_sub_epsilon_dt() {
        let time = 0.25f32;
        let adjacent = f32::from_bits(time.to_bits() + 1);
        let keys = Keys {
            times: vec![time, adjacent],
            values: vec![Quat::from_rotation_x(0.7), Quat::from_rotation_y(2.8)],
        };
        for (index, seconds) in keys.times.iter().copied().enumerate() {
            assert_eq!(
                keys.sample(seconds, Quat::IDENTITY, |_, _, _| panic!(
                    "exact native key must not interpolate"
                )),
                keys.values[index]
            );
        }
        let end = f32::from_bits(time.to_bits() + 2);
        let interpolate = Keys {
            times: vec![time, end],
            values: vec![Vec3::ZERO, Vec3::Y],
        };
        assert!((end - time).abs() < f32::EPSILON);
        assert_eq!(
            interpolate.sample(adjacent, Vec3::ZERO, Vec3::lerp),
            Vec3::Y * 0.5
        );
        let decoded =
            SourceAnimationLibrary::decode(&precise_fixture(&[time, adjacent, end])).unwrap();
        assert_eq!(
            decoded.clips[0].tracks[0].translation.times,
            vec![time, adjacent, end]
        );
    }
}
