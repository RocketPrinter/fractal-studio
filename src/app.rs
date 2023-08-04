pub mod settings;
pub mod visualizer;

use eframe::egui::{Button, CentralPanel, Frame, SidePanel, Widget, Window};
use eframe::{App, CreationContext, egui};
use crate::app::settings::Settings;
use crate::app::visualizer::Visualizer;

pub struct EguiApp {
    settings: Settings,
    settings_floating: bool,
    visualizer: Visualizer,
}

impl EguiApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings: Settings = cc.storage.and_then(|storage| eframe::get_value(storage, eframe::APP_KEY)).unwrap_or_default();

        EguiApp {
            settings,
            settings_floating: false,
            visualizer: Visualizer::new(cc.wgpu_render_state.as_ref().unwrap().target_format ),
        }

    }
}

impl App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //ctx.set_debug_on_hover(true);
        if self.settings_floating {
            Window::new("Visualizer")
                .vscroll(true)
                .open(&mut self.settings_floating)
                .show(ctx, |ui| {
                    self.settings.ui(ui);
                });
        } else {
            SidePanel::right("settings_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Visualizer");
                    if Button::new("⏏").frame(false).ui(ui).clicked() {
                        self.settings_floating = true;
                    }
                });
                ui.separator();
                self.settings.ui(ui);

            });
        }

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