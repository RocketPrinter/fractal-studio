use eframe::egui::{DragValue, Slider, Ui, Widget};
use encase::{ShaderType, UniformBuffer};
use crate::app::widgets::option_checkbox;
use crate::fractal::FractalTrait;

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
    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.label("Iterations");
        DragValue::new(&mut self.iterations).speed(1).clamp_range(1..=3000).ui(ui);

        option_checkbox(ui, &mut self.e, "Multibrot",  || 2.);
        if let Some(e) = &mut self.e {
            ui.horizontal(|ui| {
                ui.label("e");
                Slider::new(e, 1.0..=6.).drag_value_speed(0.02).clamp_to_range(false).ui(ui);
            });
        }
    }

    fn explanation(&mut self, _ui: &mut Ui) {
        // todo
    }

    fn fill_uniform_buffer(&self, mut buffer: UniformBuffer<&mut [u8]>) {
        buffer.write(&MandelbrotUniform {
            iterations: self.iterations,
            exponent: self.e.unwrap_or(2.),
        }).unwrap();
    }
}