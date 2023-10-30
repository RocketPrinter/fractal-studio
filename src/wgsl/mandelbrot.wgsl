struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Props {
    scale: vec2<f32>,
    offset: vec2<f32>,

    c: vec2<f32>,
    max_iterations: u32,
    escape_radius: f32,
    exp: f32, // only used if MULTI == true
    // 0 - render Mandelbort/base fractal
    // 1 - render a bix of both
    // 2 - render Julia fractal
    julia: i32,
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
    #if VARIANT == 2
    // the burning ship is traditionally flipped on the y axis
    if props.julia == 0 {
        out.uv *= vec2(1.,-1.);
    }
    #endif
    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    // we could turn it into a variant but it's only run once per fragment so doubling the sources isn't worth it
    if props.julia == 0 {
        let iterations = compute_iterations(vec2<f32>(), in.uv, 2., props.max_iterations);
        return vec4(vec3(f32(iterations) / f32(props.max_iterations)), 1.0);

    } else if props.julia == 1 {
        let iterations_mandelbrot = compute_iterations(vec2<f32>(), in.uv, 2., props.max_iterations/2u);
        let iterations_julia = compute_iterations(in.uv, props.c, props.escape_radius, props.max_iterations/2u);
        return vec4(vec3(f32(iterations_mandelbrot + iterations_julia) / f32(props.max_iterations)), 1.0);

    } else {
        let iterations = compute_iterations(in.uv, props.c, props.escape_radius, props.max_iterations);
        return vec4(vec3(f32(iterations) / f32(props.max_iterations)), 1.0);
    }
}

// https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
fn compute_iterations(z0: vec2<f32>, c: vec2<f32>, escape_radius: f32, max_iterations: u32) -> u32 {
    var iterations = 0u;
    var z = z0;
    let r_sq = escape_radius * escape_radius;
    while z.x * z.x + z.y * z.y <= r_sq && iterations < max_iterations {
        z = equation(z,c);
        iterations++;
    }
    return iterations;
}

fn equation(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    #if VARIANT == 0
        // mandelbrot
        return raise_power(z) + c;
    #else if VARIANT == 1
        // modified mandelbrot set
        return raise_power(z) - z + c;
    #else if VARIANT == 2
        // burning ship
        return raise_power(abs(z)) + c;
    #endif
}

fn raise_power(z: vec2<f32>) -> vec2<f32> {
    #if MULTI == false
        return vec2(z.x * z.x - z.y * z.y, 2. * z.x * z.y);
    #else
        return cpowf(z, props.exp);
    #endif
}

// complex number to the power of a real number
fn cpowf(x: vec2<f32>, y: f32) -> vec2<f32> {
    var r = pow(length(x), y);
    var theta = atan2(x.y, x.x) * y;
    return vec2<f32>(r * cos(theta), r * sin(theta));
}
