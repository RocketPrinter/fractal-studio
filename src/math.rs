use bytemuck::{Pod, Zeroable};
use eframe::egui::Vec2;
use encase::impl_vector;
use encase::vector::{AsMutVectorParts, AsRefVectorParts, FromVectorParts};
use nalgebra::Complex;

pub type C32 = Complex<f32>;

pub fn vec2_to_c32(v: &Vec2) -> C32 {
    C32::new(v.x, v.y)
}

// newtype around Complex<f32> so we can implement encase traits on it
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct UC32(pub Complex<f32>);

impl From<C32> for UC32 {
    fn from(c: C32) -> Self {
        Self(c)
    }
}

unsafe impl Zeroable for UC32 { }

unsafe impl Pod for UC32 { }

impl AsRefVectorParts<f32, 2> for UC32 {
    fn as_ref_parts(&self) -> &[f32; 2] {
        bytemuck::cast_ref(self)
    }
}

impl AsMutVectorParts<f32, 2> for UC32 {
    fn as_mut_parts(&mut self) -> &mut [f32; 2] {
        bytemuck::cast_mut(self)
    }
}

impl FromVectorParts<f32, 2> for UC32 {
    fn from_parts(parts: [f32; 2]) -> Self {
        UC32(Complex::new(parts[0], parts[1]))
    }
}

impl_vector!(2, UC32, f32);
