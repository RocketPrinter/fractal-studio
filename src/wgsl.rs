use std::hash::Hash;
use eframe::{egui::Vec2, wgpu::{include_wgsl, ShaderModuleDescriptor}};
use fractal_studio_macros::wgsl_variants;
use crate::wgsl::mandelbrot::MandelbrotShader;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Shader {
    TestGrid,
    Mandelbrot(MandelbrotShader),
    Newtons,
    Lyapunov(LyapunovShader),
}

impl Shader {
    pub fn get_shader(self) -> ShaderModuleDescriptor<'static> {
        match self {
            Shader::TestGrid => include_wgsl!("wgsl/test_grid.wgsl"),
            Shader::Mandelbrot(s) => MandelbrotShader::get_shader(s),
            Shader::Newtons => include_wgsl!("wgsl/newtons.wgsl"),
            Shader::Lyapunov(s) => s.get_shader(),
        }
    }
}

pub mod mandelbrot {
    use fractal_studio_macros::wgsl_variants;
    #[allow(unused_imports)]
    use eframe::wgpu::ShaderModuleDescriptor;

    wgsl_variants! {
        pub value_enum VARIANT as Variant: u32 {
            Mandelbrot = 0,
            Modified = 1,
            BurningShip= 2,
        }

        pub value_enum MULTI as Multi: bool { Disabled = false, Enabled = true }

        pub variants MandelbrotShader from "src/wgsl/mandelbrot.wgsl" {
            Product(Variant, Multi),
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

use num_complex::Complex32;

pub trait Vec2Ext {
    fn to_c32(&self) -> Complex32;
}

impl Vec2Ext for Vec2 {
    fn to_c32(&self) -> Complex32 {
        Complex32::new(self.x, self.y)
    }
}

pub trait Complex32Ext {
    fn to_gvec2(&self) -> glam::Vec2;
}

impl Complex32Ext for Complex32 {
    fn to_gvec2(&self) -> glam::Vec2 {
        glam::Vec2::new(self.re, self.im)
    }
}
