pub mod settings;
pub mod visualizer;
pub mod widgets;
pub mod library;

use std::sync::Arc;
use eframe::egui::{CentralPanel, Frame, Id};
use eframe::{App, CreationContext, egui};
use egui_notify::{Anchor, Toasts};
use crate::app::settings::Settings;
use crate::app::visualizer::Visualizer;

// todo: ui scaling
pub struct EguiApp {
    settings: Settings,
    visualizer: Visualizer,
    toasts: Toasts,

    set_info: bool,
}

impl EguiApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings: Settings = cc.storage.and_then(|storage| eframe::get_value(storage, eframe::APP_KEY)).unwrap_or_default();

        EguiApp {
            settings,
            visualizer: Visualizer::new(cc.wgpu_render_state.as_ref().unwrap().target_format ),
            toasts: Toasts::default().with_anchor(Anchor::TopLeft),

            set_info: false,
        }

    }
}

impl App for EguiApp {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.toasts.show(ctx);

        // used for later accessing the url
        if !self.set_info {
            self.set_info = true;
            let info = Arc::new(frame.info());
            ctx.data_mut(|data| data.insert_temp(Id::new("integration_info"), info.clone()));

            // if we're running in web try loading the fractal from the url
            #[cfg(target_arch = "wasm32")]
            {
                use crate::fractal::{Fractal, FractalTrait};
                use log::error;
                if let Some(code) = info.web_info.location.query_map.get("fractal") {
                    match Fractal::from_code(code) {
                        Ok(fractal) => self.settings.fractal = fractal,
                        Err(e) => error!("Failed to load fractal from url: {}", e),
                    }
                }
            }
        }

        self.settings.show(ctx, &mut self.toasts);

        CentralPanel::default()
            // remove margin and background
            .frame(Frame::default())
            .show(ctx, |ui| {
            self.visualizer.ui(&mut self.settings, ui);
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.settings);
    }
}
