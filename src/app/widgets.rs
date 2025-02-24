use std::fmt::Display;
use std::ops::RangeInclusive;
use eframe::egui::{CollapsingHeader, ComboBox, CornerRadius, Frame, Rect, Sense, UiBuilder, Vec2};
use eframe::egui::{color_picker::{self, Alpha}, Button, Color32, DragValue, Response, Ui, Visuals, Widget, WidgetText};
use eframe::epaint::RectShape;
use egui_notify::{Toast, ToastLevel};
use num_complex::Complex32;

pub fn c32_ui(ui: &mut Ui, v: &mut Complex32, speed: Option<f32>, range: Option<RangeInclusive<f32>>) {
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
pub fn c32_ui_full(ui: &mut Ui, label: impl Into<WidgetText>, v: &mut Complex32, speed: Option<f32>, range: Option<RangeInclusive<f32>>) -> Response {
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

pub fn palette_editor<const N: usize>(ui: &mut Ui, colors: &mut[Color32; N], label: impl Into<WidgetText>, dropdown_palettes: &[[Color32;N]]) {
    CollapsingHeader::new(label).show_unindented(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for c in colors.iter_mut() {
                color_picker::color_edit_button_srgba(ui, c, Alpha::Opaque);
            }
        });
        ComboBox::new("dropdown", "").selected_text("Pick a palette").show_ui(ui, |ui| {
            for palette in dropdown_palettes.iter() {
                let palette = &palette[0..colors.len()];

                // adapted from one of the examples
                let resp = ui.scope_builder(
                    UiBuilder::new().sense(Sense::click()),
                    |ui| {
                        let resp = ui.response();
                        let visuals = ui.style().interact(&resp);

                        Frame::canvas(ui.style())
                            .fill(visuals.bg_fill.gamma_multiply(0.3))
                            .stroke(visuals.bg_stroke)
                            .inner_margin(ui.spacing().menu_margin)
                            .show(ui, |ui| {
                                const RECT_SIZE: Vec2 = Vec2::new(30., 15.);
                                let (_, rect) = ui.allocate_space(Vec2::new(RECT_SIZE.x * palette.len() as f32, RECT_SIZE.y));
                                let painter = ui.painter();
                                for (i, c) in palette.iter().enumerate() {
                                    let shape_rect = Rect::from_min_size(rect.left_top() + Vec2::new(RECT_SIZE.x * i as f32, 0.), RECT_SIZE);

                                    let mut corner_radius = CornerRadius::ZERO;
                                    if i == 0 {
                                        corner_radius.nw = 5;
                                        corner_radius.sw = 5;
                                    }
                                    if i == palette.len() - 1 {
                                        corner_radius.ne = 5;
                                        corner_radius.se = 5;
                                    }

                                    let shape = RectShape::filled(shape_rect, corner_radius, *c);
                                    painter.add(shape);
                                }
                            });
                    }
                );

                if resp.response.clicked() {
                    colors.copy_from_slice(palette);
                }
            }
        });
    });
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
