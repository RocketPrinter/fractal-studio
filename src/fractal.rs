use eframe::egui::{DragValue, Ui, Widget};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use crate::app::visualizer::FRAGMENT_PUSH_CONSTANTS_SIZE;
use crate::helpers::extend_array;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumMessage, Hash))]
pub enum Fractal {
    #[strum_discriminants(strum(message = "Test Grid"))]
    TestGrid,
    #[strum_discriminants(strum(message = "Mandelbrot Set"))]
    Mandelbrot { iterations: u32 },
}

impl Default for Fractal {
    fn default() -> Self {
        Self::default(FractalDiscriminants::Mandelbrot)
    }
}

impl Fractal {
    pub fn default(discriminant: FractalDiscriminants) -> Self {
        match discriminant {
            FractalDiscriminants::TestGrid => Fractal::TestGrid,
            FractalDiscriminants::Mandelbrot => Fractal::Mandelbrot { iterations: 300 },
        }
    }

    pub fn settings_ui(&mut self, ui: &mut Ui) {
        match self {
            Fractal::TestGrid => (),
            Fractal::Mandelbrot { iterations } => {
                ui.label("Iterations");
                DragValue::new(iterations).speed(1).clamp_range(0..=3000).ui(ui);
                ui.end_row();
            },
        }
    }

    pub fn push_constants(&self) -> Option<[u8; FRAGMENT_PUSH_CONSTANTS_SIZE]> {
        match self {
            Fractal::TestGrid => None,
            Fractal::Mandelbrot { iterations } => Some(extend_array(&iterations.to_ne_bytes(),0)),
        }
    }
}


