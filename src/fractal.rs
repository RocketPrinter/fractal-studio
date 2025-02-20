pub mod test_grid;
pub mod mandelbrot;
pub mod newtons;
pub mod lyapunov;

use eframe::egui::{Context, Id, Painter, Ui, Vec2};
use strum::{EnumDiscriminants, EnumMessage};
use anyhow::{anyhow, bail, Result};
use base64::prelude::*;
use encase::UniformBuffer;
use enum_dispatch::enum_dispatch;
use url::Url;
use test_grid::TestGrid;
use mandelbrot::MandelbrotFamily;
use newtons::Newtons;
use lyapunov::Lyapunov;
use crate::wgsl::Shader;

#[enum_dispatch]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, EnumDiscriminants, EnumMessage)]
pub enum Fractal {
    // --- Escape time ---
    TestGrid,
    MandelbrotFamily,
    Newtons,
    Lyapunov,
}

#[enum_dispatch(Fractal)]
pub trait FractalTrait {
    fn label(&mut self) -> &'static str;
    fn settings_ui(&mut self, _ui: &mut Ui) { }
    fn get_shader(&self) -> Shader;
    fn fill_uniform_buffer(&self, _buffer: UniformBuffer<&mut [u8]>) {}
    /// mouse_pos will be Some if the mouse is hovering over the visualizer
    fn draw_extra(&mut self, _painter: &Painter, _mouse_pos: Option<Vec2>, /* evenelytodo: 2x2 matrix that converts mouse pos to shader space? */) {}
}

impl Default for Fractal {
    fn default() -> Self {
        Self::MandelbrotFamily(MandelbrotFamily::default_mandelbrot())
    }
}

impl Fractal {
    pub fn to_code(&self) -> Result<String> {
        let serialized_code = rmp_serde::to_vec_named(self)?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(serialized_code))
    }

    pub fn to_link(&self, ctx: &Context) -> Result<String> {
        // root_url should be set when creating the app
        let Some(mut url) = ctx.data(|data|data.get_temp::<String>(Id::new("root_url"))) else {
            bail!("root_url has not been set!");
        };

        url.push_str("?fractal=");

        let serialized_code = rmp_serde::to_vec_named(self)?;
        BASE64_URL_SAFE_NO_PAD.encode_string(serialized_code, &mut url);
        Ok(url)
    }

    pub fn from_code(code: &str) -> Result<Fractal> {
        let bits = BASE64_URL_SAFE_NO_PAD.decode(code)?;
        rmp_serde::decode::from_slice(&bits).map_err(|e| e.into())
    }


    pub fn from_link(link: &str) -> Result<Fractal> {
        if link.is_empty() {
            bail!("Empty string");
        }

        match Url::parse(link) {
            Ok(url) => {
                // if the url parsing was successful we extract the query param
                let code = url.query_pairs()
                    .find(|(key, _)| key == "fractal")
                    .ok_or_else(|| anyhow!("Cannot find the fractal code in the query string"))?
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
