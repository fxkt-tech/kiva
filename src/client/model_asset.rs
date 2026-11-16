//! Shared bounded binary, texture and PBR decoding for bundled model exports.
use super::character_effects::SurfaceEffect;
use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    prelude::*,
};

// Explicit LE decoding of a trusted build artifact, matching import-choulingke.py.
pub(super) struct Reader<'a>(pub(super) &'a [u8]);
impl Reader<'_> {
    pub(super) fn take<const N: usize>(&mut self) -> [u8; N] {
        self.bytes(N).try_into().unwrap()
    }
    pub(super) fn bytes(&mut self, length: usize) -> &[u8] {
        let (head, tail) = self.0.split_at(length);
        self.0 = tail;
        head
    }
    pub(super) fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }
    pub(super) fn u16(&mut self) -> usize {
        u16::from_le_bytes(self.take()) as usize
    }
    pub(super) fn u32(&mut self) -> usize {
        u32::from_le_bytes(self.take()) as usize
    }
    pub(super) fn f32(&mut self) -> f32 {
        let value = f32::from_le_bytes(self.take());
        assert!(value.is_finite(), "finite bundled model value");
        value
    }
    pub(super) fn string(&mut self) -> String {
        let length = self.u16();
        String::from_utf8(self.bytes(length).to_vec()).unwrap()
    }
    pub(super) fn vec3(&mut self) -> Vec3 {
        Vec3::new(self.f32(), self.f32(), self.f32())
    }
    pub(super) fn matrix(&mut self) -> Mat4 {
        Mat4::from_cols_array(&std::array::from_fn(|_| self.f32()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MaterialFormat {
    Legacy,
    DoubleSided,
    Emissive,
}

pub(super) struct DecodedMaterial {
    pub material: StandardMaterial,
    pub effect: SurfaceEffect,
}

pub(super) fn materials(
    world: &mut World,
    reader: &mut Reader<'_>,
    format: MaterialFormat,
) -> Vec<DecodedMaterial> {
    let texture_count = reader.u16();
    let mut textures = Vec::new();
    for _ in 0..texture_count {
        reader.string(); // Exact texture GUID is also retained in the provenance header.
        let srgb = reader.u8() != 0;
        let length = reader.u32();
        // Authored UVs include negative tiles and U > 1 (tops, pants,
        // sheath/blade). Clamp collapses these regions to edge colours.
        let mut sampler = ImageSamplerDescriptor::linear();
        sampler.address_mode_u = ImageAddressMode::Repeat;
        sampler.address_mode_v = ImageAddressMode::Repeat;
        let image = Image::from_buffer(
            reader.bytes(length),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            srgb,
            ImageSampler::Descriptor(sampler),
            RenderAssetUsages::default(),
        )
        .expect("verified bundled avatar PNG");
        textures.push(world.resource_mut::<Assets<Image>>().add(image));
    }
    let mut materials = Vec::new();
    let material_count = reader.u16();
    for _ in 0..material_count {
        reader.string();
        let base = i16::from_le_bytes(reader.take());
        let normal = i16::from_le_bytes(reader.take());
        let surface = i16::from_le_bytes(reader.take());
        assert!(
            [base, normal, surface]
                .iter()
                .all(|&index| index == -1 || (index >= 0 && (index as usize) < textures.len()))
        );
        let alpha_flag = reader.u8();
        if format == MaterialFormat::Emissive {
            assert!(alpha_flag <= 1);
        }
        let alpha = alpha_flag != 0;
        let roughness = reader.f32();
        let metallic = reader.f32();
        let reflectance = reader.f32();
        let anisotropy_strength = reader.f32();
        let clearcoat = reader.f32();
        let clearcoat_perceptual_roughness = reader.f32();
        let diffuse_transmission = reader.f32();
        let tint = reader.vec3();
        let alpha_cutoff = reader.f32();
        let double_sided = if format == MaterialFormat::Legacy {
            true
        } else {
            let flag = reader.u8();
            if format == MaterialFormat::Emissive {
                assert!(flag <= 1);
            }
            flag != 0
        };
        if format == MaterialFormat::Emissive {
            assert!(
                [
                    roughness,
                    metallic,
                    reflectance,
                    anisotropy_strength,
                    clearcoat,
                    clearcoat_perceptual_roughness,
                    diffuse_transmission,
                    alpha_cutoff,
                    tint.x,
                    tint.y,
                    tint.z
                ]
                .iter()
                .all(|v| (0.0..=1.0).contains(v)),
                "bounded authored PBR material factors"
            );
        }
        let (alpha_mode, opacity, emissive, emissive_texture) =
            if format == MaterialFormat::Emissive {
                let mode = reader.u8();
                let opacity = reader.f32();
                assert!((0.0..=1.0).contains(&opacity), "bounded material opacity");
                let emission = reader.vec3();
                assert!(
                    emission
                        .to_array()
                        .iter()
                        .all(|v| (0.0..=1000.0).contains(v)),
                    "bounded linear emissive response"
                );
                let texture = i16::from_le_bytes(reader.take());
                assert!(texture == -1 || (texture >= 0 && (texture as usize) < textures.len()));
                let mode = match mode {
                    0 => AlphaMode::Opaque,
                    1 => AlphaMode::Mask(alpha_cutoff),
                    2 => AlphaMode::Blend,
                    _ => panic!("verified material alpha mode"),
                };
                (
                    mode,
                    opacity,
                    LinearRgba::rgb(emission.x, emission.y, emission.z),
                    (texture >= 0).then(|| textures[texture as usize].clone()),
                )
            } else {
                (
                    if alpha {
                        AlphaMode::Mask(alpha_cutoff)
                    } else {
                        AlphaMode::Opaque
                    },
                    1.0,
                    LinearRgba::BLACK,
                    None,
                )
            };
        assert_eq!(
            alpha,
            matches!(alpha_mode, AlphaMode::Mask(_)),
            "consistent mask flags"
        );
        let effect = if format == MaterialFormat::Emissive {
            SurfaceEffect::from_byte(reader.u8())
        } else {
            SurfaceEffect::None
        };
        assert!(
            effect == SurfaceEffect::None || alpha_mode == AlphaMode::Blend,
            "animated surface effect requires dedicated alpha blend material"
        );
        let material = StandardMaterial {
            base_color: Color::linear_rgba(tint.x, tint.y, tint.z, opacity),
            base_color_texture: (base >= 0).then(|| textures[base as usize].clone()),
            normal_map_texture: (normal >= 0).then(|| textures[normal as usize].clone()),
            metallic_roughness_texture: (surface >= 0).then(|| textures[surface as usize].clone()),
            occlusion_texture: (surface >= 0).then(|| textures[surface as usize].clone()),
            perceptual_roughness: roughness,
            metallic,
            reflectance,
            anisotropy_strength,
            anisotropy_rotation: std::f32::consts::FRAC_PI_2,
            clearcoat,
            clearcoat_perceptual_roughness,
            diffuse_transmission,
            alpha_mode,
            emissive,
            emissive_texture,
            double_sided,
            cull_mode: (!double_sided).then_some(bevy::render::render_resource::Face::Back),
            ..default()
        };
        materials.push(DecodedMaterial { material, effect });
    }
    materials
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(format: MaterialFormat) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(0u16.to_le_bytes()); // no textures
        data.extend(1u16.to_le_bytes());
        data.extend(0u16.to_le_bytes()); // material name
        for _ in 0..3 {
            data.extend((-1i16).to_le_bytes());
        }
        data.push(0); // old mask flag
        for value in [0.7f32, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.3, 0.4, 0.5, 0.5] {
            data.extend(value.to_le_bytes());
        }
        if format != MaterialFormat::Legacy {
            data.push(1);
        }
        if format == MaterialFormat::Emissive {
            data.push(2); // alpha blend
            for value in [0.35f32, 4.0, 0.5, 0.1] {
                data.extend(value.to_le_bytes());
            }
            data.extend((-1i16).to_le_bytes());
            data.push(0); // no animated surface effect
        }
        data
    }

    fn decode(raw: &[u8], format: MaterialFormat) -> DecodedMaterial {
        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        let mut reader = Reader(raw);
        let mut decoded = materials(&mut world, &mut reader, format);
        assert!(reader.0.is_empty());
        decoded.pop().unwrap()
    }

    #[test]
    fn new_material_emission_and_alpha_do_not_change_legacy_layouts() {
        for format in [MaterialFormat::Legacy, MaterialFormat::DoubleSided] {
            let decoded = decode(&fixture(format), format);
            assert_eq!(decoded.material.alpha_mode, AlphaMode::Opaque);
            assert_eq!(decoded.material.emissive, LinearRgba::BLACK);
            assert_eq!(decoded.material.base_color.to_linear().alpha, 1.0);
        }
        let decoded = decode(&fixture(MaterialFormat::Emissive), MaterialFormat::Emissive);
        assert_eq!(decoded.material.alpha_mode, AlphaMode::Blend);
        assert_eq!(decoded.material.base_color.to_linear().alpha, 0.35);
        assert_eq!(decoded.material.emissive, LinearRgba::rgb(4.0, 0.5, 0.1));
        assert!(decoded.material.double_sided);
        let mut mask = fixture(MaterialFormat::Emissive);
        let tail = mask.len() - 20;
        mask[tail - 46] = 1;
        mask[tail] = 1;
        assert_eq!(
            decode(&mask, MaterialFormat::Emissive).material.alpha_mode,
            AlphaMode::Mask(0.5)
        );
        let mut animated = fixture(MaterialFormat::Emissive);
        *animated.last_mut().unwrap() = 1;
        assert_eq!(
            decode(&animated, MaterialFormat::Emissive).effect,
            SurfaceEffect::Ember
        );
    }

    #[test]
    fn new_material_tail_rejects_unknown_modes_indices_nonfinite_and_unbounded_values() {
        let raw = fixture(MaterialFormat::Emissive);
        let tail = raw.len() - 20;
        let mut invalid = Vec::new();
        for (offset, value) in [
            (tail, 3),
            (raw.len() - 1, 2),
            (tail - 1, 2),
            (tail - 46, 2),
            (tail, 1),
        ] {
            let mut item = raw.clone();
            item[offset] = value;
            invalid.push(item);
        }
        for (offset, value) in [
            (tail + 1, 1.1f32),
            (tail + 1, -0.1),
            (tail + 5, -1.0),
            (tail + 5, 1001.0),
            (tail + 5, f32::NAN),
            (tail - 45, -0.1),
            (tail - 17, 1.1),
        ] {
            let mut item = raw.clone();
            item[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            invalid.push(item);
        }
        let mut bad_texture = raw.clone();
        bad_texture[raw.len() - 3..raw.len() - 1].copy_from_slice(&0i16.to_le_bytes());
        invalid.push(bad_texture);
        invalid.push(raw[..raw.len() - 1].to_vec());
        for item in invalid {
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    decode(&item, MaterialFormat::Emissive);
                }))
                .is_err()
            );
        }
    }
}
