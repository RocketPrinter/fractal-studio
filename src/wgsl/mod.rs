use eframe::wgpu::{include_wgsl, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource};
use lazy_static::lazy_static;
use crate::app::settings::KindDiscriminants;


type SMD = ShaderModuleDescriptor<'static>;

lazy_static! {
    pub static ref SHADERS: [(KindDiscriminants, SMD); 2] = [
        (KindDiscriminants::Test, include_wgsl!("test.wgsl").to_owned()),
        (KindDiscriminants::Mandelbrot, include_wgsl!("mandelbrot.wgsl").to_owned()),
    ];
}