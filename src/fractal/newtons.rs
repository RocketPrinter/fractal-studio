use bytemuck::bytes_of;
use eframe::egui::{Button, CollapsingHeader, CursorIcon, DragValue, Grid, Painter, Ui, vec2, Vec2, Widget};
use encase::UniformBuffer;
use rand::Rng;
use encase::ShaderType;
use crate::app::widgets::c32_ui_full;
use crate::fractal::FractalTrait;
use crate::math::{C32, UC32, vec2_to_c32};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Newtons {
    iterations: u32,
    /// 1..=5 roots
    roots: Vec<C32>,
    /// u32 is the index of the root being picked
    pick_using_cursor: Option<Pick>,
    // extra parameters
    a: C32,
    c: C32,
    threshold: f32, // can be infinity
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
    a: UC32,
    c: UC32,
    nr_roots: u32,
    max_iterations: u32,
    threshold: f32,
}

impl Default for Newtons {
    fn default() -> Self {
        Self {
            iterations: 50,
            roots: vec![C32::new(1., 0.), C32::new(-0.5, 0.866), C32::new(-0.5, -0.866)],
            pick_using_cursor: None,
            a: C32::new(1., 0.),
            c: nalgebra::zero(),
            threshold: f32::INFINITY,
        }
    }
}

impl FractalTrait for Newtons {
    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui|{
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).clamp_range(0..=3000).ui(ui);
        });

        if self.pick_using_cursor.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            if ui.input(|input| input.pointer.any_down()) { self.pick_using_cursor = None; }
        };

        ui.horizontal(|ui|{
            ui.label("Roots");
            if ui.add_enabled(self.roots.len() < 5, Button::new("+").small().min_size(vec2(15.,0.))).clicked() {
                let mut rand = rand::thread_rng();
                self.roots.push(C32::new(rand.gen::<f32>() * 2. - 1., rand.gen::<f32>() * 2. - 1.)) ;
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
                ui.add_enabled(enabled, DragValue::new(&mut self.threshold).speed(0.1).clamp_range(0.0001..=f32::INFINITY));
            });
        });
    }

    fn explanation(&mut self, ui: &mut Ui) {
        // todo
    }

    fn fill_uniform_buffer(&self, buffer: UniformBuffer<&mut [u8]>) {
        let mut polynomial_coef: [C32; 6] = [nalgebra::zero();6];
        polynomial_coef[0] = nalgebra::one();
        for (i,root) in self.roots.iter().enumerate() {
            for j in (0..=i+1).rev() {
                if j == 0 {
                    polynomial_coef[j] = - root * polynomial_coef[j];
                } else {
                    polynomial_coef[j] = polynomial_coef[j-1] - root * polynomial_coef[j];
                }
            }
        }

        let extra_item = [nalgebra::zero()];
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
            a: self.a.into(),
            c: self.c.into(),
            nr_roots: self.roots.len() as u32,
            max_iterations: self.iterations,
            threshold: self.threshold,
        }).unwrap()
    }

    fn draw_extra(&mut self, painter: &Painter, mouse_pos: Option<Vec2>) {
        if let (Some(mouse_pos),Some(pick)) = (mouse_pos, &self.pick_using_cursor) {
            match pick {
                Pick::Root(index) => {
                    if let Some(root) = self.roots.get_mut(*index as usize) {
                        *root = vec2_to_c32(&mouse_pos);
                    }
                }
                Pick::A => self.a = vec2_to_c32(&mouse_pos),
                Pick::C => self.c = vec2_to_c32(&mouse_pos),
            }
        }
        //todo: draw roots
    }
}