use std::collections::{HashMap};
use std::sync::{Arc};
use bytemuck::bytes_of;
use eframe::egui::{Align2, PaintCallback, Sense, TextStyle, Ui, Vec2, vec2};
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
    pub fn ui(&mut self, settings: &Settings, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // changing zoom and offset
        self.offset += response.drag_delta() * vec2(-1.,1.) * DRAG_FACTOR;
        if let Some(hover_pos) = response.hover_pos() {
            ui.input(|input| {
                let mut new_scale = self.scale * (1. + input.scroll_delta.y * ZOOM_FACTOR);
                new_scale = new_scale.clamp(0.0000000001, 100000000.); // prevent zoom from becoming 0 or inf
                let delta_scale = self.scale / new_scale;
                // rescale to make zooming centered on the screen
                self.offset *= delta_scale;
                // calculate an offset to the offset that will center the zooming on the cursor
                let mut cursor_scale = 2. * (hover_pos-painter.clip_rect().min) / painter.clip_rect().size() - vec2(1.,1.);
                cursor_scale.y *= -1.;
                self.offset += cursor_scale * (delta_scale-1.);

                self.scale = new_scale;
            });
        }

        // packing
        let aspect_ratio = painter.clip_rect().aspect_ratio();
        let mut packed_constants = [0u8;16];
        packed_constants[0.. 4].copy_from_slice(&(self.scale * aspect_ratio).to_ne_bytes());
        packed_constants[4.. 8].copy_from_slice(&self.scale.to_ne_bytes());
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

        // overlay text
        painter.debug_text(painter.clip_rect().left_bottom(),
                           Align2::LEFT_BOTTOM,
                           ui.style().visuals.strong_text_color(),
                           format!("{self:?}"));
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