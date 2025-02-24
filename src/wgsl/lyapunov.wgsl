struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Props {
    scale: vec2<f32>,
    offset: vec2<f32>,

    stable_col: vec4<f32>,
    unstable_col: vec4<f32>,
    iterations: u32,
    // 1..=16
    seg_len: u32,
    // array packed in an integer, 0 is A and 1 is B
    sequence: u32,
    // extra parameter for extra fun
    c: f32,
}

const PI: f32 = 3.14159265359;

// brightness of color is calculated from lyapunov's exponent as follows: exp(ALPHA * abs(gamma))
const ALPHA = 1.;

// the first props.iterations / IGNORE_DIV iterations are ignored to avoid instabilities
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
var<uniform> props: Props;

@vertex
fn vertex(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + props.offset) * props.scale;
    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let gamma = compute_exponent(in.uv);
    return exponent_to_color(gamma);
}

fn exponent_to_color(gamma: f32) -> vec4<f32> {
    let color = mix(props.stable_col, props.unstable_col, f32(gamma > 0.0));
    return color * exp(-ALPHA * abs(gamma));
}

// https://en.wikipedia.org/wiki/Lyapunov_fractal
// https://www.youtube.com/watch?v=yGwy2WyQCQE
fn compute_exponent(ab: vec2<f32>) -> f32 {
    var xi = 0.5;
    var seq = props.sequence;
    var exp = 0.;

    var i = 1u;
    let ignore_iter = props.iterations / IGNORE_DIV;
    //ignore first iterations to avoid instability
    for (;i <= ignore_iter; i++) {
        let r = mix(ab.y, ab.x, f32(seq & 1u));
        xi = func(xi, r);
        seq = cycle_seq(seq);
    }

    for (;i <= props.iterations; i++) {
        let r = mix(ab.y, ab.x, f32(seq & 1u));
        xi = func(xi, r);
        exp += exponent(xi, r);
        seq = cycle_seq(seq);
    }

   return exp / f32(props.iterations - ignore_iter); // todo: maybe multiplying each exponent individually produces more accurate results?
}

// f(x)
fn func(x: f32, r: f32) -> f32 {
    #if FUNC == 0
        // logistical map
        return r * x * (1. - x);
    #else if FUNC == 1
        // sin map
        return r * sin(x * PI);
    #else if FUNC == 2
        // gauss map
        return exp(-5. * x * x)+r;
    #else if FUNC == 3
        // exponential function
        return pow(abs(r), sin(x));
    #else if FUNC == 4
        // circle map variation 1, omega is constant and k is r
        return fract(x + 1./3. - r * sin (2. * PI * x) / (2. * PI) );
    #else if FUNC == 5
        // circle map variation 2, omega is r and k is constant
        return fract(x + r - 2. * sin (2. * PI * x) / (2. * PI) );
    #else if FUNC == 6
        // sin
        return sin(r * x);
    #endif
}

// log(abs(f`(x)))
fn exponent(x: f32, r: f32) -> f32 {
    #if FUNC == 0
        return log(abs(r - 2. * r * x));
    #else if FUNC == 1
        return log(abs(r * cos(x * PI) * PI));
    #else if FUNC == 2
        return log(abs(exp(-5. * x * x)*-10.*x));
    #else if FUNC == 3
        let r = abs(r);
        return log(abs(pow(r, sin(x))*log(r)*cos(x)));
    #else if FUNC == 4
        return log(abs(r * cos(2. * PI * x) + 1.));
    #else if FUNC == 5
        return log(abs(2. * cos(2. * PI * x) + 1.));
    #else if FUNC == 6
        return log(abs(r * cos (r * x)));
    #endif
}

// cycles the sequence clockwise, based on seq_len
// seq needs to be initialised with props.sequence
fn cycle_seq(seq: u32) -> u32 {
    return (seq >> 1u) | ((seq & 1u) << (props.seg_len - 1u));
}

/*fn test_seq(x: f32) -> vec3<f32> {
    if x < 0. {
        return vec3<f32>(1.0, 0.0, 0.0);
    }
    let iter = i32(x);
    var seq = props.sequence;
    for (var i = 1; i <= iter; i++) {
        seq = cycle_seq(seq);
    }
    return vec3(f32(seq & 1u));
}*/
