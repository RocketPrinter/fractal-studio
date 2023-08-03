struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Constants {
    scale: vec2<f32>,
    offset: vec2<f32>,

    iterations: u32,
}

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., 1.),
    vec2<f32>( 1.,-1.),
    vec2<f32>(-1.,-1.),
    vec2<f32>(-1., 1.),
    vec2<f32>( 1., 1.),
    vec2<f32>( 1.,-1.),
);

var<push_constant> constants: Constants;

@vertex
fn vertex(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + constants.offset) * constants.scale;
    return out;
}

// https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let iterations = calc_iterations(in.uv);
    return vec4(vec3(f32(iterations - 1u) / f32(constants.iterations - 1u)), 1.0);
}


fn calc_iterations(p0: vec2<f32>) -> u32 {
    var iterations = 0u;
    var p = vec2<f32>();
    var p2 = vec2<f32>();
    var w = 0.;
    while p2.x + p2.y <= 4. && iterations < constants.iterations {
        p.x = p2.x - p2.y + p0.x;
        p.y = w - p2.x - p2.y + p0.y;
        p2 = p * p;
        w = (p.x + p.y) * (p.x + p.y);
        iterations += 1u;
    }
    return iterations;
}