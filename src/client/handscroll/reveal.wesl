import bevy_ui_render::ui_vertex_output::UiVertexOutput;

@group(1) @binding(0) var painting: texture_2d<f32>;
@group(1) @binding(1) var ink_sampler: sampler;
@group(1) @binding(2) var<uniform> spread: vec4<f32>;

fn grain(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let uv = smoothstep(vec2(0.0), vec2(1.0), fract(p));
    let seed = dot(cell, vec2(19.17, 41.07));
    let samples = fract(sin(seed + vec4(0.0, 19.17, 41.07, 60.24)) * 43758.54);
    return mix(mix(samples.x, samples.y, uv.x), mix(samples.z, samples.w, uv.x), uv.y) - 0.5;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let aspect = vec2(1.77778, 1.0);
    let p = in.uv * aspect;
    let radius = length(max(spread.xy, vec2(1.0) - spread.xy) * aspect);
    let distance = length(p - spread.xy * aspect) + 0.13 * grain(p * 11.0) + 0.035 * grain(p * 38.0);
    let opacity = smoothstep(-0.025, 0.02, distance - mix(-0.12, radius + 0.12, spread.z));
    let color = textureSample(painting, ink_sampler, in.uv);
    return vec4(color.rgb, opacity);
}
