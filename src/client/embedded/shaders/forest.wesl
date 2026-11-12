// Authored material extension for Kiva's reconstructed bamboo clearing.
// Geometry, skinning, shadow/depth vertex passes remain the standard Bevy paths.
import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, calculate_tbn_mikktspace, apply_normal_mapping},
};

import kiva::forest_view::{style, hash21, camera_leaf_discard};

@if(PREPASS_PIPELINE)
import bevy_pbr::{
    prepass::io::{VertexOutput, FragmentOutput},
    deferred::functions::deferred_output,
};
@else
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

@group(constants::MATERIAL_BIND_GROUP) @binding(101) var cover_base: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var cover_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var cover_normal: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var cover_normal_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var<uniform> cover_strength: f32;

fn noise(point: vec2<f32>) -> f32 {
    let cell = floor(point);
    let f = fract(point);
    let weight = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(cell), hash21(cell + vec2<f32>(1.0, 0.0)), weight.x),
               mix(hash21(cell + vec2<f32>(0.0, 1.0)), hash21(cell + vec2<f32>(1.0)), weight.x), weight.y);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var surface = pbr_input_from_standard_material(in, is_front);
    let p = in.world_position.xyz * style.z;
    let kind = style.x;
    let footprint = max(length(dpdx(p.xz)), length(dpdy(p.xz)));
    // Same authored 2.5m world-space UV scale as the base soil. Explicit
    // gradients keep optional layering independent of alpha/discard flow.
    let cover_uv = in.world_position.xz * 0.4;
    let cover_dx = dpdx(cover_uv);
    let cover_dy = dpdy(cover_uv);
    // Reject the transparent majority of the source branch atlas before the
    // procedural surface work. Derivatives are evaluated above the discard.
    surface.material.base_color = alpha_discard(surface.material.flags, surface.material.alpha_cutoff, surface.material.base_color);
    camera_leaf_discard(in.world_position.xyz, in.position.xy);
    let fine_visibility = 1.0 - smoothstep(0.018, 0.07, footprint);
    let grain = mix(0.5, noise(p.xz * 27.0), fine_visibility);
    let patches = noise(p.xz * 0.65) * 0.65 + noise(p.xz * 2.8) * 0.35;
    var tint = vec3<f32>(1.0);

    if kind < 0.5 {
        // Soil/loam, small stones and a restrained moist sheen. World-space
        // detail avoids stretching the original shared collision geometry.
        tint = mix(vec3<f32>(0.90, 0.94, 0.89), vec3<f32>(1.06, 1.02, 0.94), patches);
        tint *= 0.94 + grain * 0.12;
        // The material's dry roughness remains authoritative. Wetness is a
        // local artistic mask over the real base/normal maps, never metallic.
        let basin = noise(p.xz * 0.18 + vec2<f32>(11.4, -7.42)) * 0.7
                  + noise(p.xz * 0.46 + vec2<f32>(-2.7, 6.46)) * 0.3;
        // A small authored depression in the visible fighting ground. Its
        // irregular shoreline changes only material response, never geometry.
        let pool_point = (in.world_position.xz - vec2<f32>(-0.9, -2.35)) / vec2<f32>(1.35, 0.8);
        let pool_edge = (noise(in.world_position.xz * 1.3 + vec2<f32>(4.7, -2.1)) - 0.5) * 0.24;
        let local_pool = 1.0 - smoothstep(0.72, 1.0, length(pool_point) + pool_edge);
        let wetness = max(smoothstep(0.34, 0.58, basin), local_pool) * clamp(style.w, 0.0, 1.0);
        let level_ground = smoothstep(0.85, 0.98, surface.world_normal.y)
                         * (1.0 - smoothstep(0.15, 0.8, in.world_position.y));
        let puddle = max(smoothstep(0.58, 0.70, basin), local_pool) * wetness * level_ground;
        var cover = 0.0;
        if cover_strength > 0.0 {
            let cover_patch = noise(p.xz * 0.14 + vec2<f32>(3.7, 9.4)) * 0.65
                            + noise(p.xz * 0.43 + vec2<f32>(-5.1, 2.3)) * 0.35;
            cover = smoothstep(0.34, 0.64, cover_patch) * clamp(cover_strength, 0.0, 1.0)
                  * (1.0 - puddle * 0.75);
            let cover_color = textureSampleGrad(cover_base, cover_sampler, cover_uv, cover_dx, cover_dy).rgb;
            surface.material.base_color = vec4<f32>(mix(surface.material.base_color.rgb, cover_color, cover), surface.material.base_color.a);
@if(VERTEX_TANGENTS) {
            let cover_Nt = textureSampleGrad(cover_normal, cover_normal_sampler, cover_uv, cover_dx, cover_dy).rgb;
            let TBN = calculate_tbn_mikktspace(surface.world_normal, in.world_tangent);
            let cover_N = apply_normal_mapping(surface.material.flags, TBN, false, true, cover_Nt);
            surface.N = normalize(mix(surface.N, cover_N, cover * 0.7));
}
        }
        let dry_roughness = clamp(surface.material.perceptual_roughness * (0.94 + grain * 0.12), 0.35, 1.0);
        surface.material.perceptual_roughness = mix(mix(dry_roughness, 0.56, wetness), 0.14, puddle);
        let soil_luminance = dot(surface.material.base_color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        let damp_color = soil_luminance * vec3<f32>(0.64, 0.72, 0.76);
        // Damp mud loses saturation, while the recovered moss/grass layer
        // keeps its source green rather than being recoloured into grey soil.
        let desaturation = wetness * mix(0.6, 0.05, cover);
        surface.material.base_color = vec4<f32>(mix(surface.material.base_color.rgb, damp_color, desaturation), surface.material.base_color.a);
        tint *= mix(1.0, 0.86, wetness);
        let moss = smoothstep(0.62, 0.86, patches) * 0.2;
        tint = mix(tint, vec3<f32>(0.62, 0.80, 0.54), moss);
        // The recovered normal is for rough soil, not water ripples. Damp
        // earth keeps restrained detail; water cores are geometrically smooth
        // before their roughness enters the SSR range, avoiding sparkling mud.
        let normal_strength = mix(0.55, 0.25, wetness) * (1.0 - smoothstep(0.05, 0.6, puddle));
        surface.N = normalize(mix(normalize(surface.world_normal), surface.N, normal_strength));
    } else if kind < 1.5 {
        // Longitudinal culm fibres; the real bamboo mesh and joint rings stay
        // fixed. Surface variation never displaces contact geometry.
        let fibres = sin(p.x * 73.0 + p.z * 91.0 + sin(p.y * 4.0) * 0.3);
        let wax = noise(vec2<f32>(p.x + p.z, p.y) * vec2<f32>(9.0, 0.8));
        tint = vec3<f32>(0.77, 0.94, 0.80) * (0.91 + fibres * 0.07 + wax * 0.13);
        surface.material.perceptual_roughness = clamp(surface.material.perceptual_roughness * (0.85 + wax * 0.3), 0.08, 1.0);
        surface.material.clearcoat = 0.12;
        surface.material.clearcoat_perceptual_roughness = 0.3;
    } else if kind < 2.5 {
        let lichen = smoothstep(0.51, 0.78, patches) * max(surface.N.y, 0.0);
        tint = mix(vec3<f32>(0.88, 0.91, 0.87), vec3<f32>(0.65, 0.81, 0.56), lichen * 0.55);
        tint *= 0.82 + grain * 0.3;
        surface.material.perceptual_roughness = clamp(surface.material.perceptual_roughness * (0.92 + grain * 0.12), 0.08, 1.0);
    } else {
        // Thin leaves keep standard double-sided lighting and diffuse
        // transmission, with small local colour variation rather than glow.
        tint = vec3<f32>(0.84, 1.02, 0.76) * (0.88 + noise(p.xz * 11.0) * 0.16);
    }
    surface.material.base_color = vec4<f32>(surface.material.base_color.rgb * mix(vec3<f32>(1.0), tint, style.y), surface.material.base_color.a);

@if(PREPASS_PIPELINE)
    let out = deferred_output(in, surface);
@if(!PREPASS_PIPELINE)
    var out: FragmentOutput;
@if(!PREPASS_PIPELINE)
    out.color = apply_pbr_lighting(surface);
@if(!PREPASS_PIPELINE)
    out.color = main_pass_post_lighting_processing(surface, out.color);
    return out;
}
