pub mod settings;
pub mod visualizer;
pub mod widgets;

use eframe::egui::{Button, CentralPanel, Frame, SidePanel, Widget, Window};
use eframe::{App, CreationContext, egui};
use crate::app::settings::Settings;
use crate::app::visualizer::Visualizer;

pub struct EguiApp {
    settings: Settings,
    visualizer: Visualizer,
}

impl EguiApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings: Settings = cc.storage.and_then(|storage| eframe::get_value(storage, eframe::APP_KEY)).unwrap_or_default();

        EguiApp {
            settings,
            visualizer: Visualizer::new(cc.wgpu_render_state.as_ref().unwrap().target_format ),
        }

    }
}

impl App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.settings.ctx(ctx);

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