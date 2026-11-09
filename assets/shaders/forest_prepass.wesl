// Keep Bevy's standard alpha sampling, normal maps and motion vectors, adding
// only the same leaf coverage used by the forward/deferred forest material.
import bevy_pbr::{
    prepass::io::VertexOutput,
    render::pbr_prepass_functions::prepass_sample_color_and_alpha_discard,
};
import kiva::forest_view::camera_leaf_discard;

@if(PREPASS_FRAGMENT)
import bevy_pbr::prepass::io::FragmentOutput;

@if(MOTION_VECTOR_PREPASS)
import bevy_pbr::render::pbr_prepass_functions::calculate_motion_vector;

@if(NORMAL_PREPASS)
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;

@if(PREPASS_FRAGMENT) {
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var out: FragmentOutput;
@if(NORMAL_PREPASS) {
    let surface = pbr_input_from_standard_material(in, is_front);
    out.normal = vec4<f32>(surface.N * 0.5 + vec3<f32>(0.5), 1.0);
}
    prepass_sample_color_and_alpha_discard(in);
    camera_leaf_discard(in.world_position.xyz, in.position.xy);
@if(MOTION_VECTOR_PREPASS)
    out.motion_vector = calculate_motion_vector(in.world_position, in.previous_world_position);
@if(UNCLIPPED_DEPTH_ORTHO_EMULATION)
    out.frag_depth = in.unclipped_depth;
    return out;
}
}
@else {
@fragment
fn fragment(in: VertexOutput) {
    prepass_sample_color_and_alpha_discard(in);
    camera_leaf_discard(in.world_position.xyz, in.position.xy);
}
}
