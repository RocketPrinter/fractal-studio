use eframe::egui::Ui;
use strum::{EnumDiscriminants, EnumIter};
use crate::app::visualizer::FRAGMENT_PUSH_CONSTANTS_SIZE;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub kind: Kind,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut Ui) {

    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
pub enum Kind {
    #[default]
    Test,
    Mandelbrot {
        iterations: u32,
    },
}

impl Kind {
    pub fn push_constants(&self) -> Option<[u8; FRAGMENT_PUSH_CONSTANTS_SIZE as usize]> {
        match self {
            Kind::Test => None,
            Kind::Mandelbrot { .. } => {
                None // todo
            }
        }
    }
}