use eframe::egui::{ComboBox, Ui};
use strum::{EnumMessage, IntoEnumIterator};
use crate::fractal::{Fractal, FractalDiscriminants};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub fractal: Fractal,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut Ui) {
        let fractal_d = FractalDiscriminants::from(&self.fractal);
        ComboBox::from_id_source("Fractal selector")
            .selected_text(fractal_d.get_message().unwrap_or_default())
            .show_ui(ui, |ui| {
                for iter_d in FractalDiscriminants::iter() {
                    if ui.selectable_label(fractal_d == iter_d, iter_d.get_message().unwrap_or_default()).clicked() {
                        self.fractal = Fractal::default(iter_d);
                    }
                }
            });

        self.fractal.settings_ui(ui);
    }
}