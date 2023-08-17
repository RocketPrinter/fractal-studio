use eframe::egui::{CursorIcon, DragValue, Painter, Slider, Ui, Vec2, Widget};
use encase::{ShaderType, UniformBuffer};
use nalgebra::{ComplexField, UnitComplex};
use crate::app::widgets::{c32_ui_full, option_checkbox};
use crate::fractal::FractalTrait;
use crate::math::{C32, UC32, vec2_to_c32};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Julia {
    iterations: u32,
    c: C32,
    // if the exponent is != 2 then the fractal becomes a multijulia
    e: Option<f32>,
    // todo: maybe draw mandelbrot with a lowered opacity when picking using cursor to show connection
    pick_using_cursor: bool,
    animating_on_circle: bool,
}

#[derive(ShaderType)]
struct JuliaUniform {
    c: UC32,
    e: f32,
    iterations: u32,
    escape_radius: f32,
}

impl Default for Julia {
    fn default() -> Self {
        Self {
            iterations: 100,
            c: C32::new(-0.76,-0.15),
            e: None,
            pick_using_cursor: false,
            animating_on_circle: false
        }
    }
}

impl FractalTrait for Julia {
    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui|{
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).clamp_range(0..=3000).ui(ui);
        });

        if self.pick_using_cursor {
            ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            self.animating_on_circle = false;
            if ui.input(|input| input.pointer.any_down()) {
                self.pick_using_cursor = false;
            }
        }

        ui.horizontal(|ui|{
            if c32_ui_full(ui, "c", &mut self.c, Some(0.02), None).clicked() {
                self.pick_using_cursor = true;
            }
        });

        if self.animating_on_circle {
            self.animating_on_circle = !ui.button("Stop").clicked();
            self.c = UnitComplex::new(self.c.argument() + 0.01).into_inner() * self.c.abs();
            ui.ctx().request_repaint();
        } else {
            self.animating_on_circle = ui.button("Animate on circle").clicked();
        }

        option_checkbox(ui, &mut self.e, "Multijulia",  || 2.);
        if let Some(e) = &mut self.e {
            ui.horizontal(|ui| {
                ui.label("e");
                Slider::new(e, 2.0..=6.).drag_value_speed(0.02).clamp_to_range(false).ui(ui);
            });
            if *e < 2. {
                ui.colored_label(ui.style().visuals.warn_fg_color, "May not produce accurate results, still a WIP"); // todo
            }
        }
    }

    fn explanation(&mut self, _ui: &mut Ui) {
        //todo
    }

    fn fill_uniform_buffer(&self, mut buffer: UniformBuffer<&mut [u8]>) {
        let e = self.e.unwrap_or(2.);

        // solving the escapa radius
        // we solve R > 0 such that R^2 - R == |c|
        // todo: escape radius is too small for e < 2 and (maybe) too big for e > 2
        let escape_radius = (1. + (1. + 4. * self.c.abs()).sqrt()) / 2.;

        buffer.write(&JuliaUniform {
            c: self.c.into(),
            e,
            iterations: self.iterations,
            escape_radius,
        }).unwrap();
    }

    fn draw_extra(&mut self, _painter: &Painter, mouse_pos: Option<Vec2>) {
        if let (Some(mouse_pos), true) = (mouse_pos, self.pick_using_cursor) {
            self.c = vec2_to_c32(&mouse_pos);
        }
        //todo: draw c
    }
}