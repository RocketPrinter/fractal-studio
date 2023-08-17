use std::collections::{HashMap};
use std::sync::{Arc};
use bytemuck::bytes_of;
use eframe::egui::{Align2, PaintCallback, Sense, Ui, Vec2, vec2};
use eframe::egui_wgpu::CallbackFn;
use eframe::wgpu::{ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState};
use encase::UniformBuffer;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages};
use crate::app::settings::{Settings};
use crate::fractal::{FractalDiscriminants, FractalTrait};
use crate::wgsl::SHADERS;

// todo: reset button for zoom and offset
#[derive(Debug, Clone)]
pub struct Visualizer {
    scale: f32,
    offset: Vec2,

    texture_format: TextureFormat,
}

pub struct RenderData {
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    pipelines: HashMap<FractalDiscriminants,RenderPipeline>,
}

pub const UNIFORM_BUFFER_SIZE: u64 = 144;

const ZOOM_FACTOR: f32 = -0.001;
const PINCH_FACTOR: f32 = -0.3;
const DRAG_FACTOR: f32 = 0.003;

impl Visualizer {
    pub fn new(texture_format: TextureFormat) -> Self {
        Self {
            scale: 1.,
            offset: Vec2::ZERO,
            texture_format,
        }
    }

    pub fn ui(&mut self, settings: &mut Settings, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let aspect_ratio_correction = Vec2::new(painter.clip_rect().aspect_ratio(), 1.);

        // changing zoom and offset
        // todo: refactor
        let mut cursor_shader_space: Option<Vec2> = None;
        self.offset += response.drag_delta() * vec2(-1.,1.) * DRAG_FACTOR;
        if let Some(hover_pos) = response.hover_pos() {
            ui.input(|input| {
                // from -1 to 1
                let mut cursor_clip_space = 2. * (hover_pos-painter.clip_rect().min) / painter.clip_rect().size() - vec2(1., 1.);
                cursor_clip_space.y *= -1.;

                let mut new_scale = self.scale * (1. + input.scroll_delta.y * ZOOM_FACTOR);
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
        let mut buffer = [0u8;UNIFORM_BUFFER_SIZE as usize];
        buffer[0.. 8].copy_from_slice(bytes_of(&(self.scale * aspect_ratio_correction)));
        buffer[8..16].copy_from_slice(bytes_of(&self.offset));
        let settings_buffer = UniformBuffer::new(&mut buffer[16..]);
        settings.fractal.fill_uniform_buffer(settings_buffer);

        // rendering
        let fractal_d = FractalDiscriminants::from(&settings.fractal);
        let texture_format = self.texture_format;
        painter.add(PaintCallback {
            rect: painter.clip_rect(),
            callback: Arc::new(CallbackFn::default()
                // as the expose-ids feature on wgpu is not activated, we'll just have to assume that the device remains constant
                .prepare(move |device, queue, _encoder, type_map| {
                    let data = type_map.entry::<RenderData>().or_insert_with(|| RenderData::new(device, texture_format));
                    queue.write_buffer(&data.uniform_buffer, 0, &buffer);
                    vec![]
                })
                .paint(move |_info, pass, type_map| {
                    let Some(render_data) = type_map.get::<RenderData>() else {return};

                    pass.set_pipeline(render_data.pipelines.get(&fractal_d).unwrap());
                    pass.set_bind_group(0, &render_data.bind_group, &[]);

                    // vertex coordinates are hardcoded in the shader so a vertex buffer is not needed
                    pass.draw(0..6, 0..1);
                })
            ),
        });

        settings.fractal.draw_extra(&painter, cursor_shader_space);

        if settings.debug_label {
            painter.debug_text(painter.clip_rect().left_bottom(),
                               Align2::LEFT_BOTTOM,
                               ui.style().visuals.strong_text_color(),
                               format!("scale:{}, offset:{:?}, cursor:{cursor_shader_space:?}", self.scale, self.offset));
        }
    }
}

impl RenderData {
    pub fn new(device: &Device, texture_format: TextureFormat) -> Self {

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Visualizer bind group layout"),
            entries: &[BindGroupLayoutEntry{
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Visualizer uniform buffer"),
            size: UNIFORM_BUFFER_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Visualizer bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Visualizer layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
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
                        format: texture_format,
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
            uniform_buffer,
            bind_group,
            pipelines,
        }
    }
}