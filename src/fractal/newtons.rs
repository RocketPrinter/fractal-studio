use std::sync::LazyLock;

use bytemuck::bytes_of;
use ecolor::{hex_color, Color32};
use eframe::egui::{Button, CollapsingHeader, CursorIcon, DragValue, Grid, Painter, Ui, vec2, Vec2, Widget};
use encase::UniformBuffer;
use glam::{Vec2 as GVec2, Vec4 as GVec4};
use num_complex::Complex32;
use rand::Rng;
use encase::ShaderType;
use crate::app::widgets::{c32_ui_full, palette_editor};
use crate::fractal::FractalTrait;
use crate::wgsl::{Complex32Ext, Shader, Vec2Ext};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Newtons {
    iterations: u32,
    /// 1..=5 roots
    roots: Vec<Complex32>,
    /// u32 is the index of the root being picked
    // extra parameters
    a: Complex32,
    c: Complex32,
    threshold: f32, // can be infinity
    #[serde(default = "default_palette")]
    colors: [Color32; 5],

    #[serde(skip)]
    pick_using_cursor: Option<Pick>,
}

// can this be more automated?
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum Pick {
    Root(u32),
    A,
    C,
}

#[derive(ShaderType)]
struct NewtonsUniform {
    colors: [GVec4;5],
    a: GVec2,
    c: GVec2,
    nr_roots: u32,
    max_iterations: u32,
    threshold: f32,
}

impl Default for Newtons {
    fn default() -> Self {
        Self {
            iterations: 50,
            roots: vec![Complex32::new(1., 0.), Complex32::new(-0.5, 0.866), Complex32::new(-0.5, -0.866)],
            a: Complex32::ONE,
            c: Complex32::ZERO,
            threshold: f32::INFINITY,
            colors: default_palette(),

            pick_using_cursor: None,
        }
    }
}

impl FractalTrait for Newtons {
    fn label(&mut self) ->  &'static str { "Newton's Fractal" }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui|{
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).range(0..=3000).ui(ui);
        });

        if self.pick_using_cursor.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            if ui.input(|input| input.pointer.any_down()) { self.pick_using_cursor = None; }
        };

        ui.horizontal(|ui|{
            ui.label("Roots");
            if ui.add_enabled(self.roots.len() < 5, Button::new("+").small().min_size(vec2(15.,0.))).clicked() {
                let mut rand = rand::rng();
                self.roots.push(Complex32::new(rand.random::<f32>() * 2. - 1., rand.random::<f32>() * 2. - 1.)) ;
            }
            if ui.add_enabled(self.roots.len() > 2, Button::new("-").small().min_size(vec2(15.,0.))).clicked() {
                self.roots.pop();
            }
        });
        Grid::new("roots grid").min_col_width(0.).num_columns(3).striped(true).show(ui, |ui| {
            for (i,root) in self.roots.iter_mut().enumerate() {
                if c32_ui_full(ui, format!("{}", i+1), root, Some(0.02), None).clicked() {
                    self.pick_using_cursor = Some(Pick::Root(i as u32));
                }
                ui.end_row();
            }
        });
        // todo: when mouse is over any part of the roots grid show visualize location

        CollapsingHeader::new("Extra parameters").show(ui, |ui| {
            ui.horizontal(|ui|{
                ui.label("a");
                if c32_ui_full(ui, "", &mut self.a, Some(0.02), None).clicked() {
                    self.pick_using_cursor = Some(Pick::A);
                }
            });
            ui.horizontal(|ui|{
                ui.label("c");
                if c32_ui_full(ui, "", &mut self.c, Some(0.02), None).clicked() {
                    self.pick_using_cursor = Some(Pick::C);
                }
            });
            ui.horizontal(|ui|{
                let mut enabled = !self.threshold.is_infinite();
                ui.checkbox(&mut enabled, "");
                // if the checkbox is not ticked we set the threshold to infinity
                if enabled {
                    if self.threshold.is_infinite() { self.threshold = 10.; }
                } else {
                    self.threshold = f32::INFINITY;
                }
                ui.label("Threshold");
                ui.add_enabled(enabled, DragValue::new(&mut self.threshold).speed(0.1).range(0.0001..=f32::INFINITY));
            });
        });

        palette_editor(ui, &mut self.colors, "Colors", COLOR_PALETTES.as_slice());
    }

    fn get_shader(&self) -> Shader { Shader::Newtons }

    fn fill_uniform_buffer(&self, buffer: UniformBuffer<&mut [u8]>) {
        let mut polynomial_coef: [Complex32; 6] = [Complex32::ZERO;6];
        polynomial_coef[0] = Complex32::ONE;
        for (i,root) in self.roots.iter().enumerate() {
            for j in (0..=i+1).rev() {
                if j == 0 {
                    polynomial_coef[j] = - root * polynomial_coef[j];
                } else {
                    polynomial_coef[j] = polynomial_coef[j-1] - root * polynomial_coef[j];
                }
            }
        }

        let extra_item = [Complex32::ZERO];
        let interweaved_arrays = self.roots.iter().chain(extra_item.iter())
            .zip(polynomial_coef);

        // we manually write the array cause it's faster and simpler so we access the inner buffer
        let buffer = buffer.into_inner();
        let mut offset = 0;
        for (root, coefficients) in interweaved_arrays {
            buffer[offset..offset+8].copy_from_slice(bytes_of(&[root.re, root.im]));
            offset+=8;
            buffer[offset..offset+8].copy_from_slice(bytes_of(&[coefficients.re, coefficients.im]));
            offset+=8;
        }

        // we write the rest of the buffer normally
        UniformBuffer::new(&mut buffer[96..]).write(&NewtonsUniform {
            colors: self.colors.map(|c|c.to_normalized_gamma_f32().into()),
            a: self.a.to_gvec2(),
            c: self.c.to_gvec2(),
            nr_roots: self.roots.len() as u32,
            max_iterations: self.iterations,
            threshold: self.threshold,
        }).unwrap()
    }

    fn draw_extra(&mut self, _painter: &Painter, mouse_pos: Option<Vec2>) {
        if let (Some(mouse_pos),Some(pick)) = (mouse_pos, &self.pick_using_cursor) {
            match pick {
                Pick::Root(index) => {
                    if let Some(root) = self.roots.get_mut(*index as usize) {
                        *root = mouse_pos.to_c32();
                    }
                }
                Pick::A => self.a = mouse_pos.to_c32(),
                Pick::C => self.c = mouse_pos.to_c32(),
            }
        }
    }
}

fn default_palette() -> [Color32;5] { COLOR_PALETTES[0] }

pub static COLOR_PALETTES: LazyLock<Vec<[Color32;5]>> = LazyLock::new(|| vec![
    // https://coolors.co/palette/f79256-fbd1a2-7dcfb6-00b2ca-1d4e89
    [hex_color!("F79256"),hex_color!("FBD1A2"),hex_color!("7DCFB6"),hex_color!("00B2CA"),hex_color!("1D4E89")],
    // https://coolors.co/palette/f9dbbd-ffa5ab-da627d-a53860-450920
    [hex_color!("f9dbbd"),hex_color!("ffa5ab"),hex_color!("da627d"),hex_color!("a53860"),hex_color!("450920")],
    // https://coolors.co/palette/0081a7-00afb9-fdfcdc-fed9b7-f07167
    [hex_color!("0081a7"),hex_color!("00afb9"),hex_color!("fdfcdc"),hex_color!("fed9b7"),hex_color!("f07167")],
    // todo: more palettes
]);
