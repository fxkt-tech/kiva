// Authored material extension for Kiva's reconstructed bamboo clearing.
// Geometry, skinning, shadow/depth vertex passes remain the standard Bevy paths.
import bevy_pbr::render::{
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
};

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

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> style: vec4<f32>;

fn hash21(point: vec2<f32>) -> f32 {
    let p = fract(point * vec2<f32>(123.34, 456.21));
    let q = p + dot(p, p + vec2<f32>(45.32));
    return fract(q.x * q.y);
}

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
    // Reject the transparent majority of the source branch atlas before the
    // procedural surface work. Derivatives are evaluated above the discard.
    surface.material.base_color = alpha_discard(surface.material.flags, surface.material.alpha_cutoff, surface.material.base_color);
    if kind > 2.5 && kind < 3.5 {
        // Whole source branch cards can cross the follow camera. Fade only
        // nearby leaf fragments, using this view's real camera position and
        // a fixed pixel pattern so pause needs no separate animation clock.
        // Ground litter uses the soil kind and never enters this branch.
        let coverage = smoothstep(3.0, 5.5, distance(in.world_position.xyz, view.world_position));
        if coverage <= hash21(floor(in.position.xy)) {
            discard;
        }
    }
    let fine_visibility = 1.0 - smoothstep(0.018, 0.07, footprint);
    let grain = mix(0.5, noise(p.xz * 27.0), fine_visibility);
    let patches = noise(p.xz * 0.65) * 0.65 + noise(p.xz * 2.8) * 0.35;
    var tint = vec3<f32>(1.0);

    if kind < 0.5 {
        // Soil/loam, small stones and a restrained moist sheen. World-space
        // detail avoids stretching the original shared collision geometry.
        tint = mix(vec3<f32>(0.69, 0.73, 0.67), vec3<f32>(1.18, 1.10, 0.92), patches);
        tint *= 0.86 + grain * 0.24;
        surface.material.perceptual_roughness = 0.83 + grain * 0.13;
        let moss = smoothstep(0.62, 0.86, patches) * 0.2;
        tint = mix(tint, vec3<f32>(0.62, 0.80, 0.54), moss);
        let fine = vec2<f32>(sin(p.x * 83.0 + sin(p.z * 47.0)), cos(p.z * 79.0 + p.x * 29.0));
        surface.N = normalize(surface.N + vec3<f32>(fine.x, 0.0, fine.y) * 0.055 * style.y * fine_visibility);
    } else if kind < 1.5 {
        // Longitudinal culm fibres; the real bamboo mesh and joint rings stay
        // fixed. Surface variation never displaces contact geometry.
        let fibres = sin(p.x * 73.0 + p.z * 91.0 + sin(p.y * 4.0) * 0.3);
        let wax = noise(vec2<f32>(p.x + p.z, p.y) * vec2<f32>(9.0, 0.8));
        tint = vec3<f32>(0.77, 0.94, 0.80) * (0.91 + fibres * 0.07 + wax * 0.13);
        surface.material.perceptual_roughness = 0.44 + wax * 0.22;
        surface.material.clearcoat = 0.12;
        surface.material.clearcoat_perceptual_roughness = 0.3;
    } else if kind < 2.5 {
        let lichen = smoothstep(0.51, 0.78, patches) * max(surface.N.y, 0.0);
        tint = mix(vec3<f32>(0.88, 0.91, 0.87), vec3<f32>(0.65, 0.81, 0.56), lichen * 0.55);
        tint *= 0.82 + grain * 0.3;
        surface.material.perceptual_roughness = 0.87;
    } else {
        // Thin leaves keep standard double-sided lighting and diffuse
        // transmission, with small local colour variation rather than glow.
        tint = vec3<f32>(0.84, 1.02, 0.76) * (0.88 + noise(p.xz * 11.0) * 0.16);
        surface.material.perceptual_roughness = 0.64;
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
