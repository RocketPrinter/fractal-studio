pub mod widgets;
mod settings;
mod visualizer;
mod library;
mod rendering;

use std::ops::Deref;
use std::sync::Arc;

use eframe::egui::{CentralPanel, ColorImage, Frame, Id};
use eframe::{egui, App, CreationContext};
use egui_notify::{Anchor, Toasts};
use rendering::RenderData;
use crate::app::settings::Settings;
use crate::app::visualizer::Visualizer;

// todo: ui scaling
pub struct EguiApp {
    settings: Settings,
    visualizer: Visualizer,
    toasts: Toasts,
}

impl EguiApp {
    pub fn new(cc: &CreationContext<'_>) -> EguiApp {
        #[allow(unused_mut)] // it's only mutated in wasm32
        let mut settings: Settings = cc.storage.and_then(|storage| eframe::get_value(storage, eframe::APP_KEY)).unwrap_or_default();

        let wgpu = cc.wgpu_render_state.as_ref().unwrap();
        let rd = RenderData::new(&wgpu.device, wgpu.target_format);
        wgpu.renderer.write().callback_resources.insert(rd);

        // used to create sharable links, on non wasm platforms it's hardcoded
        #[cfg(target_arch = "wasm32")]
        let root_url = cc.integration_info.web_info.location.url.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let root_url = "https://rocketprinter.github.io/fractal-studio".to_string();
        cc.egui_ctx.data_mut(|data|data.insert_temp(Id::new("root_url"), root_url));

        // if we're in wasm, try to load a fractal from the current url
        #[cfg(target_arch = "wasm32")]
        {
            use crate::fractal::Fractal;
            use log::error;
            if let Some(code) = cc.integration_info.web_info.location.query_map.get("fractal") {
                // is fractal appears multiple times, we only consider the first appearance
                match Fractal::from_code(code[0].as_ref()) {
                    Ok(fractal) => settings.fractal = fractal,
                    Err(e) => error!("Failed to load fractal from url: {}", e),
                }
            }
        }

        EguiApp {
            settings,
            visualizer: Visualizer::default(),
            toasts: Toasts::default().with_anchor(Anchor::TopLeft),
        }

    }
}

impl App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // don't render most of the UI if we're taking a screenshot
        let screenshot_triggered = self.visualizer.screenshot_triggered;

        if !screenshot_triggered {
            copy_screenshots_to_clipboard(ctx, &mut self.toasts);

            self.settings.show(ctx, &mut self.toasts);

            self.toasts.show(ctx);
        }

        CentralPanel::default()
            // remove margin and background
            .frame(Frame::default())
            .show(ctx, |ui| {
            self.visualizer.ui(&mut self.settings, ui);
        });

        if screenshot_triggered {
            self.visualizer.screenshot_triggered = false;
        }
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.settings);
    }
}

fn copy_screenshots_to_clipboard(ctx: &egui::Context, toasts: &mut Toasts) {
    let screenshots: Vec<Arc<ColorImage>> = ctx.input(|i| {
        i.events.iter().filter_map(|e| {
            match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
              _ => None,
            }
        }).collect()
    });

    if !screenshots.is_empty() {
        toasts.info("Copied screenshot to the clipboard");
    }

    for s in screenshots {
        // the api forces us to clone the image
        ctx.copy_image(s.deref().clone());
    }
}
