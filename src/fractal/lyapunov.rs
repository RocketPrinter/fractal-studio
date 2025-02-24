use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use ecolor::{hex_color, Color32};
use eframe::egui::{ComboBox, DragValue, TextEdit, Ui, Widget};
use encase::{ShaderType, UniformBuffer};
use rand::{Rng, rng};
use glam::Vec4 as GVec4;
use crate::app::widgets::palette_editor;
use crate::fractal::FractalTrait;
use crate::wgsl::{LyapunovShader, Shader};

// todo: other functions?
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Lyapunov {
    iterations: u32,
    /// must only contain 'A', 'a', 'B'. 'b'; max length is 16
    sequence: String,
    variant: LyapunovShader,
    // todo: c: f32
    #[serde(default="default_palette")]
    colors: [Color32; 2],
}

#[derive(ShaderType)]
struct LyapunovUniform {
    stable_col: GVec4,
    unstable_col: GVec4,
    iterations: u32,
    // 1..=16
    seq_len: u32,
    // array packed in an integer, 0 is A and 1 is B
    sequence: u32,
}

impl Default for Lyapunov {
    fn default() -> Self {
        Self {
            iterations: 300,
            sequence: String::from("AB"),
            variant: LyapunovShader::LogisticMap,
            colors: default_palette(),
        }
    }
}

impl FractalTrait for Lyapunov {
    fn label(&mut self) ->  &'static str { "Lyapunov's Fractal" }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui|{
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).range(0..=3000).ui(ui);
        });

        ui.horizontal(|ui|{
            ui.label("Function");
            use LyapunovShader as LC;
            let variants = [LC::LogisticMap, LC::SinMap, LC::GaussMap, LC::Exponential, LC::CircleMap1, LC::CircleMap2];
            ComboBox::from_id_salt("variant selector")
                .selected_text(self.variant.to_string())
                .show_ui( ui, |ui| {
                    for variant in variants {
                        if ui.selectable_label(self.variant == variant, variant.to_string()).clicked() {
                            self.variant = variant;
                        }
                    }
                });
        });

        ui.label("Sequence (A and B only)");
        ui.horizontal(|ui|{
            TextEdit::singleline(&mut self.sequence).hint_text("AB")
                .desired_width(135.).char_limit(16).ui(ui);
            self.sequence.retain(|c| c == 'A' || c == 'B' || c == 'a' || c == 'b');
            if ui.button("🔁").clicked() {
                let mut rng = rng();
                let mut bits: u16 = rng.random();
                self.sequence =
                    (0..rng.random_range(2..=16)).map(|_| {
                        let b = bits & 1; bits >>= 1;
                        match b {
                        0 => 'A',
                        1 => 'B',
                        _ => unreachable!(),
                        }
                    }).collect();
            }
        });

        palette_editor(ui, &mut self.colors, "Colors", COLOR_PALETTES.as_slice());
    }

    fn get_shader(&self) -> Shader { Shader::Lyapunov(self.variant) }

    fn fill_uniform_buffer(&self, mut buffer: UniformBuffer<&mut [u8]>) {
        let (seq_len, sequence) = if self.sequence.is_empty() {
            (2u32, 0b10) // default AB sequence
        } else {
            (self.sequence.len() as u32,
                // packs the sequence into an u32 where 0 is A and 1 is B
             self.sequence.chars().enumerate().fold(0u32,|seq,(i,c)|{
                 seq | ( match c {
                     'A' | 'a' => 0b0,
                     'B' | 'b' => 0b1,
                        _ => unreachable!(),
                 } << i)
            }))
        };
        //println!("seq_len: {}, sequence: {:b}", seq_len, sequence);

        buffer.write(&LyapunovUniform {
            stable_col: self.colors[0].to_normalized_gamma_f32().into(),
            unstable_col: self.colors[1].to_normalized_gamma_f32().into(),
            iterations: self.iterations,
            seq_len,
            sequence,
        }).unwrap();
    }
}

impl Display for LyapunovShader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use LyapunovShader as LC;
        match self {
            LC::LogisticMap => write!(f, "Logistic map"),
            LC::SinMap      => write!(f, "Sine Map"),
            LC::GaussMap    => write!(f, "Gauss Map"),
            LC::Exponential => write!(f, "Exponential"),
            LC::CircleMap1  => write!(f, "Circle Map"),
            LC::CircleMap2  => write!(f, "Circle Map (alt)"),
        }
    }
}

fn default_palette() -> [Color32;2] { COLOR_PALETTES[0] }

pub static COLOR_PALETTES: LazyLock<Vec<[Color32;2]>> = LazyLock::new(|| vec![
    [hex_color!("FFC300"), hex_color!("0078FF")]
    // todo more palettes
]);
