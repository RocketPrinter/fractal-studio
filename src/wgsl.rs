use std::hash::{Hash};
use eframe::wgpu::{ShaderModuleDescriptor, include_wgsl};
use fractal_studio_macros::wgsl_variants;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Shader {
    TestGrid,
    Mandelbrot,
    Julia,
    Newtons,
    Lyapunov(LyapunovShader),
}

impl Shader {
    pub fn get_shader(self) -> ShaderModuleDescriptor<'static> {
        match self {
            Shader::TestGrid => include_wgsl!("wgsl/test_grid.wgsl"),
            Shader::Mandelbrot => include_wgsl!("wgsl/mandelbrot.wgsl"),
            Shader::Julia => include_wgsl!("wgsl/julia.wgsl"),
            Shader::Newtons => include_wgsl!("wgsl/newtons.wgsl"),
            Shader::Lyapunov(s) => s.get_shader(),
        }
    }
}

wgsl_variants! {
    pub variants LyapunovShader from "src/wgsl/lyapunov.wgsl" {
        LogisticMap {FUNC: u32 = 0},
        SinMap      {FUNC: u32 = 1},
        GaussMap    {FUNC: u32 = 2},
        Exponential {FUNC: u32 = 3},
        CircleMap1  {FUNC: u32 = 4},
        CircleMap2  {FUNC: u32 = 5},
    }
}
