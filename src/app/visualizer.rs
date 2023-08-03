use std::collections::{HashMap};
use std::sync::{Arc};
use bytemuck::bytes_of;
use eframe::egui::{Align2, PaintCallback, Sense, Ui, Vec2, vec2};
use eframe::egui_wgpu::CallbackFn;
use eframe::wgpu::{ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState};
use type_map::concurrent::Entry::Vacant;
use crate::app::settings::{Settings};
use crate::fractal::FractalDiscriminants;
use crate::wgsl::SHADERS;

#[derive(Debug, Clone)]
pub struct Visualizer {
    scale: f32,
    offset: Vec2,
}

pub struct RenderData {
    pipelines: HashMap<FractalDiscriminants,RenderPipeline>,
}

pub const FRAGMENT_PUSH_CONSTANTS_SIZE: usize = 16;

const ZOOM_FACTOR: f32 = 0.001;
const DRAG_FACTOR: f32 = 0.003;
//const WASD_FACTOR: f32 = 0.01;
const TEX_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

impl Default for Visualizer {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: Vec2::ZERO,
        }
    }
}

impl Visualizer {
    pub fn ui(&mut self, settings: &mut Settings, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let aspect_ratio_correction = Vec2::new(painter.clip_rect().aspect_ratio(), 1.);

        // changing zoom and offset
        let mut cursor_shader_space: Option<Vec2> = None;
        self.offset += response.drag_delta() * vec2(-1.,1.) * DRAG_FACTOR;
        if let Some(hover_pos) = response.hover_pos() {
            ui.input(|input| {
                // from -1 to 1
                let mut cursor_clip_space = 2. * (hover_pos-painter.clip_rect().min) / painter.clip_rect().size() - vec2(1., 1.);
                cursor_clip_space.y *= -1.;

                let mut new_scale = self.scale * (1. + input.scroll_delta.y * ZOOM_FACTOR);
                new_scale = new_scale.clamp(0.0000000001, 100.); // prevent zoom from becoming 0 or inf
                let delta_scale = self.scale / new_scale;
                // rescale to make zooming centered on the screen
                self.offset *= delta_scale;
                // calculate an offset to the offset that will center the zooming on the cursor
                self.offset += cursor_clip_space * (delta_scale-1.);

                self.scale = new_scale;

                cursor_shader_space = Some((cursor_clip_space + self.offset) * self.scale * aspect_ratio_correction);
                settings.fractal.cursor_shader_space(cursor_shader_space.unwrap());
            });
        }

        // packing
        let mut packed_constants = [0u8;16];
        packed_constants[0.. 8].copy_from_slice(bytes_of(&(self.scale * aspect_ratio_correction)));
        packed_constants[8..16].copy_from_slice(bytes_of(&self.offset));

        // rendering
        let fractal_d = FractalDiscriminants::from(&settings.fractal);
        let fragment_push_constants = settings.fractal.push_constants();
        painter.add(PaintCallback {
            rect: painter.clip_rect(),
            callback: Arc::new(CallbackFn::default()
                // as the expose-ids feature on wgpu is not activated, we'll just have to assume that the device remains constant
                .prepare(move |device, _queue, _encoder, type_map| {
                    if let Vacant(e) = type_map.entry::<RenderData>() {
                        e.insert(RenderData::new(device));
                    }
                    vec![]
                })
                .paint(move |_info, pass, type_map| {
                    let Some(render_data) = type_map.get::<RenderData>() else {return};

                    pass.set_pipeline(render_data.pipelines.get(&fractal_d).unwrap());

                    pass.set_push_constants(ShaderStages::VERTEX, 0, &packed_constants);
                    if let Some(fragment_push_constants) = fragment_push_constants {
                        pass.set_push_constants(ShaderStages::FRAGMENT, 16, bytes_of(&fragment_push_constants));
                    }

                    // vertex coordinates are hardcoded in the shader so a vertex buffer is not needed
                    pass.draw(0..6, 0..1);
                })
            ),
        });

        if settings.debug_label {
            painter.debug_text(painter.clip_rect().left_bottom(),
                               Align2::LEFT_BOTTOM,
                               ui.style().visuals.strong_text_color(),
                               format!("{self:?}, {cursor_shader_space:?}"));
        }
    }
}

impl RenderData {
    pub fn new(device: &Device) -> Self {

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Visualizer layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[
                PushConstantRange {
                    stages: ShaderStages::VERTEX,
                    range: 0..16, // zoom + offset
                },
                PushConstantRange {
                    stages: ShaderStages::FRAGMENT,
                    range: 16..(16+ FRAGMENT_PUSH_CONSTANTS_SIZE as u32),
                }
            ],
        });

        let pipelines = HashMap::from_iter(SHADERS.iter().map(|(kind, shader)| {
            let shader_module = device.create_shader_module(shader.to_owned());

            let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor{
                label: Some(&format!("Pipeline visualizer {kind:?}")),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vertex",
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: "fragment",
                    targets: &[Some(ColorTargetState{
                        format: TEX_FORMAT,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            });
            (kind.to_owned(), pipeline)
        }));

        Self {
            pipelines,
        }
    }
}