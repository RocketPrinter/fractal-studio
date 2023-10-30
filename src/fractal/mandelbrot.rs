use std::ops::Not;
use eframe::egui::{Button, ComboBox, CursorIcon, DragValue, Painter, Slider, Ui, Vec2, Widget, WidgetText};
use encase::{ShaderType, UniformBuffer};
use nalgebra::ComplexField;
use crate::app::widgets::{c32_ui_full, option_checkbox};
use crate::fractal::FractalTrait;
use crate::math::{C32, UC32, vec2_to_c32};
use crate::wgsl::mandelbrot::*;
use crate::wgsl::Shader;

/// Handles  all the mandelbrot-type fractals including Julia variations and Burning ship
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MandelbrotFamily {
    iterations: u32,
    variant: Variant,
    // Some if the fractal is a julia set with the constant c
    julia_c: Option<C32>,
    #[serde(skip, default = "pick_c_default")]
    pick_c_using_cursor: (bool, PickCMode),
    // Some if the fractal is multi-
    // if None e will be 2
    // z = z^e + c
    multi_e: Option<f32>,
}

fn pick_c_default() -> (bool, PickCMode) {(false, PickCMode::Both)}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u32)]
pub enum PickCMode {
    // render a mix of mandelbrot and julia
    Both = 1,
    // render julia
    Julia = 2
}

// check shader
#[derive(ShaderType)]
struct MandelbrotUniform {
    c: UC32,
    iterations: u32,
    escape_radius: f32,
    exp: f32,
    julia: u32,
}

impl MandelbrotFamily {
    pub fn default_mandelbrot() -> Self {
        Self {
            iterations: 300,
            variant: Variant::Mandelbrot,
            julia_c: None,
            pick_c_using_cursor: pick_c_default(),
            multi_e: None,
        }
    }

    pub fn default_julia() -> Self {
        Self {
            iterations: 300,
            variant: Variant::Mandelbrot,
            julia_c: Some(C32::new(-0.76,-0.15)),
            pick_c_using_cursor: pick_c_default(),
            multi_e: None,
        }
    }
    
    pub fn is_julia(&self) -> bool { self.julia_c.is_some() }
}


impl FractalTrait for MandelbrotFamily {
    fn override_label(&mut self) -> Option<&'static str> {
        match (self.julia_c.is_some(), self.multi_e.is_some()) {
            (false, false) => None,
            (true, false) => Some("Julia Set"),
            (false, true) => Some("Multibrot Set"),
            (true, true) => Some("Multijulia Set"),
        }
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).clamp_range(1..=3000).ui(ui);
        });

        ui.horizontal(|ui| {
            ui.label("Variations");
            let arr = [Variant::Mandelbrot, Variant::Modified, Variant::BurningShip];
            let mut index = arr.iter().position(|v| *v == self.variant).unwrap();
            ComboBox::from_id_source("variation_selector")
                .selected_text(self.variant)
                .show_index(ui, &mut index, arr.len(), |i|arr[i]);
            self.variant = arr[index];
        });

        if self.julia_c.is_none() {
            if ui.button("To Julia Set").clicked() {
                self.julia_c = Some(C32::i());
                self.pick_c_using_cursor.0 = true;
            }
        } else {
            // x [     ] pick
            // [mandelbrot] [julia]
            ui.horizontal(|ui| {
                if Button::new("x").small().ui(ui).clicked() {
                    self.julia_c = None;
                    self.pick_c_using_cursor.0 = false;
                } else if c32_ui_full(ui, "C", self.julia_c.as_mut().unwrap(), Some(0.02), None).clicked() {
                    self.pick_c_using_cursor.0 = true;
                }
            });

            if self.pick_c_using_cursor.0 {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);

                // we don't check for click if we are in the button area
                let check_down = ui.horizontal(|ui| {
                    let mode = &mut self.pick_c_using_cursor.1;
                    if ui.selectable_label(*mode == PickCMode::Both, "Both").clicked() {
                        *mode = PickCMode::Both;
                    }
                    if ui.selectable_label(*mode == PickCMode::Julia, "Julia only").clicked() {
                        *mode = PickCMode::Julia;
                    }
                }).response.hovered().not();

                if check_down && ui.input(|input| input.pointer.any_down()) { self.pick_c_using_cursor.0 = false; }
            }
        }

        option_checkbox(ui, &mut self.multi_e, "Custom exponent",  || 2.);
        if let Some(e) = &mut self.multi_e {
            ui.horizontal(|ui| {
                ui.label("e");
                Slider::new(e, 1.0..=6.).drag_value_speed(0.02).clamp_to_range(false).ui(ui);
            });
        }
    }

    fn explanation_ui(&mut self, _ui: &mut Ui) {
        // todo
    }

    fn get_shader(&self) -> Shader {
        Shader::Mandelbrot(MandelbrotShader::Product(
            self.variant,
            if self.multi_e.is_some() {Multi::Enabled} else {Multi::Disabled},
        ))
    }

    fn fill_uniform_buffer(&self, mut buffer: UniformBuffer<&mut [u8]>) {
        // todo: return to this
        let escape_radius = match self.julia_c {
            None => 2.,
            Some(c) => {
                // we solve R > 0 such that R^2 - R == |c|
                // todo: escape radius is too small for e < 2 and (maybe) too big for e > 2
                (1. + (1. + 4. * c.abs()).sqrt()) / 2.
            }
        };

        buffer.write(&MandelbrotUniform {
            c: self.julia_c.unwrap_or_default().into(),
            iterations: self.iterations,
            escape_radius,
            exp: self.multi_e.unwrap_or(2.),
            // 0 if not in julia mode
            // 2 if not picking
            // 0,1,2 if picking
            julia: if self.julia_c.is_none() {0}
                else if self.pick_c_using_cursor.0 { self.pick_c_using_cursor.1 as u32 } else { 2 },
        }).unwrap();
    }

    fn draw_extra(&mut self, _painter: &Painter, mouse_pos: Option<Vec2>) {
        if let (Some(mouse_pos),(true, _),Some(c)) = (mouse_pos, &self.pick_c_using_cursor, &mut self.julia_c) {
            *c = vec2_to_c32(&mouse_pos);
        }
    }
}

impl From<Variant> for WidgetText {
    fn from(value: Variant) -> Self {
        match value {
            Variant::Mandelbrot => "Classic".into(),
            Variant::Modified => "Modified".into(),
            Variant::BurningShip => "Burning Ship".into(),
        }
    }
}