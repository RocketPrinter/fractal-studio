struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Props {
    scale: vec2<f32>,
    offset: vec2<f32>,

    max_iterations: u32,
    e: f32, // if the exponent is != 2 then the fractal becomes a multibrot
    _padding: vec2<f32>,
}

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
    out.position = vec4(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + props.offset) * props.scale;
    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    var iterations = compute_iterations(in.uv);
    return vec4(vec3(f32(iterations - 1u) / f32(props.max_iterations - 1u)), 1.0);
}

fn compute_iterations(c: vec2<f32>) -> u32 {
    if (props.e == 2.) {
        return compute_iterations_simple(c);
    } else {
        return compute_iterations_generalized(c);
    }
}

// exponent is two, classic mandelbrot case
// https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
fn compute_iterations_simple(c: vec2<f32>) -> u32 {
    var iterations = 0u;
    var z = vec2<f32>();
    var z2 = vec2<f32>(); // contains z.x^2 and z.y^2 not the square of the complex number
    var w = 0.;
    while z2.x + z2.y <= 4. && iterations < props.max_iterations {
        z.x = z2.x - z2.y + c.x;
        z.y = w - z2.x - z2.y + c.y;
        z2 = z * z;
        w = (z.x + z.y) * (z.x + z.y);
        iterations += 1u;
    }
    return iterations;
}

// for cases where the exponent is != 2
// todo for cases where the exponent is negative it doesn't work, possibly because the function isn't polynomial anymore or because the escape radius is nonsensical
// see https://math.stackexchange.com/questions/1257555/how-to-compute-a-negative-multibrot-set
fn compute_iterations_generalized(c: vec2<f32>) -> u32 {
    var iterations = 0u;
    var z = vec2<f32>();
    var w = 0.;
    while z.x * z.x + z.y * z.y <= 4. && iterations < props.max_iterations {
        z = cpowf(z, props.e) + c;
        iterations += 1u;
    }
    return iterations;
}

// complex number to the power of a real number
fn cpowf(x: vec2<f32>, y: f32) -> vec2<f32> {
    var r = pow(length(x), y);
    var theta = atan2(x.y, x.x) * y;
    return vec2<f32>(r * cos(theta), r * sin(theta));
}