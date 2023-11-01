use crate::app::library::Library;
use crate::app::widgets::{error_toast};
use crate::fractal::lyapunov::Lyapunov;
use crate::fractal::mandelbrot::MandelbrotFamily;
use crate::fractal::newtons::Newtons;
use crate::fractal::test_grid::TestGrid;
use crate::fractal::{Fractal, FractalDiscriminants, FractalTrait};
use eframe::egui;
use eframe::egui::{
    vec2, Align2, Area, Button, CollapsingHeader, ComboBox, Id, RichText, SidePanel, TextEdit,
    Ui, Widget, Window,
};
use egui_extras::{Size, StripBuilder};
use std::default::Default;
use eframe::egui::menu::menu_button;
use egui_notify::{Toast, Toasts};
use strum::EnumMessage;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub fractal: Fractal,
    pub debug_label: bool,
    pub library: Library,
    pub library_window_open: bool,
    #[serde(skip)]
    pub hide: bool,
    #[serde(skip)]
    pub import_text: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fractal: Default::default(),
            library: Default::default(),
            library_window_open: false,
            debug_label: true,
            hide: false,
            import_text: "".into(),
        }
    }
}

impl Settings {
    pub fn show(&mut self, ctx: &egui::Context, toasts: &mut Toasts) {
        if self.hide {
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
            ui.horizontal(|ui| {
                ui.heading("Fractal Studio");

                ui.spacing_mut().item_spacing.x /= 2.;
                ui.menu_button("☰", |ui| self.hamburger_menu_ui(ui, toasts));
                if ui.button("⏵").clicked() {
                    self.hide = true;
                }
            });
            ui.separator();

            let height_id = ui.id().with("bottom_text_h");
            let bottom_height = ui.data(|data| data.get_temp(height_id).unwrap_or_default());
            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::exact(bottom_height))
                .vertical(|mut strip| {
                    strip.cell(|ui| self.main_ui(ui));
                    strip.cell(|ui| {
                        let new_height = Self::bottom_text_ui(ui);
                        ui.data_mut(|data| data.insert_temp(height_id, new_height));
                    })
                });
        });
    }

    fn main_ui(&mut self, ui: &mut Ui) {
        let fractal_d = FractalDiscriminants::from(&self.fractal);
        let fractal_label = self
            .fractal
            .override_label()
            .unwrap_or_else(|| fractal_d.get_documentation().unwrap_or_default());
        ui.horizontal(|ui| {
            ui.label("Fractal");
            ComboBox::from_id_source("Fractal selector")
                .selected_text(fractal_label)
                .show_ui(ui, |ui| {
                    use FractalDiscriminants as FD;
                    ui.small("Escape time fractals");

                    let is_julia = if let Fractal::MandelbrotFamily(m) = &self.fractal {
                        m.is_julia()
                    } else {
                        false
                    };

                    if ui
                        .selectable_label(
                            fractal_d == FD::MandelbrotFamily && !is_julia,
                            FD::MandelbrotFamily.get_documentation().unwrap_or_default(),
                        )
                        .clicked()
                    {
                        self.fractal =
                            Fractal::MandelbrotFamily(MandelbrotFamily::default_mandelbrot());
                    }

                    if ui
                        .selectable_label(
                            fractal_d == FD::MandelbrotFamily && is_julia,
                            "Julia Set",
                        )
                        .clicked()
                    {
                        self.fractal = Fractal::MandelbrotFamily(MandelbrotFamily::default_julia());
                    }

                    if ui
                        .selectable_label(
                            fractal_d == FD::Newtons,
                            FD::Newtons.get_documentation().unwrap_or_default(),
                        )
                        .clicked()
                    {
                        self.fractal = Fractal::Newtons(Newtons::default());
                    }

                    if ui
                        .selectable_label(
                            fractal_d == FD::Lyapunov,
                            FD::Lyapunov.get_documentation().unwrap_or_default(),
                        )
                        .clicked()
                    {
                        self.fractal = Fractal::Lyapunov(Lyapunov::default());
                    }
                    ui.small("More coming soon...")
                });
        });

        self.fractal.settings_ui(ui);

        // todo: refactor library
        Window::new("Library")
            .vscroll(true)
            .open(&mut self.library_window_open)
            .collapsible(false)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                self.library.ui(ui, &mut self.fractal);
            });
    }

    fn hamburger_menu_ui(&mut self, ui: &mut Ui, toasts: &mut Toasts) {

        //todo: make work and also not look like crap
        menu_button(ui,"Import link", |ui| {
            TextEdit::multiline(&mut self.import_text)
                .hint_text("Link or code")
                .desired_width(150.)
                .ui(ui);

            if ui
                .add_enabled(!self.import_text.is_empty(), Button::new("Import"))
                .clicked()
            {
                match Fractal::from_link(&self.import_text) {
                    Ok(fractal) => {
                        self.fractal = fractal;
                        self.import_text.clear();
                        toasts.add(Toast::success("Loaded fractal"));
                    }
                    Err(e) => { toasts.add(error_toast(e));},
                }
            }
        });

        if ui.button("Copy link to clipboard").clicked() {
            match self.fractal.to_link(ui.ctx()) {
                Ok(url) => {
                    ui.output_mut(move |output| output.copied_text = url);
                    toasts.add(Toast::success("Copied link to clipboard"));
                }
                Err(e) => { toasts.add(error_toast(e));},
            }
        }

        // todo: temp
        if ui
            .add_enabled(!self.library_window_open, Button::new("Open library"))
            .clicked()
        {
            self.library_window_open = true;
        }

        CollapsingHeader::new(RichText::new("Debug").color(ui.style().visuals.weak_text_color()))
            .show(ui, |ui| {
                ui.checkbox(&mut self.debug_label, "Debug label");
                if ui.button("Test grid").clicked() {
                    self.fractal = Fractal::TestGrid(TestGrid::default());
                }
            });
    }

    // returns the height of the widgets
    fn bottom_text_ui(ui: &mut Ui) -> f32 {
        ui.spacing_mut().item_spacing.x = 0.0;

        ui.horizontal_wrapped(|ui| {
            ui.label("Powered by ");
            ui.hyperlink_to("egui ", "https://www.egui.rs/");
            ui.label("and ");
            ui.hyperlink_to("wgpu. ", "https://wgpu.rs/");
        })
        .response
        .rect
        .height()
            + ui.horizontal_wrapped(|ui| {
                ui.label("Check the source or ");
                ui.label(RichText::new('★').color(ui.style().visuals.warn_fg_color));
                ui.label(" on ");
                ui.hyperlink_to("Github.", "https://github.com/RocketPrinter/fractal-studio");
            })
            .response
            .rect
            .height()
            + 5.
    }
}
