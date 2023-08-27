struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Constants {
    scale: vec2<f32>,
    offset: vec2<f32>,

    iterations: u32,
    // 1..=16
    seg_len: u32,
    // array packed in an integer, 0 is A and 1 is B
    sequence: u32,

}

// brightness of color is calculated from lyapunov's exponent as follows: exp(ALPHA * abs(gamma))
const ALPHA = 1.;
const STABLE_COLOR   = vec3<f32>(1.0,0.76,0.0);
const UNSTABLE_COLOR = vec3<f32>(0.,0.,1.);

// the first constants.iterations / IGNORE_DIV iterations are ignored to avoid instabilities
// it doesn't seem to have a very noticiable inpact but some areas seem to look a little better?
const IGNORE_DIV = 10u;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., 1.),
    vec2<f32>( 1.,-1.),
    vec2<f32>(-1.,-1.),
    vec2<f32>(-1., 1.),
    vec2<f32>( 1., 1.),
    vec2<f32>( 1.,-1.),
);

@group(0) @binding(0)
var<uniform> constants: Constants;

@vertex
fn vertex(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + constants.offset) * constants.scale;
    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let gamma = compute_exponent(in.uv);
    return vec4<f32>(exponent_to_color(gamma), 1.);
}

fn exponent_to_color(gamma: f32) -> vec3<f32> {
    let color = mix(STABLE_COLOR, UNSTABLE_COLOR, f32(gamma > 0.0));
    return color * exp(-ALPHA * abs(gamma));
}

// https://en.wikipedia.org/wiki/Lyapunov_fractal
// https://www.youtube.com/watch?v=yGwy2WyQCQE
fn compute_exponent(ab: vec2<f32>) -> f32 {
    var xi = 0.5;
    var seq = constants.sequence;
    var exp = 0.;

    var i = 1u;
    let ignore_iter = constants.iterations / IGNORE_DIV;
    //ignore first iterations to avoid instability
    for (;i <= ignore_iter; i++) {
        let r = mix(ab.y, ab.x, f32(seq & 1u));
        xi = r * xi * (1. - xi);
        seq = cycle_seq(seq);
    }

    for (;i <= constants.iterations; i++) {
        let r = mix(ab.y, ab.x, f32(seq & 1u));
        xi = r * xi * (1. - xi);
        exp += log(r * abs(1. - 2. * xi));
        seq = cycle_seq(seq);
    }

   return exp / f32(constants.iterations - ignore_iter); // todo: maybe multiplying each exponent individually produces more accurate results?
}

// cycles the sequence clockwise, based on seq_len
// obviously seq needs to be initialised with constants.sequence
fn cycle_seq(seq: u32) -> u32 {
    return (seq >> 1u) | ((seq & 1u) << (constants.seg_len - 1u));
}

fn test_seq(x: f32) -> vec3<f32> {
    if x < 0. {
        return vec3<f32>(1.0, 0.0, 0.0);
    }
    let iter = i32(x);
    var seq = constants.sequence;
    for (var i = 1; i <= iter; i++) {
        seq = cycle_seq(seq);
    }
    return vec3(f32(seq & 1u));
}