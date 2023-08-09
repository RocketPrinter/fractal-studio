use eframe::egui;
use eframe::egui::{Button, CollapsingHeader, ComboBox, DragValue, RichText, SidePanel, Ui, Vec2, Widget, Window};
use strum::{EnumMessage, IntoEnumIterator};
use crate::fractal::{Fractal, FractalDiscriminants};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub fractal: Fractal,
    #[serde(skip)]
    pub floating: bool,
    pub debug_label: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fractal: Default::default(),
            debug_label: true,
            floating: false,
        }
    }
}

impl Settings {
    pub fn ctx(&mut self, ctx: &egui::Context) {
        if self.floating {
            let mut floating = self.floating;
            Window::new("Fractals")
                .vscroll(true)
                .open(&mut floating)
                .show(ctx, |ui| {
                    self.ui(ui);
                });
            self.floating = floating;
        } else {
            SidePanel::right("settings_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Fractals");
                    if Button::new("⏏").frame(false).ui(ui).clicked() {
                        self.floating = true;
                    }
                });
                ui.separator();
                self.ui(ui);

            });
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        let fractal_d = FractalDiscriminants::from(&self.fractal);
        ComboBox::from_id_source("Fractal selector")
            .selected_text(fractal_d.get_documentation().unwrap_or_default())
            .show_ui(ui, |ui| {
                // skip(1) skips TestGrid
                for iter_d in FractalDiscriminants::iter().skip(1) {
                    if ui.selectable_label(fractal_d == iter_d, iter_d.get_documentation().unwrap_or_default()).clicked() {
                        self.fractal = Fractal::default(iter_d);
                    }
                }
            });

        self.fractal.settings_ui(ui);

        ui.separator();
        CollapsingHeader::new(RichText::new("Debug").color(ui.style().visuals.weak_text_color())).show(ui,|ui| {
            ui.checkbox(&mut self.debug_label, "Debug label");
            if ui.button("Test grid").clicked() {
                self.fractal = Fractal::TestGrid;
            }

        });

        ui.horizontal_wrapped(|ui|{
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Powered by ");
            ui.hyperlink_to("egui ","https://www.egui.rs/");
            ui.label("and ");
            ui.hyperlink_to("wgpu", "https://wgpu.rs/");
            ui.label(". Check the source on ");
            ui.hyperlink_to(" Github","https://github.com/RocketPrinter/fractal_visualizer");
            ui.label(".")
        });
    }
}