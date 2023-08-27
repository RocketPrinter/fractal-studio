use eframe::egui::{DragValue, TextEdit, Ui, Widget};
use encase::{ShaderType, UniformBuffer};
use rand::{Rng, thread_rng};
use crate::fractal::FractalTrait;

// todo: other functions?
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Lyapunov {
    iterations: u32,
    /// must only contain 'A', 'a', 'B'. 'b'; max length is 16
    sequence: String,
}

#[derive(ShaderType)]
struct LyapunovUniform {
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
            sequence: String::from("AB")
        }
    }
}

impl FractalTrait for Lyapunov {
    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui|{
            ui.label("Iterations");
            DragValue::new(&mut self.iterations).speed(1).clamp_range(0..=3000).ui(ui);
        });

        ui.label("Sequence (A and B only)");
        ui.horizontal(|ui|{
            TextEdit::singleline(&mut self.sequence).hint_text("AB")
                .desired_width(135.).char_limit(16).ui(ui);
            self.sequence.retain(|c| c == 'A' || c == 'B' || c == 'a' || c == 'b');
            if ui.button("🔁").clicked() {
                let mut rng = thread_rng();
                let mut bits: u16 = rng.gen();
                self.sequence =
                    (0..rng.gen_range(2..=16)).map(|i| {
                        let b = bits & 1; bits >>= 1;
                        match b {
                        0 => 'A',
                        1 => 'B',
                        _ => unreachable!(),
                        }
                    }).collect();
            }
        });
    }

    fn explanation_ui(&mut self, ui: &mut Ui) {
        ui.label("todo"); // todo
    }

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
            iterations: self.iterations,
            seq_len,
            sequence,
        }).unwrap();
    }
}