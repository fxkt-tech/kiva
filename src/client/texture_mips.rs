//! One-time mip generation for the bundled opaque, repeating RGBA terrain tiles.
//! Atlas cutouts use a separate coverage policy and do not call this helper.
use bevy::{prelude::*, render::render_resource::TextureFormat};

pub(super) fn generate(image: &mut Image, normal: bool) {
    if image.texture_descriptor.mip_level_count > 1 {
        return;
    }
    let format = image.texture_descriptor.format;
    assert!(matches!(
        format,
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb
    ));
    let srgb = format.is_srgb();
    assert!(!normal || !srgb, "normal tiles must use linear samples");
    let size = image.texture_descriptor.size;
    let (mut width, mut height) = (size.width as usize, size.height as usize);
    assert!(width.is_power_of_two() && height.is_power_of_two());
    assert_eq!(size.depth_or_array_layers, 1);
    let data = image
        .data
        .as_mut()
        .expect("bundled tile retains CPU texels");
    assert_eq!(data.len(), width * height * 4);
    // Keep level zero byte-for-byte identical to the provenance-checked source.
    let mut level: Vec<Vec4> = data
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| {
            let color = pixel.map(|value| f32::from(value) / 255.0);
            let rgb = if normal {
                (Vec3::from_array([color[0], color[1], color[2]]) * 2.0 - Vec3::ONE)
                    .normalize_or(Vec3::Z)
            } else if srgb {
                Vec3::new(
                    Srgba::gamma_function(color[0]),
                    Srgba::gamma_function(color[1]),
                    Srgba::gamma_function(color[2]),
                )
            } else {
                Vec3::new(color[0], color[1], color[2])
            };
            rgb.extend(color[3])
        })
        .collect();
    while width > 1 || height > 1 {
        let (next_width, next_height) = ((width / 2).max(1), (height / 2).max(1));
        let mut next = Vec::with_capacity(next_width * next_height);
        for y in 0..next_height {
            for x in 0..next_width {
                let mut value = Vec4::ZERO;
                for dy in 0..2 {
                    for dx in 0..2 {
                        value += level
                            [(2 * y + dy).min(height - 1) * width + (2 * x + dx).min(width - 1)]
                            * 0.25;
                    }
                }
                if normal {
                    value = value.xyz().normalize_or(Vec3::Z).extend(value.w);
                }
                let rgb = if normal {
                    value.xyz() * 0.5 + Vec3::splat(0.5)
                } else if srgb {
                    Vec3::new(
                        Srgba::gamma_function_inverse(value.x),
                        Srgba::gamma_function_inverse(value.y),
                        Srgba::gamma_function_inverse(value.z),
                    )
                } else {
                    value.xyz()
                };
                data.extend(
                    rgb.extend(value.w)
                        .to_array()
                        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
                );
                next.push(value);
            }
        }
        level = next;
        (width, height) = (next_width, next_height);
        image.texture_descriptor.mip_level_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::RenderAssetUsages,
        render::render_resource::{Extent3d, TextureDimension},
    };

    fn tile(width: u32, height: u32, data: Vec<u8>, format: TextureFormat) -> Image {
        Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            format,
            RenderAssetUsages::default(),
        )
    }

    #[test]
    fn color_mips_average_light_and_preserve_original_level() {
        let source = [
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [0, 0, 0, 255],
        ]
        .concat();
        let mut image = tile(2, 2, source.clone(), TextureFormat::Rgba8UnormSrgb);
        generate(&mut image, false);
        let data = image.data.as_ref().unwrap();
        assert_eq!(&data[..16], source);
        assert_eq!(&data[16..], &[188, 188, 188, 255]);
        assert_eq!(image.texture_descriptor.mip_level_count, 2);
    }

    #[test]
    fn normal_mips_average_directions_without_srgb_or_losing_unit_length() {
        let source = [
            [218, 128, 218, 255],
            [37, 128, 218, 255],
            [128, 218, 218, 255],
            [128, 37, 218, 255],
        ]
        .concat();
        let mut image = tile(2, 2, source, TextureFormat::Rgba8Unorm);
        generate(&mut image, true);
        assert_eq!(&image.data.as_ref().unwrap()[16..], &[128, 128, 255, 255]);
    }

    #[test]
    fn rectangular_chain_reaches_one_texel_and_is_not_generated_twice() {
        let mut image = tile(4, 2, vec![128; 32], TextureFormat::Rgba8UnormSrgb);
        generate(&mut image, false);
        assert_eq!(image.texture_descriptor.mip_level_count, 3);
        assert_eq!(image.data.as_ref().unwrap().len(), 32 + 8 + 4);
        let data = image.data.clone();
        generate(&mut image, false);
        assert_eq!(image.data, data);
    }
}
