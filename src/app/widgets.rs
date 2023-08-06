use std::ops::RangeInclusive;
use eframe::egui::{Button, DragValue, Ui, Vec2, Widget, WidgetText};

pub fn vec2_ui(ui: &mut Ui, v: &mut Vec2, complex: bool, speed: Option<f32>, clamp_range: Option<RangeInclusive<f32>>) {
    ui.horizontal(|ui| {
        let mut x = DragValue::new(&mut v.x);
        let mut y = DragValue::new(&mut v.y);
        if complex {
            y = y.suffix("i");
        } else {
            x = x.prefix("x=");
            y = y.prefix("y=");
        }
        if let Some(speed) = speed {
            x = x.speed(speed);
            y = y.speed(speed);
        }
        if let Some(clamp_range) = clamp_range {
            x = x.clamp_range(clamp_range.clone());
            y = y.clamp_range(clamp_range);
        }
        x.ui(ui);
        ui.add_space(-6.);
        y.ui(ui);
    });
}

/// also has a label and a button for enabling picking
pub fn vec2_ui_full(ui: &mut Ui, label: impl Into<WidgetText>, v: &mut Vec2,  complex: bool, speed: Option<f32>, clamp_range: Option<RangeInclusive<f32>>) -> bool {
    ui.label(label);
    vec2_ui(ui, v, complex, speed, clamp_range);
    Button::new("🖱").small().ui(ui).clicked()
}