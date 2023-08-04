use bytemuck::bytes_of;
use eframe::egui::{CollapsingHeader, CursorIcon, DragValue, Ui, Vec2, Widget};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use crate::app::settings::vec2_ui;

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
        pick_using_cursor: bool,
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
            FractalDiscriminants::Julia => Fractal::Julia { iterations: 100, c: Vec2::new(-0.76,-0.15), pick_using_cursor: false, animating_on_circle: false },
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
            Fractal::Julia { iterations, c, pick_using_cursor, animating_on_circle } => {
                ui.horizontal(|ui|{
                    ui.label("Iterations");
                    DragValue::new(iterations).speed(1).clamp_range(0..=3000).ui(ui);
                });

                ui.horizontal(|ui|{
                    ui.label("C");
                    vec2_ui(ui, c, true, Some(0.02), None);
                });

                let pick_using_cursor_button = ui.button("Pick with cursor").clicked();
                if *pick_using_cursor {
                    *animating_on_circle = false;
                    *pick_using_cursor = !ui.input(|input| input.pointer.any_down());
                    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                    // the visualizer will call the method
                } else  {
                    *pick_using_cursor = pick_using_cursor_button;
                }

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

    /// cursor's position in the coordinate system of the fragment shader
    /// will only be executed if cursor is hovering over the visualizer
    pub fn cursor_shader_space(&mut self, pos: Vec2) {
        match self {
            Fractal::Julia { c, pick_using_cursor, .. } if *pick_using_cursor => {
                *c = pos;
            },
            _ => (),
        }
    }

    pub fn uniform_buffer_data(&self) -> Option<Vec<u8>> {
        match self {
            Fractal::TestGrid => None,
            Fractal::Mandelbrot { iterations } => Some(iterations.to_ne_bytes().into()),
            Fractal::Julia { iterations, c, ..} => {
                // solving the escape radius aka R
                // choose R > 0 such that R**2 - R >= sqrt(cx**2 + cy**2)
                let r = (1. + (1. + 4. * c.length()).sqrt()) / 2.;
                let mut buffer = vec![0; 16];
                buffer[0..4].copy_from_slice(&iterations.to_ne_bytes());
                buffer[4..8].copy_from_slice(&r.to_ne_bytes());
                buffer[8..16].copy_from_slice(bytes_of(c));
                Some(buffer)
            },
        }
    }
}

