struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Constants {
    scale: vec2<f32>,
    offset: vec2<f32>,

    c: vec2<f32>,
    e: f32, // if the exponent is != 2 then the fractal becomes a multijulia
    max_iterations: u32,
    escape_radius: f32, // escape radius (choose R > 0 such that R**2 - R >= sqrt(cx**2 + cy**2))
    _padding: vec3<f32>,
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
var<uniform> constants: Constants;

@vertex
fn vertex(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + constants.offset) * constants.scale;
    return out;
}

// https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let iterations = compute_iterations(in.uv);
    return vec4(vec3(f32(iterations) / f32(constants.max_iterations)), 1.0);
}

fn compute_iterations(c: vec2<f32>) -> u32 {
    if (constants.e == 2.) {
        return compute_iterations_simple(c);
    } else {
        return compute_iterations_generalized(c);
    }
}

// exponent is 2, classic julia case
// https://en.wikipedia.org/wiki/Julia_set#Pseudocode
fn compute_iterations_simple(z: vec2<f32>) -> u32 {
    var iterations = 0u;
    var z = z;
    var z2 = vec2(z.x * z.x, z.y * z.y); // contains z.x^2 and z.y^2 not the square of the complex number
    let r_sq = constants.escape_radius * constants.escape_radius;
    while (z2.x + z2.y < r_sq  && iterations < constants.max_iterations) {
        z = vec2(z2.x - z2.y + constants.c.x, 2. * z.x * z.y + constants.c.y);
        z2 = vec2(z.x * z.x, z.y * z.y);
        iterations++;
    }
    return iterations;
}

// for cases where the exponent is != 2
fn compute_iterations_generalized(z: vec2<f32>) -> u32 {
    var iterations = 0u;
    var z = z;
    let r_sq = constants.escape_radius * constants.escape_radius;
    while (z.x * z.x + z.y * z.y < r_sq  && iterations < constants.max_iterations) {
        z = cpowf(z, constants.e) + constants.c;
        iterations++;
    }
    return iterations;
}

// complex number to the power of a real number
fn cpowf(x: vec2<f32>, y: f32) -> vec2<f32> {
    var r = pow(length(x), y);
    var theta = atan2(x.y, x.x) * y;
    return vec2<f32>(r * cos(theta), r * sin(theta));
}