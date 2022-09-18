use bevy_ecs::{
    prelude::DetectChanges,
    query::Changed,
    schedule::{Stage, SystemStage},
    system::{Query, Res, ResMut},
};
use glam::Vec2;
use log::error;
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{
    event::{ElementState, KeyboardInput, WindowEvent},
    window::Window,
};

use crate::{
    camera::Camera2D,
    input::Keyboard,
    instance::{Inst, Instance2D},
    shape::{DrawShape2D, Shape2DVertex},
    shape_registry::ShapeRegistry,
    time::Time,
    vertex::Vertex,
};

pub struct State {
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    keyboard: Keyboard,
    world: bevy_ecs::world::World,
    schedule: bevy_ecs::schedule::Schedule,
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
        error!("Starting with backend: {:?}", adapter.get_info().backend);

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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<Inst>() * 10000) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let time = Time::new();
        let mut shape_registry = ShapeRegistry::new();
        shape_registry.register_builtin_shapes(&device);

        let keyboard = Keyboard::new();
        let mut world = bevy_ecs::world::World::default();

        let render_state = RenderState {
            surface,
            queue,
            camera_buffer,
            device,
            render_pipeline,
            camera_bind_group,
            instance_buffer,
        };
        world.insert_resource(camera2d);
        world.insert_resource(render_state);
        world.insert_resource(time);
        world.insert_resource(shape_registry);
        world.insert_resource(keyboard.clone());
        world.insert_resource(Renderables(Vec::with_capacity(1000)));
        world.insert_resource(RenderResult(Ok(())));
        world.insert_resource(FpsCounter::new());

        let mut schedule = bevy_ecs::schedule::Schedule::default();

        schedule.add_stage(
            "pre_update",
            SystemStage::parallel().with_system(update_time),
        );
        schedule.add_stage("update", SystemStage::parallel());
        schedule.add_stage(
            "post_update",
            SystemStage::parallel().with_system(update_camera),
        );
        schedule.add_stage(
            "collect",
            SystemStage::single_threaded()
                .with_system(sync_matrix)
                .with_system(collect_instances),
        );
        schedule.add_stage(
            "render",
            SystemStage::parallel().with_system(main_render_pass),
        );
        schedule.add_stage(
            "log",
            SystemStage::single_threaded().with_system(fps_counter),
        );

        Self {
            config,
            size,
            keyboard,
            world,
            schedule,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            if let Some(render_state) = self.world.get_resource_mut::<RenderState>() {
                render_state
                    .surface
                    .configure(&render_state.device, &self.config);
            }
            if let Some(mut camera2d) = self.world.get_resource_mut::<Camera2D>() {
                camera2d.rescale(Vec2::new(new_size.width as f32, new_size.height as f32));
            }
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => self.keyboard.pressed(*keycode),
                ElementState::Released => self.keyboard.released(*keycode),
            },
            _ => {}
        }
        false
    }

    pub fn update(&mut self) {
        if let Some(mut k) = self.world.get_resource_mut::<Keyboard>() {
            *k = self.keyboard.clone();
            self.keyboard.update();
        }
        self.schedule.run(&mut self.world);
    }

    #[inline(always)]
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    #[inline(always)]
    pub fn render_result(&self) -> Result<(), wgpu::SurfaceError> {
        if let Some(res) = self.world.get_resource::<RenderResult>() {
            res.0.clone()
        } else {
            Ok(())
        }
    }

    pub fn borrow_world(&mut self) -> &mut bevy_ecs::world::World {
        &mut self.world
    }

    pub fn borrow_schedule(&mut self) -> &mut bevy_ecs::schedule::Schedule {
        &mut self.schedule
    }

    // pub fn register_shape(&mut self, name: String, points: Vec<Vec2>, indices: Vec<u32>) -> u32 {
    //     if let Some(shape_registry) = self.world.get_resource_mut::<ShapeRegistry>() {

    //     }
    // }

    // #[inline(always)]
    // pub fn get_shape_id(&self, name: &str) -> Option<u32> {
    //     self.shape_registry.get_id(name)
    // }
}

fn update_time(mut time: ResMut<Time>) {
    time.update();
}

fn update_camera(mut camera2d: ResMut<Camera2D>) {
    if camera2d.is_changed() {
        camera2d.update();
    }
}
pub struct Renderables(Vec<(Instance2D, Inst)>);

fn sync_matrix(mut instances: Query<(&Instance2D, &mut Inst), Changed<Instance2D>>) {
    for (changed, mut inst) in &mut instances {
        *inst = changed.to_matrix();
    }
}

fn collect_instances(
    instance_query: Query<(&Instance2D, &mut Inst)>,
    mut renderables: ResMut<Renderables>,
    render_state: Res<RenderState>,
    camera2d: Res<Camera2D>,
) {
    renderables.0.clear();

    for (inst, render_inst) in &instance_query {
        if inst.position.x - inst.scale.x < camera2d.position.x + camera2d.scale.x
            && inst.position.x + inst.scale.x > camera2d.position.x - camera2d.scale.x
            && inst.position.y - inst.scale.y < camera2d.position.y + camera2d.scale.y
            && inst.position.y + inst.scale.y > camera2d.position.y - camera2d.scale.y
        {
            renderables.0.push((*inst, *render_inst));
        }
    }
    renderables.0.sort_by(|a, b| a.0.shape.cmp(&b.0.shape));
    let shape2d_instances_data = renderables.0.iter().map(|(_a, b)| *b).collect::<Vec<_>>();

    render_state.queue.write_buffer(
        &render_state.instance_buffer,
        0,
        bytemuck::cast_slice(&shape2d_instances_data),
    );
}

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub camera_buffer: wgpu::Buffer,
    pub device: wgpu::Device,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub instance_buffer: wgpu::Buffer,
}

fn main_render_pass(
    render_state: ResMut<RenderState>,
    shape_registry: Res<ShapeRegistry>,
    renderables: Res<Renderables>,
    camera2d: Res<Camera2D>,
    mut render_result: ResMut<RenderResult>,
) {
    if renderables.0.is_empty() {
        *render_result = RenderResult(Ok(()));
        return;
    }
    let output = match render_state.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(err) => {
            *render_result = RenderResult(Err(err));
            return;
        }
    };
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    render_state.queue.write_buffer(
        &render_state.camera_buffer,
        0,
        bytemuck::cast_slice(&[camera2d.get_view()]),
    );
    let mut encoder = render_state
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

        render_pass.set_pipeline(&render_state.render_pipeline);
        render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);

        let mut s = renderables.0.first().unwrap().0.shape;
        let mut start: usize = 0;

        let total_len = renderables.0.len();
        render_pass.set_vertex_buffer(1, render_state.instance_buffer.slice(..));
        for i in 0..total_len {
            if renderables.0[i].0.shape == s && i != total_len - 1 {
                continue;
            }

            let end = if i == total_len - 1 { total_len } else { i };
            render_pass
                .draw_shape2d_instanced(shape_registry.get_shape(s), start as u32..end as u32);
            s = renderables.0[i].0.shape;
            start = i;
        }
    }
    render_state.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

pub struct RenderResult(Result<(), wgpu::SurfaceError>);

pub struct FpsCounter {
    start: instant::Instant,
    frames: u32,
}
impl FpsCounter {
    pub fn new() -> Self {
        Self {
            start: instant::Instant::now(),
            frames: 0,
        }
    }
}

fn fps_counter(mut counter: ResMut<FpsCounter>) {
    counter.frames += 1;
    let now = instant::Instant::now();
    if now.duration_since(counter.start).as_secs() >= 1 {
        error!("FPS: {}", counter.frames);
        counter.start = now;
        counter.frames = 0;
    }
}
