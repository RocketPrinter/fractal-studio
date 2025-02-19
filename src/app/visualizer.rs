use bytemuck::bytes_of;
use eframe::egui::{vec2, Align, Align2, Button, Layout, Sense, Ui, UiBuilder, Vec2, Widget};
use eframe::egui_wgpu::Callback;
use encase::UniformBuffer;
use crate::app::settings::Settings;
use crate::app::widgets::get_transparent_button_fill;
use crate::fractal::FractalTrait;

use super::rendering::{RendererCallback, MAIN_UNIFORM_BUFFER_SIZE};
// todo: reset zoom and offset when changing fractal
#[derive(Debug, Clone)]
pub struct Visualizer {
    scale: f32,
    offset: Vec2,
}

const ZOOM_FACTOR: f32 = -0.001;

impl Default for Visualizer {
    fn default() -> Self {
        Self { scale: 1., offset: Vec2::ZERO, }
    }
}

impl Visualizer {
    pub fn ui(&mut self, settings: &mut Settings, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let aspect_ratio_correction = Vec2::new(painter.clip_rect().aspect_ratio(), 1.);

        // changing zoom and offset
        // todo: refactor
        let mut cursor_shader_space: Option<Vec2> = None;
        self.offset += response.drag_delta() / painter.clip_rect().size() * vec2(-1.,1.) * 2.0;
        if let Some(hover_pos) = response.hover_pos() {
            ui.input(|input| {
                // from -1 to 1
                let mut cursor_clip_space = 2. * (hover_pos-painter.clip_rect().min) / painter.clip_rect().size() - vec2(1., 1.);
                cursor_clip_space.y *= -1.;

                let zoom = input
                    .multi_touch()
                    .map(|mt| 1. / mt.zoom_delta)
                    .unwrap_or_else(|| {
                        1. + input.smooth_scroll_delta.y * ZOOM_FACTOR
                    });

                let mut new_scale = self.scale * zoom;
                new_scale = new_scale.clamp(0.0000000001, 10000.); // prevent zoom from becoming 0 or inf
                let delta_scale = self.scale / new_scale;
                // rescale to make zooming centered on the screen
                self.offset *= delta_scale;
                // calculate an offset to the offset that will center the zooming on the cursor
                self.offset += cursor_clip_space * (delta_scale-1.);

                self.scale = new_scale;

                cursor_shader_space = Some((cursor_clip_space + self.offset) * self.scale * aspect_ratio_correction);
            });
        }

        // preparing data for writing to the uniform buffer
        let mut buffer = [0u8; MAIN_UNIFORM_BUFFER_SIZE];
        buffer[0.. 8].copy_from_slice(bytes_of(&(self.scale * aspect_ratio_correction)));
        buffer[8..16].copy_from_slice(bytes_of(&self.offset));
        let settings_buffer = UniformBuffer::new(&mut buffer[16..]);
        settings.fractal.fill_uniform_buffer(settings_buffer);

        // rendering
        let callback = RendererCallback {
            shader_code: settings.fractal.get_shader(),
            main_data: buffer,
        };

        painter.add(Callback::new_paint_callback(painter.clip_rect(), callback));

        // fractals can draw extra stuff
        settings.fractal.draw_extra(&painter, cursor_shader_space);

        if settings.debug_label {
            painter.debug_text(painter.clip_rect().left_bottom(),
                               Align2::LEFT_BOTTOM,
                               ui.style().visuals.strong_text_color(),
                               format!("scale:{}, offset:{:?}, cursor:{cursor_shader_space:?}", self.scale, self.offset));
        }

        // position reset button, we have to do some funky stuff to get it to the right place
        ui.allocate_new_ui(UiBuilder::new().max_rect(ui.max_rect().shrink(5.)), |ui| {
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                if !settings.hide && Button::new("🏠").fill(get_transparent_button_fill(ui.visuals(), 0.7)).ui(ui).clicked() {
                    self.scale = 1.;
                    self.offset = Vec2::ZERO;
                }
            });
        });
    }
}
