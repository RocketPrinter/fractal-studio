use eframe::egui::{DragValue, Slider, Ui, Widget};
use encase::{ShaderType, UniformBuffer};
use crate::app::widgets::option_checkbox;
use crate::fractal::FractalTrait;
use crate::wgsl::ShaderCode;

// todo: make Julia sets be a subtype of Mandelbrot
// also add Burning Ship
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Mandelbrot {
    iterations: u32,
    // z_n+1 = z ^ e + c;
    // if the exponent is != 2 then the fractal becomes a multibrot
    e: Option<f32>,
}

#[derive(ShaderType)]
struct MandelbrotUniform {
    iterations: u32,
    exponent: f32,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            iterations: 300,
            e: None,
        }
    }
}

impl FractalTrait for Mandelbrot {
    fn override_label(&mut self) -> Option<&'static str> {
        if self.e.is_some() {
            Some("Multibrot set")
        } else {
            None
        }
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).clamp_range(1..=3000).ui(ui);
        });

        option_checkbox(ui, &mut self.e, "Multibrot",  || 2.);
        if let Some(e) = &mut self.e {
            ui.horizontal(|ui| {
                ui.label("e");
                Slider::new(e, 1.0..=6.).drag_value_speed(0.02).clamp_to_range(false).ui(ui);
            });
        }
    }

    fn explanation_ui(&mut self, _ui: &mut Ui) {
        // todo
    }

    fn get_shader_code(&self) -> ShaderCode { ShaderCode::Mandelbrot }

    fn fill_uniform_buffer(&self, mut buffer: UniformBuffer<&mut [u8]>) {
        buffer.write(&MandelbrotUniform {
            iterations: self.iterations,
            exponent: self.e.unwrap_or(2.),
        }).unwrap();
    }
}