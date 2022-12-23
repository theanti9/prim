use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, Device, Extent3d, Face, FragmentState, FrontFace,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipelineDescriptor, ShaderStages, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureUsages, TextureViewDescriptor, VertexState,
};

use crate::{
    camera::Camera2D,
    instance::{Inst, Instance2D},
    shape::Shape2DVertex,
    vertex::Vertex,
};

pub(crate) struct PrimShaderModules {
    pub shape_shader_module: wgpu::ShaderModule,
}

pub(crate) struct PrimBindGroupLayouts {
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

pub(crate) struct PrimPipelines {
    pub shape_pipeline: wgpu::RenderPipeline,
}

pub(crate) struct PrimTargets {
    pub multisample_buffer: wgpu::TextureView,
}

pub(crate) struct PrimBuffers {
    pub camera_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    #[allow(unused)]
    pub time_buffer: wgpu::Buffer,
}

pub(crate) struct PrimBindGroups {
    pub camera_bind_group: wgpu::BindGroup,
}

impl PrimShaderModules {
    #[must_use]
    pub fn new(device: &Device) -> Self {
        Self {
            shape_shader_module: device.create_shader_module(include_wgsl!("shader2d.wgsl")),
        }
    }
}

impl PrimBindGroupLayouts {
    #[must_use]
    pub fn new(device: &Device) -> Self {
        Self {
            camera_bind_group_layout: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Prim Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        }
    }
}

impl PrimPipelines {
    #[must_use]
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        layouts: &PrimBindGroupLayouts,
        shaders: &PrimShaderModules,
        multisample_count: u32,
    ) -> Self {
        let shape_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Shape Pipeline Layout"),
            bind_group_layouts: &[&layouts.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shape_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&shape_pipeline_layout),
            vertex: VertexState {
                module: &shaders.shape_shader_module,
                entry_point: "vs_main",
                buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
            },
            fragment: Some(FragmentState {
                module: &shaders.shape_shader_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: multisample_count,
                ..Default::default()
            },
            multiview: None,
        });

        Self { shape_pipeline }
    }
}

impl PrimTargets {
    #[must_use]
    pub fn new(device: &Device, config: &SurfaceConfiguration, sample_count: u32) -> Self {
        let texture_extent = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let frame_descriptor = &TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
        };

        Self {
            multisample_buffer: device
                .create_texture(frame_descriptor)
                .create_view(&TextureViewDescriptor::default()),
        }
    }
}

impl PrimBuffers {
    #[must_use]
    pub fn new(
        device: &Device,
        #[allow(unused)] config: &SurfaceConfiguration,
        camera: &Camera2D,
    ) -> Self {
        Self {
            camera_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera.get_view()]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
            instance_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (std::mem::size_of::<Inst>() * 100_000) as BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            time_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Time Buffer"),
                size: std::mem::size_of::<f32>() as BufferAddress,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }
}

impl PrimBindGroups {
    #[must_use]
    pub fn new(
        device: &Device,
        #[allow(unused)] config: &SurfaceConfiguration,
        layouts: &PrimBindGroupLayouts,
        buffers: &PrimBuffers,
    ) -> Self {
        Self {
            camera_bind_group: device.create_bind_group(&BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &layouts.camera_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: buffers.camera_buffer.as_entire_binding(),
                }],
            }),
        }
    }
}
