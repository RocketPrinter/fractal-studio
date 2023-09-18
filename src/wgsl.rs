use std::hash::{Hash};
use eframe::wgpu::{ShaderModuleDescriptor, include_wgsl};
use lazy_static::lazy_static;
use fractal_visualizer_macros::include_wgsl_variants;
use crate::fractal::FractalDiscriminants;
use crate::fractal::FractalDiscriminants::*;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum ShaderCode {
    TestGrid,
    Mandelbrot,
    Julia,
    Newtons,
    Lyapunov(LyapunovVariant),
}

impl ShaderCode {
    pub fn get_shader(self) -> ShaderModuleDescriptor<'static> {
        match self {
            ShaderCode::TestGrid => include_wgsl!("wgsl/test_grid.wgsl"),
            ShaderCode::Mandelbrot => include_wgsl!("wgsl/mandelbrot.wgsl"),
            ShaderCode::Julia => include_wgsl!("wgsl/julia.wgsl"),
            ShaderCode::Newtons => include_wgsl!("wgsl/newtons.wgsl"),
            ShaderCode::Lyapunov(s) => s.get_shader(),
        }
    }
}

include_wgsl_variants! {
    pub variants LyapunovVariant from "src/wgsl/lyapunov.wgsl" {
        LogisticMap: {FUNC: u32 = 0},
        SinMap:      {FUNC: u32 = 1},
        GaussMap:    {FUNC: u32 = 2},
        Exponential: {FUNC: u32 = 3},
        CircleMap1:  {FUNC: u32 = 4},
        CircleMap2:  {FUNC: u32 = 5},
    }
}