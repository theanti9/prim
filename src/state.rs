use std::cell::RefCell;

use glam::{Vec2, Vec4};
use rand::{thread_rng, Rng};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::Camera2D,
    instance::{Inst, Instance2D},
    object_registry::{Component, ObjectRegistry},
    shape::{DrawShape2D, Shape2DVertex},
    shape_registry::ShapeRegistry,
    time::Time,
    vertex::Vertex,
};

const NUM_INSTANCES_PER_ROW: u32 = 100;

pub struct State {
    time: Time,
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
    shape2d_instances_data: Vec<Inst>,
    instance_buffer: wgpu::Buffer,
    shape_registry: ShapeRegistry,
    object_registry: RefCell<ObjectRegistry>,
}

impl State {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
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
        ))
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

        let object_registry = RefCell::new(ObjectRegistry::new());
        {
            let mut reg = object_registry.borrow_mut();
            let obj = reg.spawn_object();
            let spinners = (0..NUM_INSTANCES_PER_ROW)
                .flat_map(|y| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                        let position = Vec2::new(
                            (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0) * 40.0,
                            (y as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0) * 40.0,
                        );
                        let mut spinner = Spinner::new(position);
                        spinner.instances[0].scale = Vec2::splat(35.0);
                        spinner.instances[0].color = Vec4::new(
                            position.x / 50.0 / NUM_INSTANCES_PER_ROW as f32,
                            position.y / 50.0 / NUM_INSTANCES_PER_ROW as f32,
                            0.2,
                            1.0,
                        );
                        spinner.instances[0].shape = (x + y) % 2;
                        spinner
                    })
                })
                .collect::<Vec<_>>();

            for spinner in spinners {
                obj.add_component(spinner);
            }
        }

        let shape2d_instances = object_registry
            .borrow()
            .collect_renderables()
            .iter()
            .map(|i| **i)
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

        let time = Time::new();
        let mut shape_registry = ShapeRegistry::new();
        shape_registry.register_shape(
            "Triangle".to_string(),
            Vec::from([
                Vec2::new(0.0, 0.5),
                Vec2::new(-0.5, -0.5),
                Vec2::new(0.5, -0.5),
            ]),
            Vec::from([0, 1, 2]),
            &device,
        );
        shape_registry.register_shape(
            "Square".to_string(),
            Vec::from([
                Vec2::new(0.5, 0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::new(-0.5, -0.5),
                Vec2::new(0.5, -0.5),
            ]),
            Vec::from([0, 1, 2, 0, 2, 3]),
            &device,
        );

        Self {
            time,
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
            shape_registry,
            object_registry,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera2d
                .rescale(Vec2::new(new_size.width as f32, new_size.height as f32));
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn filter_visible_instances(&mut self) {
        self.shape2d_instances = self
            .shape2d_instances
            .iter()
            .filter_map(|inst| {
                if inst.position.x - inst.scale.x < self.camera2d.position.x + self.camera2d.scale.x
                    && inst.position.x + inst.scale.x
                        > self.camera2d.position.x - self.camera2d.scale.x
                    && inst.position.y - inst.scale.y
                        < self.camera2d.position.y + self.camera2d.scale.y
                    && inst.position.y + inst.scale.y
                        > self.camera2d.position.y - self.camera2d.scale.y
                {
                    Some(*inst)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn update(&mut self) {
        self.time.update();

        self.object_registry.borrow_mut().update(&self.time, &self);

        self.shape2d_instances = self
            .object_registry
            .borrow()
            .collect_renderables()
            .iter()
            .map(|i| **i)
            .collect::<Vec<_>>();
        self.filter_visible_instances();
        self.shape2d_instances_data = self
            .shape2d_instances
            .iter()
            .map(Instance2D::to_matrix)
            .collect::<Vec<_>>();

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

            if self.shape2d_instances.is_empty() {
                return Ok(());
            }
            let mut s = self.shape2d_instances.first().unwrap().shape;
            let mut start: usize = 0;

            let total_len = self.shape2d_instances.len();

            for i in 0..total_len {
                if self.shape2d_instances[i].shape == s && i != total_len - 1 {
                    continue;
                }

                let end = if i == total_len - 1 { total_len } else { i };

                render_pass.draw_shape2d_instanced(
                    self.shape_registry.get_shape(s),
                    start as u32..end as u32,
                );
                s = self.shape2d_instances[i].shape;
                start = i;
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[inline(always)]
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn register_shape(&mut self, name: String, points: Vec<Vec2>, indices: Vec<u32>) -> u32 {
        self.shape_registry
            .register_shape(name, points, indices, &self.device)
    }

    pub fn get_shape_id(&self, name: &str) -> Option<u32> {
        self.shape_registry.get_id(name)
    }

    pub fn rotate_instances(&mut self, time: Time) {
        for instance in &mut self.shape2d_instances {
            instance.rotation += time.delta_seconds();
        }
    }
}

pub struct Spinner {
    instances: Vec<Instance2D>,
    multiplier: f32,
}
impl Spinner {
    pub fn new(position: Vec2) -> Self {
        let mut rng = thread_rng();
        Self {
            instances: vec![Instance2D {
                position,
                rotation: 0.0,
                scale: Vec2::splat(35.0),
                color: Vec4::new(1.0, 0.5, 0.2, 1.0),
                shape: 0,
            }],
            multiplier: rng.gen_range(0.2..2.0),
        }
    }
}
impl Component for Spinner {
    fn update(&mut self, time: &Time, _state: &State) {
        self.instances[0].rotation += self.multiplier * time.delta_seconds();
    }

    fn get_renderables(&self) -> &Vec<Instance2D> {
        &self.instances
    }
}
