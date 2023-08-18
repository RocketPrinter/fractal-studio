use eframe::egui::{Button, CollapsingHeader, Grid, TextEdit, Ui};
use crate::app::widgets::dismissible_error;
use crate::fractal::Fractal;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Library {
    user_fractals: Vec<(String, String)>,
    #[serde(skip)]
    add_text: String,
    #[serde(skip)]
    last_error: Option<anyhow::Error>,
}

const INCLUDED_FRACTALS: &[(&str,&str)] = &[
    ("Starfish", "gaVKdWxpYYWqaXRlcmF0aW9uc2ShY5LKPqb2F8q_M-MMoWXKQKAAALFwaWNrX3VzaW5nX2N1cnNvcsKzYW5pbWF0aW5nX29uX2NpcmNsZcI"),
    ("Sun", "gaVKdWxpYYWqaXRlcmF0aW9uc2ShY5LKvdRBCso_X1xGoWXKQVAAALFwaWNrX3VzaW5nX2N1cnNvcsKzYW5pbWF0aW5nX29uX2NpcmNsZcI"),
    ("Nebula", "gaVKdWxpYYWqaXRlcmF0aW9uc2ShY5LKvdRBCso_X1xGoWXKQCmZmrFwaWNrX3VzaW5nX2N1cnNvcsKzYW5pbWF0aW5nX29uX2NpcmNsZcI"),
    ("Shuriken", "gaVKdWxpYYWqaXRlcmF0aW9uc2ShY5LKPxT7Xsq_c56ToWXKQIAAALFwaWNrX3VzaW5nX2N1cnNvcsKzYW5pbWF0aW5nX29uX2NpcmNsZcI"),
    ("Decorations","gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5WSyr_zGPzKPzOcD5LKvwAAAMo_XbItksq_AAAAyr9dsi2Syj8sVtbKvy9B8pLKv0px3sq_EQyzsXBpY2tfdXNpbmdfY3Vyc29ywKFhkso_TMzNyj-AAAChY5LKvczMzcoAAAAAqXRocmVzaG9sZMp_gAAA"),
    ("Helix","gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5WSyr_zGPzKPzOcD5LKvwAAAMo_XbItksq_AAAAyr9dsi2Syj8sVtbKvy9B8pLKv0px3sq_EQyzsXBpY2tfdXNpbmdfY3Vyc29ywKFhkso_TMzNyr-AAAChY5LKvczMzcoAAAAAqXRocmVzaG9sZMp_gAAA"),
    ("Meta", "gadOZXd0b25zhqppdGVyYXRpb25zZKVyb290c5OSyj-AAADKAAAAAJLKvwAAAMo_XbItksq_AAAAyr9dsi2xcGlja191c2luZ19jdXJzb3LAoWGSyj3sv7HKvszMzaFjksoAAAAAygAAAACpdGhyZXNob2xkyn-AAAA"),
    ("Blobs", "gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5OSyj-AAADKAAAAAJLKvwAAAMo_XbItksq_AAAAyr9dsi2xcGlja191c2luZ19jdXJzb3LAoWGSyj-AAADKP564UqFjksoAAAAAygAAAACpdGhyZXNob2xkyj9MzM0"),
    ("Spiral blobs", "gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5OSyj-AAADKAAAAAJLKvwAAAMo_XbItksq_AAAAyr9dsi2xcGlja191c2luZ19jdXJzb3LAoWGSyj-AAADKP564UqFjksoAAAAAyj51wo-pdGhyZXNob2xkyj8mZmY"),
];

impl Library {
    pub fn ui(&mut self, ui: &mut Ui, fractal: &mut Fractal) {
        CollapsingHeader::new("Examples")
            .default_open(false)
            .show(ui, |ui| {
                ui.scope(|ui|{
                    // force the text to wrap
                    ui.set_max_width(230.);
                    ui.label("Here's some cool fractals I found while developing this project:");
                });

                const NR_COL: usize = 3;
                Grid::new("included_fractals")
                    // we put two fractals per column to save space
                    .num_columns(NR_COL)
                    .show(ui, |ui| {
                        for (i,(name, code)) in INCLUDED_FRACTALS.iter().enumerate() {
                            if ui.selectable_label(false, *name).clicked() {
                                match Fractal::from_code(code) {
                                    Ok(new_fractal) => {
                                        *fractal = new_fractal;
                                        self.last_error = None;
                                    }
                                    Err(e) => self.last_error = Some(e),
                                }
                            }
                            if i % NR_COL == NR_COL - 1 {
                                ui.end_row();
                            }
                        }
                    });
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
            TextEdit::singleline(&mut self.add_text)
                .hint_text("Name")
                .desired_width(130.)
                .show(ui);

            if ui.add_enabled(!self.add_text.is_empty(), Button::new("Add")).clicked() {
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

        dismissible_error(ui, &mut self.last_error);
    }
}