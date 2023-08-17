mod test_grid;
mod mandelbrot;
mod julia;
mod newtons;

use eframe::egui::{Context, Painter, Ui, Vec2};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use anyhow::{anyhow, Result};
use base64::prelude::*;
use encase::UniformBuffer;
use enum_dispatch::enum_dispatch;
use url::Url;
use test_grid::TestGrid;
use mandelbrot::Mandelbrot;
use julia::Julia;
use newtons::Newtons;

#[enum_dispatch]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumMessage, Hash))]
pub enum Fractal {
    /// Test Grid
    TestGrid,
    /// Mandelbrot Set
    Mandelbrot,
    /// Julia Set
    Julia,
    /// Newton's fractal
    Newtons,
}

#[enum_dispatch(Fractal)]
pub trait FractalTrait {
    fn settings_ui(&mut self, ui: &mut Ui);
    fn explanation(&mut self, ui: &mut Ui);
    fn fill_uniform_buffer(&self, _buffer: UniformBuffer<&mut [u8]>) {}
    /// mouse_pos will be Some if the mouse is hovering over the visualizer
    fn draw_extra(&mut self, _painter: &Painter, _mouse_pos: Option<Vec2>, /* todo: 2x2 matrix that converts mouse pos to shader space? */) {}
}

impl Default for Fractal {
    fn default() -> Self {
        Self::new(FractalDiscriminants::Mandelbrot)
    }
}

impl Fractal {
    pub fn new(discriminant: FractalDiscriminants) -> Self {
        match discriminant {
            FractalDiscriminants::TestGrid => TestGrid::default().into(),
            FractalDiscriminants::Mandelbrot => Mandelbrot::default().into(),
            FractalDiscriminants::Julia => Julia::default().into(),
            FractalDiscriminants::Newtons => Newtons::default().into(),
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
                else {anyhow::bail!("Cannot get the integration info")};
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