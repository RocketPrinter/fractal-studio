use eframe::egui::{Button, Grid, Separator, TextEdit, Ui, Widget};
use egui_notify::Toasts;
use crate::app::widgets::error_toast;
use crate::fractal::Fractal;
use crate::evenly_spaced_out;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Library {
    pub user_fractals: Vec<(String, String)>,
    #[serde(skip)]
    pub tab: Tab,
    #[serde(skip)]
    add_text: String,
}

const EXAMPLE_FRACTALS: &[(&str, &str)] = &[
    // madelbrot
    ("Starfish", "gbBNYW5kZWxicm90RmFtaWx5hKppdGVyYXRpb25zZKd2YXJpYW50qk1hbmRlbGJyb3SnanVsaWFfY5LKO4bWMMo_DtV6p211bHRpX2XKQKAAAA"),
    ("Sun", "gbBNYW5kZWxicm90RmFtaWx5hKppdGVyYXRpb25zZKd2YXJpYW50qk1hbmRlbGJyb3SnanVsaWFfY5LKvdXPq8o_X1wpp211bHRpX2XKQVAAAA"),
    ("Shattered", "gbBNYW5kZWxicm90RmFtaWx5hKppdGVyYXRpb25zzMindmFyaWFudKpNYW5kZWxicm90p2p1bGlhX2OSyr69cKTKPSPXCqdtdWx0aV9lyj_gAAA"),
    ("Struts", "gbBNYW5kZWxicm90RmFtaWx5hKppdGVyYXRpb25zzQEsp3ZhcmlhbnSrQnVybmluZ1NoaXCnanVsaWFfY5LKv3sx8Mq_Xjv5p211bHRpX2XA"),
    ("Trees", "gbBNYW5kZWxicm90RmFtaWx5hKppdGVyYXRpb25zzJandmFyaWFudKtCdXJuaW5nU2hpcKdqdWxpYV9jkso_I9cKyj9--dunbXVsdGlfZcrAIAAA"),
    ("Hypersphere", "gbBNYW5kZWxicm90RmFtaWx5hKppdGVyYXRpb25zzQEsp3ZhcmlhbnSrQnVybmluZ1NoaXCnanVsaWFfY5LKv6AAAMo_oAAAp211bHRpX2XKv4AAAA"),
    // newton
    ("Decorations","gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5WSyr_zGPzKPzOcD5LKvwAAAMo_XbItksq_AAAAyr9dsi2Syj8sVtbKvy9B8pLKv0px3sq_EQyzsXBpY2tfdXNpbmdfY3Vyc29ywKFhkso_TMzNyj-AAAChY5LKvczMzcoAAAAAqXRocmVzaG9sZMp_gAAA"),
    ("Helix","gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5WSyr_zGPzKPzOcD5LKvwAAAMo_XbItksq_AAAAyr9dsi2Syj8sVtbKvy9B8pLKv0px3sq_EQyzsXBpY2tfdXNpbmdfY3Vyc29ywKFhkso_TMzNyr-AAAChY5LKvczMzcoAAAAAqXRocmVzaG9sZMp_gAAA"),
    ("Swirls", "gadOZXd0b25zhqppdGVyYXRpb25zZKVyb290c5OSyj-AAADKAAAAAJLKvwAAAMo_XbItksq_AAAAyr9dsi2xcGlja191c2luZ19jdXJzb3LAoWGSyj3sv7HKvszMzaFjksoAAAAAygAAAACpdGhyZXNob2xkyn-AAAA"),
    ("Spiral blobs", "gadOZXd0b25zhqppdGVyYXRpb25zMqVyb290c5OSyj-AAADKAAAAAJLKvwAAAMo_XbItksq_AAAAyr9dsi2xcGlja191c2luZ19jdXJzb3LAoWGSyj-AAADKP564UqFjksoAAAAAyj51wo-pdGhyZXNob2xkyj8mZmY"),
    // lyapunov
    ("Zircon City", "gahMeWFwdW5vdoOqaXRlcmF0aW9uc80BLKhzZXF1ZW5jZaxCQkJCQkJBQUFBQUGndmFyaWFudKtMb2dpc3RpY01hcA"),
];

#[derive(Debug, Default, Clone, Copy)]
pub enum Tab {
    Examples,
    #[default] User,
}

impl Library {
    pub fn ui(&mut self, ui: &mut Ui, fractal: &mut Fractal, toasts: &mut Toasts) {

        evenly_spaced_out!{ ui, horizontal,
            |ui| {
               if ui.selectable_label(matches!(self.tab, Tab::Examples), "Examples").clicked() {
                    self.tab = Tab::Examples;
                }
            },
            |ui| {
                if ui.selectable_label(matches!(self.tab, Tab::User), "Your fractals").clicked() {
                    self.tab = Tab::User;
                }
            },
        }

        // .grow makes it take a few extra pixels
        Separator::default().grow(200.).ui(ui);

        match self.tab {
            Tab::Examples => {

                const NR_COL: usize = 3;
                Grid::new("included_fractals")
                    // we put two fractals per column to save space
                    .num_columns(NR_COL)
                    .show(ui, |ui| {
                        for (i,(name, code)) in EXAMPLE_FRACTALS.iter().enumerate() {
                            if ui.selectable_label(false, *name).clicked() {
                                match Fractal::from_code(code) {
                                    Ok(new_fractal) => {
                                        *fractal = new_fractal;
                                        toasts.success("Loaded fractal");
                                    }
                                    Err(e) => {toasts.add(error_toast(e));},
                                }
                            }
                            if i % NR_COL == NR_COL - 1 {
                                ui.end_row();
                            }
                        }
                    });
            }
            Tab::User => {
                ui.small("Stored in local storage");
                Grid::new("user_fractals")
                    .num_columns(2)
                    .show(ui, |ui| {
                        let mut delete_action = None;
                        for (i,(name, code)) in self.user_fractals.iter().enumerate() {
                            if ui.selectable_label(false, name).clicked() {
                                match Fractal::from_code(code) {
                                    Ok(new_fractal) => {
                                        *fractal = new_fractal;
                                        toasts.success("Loaded fractal");
                                    }
                                    Err(e) => {toasts.add(error_toast(e));},
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

                    if ui.add_enabled(!self.add_text.is_empty(), Button::new("Add fractal")).clicked() {
                        match fractal.to_code() {
                            Ok(code) => {
                                self.user_fractals.push((
                                    std::mem::replace(&mut self.add_text, "Title".into()),
                                    code
                                ));
                                toasts.success("Saved fractal");
                            }
                            Err(e) => {toasts.add(error_toast(e));},
                        }
                    }
                });
            }
        }

        // setting the next width of the bar
        //ui.data_mut(|data|data.insert_temp(tab_bar_width_id, ui.min_rect().width()))
    }
}
