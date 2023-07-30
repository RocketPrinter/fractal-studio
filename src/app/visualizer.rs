use std::collections::{HashMap};
use std::sync::{Arc, OnceLock};
use bytemuck::bytes_of;
use eframe::egui::{Align2, Key, PaintCallback, Pos2, Sense, TextStyle, Ui, Vec2, vec2};
use eframe::egui_wgpu::CallbackFn;
use eframe::wgpu::{ColorTargetState, ColorWrites, Device, Features, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, QuerySetDescriptor, QueryType, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, VertexState};
use lazy_static::lazy_static;
use type_map::concurrent::Entry::Vacant;
use crate::app::settings::{Kind, KindDiscriminants, Settings};
use crate::wgsl::{SHADERS};

#[derive(Debug, Clone)]
pub struct Visualizer {
    scale: f32,
    offset: Vec2,
}

pub struct RenderData {
    pipelines: HashMap<KindDiscriminants,RenderPipeline>,
}

const ZOOM_FACTOR: f32 = 0.001;
const DRAG_FACTOR: f32 = 0.003;
//const WASD_FACTOR: f32 = 0.01;
const TEX_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;
pub const FRAGMENT_PUSH_CONSTANTS_SIZE: u32 = 16;

lazy_static!{
    static ref PUSH_CONSTANTS_SUPPORTED: OnceLock<bool> = OnceLock::default();
}

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
        if PUSH_CONSTANTS_SUPPORTED.get() == Some(&false) {
            ui.colored_label(ui.style().visuals.error_fg_color, "Push constants are not supported by the device");
            return;
        };

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // changing zoom and offset
        if response.hovered() {
            ui.input(|input| {
                self.scale *= 1. + input.scroll_delta.y * ZOOM_FACTOR;
                self.scale = self.scale.clamp(0.0001, 100000000.); // prevent zoom from becoming 0 or inf
            });
        }
        self.offset += response.drag_delta() * vec2(-1.,1.) * DRAG_FACTOR;

        // packing
        let aspect_ratio = painter.clip_rect().aspect_ratio();
        let packed_constants = [
            (self.scale * aspect_ratio).to_ne_bytes(), self.scale.to_ne_bytes(), // scale w/ aspect ratio correction
            self.offset.x.to_ne_bytes(), self.offset.y.to_ne_bytes() // offset
        ];

        // rendering
        let kind_discriminant: KindDiscriminants = (&settings.kind).into();
        let fragment_push_constants = settings.kind.push_constants();
        painter.add(PaintCallback {
            rect: painter.clip_rect(),
            callback: Arc::new(CallbackFn::default()
                // as the expose-ids feature on wgpu is not activated, we'll just have to assume that the device remains constant
                .prepare(move |device, _queue, _encoder, type_map| {
                    if let Vacant(e) = type_map.entry::<RenderData>() {
                        if let Some(render_data) = RenderData::new(device) {
                            e.insert(render_data);
                        }
                    }
                    vec![]
                })
                .paint(move |_info, pass, type_map| {
                    let Some(render_data) = type_map.get::<RenderData>() else {return};

                    pass.set_pipeline(render_data.pipelines.get(&kind_discriminant).unwrap());

                    pass.set_push_constants(ShaderStages::VERTEX, 0, bytes_of(&packed_constants));
                    if let Some(fragment_push_constants) = fragment_push_constants {
                        pass.set_push_constants(ShaderStages::FRAGMENT, 16, bytes_of(&fragment_push_constants));
                    }
                    
                    // vertex coordinates are hardcoded in the shader so a vertex buffer is not needed
                    pass.draw(0..6, 0..1);
                })
            ),
        });
        
        // overlay text
        painter.text(painter.clip_rect().left_bottom(),
                     Align2::LEFT_BOTTOM, format!("{self:?}"),
                     ui.style().text_styles.get(&TextStyle::Body).cloned().unwrap_or_default(),
                     ui.style().visuals.strong_text_color());
    }
}

impl RenderData {
    pub fn new(device: &Device) -> Option<Self> {
        if !(*PUSH_CONSTANTS_SUPPORTED.get_or_init(|| device.features().intersects(Features::PUSH_CONSTANTS))) {
            return None;
        }

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
                    range: 16..(16+ FRAGMENT_PUSH_CONSTANTS_SIZE),
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

        Some(Self {
            pipelines,
        })
    }
}