use eframe::egui::{Color32, Ui};
use crate::app::widgets::palette_picker;
use crate::fractal::FractalTrait;
use crate::wgsl::Shader;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestGrid {}

impl FractalTrait for TestGrid {
    fn label(&mut self) ->  &'static str { "Test Grid" }

    fn get_shader(&self) -> Shader { Shader::TestGrid }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.label("This is a test grid.");

        let mut colors = [Color32::LIGHT_RED; 5];
        palette_picker(ui, colors.as_mut_slice(), "Colors");
    }
}
