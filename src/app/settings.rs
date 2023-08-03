use std::ops::RangeInclusive;
use eframe::egui::{ComboBox, DragValue, Ui, Vec2, Widget};
use strum::{EnumMessage, IntoEnumIterator};
use crate::fractal::{Fractal, FractalDiscriminants};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    // todo: move scale and offset here
    pub fractal: Fractal,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut Ui) {
        let fractal_d = FractalDiscriminants::from(&self.fractal);
        ComboBox::from_id_source("Fractal selector")
            .selected_text(fractal_d.get_documentation().unwrap_or_default())
            .show_ui(ui, |ui| {
                for iter_d in FractalDiscriminants::iter() {
                    if ui.selectable_label(fractal_d == iter_d, iter_d.get_documentation().unwrap_or_default()).clicked() {
                        self.fractal = Fractal::default(iter_d);
                    }
                }
            });

        self.fractal.settings_ui(ui);
    }
}

pub fn vec2_ui(ui: &mut Ui, v: &mut Vec2, complex: bool, speed: Option<f32>, clamp_range: Option<RangeInclusive<f32>>) {
    ui.horizontal(|ui| {
        let mut x = DragValue::new(&mut v.x);
        let mut y = DragValue::new(&mut v.y);
        if complex {
            y = y.suffix("i");
        } else {
            x = x.prefix("x=");
            y = y.prefix("y=");
        }
        if let Some(speed) = speed {
            x = x.speed(speed);
            y = y.speed(speed);
        }
        if let Some(clamp_range) = clamp_range {
            x = x.clamp_range(clamp_range.clone());
            y = y.clamp_range(clamp_range);
        }
        x.ui(ui);
        ui.add_space(-6.);
        y.ui(ui);
    });
}