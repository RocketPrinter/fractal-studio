use eframe::egui::Ui;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    kind: Kind,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Kind {
    #[default]
    Test,
    Mandelbrot,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut Ui) {

    }
}