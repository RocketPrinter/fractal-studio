use std::collections::{HashMap};
use std::sync::{Arc, OnceLock};
use eframe::egui::{PaintCallback, Sense, Ui};
use eframe::egui_wgpu::CallbackFn;
use eframe::wgpu::{ColorTargetState, ColorWrites, Device, Features, FragmentState, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, PushConstantRange, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, VertexState};
use lazy_static::lazy_static;
use type_map::concurrent::Entry::Vacant;
use crate::app::settings::{Kind, Settings};
use crate::wgsl::{SHADERS};

#[derive(Default)]
pub struct Visualizer {

}

pub struct RenderData {
    pipelines: HashMap<Kind,RenderPipeline>,
}

const TEX_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

lazy_static!{
    static ref PUSH_CONSTANTS_SUPPORTED: OnceLock<bool> = OnceLock::default();
}

impl Visualizer {
    pub fn ui(&mut self, settings: &Settings, ui: &mut Ui) {
        if PUSH_CONSTANTS_SUPPORTED.get() == Some(&false) {
            ui.colored_label(ui.style().visuals.error_fg_color, "Push constants are not supported by the device");
            return;
        };
        let (_response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());

        let kind = settings.kind;

        ui.painter().add(PaintCallback {
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

                    pass.set_pipeline(render_data.pipelines.get(&kind).unwrap());
                    pass.draw(0..3, 0..1);
                })
            ),
        });
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
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::FRAGMENT,
                range: 0..16, // 16 bytes, for now
            }],
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