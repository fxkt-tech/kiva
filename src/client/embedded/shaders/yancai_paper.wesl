// Yancai pass 2 (display values, after tonemapping, before FXAA): pull hues
// toward a mineral palette, pool pigment where colour fields meet, and lay
// the image on fibrous paper. Paper is procedural, in screen space, because
// it is the canvas rather than a surface in the world.
import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct Paper {
    palette: f32,
    paper: f32,
    pooling: f32,
    _pad: f32,
}
@group(0) @binding(2) var<uniform> settings: Paper;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const PAPER_TONE: vec3<f32> = vec3<f32>(0.93, 0.89, 0.80);

// 石青 azurite, 石绿 malachite, 朱砂 cinnabar, 赭石 ochre, 金 gold.
// Ink and paper white are value extremes and need no hue.
const PIGMENTS: array<vec3<f32>, 5> = array<vec3<f32>, 5>(
    vec3<f32>(0.06, 0.15, 0.30),
    vec3<f32>(0.10, 0.30, 0.17),
    vec3<f32>(0.55, 0.07, 0.04),
    vec3<f32>(0.40, 0.23, 0.10),
    vec3<f32>(0.62, 0.45, 0.14),
);

fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(123.34, 456.21));
    let r = q + dot(q, q + vec2<f32>(45.32));
    return fract(r.x * r.y);
}

fn noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(cell), hash(cell + vec2<f32>(1.0, 0.0)), w.x),
               mix(hash(cell + vec2<f32>(0.0, 1.0)), hash(cell + vec2<f32>(1.0)), w.x), w.y);
}

/// Keep value and chroma strength, turn the hue toward the closest pigment.
fn palette(color: vec3<f32>) -> vec3<f32> {
    let value = dot(color, LUMA);
    let chroma = color - value;
    let strength = length(chroma);
    if strength < 1e-4 {
        return color;
    }
    var best = chroma / strength;
    var score = -2.0;
    for (var i = 0; i < 5; i++) {
        let pc = PIGMENTS[i] - dot(PIGMENTS[i], LUMA);
        let direction = pc / length(pc);
        let s = dot(direction, chroma / strength);
        if s > score {
            score = s;
            best = direction;
        }
    }
    return mix(color, value + best * strength, settings.palette);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(in.position.xy);
    let source = textureLoad(screen, p, 0);
    var color = palette(source.rgb);

    // Pigment pools at the edges of colour fields.
    let l = dot(source.rgb, LUMA);
    var gradient = 0.0;
    let offsets = array(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
    for (var i = 0; i < 4; i++) {
        gradient = max(gradient, abs(dot(textureLoad(screen, p + offsets[i], 0).rgb, LUMA) - l));
    }
    color *= 1.0 - settings.pooling * smoothstep(0.04, 0.2, gradient);

    // Fibres (stretched fine noise) over soft blotches of absorbency.
    let xy = in.position.xy;
    let fibre = noise(vec2<f32>(xy.x * 0.9, xy.y * 0.12)) * 0.6 + noise(xy * 0.45) * 0.4;
    let blotch = noise(xy * 0.012) * 0.7 + noise(xy * 0.04) * 0.3;
    let grain = (fibre - 0.5) * 0.6 + (blotch - 0.5);
    color *= 1.0 + grain * settings.paper;
    // Highlights take on the paper tone instead of clipping to white.
    color = mix(color, color * PAPER_TONE, smoothstep(0.55, 1.0, l) * 0.6);
    return vec4<f32>(color, source.a);
}
