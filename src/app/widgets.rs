use std::ops::RangeInclusive;
use eframe::egui::{Button, DragValue, Response, Ui, Widget, WidgetText};
use crate::math::C32;

pub fn c32_ui(ui: &mut Ui, v: &mut C32, speed: Option<f32>, clamp_range: Option<RangeInclusive<f32>>) {
    ui.horizontal(|ui| {
        let mut x = DragValue::new(&mut v.re);
        let mut y = DragValue::new(&mut v.im);
        y = y.suffix("i");
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
pub fn c32_ui_full(ui: &mut Ui, label: impl Into<WidgetText>, v: &mut C32, speed: Option<f32>, clamp_range: Option<RangeInclusive<f32>>) -> Response {
    ui.label(label);
    c32_ui(ui, v, speed, clamp_range);
    Button::new("🖱").small().ui(ui)
}

pub fn option_checkbox<T>(ui: &mut Ui, value: &mut Option<T>, label: impl Into<WidgetText>, default_if_some: impl FnOnce() -> T) {
    let mut checked = value.is_some();
    ui.checkbox(&mut checked, label);
    if checked {
        if value.is_none() {
            *value = Some(default_if_some());
        }
    } else {
        *value = None;
    }
}