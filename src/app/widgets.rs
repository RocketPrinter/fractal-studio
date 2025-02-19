use std::fmt::Display;
use std::ops::RangeInclusive;
use eframe::egui::{Button, Color32, DragValue, Response, Ui, Visuals, Widget, WidgetText};
use egui_notify::{Toast, ToastLevel};
use crate::math::C32;

pub fn c32_ui(ui: &mut Ui, v: &mut C32, speed: Option<f32>, range: Option<RangeInclusive<f32>>) {
    ui.horizontal(|ui| {
        let mut x = DragValue::new(&mut v.re);
        let mut y = DragValue::new(&mut v.im);
        y = y.suffix("i");
        if let Some(speed) = speed {
            x = x.speed(speed);
            y = y.speed(speed);
        }
        if let Some(range) = range {
            x = x.range(range.clone());
            y = y.range(range);
        }
        x.ui(ui);
        ui.add_space(-6.);
        y.ui(ui);
    });
}

/// also has a label and a button for enabling picking
pub fn c32_ui_full(ui: &mut Ui, label: impl Into<WidgetText>, v: &mut C32, speed: Option<f32>, range: Option<RangeInclusive<f32>>) -> Response {
    ui.label(label);
    c32_ui(ui, v, speed, range);
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

pub fn error_toast(err: impl Display) -> Toast {
    let mut toast = Toast::basic(format!("Error: {err}"));
    toast.duration(None).level(ToastLevel::Error);
    toast
}

pub fn get_transparent_button_fill(visuals: &Visuals, gamma_mul: f32) -> Color32 {
    visuals.widgets.noninteractive.weak_bg_fill.gamma_multiply(gamma_mul)
}

#[macro_export]
macro_rules! __count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::__count!($($xs)*));
}

/// ONE MACRO TO RULE ALL THE DUMB TRICKS
#[macro_export]
macro_rules! evenly_spaced_out {
    ($ui:ident, horizontal, $(|$item_ui:ident|$item: tt,)+) => {
        $ui.horizontal(|ui|{
            const SIZE: usize = $crate::__count!($($item_ui)*);

            let id = ui.id().with("_ultimate_centerer");

            let mut width_arr = ui.data(|data|data.get_temp::<[f32;SIZE]>(id).unwrap_or_default());
            let expected_space = (ui.available_width() - width_arr.iter().sum::<f32>()) / (SIZE + 1) as f32;

            let mut width_changed = false;
            // != 0 if the previous element has changed width
            let mut last_width_change = 0.;

            let mut i = 0;
            $(
                // if the previous item was longer or shorter than it needs to be we account for that so the rest of the items don't shift
                ui.add_space((expected_space -  last_width_change).max(0.));

                let new_width = ui.scope(|$item_ui| $item).response.rect.width();

                #[allow(unused_assignments)]{last_width_change = new_width - width_arr[i];}

                if width_arr[i] != new_width {
                    width_changed = true;
                    width_arr[i] = new_width;
                }

                #[allow(unused_assignments)]{i+=1;}
            )+

            ui.data_mut(|data|data.insert_temp(id, width_arr));
            if width_changed {
                ui.ctx().request_repaint();
            }
        });

    };
    ($ui:ident, vertical, $($item: tt,)+) => {
        // to implement when needed
    };
}
