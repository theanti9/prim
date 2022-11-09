use glam::Vec2;
use wgpu::{include_wgsl, util::DeviceExt, BufferUsages};

use crate::{
    camera::Camera2D,
    instance::{Inst, Instance2D},
    jump_flood::{num_passes, JumpFloodParams, MAX_JUMP_FLOOD_PASSES},
    shape::Shape2DVertex,
    vertex::Vertex,
};

pub(crate) struct PrimShaderModules {
    pub emitter_occluder_shader: wgpu::ShaderModule,
    pub jump_seed_shader: wgpu::ShaderModule,
    pub jump_flood_shader: wgpu::ShaderModule,
    pub distance_field_shader: wgpu::ShaderModule,
    pub raymarch_shader: wgpu::ShaderModule,
}
pub(crate) struct PrimBindGroupLayouts {
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub jump_seed_bind_group_layout: wgpu::BindGroupLayout,
    pub jump_flood_bind_group_layout: wgpu::BindGroupLayout,
    pub distance_field_bind_group_layout: wgpu::BindGroupLayout,
    pub raymarch_bind_group_layout: wgpu::BindGroupLayout,
}

pub(crate) struct PrimPipelines {
    pub emitter_occluder_pipeline: wgpu::RenderPipeline,
    pub jump_seed_pipeline: wgpu::RenderPipeline,
    pub jump_flood_pipeline: wgpu::RenderPipeline,
    pub distance_field_pipeline: wgpu::RenderPipeline,
    pub raymarch_pipeline: wgpu::RenderPipeline,
}

pub(crate) struct PrimTargets {
    pub emitter_occluder_target: wgpu::TextureView,
    pub jump_seed_target: wgpu::TextureView,
    pub jump_flood_targets: Vec<wgpu::TextureView>,
    pub distance_field_target: wgpu::TextureView,
}

pub(crate) struct PrimBuffers {
    pub camera_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub quad_buffer: wgpu::Buffer,
    pub jump_flood_params_buffer: wgpu::Buffer,
    pub time_buffer: wgpu::Buffer,
}

pub(crate) struct PrimBindGroups {
    pub camera_bind_group: wgpu::BindGroup,
    pub jump_seed_bind_group: wgpu::BindGroup,
    pub jump_flood_bind_groups: Vec<wgpu::BindGroup>,
    pub distance_field_bind_group: wgpu::BindGroup,
    pub raymarch_bind_group: wgpu::BindGroup,
}

impl PrimShaderModules {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            emitter_occluder_shader: device
                .create_shader_module(include_wgsl!("EmitterOccluder.wgsl")),
            jump_seed_shader: device.create_shader_module(include_wgsl!("JumpSeed.wgsl")),
            jump_flood_shader: device.create_shader_module(include_wgsl!("JumpFlood.wgsl")),
            distance_field_shader: device.create_shader_module(include_wgsl!("DistanceField.wgsl")),
            raymarch_shader: device.create_shader_module(include_wgsl!("RayMarch.wgsl")),
        }
    }
}

impl PrimBindGroupLayouts {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera_bind_group_layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                },
            ),
            jump_seed_bind_group_layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("jump_seed_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                },
            ),
            jump_flood_bind_group_layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("jump_flood_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: true,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    JumpFloodParams,
                                >(
                                )
                                    as _),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                },
            ),
            distance_field_bind_group_layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("distance_field_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                },
            ),
            raymarch_bind_group_layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("raymarch_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                },
            ),
        }
    }
}

impl PrimPipelines {
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &PrimBindGroupLayouts,
        shaders: &PrimShaderModules,
    ) -> Self {
        let emitter_occluder_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Emitter Occluder Pipeline Layout"),
                bind_group_layouts: &[&layouts.camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let emitter_occluder_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Emitter Occluder Pipeline"),
                layout: Some(&emitter_occluder_layout),
                vertex: wgpu::VertexState {
                    module: &shaders.emitter_occluder_shader,
                    entry_point: "vs_main",
                    buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders.emitter_occluder_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let jump_seed_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Jump Seed Pipeline Layout"),
            bind_group_layouts: &[
                &layouts.camera_bind_group_layout,
                &layouts.jump_seed_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let jump_seed_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Jump Seed Pipeline"),
            layout: Some(&jump_seed_layout),
            vertex: wgpu::VertexState {
                module: &shaders.jump_seed_shader,
                entry_point: "vs_main",
                buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shaders.jump_seed_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let jump_flood_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Jump Flood Pipeline Layout"),
            bind_group_layouts: &[
                &layouts.camera_bind_group_layout,
                &layouts.jump_flood_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let jump_flood_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Jump Flood Pipeline"),
            layout: Some(&jump_flood_layout),
            vertex: wgpu::VertexState {
                module: &shaders.jump_flood_shader,
                entry_point: "vs_main",
                buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shaders.jump_flood_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let distance_field_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Distance Field Pipeline Layout"),
                bind_group_layouts: &[
                    &layouts.camera_bind_group_layout,
                    &layouts.distance_field_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let distance_field_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Distance Field Pipeline"),
                layout: Some(&distance_field_layout),
                vertex: wgpu::VertexState {
                    module: &shaders.distance_field_shader,
                    entry_point: "vs_main",
                    buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders.distance_field_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let raymarch_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raymarch Pipeline Layout"),
            bind_group_layouts: &[
                &layouts.camera_bind_group_layout,
                &layouts.raymarch_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let raymarch_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raymarch Pipeline"),
            layout: Some(&raymarch_layout),
            vertex: wgpu::VertexState {
                module: &shaders.raymarch_shader,
                entry_point: "vs_main",
                buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shaders.raymarch_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            emitter_occluder_pipeline,
            jump_seed_pipeline,
            jump_flood_pipeline,
            distance_field_pipeline,
            raymarch_pipeline,
        }
    }
}

impl PrimTargets {
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let max_dimension = width.max(height);
        let size_squared = wgpu::Extent3d {
            // width: max_dimension / 2,
            width: max_dimension,
            // height: max_dimension / 2,
            height: max_dimension,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };

        let square_texture_descriptor = wgpu::TextureDescriptor {
            size: size_squared,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };

        let passes = num_passes(config) as u32;
        let mut pass_targets = Vec::with_capacity(passes as usize);

        for i in 0..passes {
            pass_targets.push(
                device
                    .create_texture(&square_texture_descriptor)
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some(format!("Jump Flood Pass {}", i).as_str()),
                        ..Default::default()
                    }),
            )
        }

        Self {
            emitter_occluder_target: device
                .create_texture(&square_texture_descriptor)
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Emitter Occluder Target View"),
                    ..Default::default()
                }),
            jump_seed_target: device
                .create_texture(&square_texture_descriptor)
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Jump Seed Target View"),
                    ..Default::default()
                }),
            jump_flood_targets: pass_targets,
            distance_field_target: device
                .create_texture(&square_texture_descriptor)
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Distance Field Target View"),
                    ..Default::default()
                }),
        }
    }
}

impl PrimBuffers {
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera: &Camera2D,
    ) -> Self {
        let quad = Instance2D {
            scale: Vec2::new(config.width as f32, config.height as f32),
            shape: 2,
            ..Default::default()
        };

        let params_size = device
            .limits()
            .min_uniform_buffer_offset_alignment
            .max(std::mem::size_of::<JumpFloodParams>() as u32);

        Self {
            camera_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera.get_view(config.width, config.height)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            instance_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (std::mem::size_of::<Inst>() * 100_000) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            quad_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Buffer"),
                contents: bytemuck::cast_slice(&[quad.to_matrix()]),
                usage: BufferUsages::all(),
            }),
            jump_flood_params_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Jump Flood Params Buffer"),
                size: (params_size as usize * MAX_JUMP_FLOOD_PASSES) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            time_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Time Buffer"),
                size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }
}

impl PrimBindGroups {
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &PrimBindGroupLayouts,
        targets: &PrimTargets,
        buffers: &PrimBuffers,
    ) -> Self {
        let passes = num_passes(config) as usize;
        let mut pass_bind_groups = Vec::with_capacity(passes);
        for i in 0..passes {
            let input_tex = if i == 0 {
                &targets.jump_seed_target
            } else {
                &targets.jump_flood_targets[i - 1]
            };

            pass_bind_groups.push(
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(format!("Jump Flood Pass {} Bind Group", i).as_str()),
                    layout: &layouts.jump_flood_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &buffers.jump_flood_params_buffer,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<JumpFloodParams>() as _
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                                &wgpu::SamplerDescriptor {
                                    label: Some("Jump Flood Sampler"),
                                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                                    mag_filter: wgpu::FilterMode::Linear,
                                    min_filter: wgpu::FilterMode::Linear,
                                    mipmap_filter: wgpu::FilterMode::Linear,
                                    ..Default::default()
                                },
                            )),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(input_tex),
                        },
                    ],
                }),
            );
        }

        Self {
            camera_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &layouts.camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.camera_buffer.as_entire_binding(),
                }],
            }),
            jump_seed_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Jump Seed Bind Group"),
                layout: &layouts.jump_seed_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                            &wgpu::SamplerDescriptor {
                                label: Some("Jump Seed Sampler"),
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Linear,
                                ..Default::default()
                            },
                        )),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &targets.emitter_occluder_target,
                        ),
                    },
                ],
            }),
            jump_flood_bind_groups: pass_bind_groups,
            distance_field_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Distance Field Bind Group"),
                layout: &layouts.distance_field_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                            &wgpu::SamplerDescriptor {
                                label: Some("Jump Seed Sampler"),
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Nearest,
                                ..Default::default()
                            },
                        )),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &targets.jump_flood_targets[&targets.jump_flood_targets.len() - 1],
                        ),
                    },
                ],
            }),
            raymarch_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Raymarch Bind Group"),
                layout: &layouts.raymarch_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                            &wgpu::SamplerDescriptor {
                                label: Some("Jump Seed Sampler"),
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Linear,
                                ..Default::default()
                            },
                        )),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &targets.distance_field_target,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &targets.emitter_occluder_target,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: buffers.time_buffer.as_entire_binding(),
                    },
                ],
            }),
        }
    }
}
