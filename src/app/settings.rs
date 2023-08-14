use eframe::egui;
use eframe::egui::{Button, CollapsingHeader, ComboBox, RichText, SidePanel, TextEdit, Ui, Vec2, Widget, Window};
use strum::{EnumMessage, IntoEnumIterator};
use crate::app::library::Library;
use crate::fractal::{Fractal, FractalDiscriminants};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub fractal: Fractal,
    pub debug_label: bool,
    pub library: Library,
    pub library_window_open: bool,
    #[serde(skip)]
    pub floating: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fractal: Default::default(),
            library: Default::default(),
            library_window_open: false,
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
                    self.ui(ui)
                });
            self.floating = floating;
        } else {
            SidePanel::right("settings_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Fractals");
                    // todo: instead of popping into a floating window it would be more useful if the button hide the side panel and left a tiny button to unhide
                    if Button::new("⏏").frame(false).ui(ui).clicked() {
                        self.floating = true;
                    }
                });
                ui.separator();
                self.ui(ui)
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

        ui.horizontal(|ui| {
            ui.menu_button("Import", |ui| {
                ui.label("todo");//todo
            });
            ui.menu_button("Export", |ui| {
                match self.fractal.to_link(ui.ctx()) {
                    Ok(url) => {
                        // trick to force the text to be not editable
                        ui.text_edit_multiline(&mut (&url[..]));
                        if ui.button("Copy to clipboard").clicked() {
                            ui.output_mut(move |output|output.copied_text = url)
                        }
                    },
                    Err(e) => {
                        ui.colored_label(ui.style().visuals.error_fg_color, format!("Error: {}", e));
                    }
                }

            });
        });

        ui.separator();

        if ui.add_enabled(!self.library_window_open, Button::new("Open library")).clicked() {
            self.library_window_open = true;
        }

        Window::new("Library")
            .vscroll(true)
            .open(&mut self.library_window_open)
            .collapsible(false)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                self.library.ui(ui, &mut self.fractal);
            });

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