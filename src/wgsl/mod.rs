use eframe::wgpu::{Device, include_wgsl, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VERTEX_SHADER: ShaderModuleDescriptor<'static> = include_wgsl!("vertex.wgsl");

    pub static ref TEST_SHADER: ShaderModuleDescriptor<'static> = include_wgsl!("test.wgsl");
    pub static ref MANDELBROT_SHADER: ShaderModuleDescriptor<'static> = include_wgsl!("mandelbrot.wgsl");
}