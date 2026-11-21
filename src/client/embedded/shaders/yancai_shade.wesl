// Yancai pass 1 (HDR, before tonemapping): band the light on deferred
// surfaces and draw ink outlines. Forward pixels (lighting id 0) carry stale
// gbuffer data from the surface behind them, so they are never re-shaded;
// the forest material bands them itself.
import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var gbuffer: texture_2d<u32>;
@group(0) @binding(4) var lighting_id: texture_2d<u32>;
// Interpolated geometric normals (no normal maps), forward and deferred.
@group(0) @binding(5) var prepass_normal: texture_2d<f32>;

struct Shade {
    view_from_clip: mat4x4<f32>,
    fog_color: vec4<f32>,
    // x = exposure * key illuminance / pi, y = bands, z = exp² fog density.
    params: vec4<f32>,
}
@group(0) @binding(6) var<uniform> shade: Shade;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const UNLIT_FLAG: u32 = 1u; // DEFERRED_FLAGS_UNLIT_BIT
const INK: vec3<f32> = vec3<f32>(0.012, 0.014, 0.022);
const SHADOW_TINT: vec3<f32> = vec3<f32>(0.36, 0.50, 0.62); // azurite
const LIGHT_TINT: vec3<f32> = vec3<f32>(1.0, 0.95, 0.86); // warm paper

fn rgb9e5(v: u32) -> vec3<f32> {
    let exponent = i32(extractBits(v, 27u, 5u)) - 15 - 9;
    return vec3<f32>(
        f32(extractBits(v, 0u, 9u)),
        f32(extractBits(v, 9u, 9u)),
        f32(extractBits(v, 18u, 9u)),
    ) * exp2(f32(exponent));
}

fn albedo_at(g: vec4<u32>) -> vec3<f32> {
    return pow(unpack4x8unorm(g.r).rgb, vec3<f32>(2.2));
}

fn distance_at(p: vec2<i32>, d: f32) -> f32 {
    let size = vec2<f32>(textureDimensions(depth));
    let uv = (vec2<f32>(p) + 0.5) / size;
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    let view = shade.view_from_clip * ndc;
    return length(view.xyz / view.w);
}

fn fog_factor(dist: f32) -> f32 {
    let density = shade.params.z;
    if density <= 0.0 {
        return 0.0;
    }
    let x = dist * density;
    return (1.0 - 1.0 / exp(x * x)) * shade.fog_color.a;
}

/// Light received relative to the key light, from fog-free lit colour.
fn received(lit: vec3<f32>, albedo: vec3<f32>, f: f32) -> f32 {
    let surface = (lit - shade.fog_color.rgb * f) / max(1.0 - f, 1e-3);
    return dot(surface, LUMA) / (max(dot(albedo, LUMA), 1e-3) * shade.params.x);
}

fn band(light: f32) -> f32 {
    let steps = max(shade.params.y, 1.0);
    let raw = clamp(light, 0.0, 1.0) * steps;
    return (floor(raw) + smoothstep(0.42, 0.58, fract(raw))) / steps;
}

fn geometric_normal(p: vec2<i32>) -> vec3<f32> {
    return normalize(textureLoad(prepass_normal, p, 0).xyz * 2.0 - 1.0);
}

/// Ink where depth or geometric normals break. Wherever forward foliage is
/// involved only the depth silhouette counts, so grass is not scribbled.
fn ink(p: vec2<i32>, centre_depth: f32, dist: f32, deferred: bool) -> f32 {
    var depth_edge = 0.0;
    var normal_edge = 0.0;
    let n = geometric_normal(p);
    let offsets = array(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
    var foliage = !deferred;
    for (var i = 0; i < 4; i++) {
        let q = p + offsets[i];
        let d = textureLoad(depth, q, 0);
        let other = select(1e4, distance_at(q, d), d > 0.0);
        depth_edge = max(depth_edge, (other - dist) / dist);
        let neighbour_deferred = textureLoad(lighting_id, q, 0).r != 0u;
        foliage = foliage || !neighbour_deferred;
        if deferred && neighbour_deferred {
            normal_edge = max(normal_edge, 1.0 - dot(geometric_normal(q), n));
        }
    }
    // Only the nearer side of a depth break draws, keeping lines one pixel.
    var e = smoothstep(0.03, 0.09, depth_edge);
    if foliage {
        e = smoothstep(0.25, 0.6, depth_edge);
    } else {
        e = max(e, smoothstep(0.12, 0.35, normal_edge));
    }
    // Distant detail fades so the bamboo stand does not turn to scribble.
    return e * (1.0 - smoothstep(14.0, 42.0, dist));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(in.position.xy);
    let lit = textureLoad(scene, p, 0);
    let d = textureLoad(depth, p, 0);
    if d <= 0.0 {
        return lit; // sky / clear colour
    }
    let dist = distance_at(p, d);
    let f = fog_factor(dist);
    let deferred = textureLoad(lighting_id, p, 0).r != 0u;
    var color = lit.rgb;
    if deferred {
        let g = textureLoad(gbuffer, p, 0);
        let albedo = albedo_at(g);
        let unlit = ((g.a >> 24u) & UNLIT_FLAG) != 0u;
        let glow = dot(rgb9e5(g.g), LUMA);
        if !unlit && glow < 0.02 {
            // Smooth received light over nearby same-surface pixels before
            // banding: normal-map and occlusion detail otherwise shatters
            // figures into speckles. Albedo detail stays per pixel.
            var total = received(lit.rgb, albedo, f);
            var weight = 1.0;
            let taps = array(
                vec2<i32>(2, 0), vec2<i32>(-2, 0), vec2<i32>(0, 2), vec2<i32>(0, -2),
                vec2<i32>(2, 2), vec2<i32>(-2, 2), vec2<i32>(2, -2), vec2<i32>(-2, -2),
            );
            for (var i = 0; i < 8; i++) {
                let q = p + taps[i];
                let qd = textureLoad(depth, q, 0);
                if qd <= 0.0 || textureLoad(lighting_id, q, 0).r == 0u {
                    continue;
                }
                if abs(distance_at(q, qd) - dist) > dist * 0.02 {
                    continue;
                }
                let qg = textureLoad(gbuffer, q, 0);
                total += received(textureLoad(scene, q, 0).rgb, albedo_at(qg), f);
                weight += 1.0;
            }
            let level = band(total / weight);
            let tint = mix(SHADOW_TINT, LIGHT_TINT, level);
            let banded = albedo * tint * mix(0.55, 1.0, level) * shade.params.x;
            let surface = (lit.rgb - shade.fog_color.rgb * f) / max(1.0 - f, 1e-3);
            // Near-black albedo gives an unstable ratio; keep its lit colour.
            let confidence = smoothstep(0.015, 0.06, dot(albedo, LUMA));
            color = mix(mix(surface, banded, confidence), shade.fog_color.rgb, f);
        }
    }
    let line = ink(p, d, dist, deferred) * (1.0 - f);
    color = mix(color, INK * shade.params.x, line);
    return vec4<f32>(color, lit.a);
}
