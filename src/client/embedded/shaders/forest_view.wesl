// Shared coverage for the visible, deferred and depth/normal forest passes.
import bevy_pbr::render::mesh_view_bindings::view;

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> style: vec4<f32>;

fn hash21(point: vec2<f32>) -> f32 {
    let p = fract(point * vec2<f32>(123.34, 456.21));
    let q = p + dot(p, p + vec2<f32>(45.32));
    return fract(q.x * q.y);
}

fn camera_leaf_discard(world_position: vec3<f32>, pixel: vec2<f32>) {
    // Bevy's shadow views retain the primary camera as their LOD origin;
    // the visible view has matching world/LOD positions. Preserve the atlas
    // shadow silhouette while matching camera depth and colour coverage.
    let camera_view = distance(view.world_position, view.lod_view_world_position) < 0.001;
    if camera_view && style.x > 2.5 && style.x < 3.5 {
        let coverage = smoothstep(3.0, 5.5, distance(world_position, view.world_position));
        if coverage <= hash21(floor(pixel)) {
            discard;
        }
    }
}
