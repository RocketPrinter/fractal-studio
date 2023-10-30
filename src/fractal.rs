pub mod test_grid;
pub mod mandelbrot;
pub mod newtons;
pub mod lyapunov;

use eframe::egui::{Context, Painter, Ui, Vec2};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use anyhow::{anyhow, Result};
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
#[strum_discriminants(derive(EnumIter, EnumMessage, Hash))]
pub enum Fractal {
    // --- Escape time ---
    /// Test Grid
    TestGrid,
    /// Mandelbrot Set
    MandelbrotFamily,
    /// Newton's fractal
    Newtons,
    /// Lyapunov's fractal
    Lyapunov,
}

#[enum_dispatch(Fractal)]
pub trait FractalTrait {
    fn override_label(&mut self) -> Option<&'static str> { None }
    fn settings_ui(&mut self, _ui: &mut Ui) { }
    fn explanation_ui(&mut self, _ui: &mut Ui) { }
    fn get_shader(&self) -> Shader;
    fn fill_uniform_buffer(&self, _buffer: UniformBuffer<&mut [u8]>) {}
    /// mouse_pos will be Some if the mouse is hovering over the visualizer
    fn draw_extra(&mut self, _painter: &Painter, _mouse_pos: Option<Vec2>, /* todo: 2x2 matrix that converts mouse pos to shader space? */) {}
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

    pub fn to_link(&self, _ctx: &Context) -> Result<String> {
        // if on native, hardcode the url
        #[cfg(not(target_arch = "wasm32"))]
        let mut url = "https://rocketprinter.github.io/fractal_visualizer?fractal=".to_string();

        //uses "integration_info" data to get the integration info, which should be set in app.rs
        #[cfg(target_arch = "wasm32")]
        let mut url = {
            use eframe::egui::Id;
            use std::sync::Arc;
            let Some(integration_info) = _ctx.data(|data|data.get_temp::<Arc<eframe::IntegrationInfo>>(Id::new("integration_info")))
                else {anyhow::bail!("Cannot get the integration info")};
            let mut url = integration_info.web_info.location.url.clone();
            url.push_str("?fractal=");
            url
        };

        let serialized_code = rmp_serde::to_vec_named(self)?;
        BASE64_URL_SAFE_NO_PAD.encode_string(serialized_code, &mut url);
        Ok(url)
    }

    pub fn from_code(code: &str) -> Result<Fractal> {
        let bits = BASE64_URL_SAFE_NO_PAD.decode(code)?;
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