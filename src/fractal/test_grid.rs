use eframe::egui::Ui;
use crate::fractal::FractalTrait;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestGrid {}

impl FractalTrait for TestGrid {
    fn settings_ui(&mut self, _ui: &mut Ui) {

    }

    fn explanation(&mut self, ui: &mut Ui) {
        ui.label("This is a test grid.");
    }
}