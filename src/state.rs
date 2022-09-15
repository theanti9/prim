use glam::{Mat4, Vec2};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::Camera2D,
    instance::Instance2D,
    shape::{DrawShape2D, Shape2D, Shape2DVertex},
    vertex::Vertex,
};

const NUM_INSTANCES_PER_ROW: u32 = 250;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    camera2d: Camera2D,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    shape2d_instances: Vec<Instance2D>,
    shape2d_instances_data: Vec<Mat4>,
    instance_buffer: wgpu::Buffer,
    shape2d: Shape2D,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
        };
        surface.configure(&device, &config);

        let shader2d = device.create_shader_module(include_wgsl!("shader2d.wgsl"));

        let camera2d = Camera2D::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(size.width as f32 / 2.0, size.height as f32 / 2.0),
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera2d.get_view()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader2d,
                entry_point: "vs_main",
                buffers: &[Shape2DVertex::desc(), Instance2D::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader2d,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let shape2d_instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|y| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = Vec2::new(x as f32 * 100.0, y as f32 * 100.0);
                    let rotation = f32::to_radians(0.0);

                    Instance2D {
                        position,
                        rotation,
                        scale: Vec2::splat(100.0),
                    }
                })
            })
            .collect::<Vec<_>>();

        let shape2d_instances_data = shape2d_instances
            .iter()
            .map(Instance2D::to_matrix)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&shape2d_instances_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let shape2d = Shape2D::create_from_points(
            "Triangle".to_string(),
            Vec::from([
                Vec2::new(0.0, 0.5),
                Vec2::new(-0.5, -0.5),
                Vec2::new(0.5, -0.5),
            ]),
            Vec::from([0, 1, 2]),
            &device,
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera2d,
            camera_buffer,
            camera_bind_group,
            shape2d_instances,
            shape2d_instances_data,
            instance_buffer,
            shape2d,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        // let start = std::time::Instant::now();
        self.rotate_instances();

        // error!(
        //     "{}",
        //     std::time::Instant::now()
        //         .duration_since(start)
        //         .as_secs_f32()
        // );
        for i in 0..self.shape2d_instances.len() {
            self.shape2d_instances_data[i] = self.shape2d_instances[i].to_matrix();
        }

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.shape2d_instances_data),
        );
        self.camera2d.update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera2d.get_view()]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass
                .draw_shape2d_instanced(&self.shape2d, 0..self.shape2d_instances.len() as u32)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[inline(always)]
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn rotate_instances(&mut self) {
        for instance in &mut self.shape2d_instances {
            instance.rotation += f32::to_radians(0.01);
        }
    }
}
