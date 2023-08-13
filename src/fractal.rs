use bytemuck::bytes_of;
use eframe::egui::{Button, CollapsingHeader, Context, CursorIcon, DragValue, Grid, Id, Painter, Ui, Vec2, vec2, Widget};
use num_complex::Complex32;
use num_traits::One;
use rand::Rng;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use crate::app::widgets::{vec2_ui_full};
use anyhow::{anyhow, bail, Result};
use base64::prelude::*;
use url::Url;

// todo: allow exporting config to a sharable link
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumMessage, Hash))]
pub enum Fractal {
    /// Test Grid
    TestGrid,
    /// Mandelbrot Set
    Mandelbrot { iterations: u32 },
    // todo: add option to draw path of point
    /// Julia Set
    Julia {
        iterations: u32,
        c: Vec2,
        pick_using_cursor: bool,
        animating_on_circle: bool,
    },
    /// Newton's fractal
    Netwtons {
        // todo: add a threshold for black pixels and arguments a and c from the wiki page
        iterations: u32,
        /// 1..=5 roots
        roots: Vec<Vec2>,
        /// u32 is the index of the root being picked
        pick_using_cursor: Option<u32>,
        // extra parameters
        a: Vec2,
        c: Vec2,

    }
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
            FractalDiscriminants::Netwtons => Fractal::Netwtons {
                iterations: 50,
                roots: vec![vec2(1., 0.), vec2(-0.5, 0.866), vec2(-0.5, -0.866)],
                pick_using_cursor: None,
                a: Vec2::new(1., 0.),
                c: Vec2::ZERO,
            },
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

                if *pick_using_cursor {
                    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                    *animating_on_circle = false;
                    if ui.input(|input| input.pointer.any_down()) {
                        *pick_using_cursor = false;
                    }
                }

                ui.horizontal(|ui|{
                    if vec2_ui_full(ui, "c", c, true, Some(0.02), None) {
                        *pick_using_cursor = true;
                    }
                });

                if *animating_on_circle {
                    *animating_on_circle = !ui.button("Stop").clicked();
                    *c = Vec2::angled(c.angle() + 0.01) * c.length();
                    ui.ctx().request_repaint();
                } else {
                    * animating_on_circle = ui.button("Animate on circle").clicked();
                }
            },
            Fractal::Netwtons { iterations, roots, pick_using_cursor, a, c } => {
                ui.horizontal(|ui|{
                    ui.label("Iterations");
                    DragValue::new(iterations).speed(1).clamp_range(0..=3000).ui(ui);
                });

                if pick_using_cursor.is_some() {
                    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                    if ui.input(|input| input.pointer.any_down()) { *pick_using_cursor = None; }
                };

                ui.horizontal(|ui|{
                    ui.label("Roots");
                    if ui.add_enabled(roots.len() < 5, Button::new("+").small().min_size(vec2(15.,0.))).clicked() {
                        let mut rand = rand::thread_rng();
                        roots.push(vec2(rand.gen::<f32>() * 2. - 1., rand.gen::<f32>() * 2. - 1.)) ;
                    }
                    if ui.add_enabled(roots.len() > 2, Button::new("-").small().min_size(vec2(15.,0.))).clicked() {
                        roots.pop();
                    }
                });
                Grid::new("roots grid").min_col_width(0.).num_columns(3).striped(true).show(ui, |ui| {
                    for (i,root) in roots.iter_mut().enumerate() {
                        if vec2_ui_full(ui, format!("{}",i+1), root,true, Some(0.02), None) {
                            *pick_using_cursor = Some(i as u32);
                        }
                        ui.end_row();
                    }
                });
                // todo: when mouse is over any part of the roots grid show visualize location

                CollapsingHeader::new("Extra parameters").show(ui, |ui| {
                    ui.horizontal(|ui|{
                        ui.label("a");
                        if vec2_ui_full(ui, "", a, true, Some(0.02), None) {
                            *pick_using_cursor = Some(5); //todo: hack
                        }
                    });
                    ui.horizontal(|ui|{
                        ui.label("c");
                        if vec2_ui_full(ui, "", c, true, Some(0.02), None) {
                            *pick_using_cursor = Some(6); //todo: hack
                        }
                    });
                });
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
                Fractal::Netwtons { .. } => {
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
            Fractal::Netwtons { roots, pick_using_cursor: Some(root_index), a, c, .. } => {
                match root_index {
                    0..=4 =>
                        if let Some(root) = roots.get_mut(*root_index as usize) {
                            *root = pos;
                        },
                    5 => *a = pos,
                    6 => *c = pos,
                    _ => (),
                }

                if let Some(root) = roots.get_mut(*root_index as usize) {
                    *root = pos;
                }
            }
            _ => (),
        }
    }

    pub fn fill_uniform_buffer(&self, buffer: &mut [u8]) {
        match self {
            Fractal::TestGrid => (),
            Fractal::Mandelbrot { iterations } => buffer[16..20].copy_from_slice(&iterations.to_ne_bytes()),
            Fractal::Julia { iterations, c, ..} => {
                // solving the escape radius aka R
                // choose R > 0 such that R**2 - R >= sqrt(cx**2 + cy**2)
                let r = (1. + (1. + 4. * c.length()).sqrt()) / 2.;
                buffer[16..20].copy_from_slice(&iterations.to_ne_bytes());
                buffer[20..24].copy_from_slice(&r.to_ne_bytes());
                buffer[24..32].copy_from_slice(bytes_of(c));
            },
            Fractal::Netwtons { iterations, roots, a, c, .. } => {
                let mut polynomial_coef = [Complex32::default();6];
                polynomial_coef[0] = Complex32::one();
                for (i,root) in roots.iter().enumerate() {
                    let root = Complex32::new(root.x,root.y);
                    for j in (0..=i+1).rev() {
                        if j == 0 {
                            polynomial_coef[j] = - root * polynomial_coef[j];
                        } else {
                            polynomial_coef[j] = polynomial_coef[j-1] - root * polynomial_coef[j];
                        }
                    }
                }
                let interweaved_arrays = roots.iter().chain([Vec2::ZERO].iter())
                    .zip(polynomial_coef);
                let mut offset = 16;
                for (root, coefficients) in interweaved_arrays {
                    buffer[offset..offset+8].copy_from_slice(bytes_of(root));
                    offset+=8;
                    buffer[offset..offset+8].copy_from_slice(bytes_of(&coefficients));
                    offset+=8;
                }
                buffer[112..120].copy_from_slice(bytes_of(a));
                buffer[120..128].copy_from_slice(bytes_of(c));
                buffer[128..132].copy_from_slice(&(roots.len() as u32).to_ne_bytes());
                buffer[132..136].copy_from_slice(&iterations.to_ne_bytes());
            },
        }
    }

    pub fn to_code(&self) -> Result<String> {
        let serialized_code = rmp_serde::to_vec_named(self)?;
        Ok(BASE64_URL_SAFE.encode(serialized_code))
    }

    pub fn to_link(&self, _ctx: &Context) -> Result<String> {
        // if on native, hardcode the url
        #[cfg(not(target_arch = "wasm32"))]
        let mut url = "https://rocketprinter.github.io/fractal_visualizer?fractal=".to_string();

        //uses "integration_info" data to get the integration info, which should be set in app.rs
        #[cfg(target_arch = "wasm32")]
        let mut url = {
            let Some(integration_info) = _ctx.data(|data|data.get_temp::<std::sync::Arc<eframe::IntegrationInfo>>(Id::new("integration_info")))
                else {bail!("Cannot get the integration info")};
            let mut url = integration_info.web_info.location.url.clone();
            url.push_str("?fractal=");
            url
        };

        let serialized_code = rmp_serde::to_vec_named(self)?;
        BASE64_URL_SAFE.encode_string(serialized_code, &mut url);
        Ok(url)
    }

    pub fn from_code(code: &str) -> Result<Fractal> {
        let bits = BASE64_URL_SAFE.decode(code)?;
        rmp_serde::decode::from_slice(&bits).map_err(|e| e.into())
    }


    pub fn from_link(link: &str) -> Result<Fractal> {
        match Url::parse(link) {
            Ok(url) => {
                // if the url parsing was successful we extract the query param
                let code = url.query_pairs()
                    .find(|(key, _)| key == "fractal")
                    .ok_or_else(|| anyhow!("Cannot find the fractal in the query string"))?
                    .1;
                Self::from_code(&code)
            },
            Err(_) => {
                // otherwise we can only assume that the whole string is the base64 code
                Self::from_code(link)
            },
        }
    }
}