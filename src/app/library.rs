use eframe::egui::{CollapsingHeader, Grid, Label, RichText, TextEdit, Ui};
use crate::fractal::Fractal;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Library {
    user_fractals: Vec<(String, String)>,
    #[serde(skip)]
    add_text: String,
    #[serde(skip)]
    last_error: Option<anyhow::Error>,
}

const INCLUDED_FRACTALS: [(&str,&str); 1] = [
    ("Test", "owo"), // todo: fill this in
];

impl Library {
    pub fn ui(&mut self, ui: &mut Ui, fractal: &mut Fractal) {
        CollapsingHeader::new("Included")
            .default_open(false)
            .show(ui, |ui| {
                ui.scope(|ui|{
                    // force the text to wrap
                    ui.set_max_width(150.);
                    ui.label("Here's some cool fractals I found while developing this project:");
                });
                for (name, code) in INCLUDED_FRACTALS.iter() {
                    if ui.selectable_label(false, *name).clicked() {
                        match Fractal::from_code(code) {
                            Ok(new_fractal) => {
                                *fractal = new_fractal;
                                self.last_error = None;
                            }
                            Err(e) => self.last_error = Some(e),
                        }
                    }
                }
            });

        Grid::new("user_fractals")
            .num_columns(2)
            .show(ui, |ui| {
                let mut delete_action = None;
                for (i,(name, code)) in self.user_fractals.iter().enumerate() {
                    if ui.selectable_label(false, name).clicked() {
                        match Fractal::from_code(code) {
                            Ok(new_fractal) => {
                                *fractal = new_fractal;
                                self.last_error = None;
                            }
                            Err(e) => self.last_error = Some(e),
                        }
                    }
                    if ui.small_button("x").clicked() {
                        delete_action = Some(i);
                    }
                    ui.end_row();
                }
                if let Some(i) = delete_action {
                    self.user_fractals.remove(i);
                }
            });

        ui.horizontal(|ui|{
            if self.add_text.is_empty() {
                self.add_text.push_str("Title");
            }
            TextEdit::singleline(&mut self.add_text).desired_width(130.).show(ui);
            if ui.button("Add").clicked() {
                match fractal.to_code() {
                    Ok(code) => {
                        self.user_fractals.push((
                            std::mem::replace(&mut self.add_text, "Title".into()),
                            code
                        ));
                    }
                    Err(e) => self.last_error = Some(e),
                }
            }
        });

        if let Some(e) = &self.last_error {
            let text = RichText::new(format!("Error: {e}")).color(ui.style().visuals.error_fg_color);
            if ui.selectable_label(false, text).clicked() {
                self.last_error = None;
            }
        }
    }
}