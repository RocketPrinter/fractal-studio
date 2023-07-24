use eframe::wgpu::{include_wgsl, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource};
use lazy_static::lazy_static;
use crate::app::settings::Kind;

type SMD = ShaderModuleDescriptor<'static>;

lazy_static! {
    pub static ref SHADERS: [(Kind, SMD);1] = [
    (Kind::Test, include_wgsl!("test.wgsl").to_owned()),
];
}