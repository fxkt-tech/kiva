import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var reflection: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var water_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var<uniform> seconds: f32;

// A few small drops from the broken eaves. Each wave dissipates into still water.
fn drop(p: vec2<f32>, phase: f32) -> f32 {
    let age = fract(phase + seconds * 0.34);
    let front = length(p) - age * 0.77;
    let envelope = (1.0 - smoothstep(0.0, 0.17, abs(front))) * (1.0 - age);
    return sin(front * 126.0) * envelope;
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let p = in.world_position.xz;
    let wind = sin(p.x * 4.1 + p.y * 12.3 + sin(p.y * 1.7) + seconds * 1.1);
    let crosswind = sin(p.y * 16.8 - p.x * 3.1 + seconds * 1.7);
    let ripple = drop(p - vec2<f32>(-1.72, 2.7), 0.24)
               + drop(p - vec2<f32>(1.96, 0.8), 0.53)
               + drop(p - vec2<f32>(-0.82, -0.83), 0.87);
    let perturbation = vec2<f32>(wind + ripple, crosswind + ripple) * 0.00084;
    let radiance = textureSample(reflection, water_sampler, in.uv + perturbation).rgb;
    let crest = max(0.0, wind * 0.58 + crosswind * 0.42);
    let silver = 0.009 * pow(crest, 7.0) + 0.025 * max(0.0, ripple);
    var out: FragmentOutput;
    out.color = vec4<f32>(radiance * in.color.rgb + vec3<f32>(silver), in.color.a);
    return out;
}
