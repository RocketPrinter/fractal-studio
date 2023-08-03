use bytemuck::bytes_of;
use eframe::egui::{CollapsingHeader, DragValue, Grid, Ui, Vec2, Widget};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use crate::app::settings::vec2_ui;
use crate::app::visualizer::FRAGMENT_PUSH_CONSTANTS_SIZE;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumMessage, Hash))]
pub enum Fractal {
    /// Test Grid
    TestGrid,
    /// Mandelbrot Set
    Mandelbrot { iterations: u32 },
    /// Julia Set
    Julia {
        iterations: u32,
        c: Vec2,
        eyedropping: bool,
        animating_on_circle: bool,
    },
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
            FractalDiscriminants::Julia => Fractal::Julia { iterations: 100, c: Vec2::new(0.25,-0.55), eyedropping: false, animating_on_circle: false },
        }
    }

    // todo: write a neat description for each
    pub fn settings_ui(&mut self, ui: &mut Ui) {
        match self {
            Fractal::TestGrid => (),
            Fractal::Mandelbrot { iterations } => {
                ui.horizontal(|ui| {
                    ui.label("Iterations");
                    DragValue::new(iterations).speed(1).clamp_range(1..=3000).ui(ui);
                });
            },
            Fractal::Julia { iterations, c, eyedropping, animating_on_circle } => {
                ui.horizontal(|ui|{
                    ui.label("Iterations");
                    DragValue::new(iterations).speed(1).clamp_range(0..=3000).ui(ui);
                });

                ui.horizontal(|ui|{
                    ui.label("C");
                    vec2_ui(ui, c, true, Some(0.02), None);
                });

                /* todo
                let eyedropper_button = ui.button("Pick with cursor").clicked();
                if *eyedropping {
                    *animating_unit_circle = None;
                    *eyedropping = !ui.input(|input| input.pointer.any_down());
                    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                    // the visualizer will call the method
                } else  {
                    *eyedropping = eyedropper_button;
                }
                */

                if *animating_on_circle {
                    *animating_on_circle = !ui.button("Stop").clicked();
                    *c = Vec2::angled(c.angle() + 0.01) * c.length();
                    ui.ctx().request_repaint();
                } else {
                    * animating_on_circle = ui.button("Animate on circle").clicked();
                }
            },
        }

        CollapsingHeader::new("Explanation").show(ui, |ui| {
            match self {
                Fractal::TestGrid => {
                    ui.label("This is a test grid.");
                }
                Fractal::Mandelbrot { .. } => {
                    ui.label("todo"); // todo
                }
                Fractal::Julia { .. } => {
                    ui.label("todo"); // todo
                }
            }
        });
    }

    /* todo
    /// called by the visualizer to determine if it should enable eyedropping mode
    pub fn enable_eyedrop(&mut self) -> bool {
        if let Fractal::Julia { eyedropping, .. } = self { *eyedropping } else { false }
    }

    pub fn eyedrop_result(&mut self, result: Vec2) {
        if let Fractal::Julia { c, .. } = self {
            *c = result
        }
    }
    */

    pub fn push_constants(&self) -> Option<[u8; FRAGMENT_PUSH_CONSTANTS_SIZE]> {
        match self {
            Fractal::TestGrid => None,
            Fractal::Mandelbrot { iterations } => {
                let mut buffer = [0; FRAGMENT_PUSH_CONSTANTS_SIZE];
                buffer[0..4].copy_from_slice(&iterations.to_ne_bytes());
                Some(buffer)
            },
            Fractal::Julia { iterations, c, ..} => {
                // solving the escape radius aka R
                // choose R > 0 such that R**2 - R >= sqrt(cx**2 + cy**2)
                let r = (1. + (1. + 4. * c.length()).sqrt()) / 2.;
                let mut buffer = [0; FRAGMENT_PUSH_CONSTANTS_SIZE];
                buffer[0..4].copy_from_slice(&iterations.to_ne_bytes());
                buffer[4..8].copy_from_slice(&r.to_ne_bytes());
                buffer[8..16].copy_from_slice(bytes_of(c));
                Some(buffer)
            },
        }
    }
}
