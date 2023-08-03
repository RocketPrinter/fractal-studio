struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Constants {
    scale: vec2<f32>,
    offset: vec2<f32>,

    iterations: u32,
    r: f32, // escape radius (choose R > 0 such that R**2 - R >= sqrt(cx**2 + cy**2))
    c: vec2<f32>,
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
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = (v_positions[v_idx] + constants.offset) * constants.scale;
    return out;
}

// https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set
@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let iterations = calc_iterations(in.uv);
    return vec4<f32>(vec3<f32>(f32(iterations) / f32(constants.iterations)), 1.0);
}

// https://en.wikipedia.org/wiki/Julia_set#Pseudocode
fn calc_iterations(z: vec2<f32>) -> u32 {
    var iterations = 0u;
    var z = z;
    var z2 = vec2(z.x * z.x, z.y * z.y); // containts z.x^2 and z.y^2 not the square of the complex number
    let r_sq = constants.r * constants.r;
    while (z2.x + z2.y < r_sq  && iterations < constants.iterations) {
        z = vec2(z2.x - z2.y + constants.c.x, 2. * z.x * z.y + constants.c.y);
        z2 = vec2(z.x * z.x, z.y * z.y);
        iterations++;
    }
    return iterations;
}

/*
R = escape radius  # choose R > 0 such that R**2 - R >= sqrt(cx**2 + cy**2)

for each pixel (x, y) on the screen, do:
{
    zx = scaled x coordinate of pixel # (scale to be between -R and R)
       # zx represents the real part of z.
    zy = scaled y coordinate of pixel # (scale to be between -R and R)
       # zy represents the imaginary part of z.

    iteration = 0
    max_iteration = 1000

    while (zx * zx + zy * zy < R**2  AND  iteration < max_iteration)
    {
        xtemp = zx * zx - zy * zy
        zy = 2 * zx * zy  + cy
        zx = xtemp + cx

        iteration = iteration + 1
    }

    if (iteration == max_iteration)
        return black;
    else
        return iteration;
}
*/