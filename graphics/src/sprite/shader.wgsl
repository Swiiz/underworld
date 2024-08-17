// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(2) model_mat_0: vec3<f32>,
    @location(3) model_mat_1: vec3<f32>,
    @location(4) model_mat_2: vec3<f32>,
    @location(5) tex_pos: vec2<f32>,
    @location(6) tex_dims: vec2<f32>,
    @location(7) tint: vec3<f32>,
    @location(8) z_index: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tint: vec3<f32>,
};


@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = mat3x3<f32>(
        instance.model_mat_0, instance.model_mat_1, instance.model_mat_2
    );
    out.tex_coords = instance.tex_pos + model.tex_coords * instance.tex_dims;
    let pos = model_matrix * vec3<f32>(model.position, 1.0);
    out.clip_position = vec4<f32>(pos.xy, instance.z_index / 100.0, 1.0);
    out.tint = instance.tint;
    
    return out;
}

// Fragment shader

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0)@binding(1)
var sam: sampler;

struct FragmentOutput {
    @location(0) rgba: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs_main(in: VertexOutput, ) -> FragmentOutput {
    var sample: vec4<f32> = textureSample(tex, sam, in.tex_coords);
    var rgb: vec3<f32> = sample.xyz * in.tint;
    var out: FragmentOutput;
    out.rgba = vec4<f32>(rgb, sample.a);
    out.depth = in.clip_position.z + (1.0 - sample.a);
    return out;
}