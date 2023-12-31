struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Props {
    scale: vec2<f32>, //0..8
    offset: vec2<f32>,//8..16

    arr: array<Element, 6>,//16..112
    a: vec2<f32>,//112..120
    c: vec2<f32>,//120..128
    nr_roots: u32,//128..132
    max_iterations: u32,//132..136
    threshold: f32,//136..140
    _padding: f32,//140..144
}
// array elements must have a size of 16 so we interweave the roots and polynomial constant arraysa
struct Element {
    root: vec2<f32>,
    coefficient: vec2<f32>,
}

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., 1.),
    vec2<f32>( 1.,-1.),
    vec2<f32>(-1.,-1.),
    vec2<f32>(-1., 1.),
    vec2<f32>( 1., 1.),
    vec2<f32>( 1.,-1.),
);

// https://coolors.co/palette/f79256-fbd1a2-7dcfb6-00b2ca-1d4e89
var<private> root_colors: array<vec4<f32>, 5> = array<vec4<f32>, 5>(
    vec4<f32>(0.96,0.57,0.33,1.),
    vec4<f32>(0.98,0.81,0.63,1.),
    vec4<f32>(0.48,0.80,0.71,1.),
    vec4<f32>(0.00,0.69,0.78,1.),
    vec4<f32>(0.11,0.30,0.53,1.),
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

// https://youtu.be/-RdOwhmqP5s
@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let root = newtons_method(in.uv);
    if (root == -1) {
        discard;
    }
    return root_colors[root];
}

// https://en.wikipedia.org/wiki/Newton_fractal#Implementation
// will return -1 if it's not close enough to any of the roots and the root index otherwise
fn newtons_method(z: vec2<f32>) -> i32 {
    var z = z;
    for(var iteration = 0u; iteration < props.max_iterations; iteration++) {
        var zp = vec2(1.,0.);
        var prev = vec2<f32>();
        var f = vec2<f32>();
        var fd = vec2<f32>();
        for (var i=0;i<=5;i++) {
            let coef = props.arr[i].coefficient;
            f += cmul(coef, zp);
            fd += cmul(coef, prev) * f32(i);
            prev = zp;
            zp = cmul(cmul(props.a,zp), z) + props.c;
        }
        z = z - cdiv(f,fd);
    }
    var closest_root = -1; var closest_dist = props.threshold;
    for (var i=0u;i<props.nr_roots;i++) {
        let d = distance(z,props.arr[i].root);
        if (d < closest_dist) {
            closest_root = i32(i);
            closest_dist = d;
        }
    }
    return closest_root;
}

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn csq(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2. * z.x * z.y);
}

// maybe use https://arxiv.org/pdf/1608.07596.pdf
// from num_complex crate
fn cdiv(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let norm_sqr = b.x * b.x + b.y * b.y;
    let re = a.x * b.x + a.y * b.y;
    let im = a.y * b.x - a.x * b.y;
    return vec2(re / norm_sqr, im / norm_sqr);
}