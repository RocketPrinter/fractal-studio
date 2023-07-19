use std::collections::HashMap;
use std::sync::{Arc};
use eframe::egui::{Image, PaintCallback, Sense, TextureId, Ui, Widget};
use eframe::egui_wgpu::CallbackFn;
use eframe::wgpu::{Color, ColorTargetState, ColorWrites, Device, Extent3d, FragmentState, FrontFace, LoadOp, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, VertexState};
use crate::app::settings::{Kind};
use crate::wgsl::{TEST_SHADER, VERTEX_SHADER};

#[derive(Default)]
pub struct Visualizer {

}

pub struct RenderData {
    tex: (Texture, TextureView),
    pipeline_cache: PipelineCache,
}

pub struct PipelineCache {
    pipeline_layout: Arc<PipelineLayout>,
    pipelines: HashMap<Kind,RenderPipeline>,
}

const TEX_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

impl Visualizer {
    pub fn ui(&mut self, ui: &mut Ui) {
        let (_response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());

        let size = ui.clip_rect().size() * ui.ctx().pixels_per_point();
        let size = Extent3d {
            width: size.x as u32,
            height: size.y as u32,
            depth_or_array_layers: 1,
        };

        ui.painter().add(PaintCallback {
            rect: painter.clip_rect(),
            callback: Arc::new(CallbackFn::default()
                // todo: as the expose-ids feature on wgpu is not activated, we'll just have to assume that the device remains constant
                .prepare(move |device, _queue, encoder, type_map| {
                    let render_data = type_map.entry().or_insert_with(|| RenderData::new(device, size));
                    // the size of the element was changed so we realloc the texture
                    // todo: is this a good idea?
                    if render_data.tex.0.size() != size {
                        render_data.tex = create_tex(device, size);
                        println!("reallocated texture"); // todo: remove later
                    }

                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Visualizer pass"),
                        color_attachments: &[Some(RenderPassColorAttachment{
                            view: &render_data.tex.1,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear( Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    pass.set_pipeline(render_data.pipeline_cache.get_pipeline(device, Kind::Test));

                    // todo:

                    drop(pass);
                    vec![]
                })
                .paint(|_info,_pass,_typemap| {
                    ()
                })
            ),
        });
        let _response = Image::new(TextureId::Managed(0), ui.available_size()).sense(Sense::drag()).ui(ui);
    }
}

impl RenderData {
    pub fn new(device: &Device, size: Extent3d) -> Self {
        Self {
            tex: create_tex(device, size),
            pipeline_cache: PipelineCache::new(device),
        }
    }
}

impl PipelineCache {
    pub fn new(device: &Device) -> Self {
        Self {
            pipeline_layout: Arc::new(device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Visualizer layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })),
            pipelines: Default::default(),
        }
    }

    pub fn get_pipeline(&mut self, device: &Device, kind: Kind) -> &RenderPipeline  {
        self.pipelines.entry(kind).or_insert_with(||
            device.create_render_pipeline(&RenderPipelineDescriptor{
                label: None, // todo:
                layout: Some(&*self.pipeline_layout.clone()),
                vertex: VertexState {
                    module: &device.create_shader_module(VERTEX_SHADER.clone()),
                    entry_point: "vertex",
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &device.create_shader_module(TEST_SHADER.clone()), // todo
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
            })
        )
    }
}

fn create_tex(device: &Device, size: Extent3d) -> (Texture, TextureView) {
    let tex = device.create_texture(&TextureDescriptor{
        label: Some("Visualizer target tex"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TEX_FORMAT,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
        usage: TextureUsages::all() - TextureUsages::STORAGE_BINDING,
    });
    let view = tex.create_view(&Default::default());
    (tex, view)
}