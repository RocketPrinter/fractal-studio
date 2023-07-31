use eframe::egui::{ComboBox, DragValue, Grid, Ui, Widget};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, EnumVariantNames};
use crate::app::visualizer::FRAGMENT_PUSH_CONSTANTS_SIZE;
use crate::helpers::extend_array;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub kind: Kind,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut Ui) {
        // todo: save settings in another place so switching between visualizers doesn't reset them (also add a reset settings button)
        ComboBox::from_id_source("Kind selector")
            .selected_text(self.kind.get_message().unwrap())
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut self.kind, Kind::Test, "Test grid").clicked() {
                    self.kind = Kind::Test;
                }
                if ui.selectable_label(false, "Mandelbrot set").clicked() {
                    self.kind = Kind::Mandelbrot { iterations: 100 };
                }
            });

        self.kind.ui(ui);
    }
}

// todo: consider switching to a trait
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, EnumDiscriminants, EnumMessage)]
#[strum_discriminants(derive(EnumIter, Hash))]
pub enum Kind {
    #[default]
    #[strum(message="Test grid")]
    Test,
    #[strum(message="Mandelbrot set")]
    Mandelbrot {
        iterations: u32,
    },
}

impl Kind {
    pub fn push_constants(&self) -> Option<[u8; FRAGMENT_PUSH_CONSTANTS_SIZE]> {
        match self {
            Kind::Test => None,
            Kind::Mandelbrot { iterations } => {
                Some(extend_array(&iterations.to_ne_bytes(),0))
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        Grid::new("Kind grid")
            .num_columns(2)
            .show(ui, |ui|{
            match self {
                Kind::Test => (),
                Kind::Mandelbrot { iterations } => {
                    ui.label("Iterations");
                    DragValue::new(iterations).speed(1).clamp_range(0..=3000).ui(ui);
                    ui.end_row();
                }
            }
        });
    }
}