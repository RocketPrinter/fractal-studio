use eframe::{egui::ahash::HashMap, egui_wgpu::CallbackTrait};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState};

use crate::wgsl::Shader;

pub struct RenderData {
    main_uniform_buffer: Buffer,
    bind_group: BindGroup,
    pipeline_layout: PipelineLayout,
    pipelines: HashMap<Shader, RenderPipeline>,

    target_format: TextureFormat,
}

impl RenderData {
    pub fn new(device: &Device, target_format: TextureFormat) -> Self {

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Fractal bind group layout"),
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

        let main_uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Fractal main uniform"),
            size: MAIN_UNIFORM_BUFFER_SIZE as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Fractal bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(main_uniform_buffer.as_entire_buffer_binding()),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Fractal visualizer layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Self {
            main_uniform_buffer,
            bind_group,
            pipeline_layout,
            pipelines: HashMap::default(),
            target_format,
        }
    }

    fn ensure_pipeline_created(&mut self, device: &Device, shader_code: Shader) {
        let descriptor = shader_code.get_shader();
        let label=  format!("Pipeline visualizer {:?}", descriptor.label);
        let shader_module = device.create_shader_module(descriptor);

        self.pipelines.entry(shader_code).or_insert_with(||
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&label),
                layout: Some(&self.pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: None, // picks the default one
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: None,
                    targets: &[Some(ColorTargetState {
                        format: self.target_format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        );
    }
}

pub const MAIN_UNIFORM_BUFFER_SIZE: usize = 224;

pub struct RendererCallback {
    pub shader_code: Shader,
    pub main_data: [u8; MAIN_UNIFORM_BUFFER_SIZE],
}

impl CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let render_data = callback_resources.get_mut::<RenderData>().expect("Should be created and inserted when creating the app");
        render_data.ensure_pipeline_created(device, self.shader_code);
        queue.write_buffer(&render_data.main_uniform_buffer, 0, &self.main_data);
        vec![]
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let Some(render_data) = callback_resources.get::<RenderData>() else {return};

        pass.set_pipeline(render_data.pipelines.get(&self.shader_code).unwrap());
        pass.set_bind_group(0, &render_data.bind_group, &[]);

        // vertex coordinates are hardcoded in the shader so a vertex buffer is not needed
        pass.draw(0..6, 0..1);
    }
}
