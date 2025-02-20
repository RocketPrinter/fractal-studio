use crate::app::library::Library;
use crate::app::widgets::error_toast;
use crate::fractal::lyapunov::Lyapunov;
use crate::fractal::mandelbrot::MandelbrotFamily;
use crate::fractal::newtons::Newtons;
use crate::fractal::test_grid::TestGrid;
use crate::fractal::{Fractal, FractalDiscriminants, FractalTrait};
use eframe::egui::{self, vec2, Align, Align2, Area, Button, CollapsingHeader, ComboBox, Id, Layout, Modal, RichText, SidePanel, Sides, TextEdit, Ui, UiBuilder, Vec2, Widget, Window};
use std::default::Default;
use egui_notify::Toasts;
use crate::app::library;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub fractal: Fractal,
    pub library: Library,
    pub debug_label: bool,
    pub library_window_open: bool,
    pub welcome_window_open: bool,
    #[serde(skip)]
    pub hide: bool,
    #[serde(skip)]
    import_modal: (bool, String),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fractal: Default::default(),
            library: Default::default(),
            welcome_window_open: true,
            library_window_open: false,
            debug_label: true,
            hide: false,
            import_modal: (false, String::new())
        }
    }
}

impl Settings {
    pub fn show(&mut self, ctx: &egui::Context, toasts: &mut Toasts) {
        if self.hide {
            // draw a little triangle to reopen the menu
            Area::new(Id::new("settings_open"))
                .anchor(Align2::RIGHT_TOP, vec2(0., 8.))
                .show(ctx, |ui| {
                    if Button::new("⏴").frame(false).ui(ui).clicked() {
                        self.hide = false;
                    }
                });
        }

        SidePanel::right("settings_panel").show_animated(ctx, !self.hide, |ui| {
            ui.add_space(5.);
            Sides::new().show(ui,
                |ui|ui.heading("Fractal Studio"),
                |ui| {
                    ui.spacing_mut().item_spacing.x /= 2.;
                    if ui.button("⏵").clicked() {
                        self.hide = true;
                    }
                    ui.menu_button("☰", |ui| self.hamburger_menu_ui(ui, toasts));
                }
            );
            ui.separator();

            self.main_ui(ui);
            ui.scope_builder(
                UiBuilder::new().sizing_pass().layout(Layout::bottom_up(Align::Min)),
                Self::bottom_text_ui,
            );
        });

        Window::new("Library")
            .vscroll(true)
            .open(&mut self.library_window_open)
            .collapsible(false)
            .constrain(false)
            .show(ctx, |ui| {
                self.library.ui(ui, &mut self.fractal, toasts);
            });

        self.welcome_window(ctx);

        self.import_modal(ctx, toasts);
    }

    fn main_ui(&mut self, ui: &mut Ui) {
        let fractal_d = FractalDiscriminants::from(&self.fractal);
        let fractal_label = self.fractal.label();
        ui.horizontal(|ui| {
            ui.label("Fractal");
            ComboBox::from_id_salt("Fractal selector")
                .selected_text(fractal_label)
                .show_ui(ui, |ui| {
                    use FractalDiscriminants as FD;
                    ui.small("Escape time fractals");

                    let is_julia = if let Fractal::MandelbrotFamily(m) = &self.fractal {
                        m.is_julia()
                    } else {
                        false
                    };

                    if ui.selectable_label(
                            fractal_d == FD::MandelbrotFamily && !is_julia,
                            "Mandelbrot Set",
                        ).clicked() {
                        self.fractal =
                            Fractal::MandelbrotFamily(MandelbrotFamily::default_mandelbrot());
                    }

                    if ui.selectable_label(
                            fractal_d == FD::MandelbrotFamily && is_julia,
                            "Julia Set",
                        ).clicked() {
                        self.fractal = Fractal::MandelbrotFamily(MandelbrotFamily::default_julia());
                    }

                    if ui.selectable_label(
                            fractal_d == FD::Newtons,
                            "Newton's Fractal",
                        ).clicked() {
                        self.fractal = Fractal::Newtons(Newtons::default());
                    }

                    if ui.selectable_label(
                            fractal_d == FD::Lyapunov,
                            "Lyapunov's Fractal",
                        ).clicked() {
                        self.fractal = Fractal::Lyapunov(Lyapunov::default());
                    }
                    ui.small("More coming soon...")
                });
        });

        self.fractal.settings_ui(ui);
    }

    fn hamburger_menu_ui(&mut self, ui: &mut Ui, toasts: &mut Toasts) {

        if ui.button("Import from link").clicked() {
            self.import_modal.0 = true;
        }

        if ui.button("Copy link to clipboard").clicked() {
            match self.fractal.to_link(ui.ctx()) {
                Ok(url) => {
                    ui.ctx().copy_text(url);
                    toasts.success("Copied link to clipboard");
                }
                Err(e) => { toasts.add(error_toast(e));},
            }
        }

        if ui.add_enabled(!self.welcome_window_open, Button::new("Show Welcome"))
            .clicked() {
            self.welcome_window_open = true;
        }

        if ui.add_enabled(!self.library_window_open, Button::new("Show Library"))
            .clicked() {
            self.library_window_open = true;
        }

        CollapsingHeader::new(RichText::new("Debug").color(ui.style().visuals.weak_text_color()))
            .show(ui, |ui| {
                ui.checkbox(&mut self.debug_label, "Debug label");
                let mut debug_on_hover = ui.ctx().debug_on_hover();
                if ui.checkbox(&mut debug_on_hover, "Debug on hover" ).changed() {
                    ui.ctx().set_debug_on_hover(debug_on_hover);
                }

                if ui.button("Test grid").clicked() {
                    self.fractal = Fractal::TestGrid(TestGrid::default());
                }

                ui.label("egui menus");

                let ctx = ui.ctx().clone();
                CollapsingHeader::new("settings").show(ui, |ui| {
                    ctx.settings_ui(ui);
                });
                CollapsingHeader::new("inspection").show(ui, |ui| {
                    ctx.inspection_ui(ui);
                });
                CollapsingHeader::new("memory").show(ui, |ui| {
                    ctx.memory_ui(ui);
                });
                CollapsingHeader::new("texture").show(ui, |ui| {
                    ctx.texture_ui(ui);
                });
            });
    }

    fn bottom_text_ui(ui: &mut Ui) {
        ui.spacing_mut().item_spacing.x = 0.0;

        ui.horizontal_wrapped(|ui| {
            ui.label("Read the source on ");
            ui.hyperlink_to("Github.", "https://github.com/RocketPrinter/fractal-studio");
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Powered by ");
            ui.hyperlink_to("egui ", "https://www.egui.rs/");
            ui.label("and ");
            ui.hyperlink_to("wgpu. ", "https://wgpu.rs/");
        });

        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
    }

    fn welcome_window(&mut self, ctx: &egui::Context) {
        let mut close_window = false;
        Window::new("Welcome!")
            .open(&mut self.welcome_window_open)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .resizable(false)
            .default_width(300.)
            .show(ctx, |ui| {
                ui.add_space(7.);
                ui.label("Explore multiple fractals by dragging and zooming.");
                ui.add_space(7.);
                ui.label("Tweak the settings of each fractal and discover new pretty things! (and use the link to share them)");
                ui.add_space(7.);

                ui.with_layout(Layout::right_to_left(Align::TOP),|ui|{
                    if ui.button("Check some examples >").clicked() {
                        self.library_window_open = true;
                        // we usually want the user tab to be open by default but this time we want the examples to be open
                        self.library.tab = library::Tab::Examples;
                        close_window = true;
                    }
                });
            });

        if close_window {self.welcome_window_open = false}
    }

    fn import_modal(&mut self, ctx: &egui::Context, toasts: &mut Toasts) {
        if !self.import_modal.0 { return; }

        let modal = Modal::new(Id::new("import_modal"))
            .show(ctx, |ui| {
            ui.set_width(250.);

            ui.heading("Import from link");

            TextEdit::multiline(&mut self.import_modal.1)
                .hint_text("paste link here")
                .ui(ui);

            ui.separator();

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if ui.button("Ok").clicked() {
                    match Fractal::from_link(&self.import_modal.1) {
                        Ok(fractal) => {
                            self.fractal = fractal;
                            self.import_modal.1.clear();
                            toasts.success("Loaded fractal");
                            self.import_modal.0 = false;
                        }
                        Err(e) => { toasts.add(error_toast(e));},
                    }
                }

                if ui.button("Cancel").clicked() {
                    self.import_modal.0 = false;
                }
            });
        });

        if modal.should_close() {
            self.import_modal.0 = false;
        }
    }
}
