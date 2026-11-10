//! Shared bounded binary, texture and PBR decoding for bundled model exports.
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

pub(super) fn materials(
    world: &mut World,
    reader: &mut Reader<'_>,
    source_pose: bool,
) -> Vec<StandardMaterial> {
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
        let alpha = reader.u8() != 0;
        let roughness = reader.f32();
        let metallic = reader.f32();
        let reflectance = reader.f32();
        let anisotropy_strength = reader.f32();
        let clearcoat = reader.f32();
        let clearcoat_perceptual_roughness = reader.f32();
        let diffuse_transmission = reader.f32();
        let tint = reader.vec3();
        let alpha_cutoff = reader.f32();
        let double_sided = !source_pose || reader.u8() != 0;
        let material = StandardMaterial {
            base_color: Color::linear_rgb(tint.x, tint.y, tint.z),
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
            alpha_mode: if alpha {
                AlphaMode::Mask(alpha_cutoff)
            } else {
                AlphaMode::Opaque
            },
            double_sided,
            cull_mode: (!double_sided).then_some(bevy::render::render_resource::Face::Back),
            ..default()
        };
        materials.push(material);
    }
    materials
}
