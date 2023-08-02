use eframe::wgpu::{ShaderModuleDescriptor, include_wgsl};
use lazy_static::lazy_static;
use crate::fractal::FractalDiscriminants;
use crate::fractal::FractalDiscriminants::*;

pub type Smd = ShaderModuleDescriptor<'static>;

lazy_static!{
    pub static ref SHADERS: [(FractalDiscriminants, Smd);2] = [
        (TestGrid,include_wgsl!("test_grid.wgsl")),
        (Mandelbrot,include_wgsl!("mandelbrot.wgsl")),
    ];
}