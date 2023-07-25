struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Constants {
    scale: vec2<f32>,
    offset: vec2<f32>,
}

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., 1.),
    vec2<f32>( 1.,-1.),
    vec2<f32>(-1.,-1.),
    vec2<f32>(-1., 1.),
    vec2<f32>( 1., 1.),
    vec2<f32>( 1.,-1.),
);

var<private> v_colors: array<vec4<f32>, 3> = array<vec4<f32>, 3>(
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0, 1.0, 0.0, 1.0),
    vec4<f32>(0.0, 0.0, 1.0, 1.0),
);

var<push_constant> constants: Constants;

@vertex
fn vertex(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + constants.offset) * constants.scale;
    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.uv, 0.0, 1.0);
}