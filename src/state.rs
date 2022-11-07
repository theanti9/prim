use bevy_ecs::{
    prelude::{Bundle, Component, DetectChanges, Events},
    query::{Changed, With},
    schedule::{Schedule, Stage, SystemStage},
    system::{Query, Res, ResMut},
    world::{Mut, World},
};
use glam::{Vec2, Vec3, Vec4};
use log::{error, info};
use wgpu::{include_wgsl, util::DeviceExt, PipelineLayout, ShaderModule, VertexBufferLayout};
use wgpu_text::section::{OwnedText, Section, Text};
use winit::{
    event::{ElementState, KeyboardInput, WindowEvent},
    window::Window,
};

use crate::{
    camera::Camera2D,
    initialization::{InitializeCommand, InitializerQueue},
    input::{Keyboard, Mouse},
    instance::{Inst, Instance2D},
    jump_flood::{num_passes, JumpFloodParams},
    pipelines::{
        PrimBindGroupLayouts, PrimBindGroups, PrimBuffers, PrimPipelines, PrimShaderModules,
        PrimTargets,
    },
    shape::{DrawShape2D, Shape2DVertex},
    shape_registry::ShapeRegistry,
    text::{FontRegistry, TextSection},
    time::Time,
    vertex::Vertex,
    window::{PrimWindow, PrimWindowResized},
};

/// The main application state container.
///
/// This contains current state for the window, inputs, world entities, execution schedule,
/// and rendering components.
pub struct State {
    size: winit::dpi::PhysicalSize<u32>,
    keyboard: Keyboard,
    mouse: Mouse,
    world: World,
    schedule: Schedule,
    initializer_queue: InitializerQueue,
}

impl State {
    /// Givien a [`winit::window::Window`], start a new application state within it.
    ///
    /// This attempts to initialize all the necessary wgpu information.
    ///
    /// # Panics
    /// This method panics if wgpu fails to find or initialize an adapter with the specified options,
    /// or if it is unable to initialize the device and queue.
    #[must_use]
    pub fn new(window: &Window, vsync: bool, clear_color: Vec3, sample_count: u32) -> Self {
        let size = window.inner_size();
        // Let wgpu decide the best backend based on what's available for the platform.
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();
        info!("Starting with backend: {:?}", adapter.get_info().backend);

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
            present_mode: if vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
        };
        surface.configure(&device, &config);

        #[allow(clippy::cast_precision_loss)]
        let camera2d = Camera2D::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(size.width as f32 / 2.0, size.height as f32 / 2.0),
        );

        let time = Time::new();
        let mut shape_registry = ShapeRegistry::new();
        shape_registry.register_builtin_shapes(&device);

        let keyboard = Keyboard::new();
        let mouse = Mouse::new();

        let render_state = Self::create_render_state(
            config,
            surface,
            device,
            queue,
            &camera2d,
            clear_color,
            sample_count,
        );

        let mut world = World::default();

        Self::setup_world(
            &mut world,
            camera2d,
            render_state,
            time,
            shape_registry,
            keyboard.clone(),
            mouse.clone(),
        );

        let mut schedule = Schedule::default();
        Self::setup_schedule(&mut schedule);

        let initializer_queue = InitializerQueue::new();

        Self {
            size,
            keyboard,
            mouse,
            world,
            schedule,
            initializer_queue,
        }
    }

    /// Add an initializer command to the queue to be processed before the world systems are started.
    pub fn add_initializer(&mut self, command: InitializeCommand) {
        self.initializer_queue.queue.push_back(command);
    }

    pub(crate) fn run_initializer_queue(&mut self) {
        for cmd in &self.initializer_queue.queue {
            match cmd {
                InitializeCommand::InitializeFont(initialize_font) => {
                    self.world
                        .resource_scope(|world, mut font_registry: Mut<FontRegistry>| {
                            if let Some(render_state) = world.get_resource::<RenderState>() {
                                match font_registry.initialize_font(
                                    initialize_font.name.clone(),
                                    initialize_font.bytes,
                                    &render_state.device,
                                    &render_state.config,
                                ) {
                                    Ok(_) => {}
                                    Err(err) => error!(
                                        "Error loading font {}: {}",
                                        &initialize_font.name, err
                                    ),
                                }
                            }
                        });
                }
                InitializeCommand::InitializeShape(initialize_shape) => {
                    self.world
                        .resource_scope(|world, mut shape_registry: Mut<ShapeRegistry>| {
                            if let Some(render_state) = world.get_resource::<RenderState>() {
                                shape_registry.register_shape(
                                    initialize_shape.name.clone(),
                                    initialize_shape.vertices.clone(),
                                    initialize_shape.indices.clone(),
                                    &render_state.device,
                                );
                            }
                        });
                }
            }
        }

        // This should only be run once. Clear the queue and zero it in size to release the memory,
        // in case the user initialized a lot of stuff.
        self.initializer_queue.queue.clear();
        self.initializer_queue.queue.shrink_to_fit();
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: Option<&'static str>,
        layout: &PipelineLayout,
        shader_module: &ShaderModule,
        vertex_buffers: &[VertexBufferLayout],
        sample_count: u32,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
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
                count: sample_count,
                ..Default::default()
            },
            multiview: None,
        })
    }

    fn create_multisample_framebuffer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let frame_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };

        device
            .create_texture(frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_render_state(
        config: wgpu::SurfaceConfiguration,
        surface: wgpu::Surface,
        device: wgpu::Device,
        queue: wgpu::Queue,
        camera2d: &Camera2D,
        clear_color: Vec3,
        sample_count: u32,
    ) -> RenderState {
        let shader2d = device.create_shader_module(include_wgsl!("shader2d.wgsl"));
        let emitter_occluder_shader =
            device.create_shader_module(include_wgsl!("EmitterOccluder.wgsl"));

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera2d.get_view(config.width, config.height)]),
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

        let render_pipeline = Self::create_render_pipeline(
            &device,
            &config,
            Some("Render Pipeline"),
            &render_pipeline_layout,
            &shader2d,
            &[Shape2DVertex::desc(), Instance2D::desc()],
            sample_count,
        );

        let emitter_occluder_render_pipeline = Self::create_render_pipeline(
            &device,
            &config,
            Some("Emitter Occluder Render"),
            &render_pipeline_layout,
            &emitter_occluder_shader,
            &[Shape2DVertex::desc(), Instance2D::desc()],
            sample_count,
        );

        // Create an instance buffer for up to 100,000 entities.
        // Currently, having more items than this rendered at once will cause the program to crash.
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<Inst>() * 100_000) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let multisample_framebuffer =
            State::create_multisample_framebuffer(&device, &config, sample_count);

        let shaders = PrimShaderModules::new(&device);
        let bind_group_layouts = PrimBindGroupLayouts::new(&device);
        let pipelines = PrimPipelines::new(&device, &config, &bind_group_layouts, &shaders);
        let targets = PrimTargets::new(&device, &config, config.width, config.height);
        let buffers = PrimBuffers::new(&device, &config, camera2d);
        let bind_groups =
            PrimBindGroups::new(&device, &config, &bind_group_layouts, &targets, &buffers);

        RenderState {
            config,
            surface,
            queue,
            camera_buffer,
            device,
            render_pipeline,
            emitter_occluder_render_pipeline,
            camera_bind_group,
            instance_buffer,
            // TODO: Make configurable
            sort_renderables: false,
            clear_color: wgpu::Color {
                r: f64::from(clear_color.x),
                g: f64::from(clear_color.y),
                b: f64::from(clear_color.z),
                a: 1.0,
            },
            sample_count,
            multisample_framebuffer,
            recreate_framebuffer: false,
            shaders,
            bind_group_layouts,
            pipelines,
            targets,
            buffers,
            bind_groups,
        }
    }

    /// Registers resources needed by most systems or the rendering process in the ecs world.
    fn setup_world(
        world: &mut World,
        camera2d: Camera2D,
        render_state: RenderState,
        time: Time,
        shape_registry: ShapeRegistry,
        keyboard: Keyboard,
        mouse: Mouse,
    ) {
        world.insert_resource(Events::<PrimWindowResized>::default());
        world.insert_resource(PrimWindow::new(&render_state.config));
        world.insert_resource(camera2d);
        world.insert_resource(render_state);
        world.insert_resource(time);
        world.insert_resource(shape_registry);
        world.insert_resource(keyboard);
        world.insert_resource(mouse);
        world.insert_resource(FontRegistry::new());
        world.insert_resource(Renderables(Vec::with_capacity(1000)));
        world.insert_resource(RenderResult(Ok(())));
        world.insert_resource(FpsCounter::new());
    }

    /// Sets up the main stages of execution for the given [`Schedule`]
    ///
    /// The following stages are executed in order:
    /// - `pre_updated`: Used for updating items that need to be consistent for the duration of any parallel systems for the frame.
    /// - `update`: Used for any game logic.
    /// - `post_update`: Used to sync any computations necessary after game logic executes, such as view and transformation matrices.
    /// - `collect`: Finds all renderable instances and their matrices.
    /// - `render`: Sends instance information to the GPU and presents.
    fn setup_schedule(schedule: &mut Schedule) {
        schedule.add_stage(
            "pre_update",
            SystemStage::parallel()
                .with_system(update_time)
                .with_system(update_events::<PrimWindowResized>),
        );
        schedule.add_stage("update", SystemStage::parallel().with_system(fps_counter));
        schedule.add_stage(
            "post_update",
            SystemStage::parallel()
                .with_system(update_camera)
                .with_system(sync_matrix),
        );
        schedule.add_stage(
            "collect",
            SystemStage::single_threaded().with_system(collect_instances),
        );
        schedule.add_stage(
            "render",
            SystemStage::parallel().with_system(main_render_pass),
        );
    }

    /// Add an event to the world that will have its reader/writer updated each frame.
    pub fn add_event<T>(&mut self)
    where
        T: Send + Sync + 'static,
    {
        self.world.insert_resource(Events::<T>::default());
        self.schedule
            .add_system_to_stage("pre_update", update_events::<T>);
    }

    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;

            self.world
                .resource_scope(|world, mut render_state: Mut<RenderState>| {
                    render_state.config.width = new_size.width;
                    render_state.config.height = new_size.height;
                    render_state.recreate_framebuffer = true;
                    render_state
                        .surface
                        .configure(&render_state.device, &render_state.config);
                    world.send_event(PrimWindowResized::from_size(
                        new_size.width,
                        new_size.height,
                    ));
                    if let Some(mut font_registry) = world.get_resource_mut::<FontRegistry>() {
                        font_registry.fonts_mut().iter_mut().for_each(|f| {
                            f.resize_view(
                                new_size.width as f32,
                                new_size.height as f32,
                                &render_state.queue,
                            );
                        });
                    }

                    if let Some(mut prim_window) = world.get_resource_mut::<PrimWindow>() {
                        prim_window.update(&render_state.config);
                    }
                });

            if let Some(mut camera2d) = self.world.get_resource_mut::<Camera2D>() {
                camera2d.rescale(Vec2::new(new_size.width as f32, new_size.height as f32));
            }
        }
    }

    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        #[allow(clippy::single_match)]
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
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => self.mouse.pressed(*button),
                ElementState::Released => self.mouse.released(*button),
            },
            _ => {}
        }
        false
    }

    pub(crate) fn update(&mut self) {
        if let Some(mut k) = self.world.get_resource_mut::<Keyboard>() {
            *k = self.keyboard.clone();
            self.keyboard.update();
        }

        if let Some(mut m) = self.world.get_resource_mut::<Mouse>() {
            *m = self.mouse.clone();
            self.mouse.update();
        }

        self.schedule.run(&mut self.world);
        self.world.clear_trackers();
    }

    #[inline(always)]
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    /// Fetches the result of the last render call from the ecs world.
    ///
    /// # Errors
    /// Returns a `wgpu::SurfaceError` if there were any issues during rendering.
    /// These generally indicate that the surface needs to be resized or recreated.
    #[inline(always)]
    pub(crate) fn render_result(&self) -> Result<(), wgpu::SurfaceError> {
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
}

/// Run in the `pre_update` stage, updates the timestep for the upcoming frame.
fn update_time(mut time: ResMut<Time>) {
    time.update();
}

fn update_events<T>(mut events: ResMut<Events<T>>)
where
    T: Send + Sync + 'static,
{
    events.update();
}

/// Run in the `post_update` stage, recomputes the view matrix if the camera transform has changed.
fn update_camera(mut camera2d: ResMut<Camera2D>) {
    if camera2d.is_changed() {
        camera2d.update();
    }
}

/// Contains the collected list of renderable items.
struct Renderables(Vec<(Instance2D, Inst)>);

/// Run in the `post_update` stage, syncs any changes from the transform values to the transformation matrix that'll be
/// passed to the instance buffer.
fn sync_matrix(mut instances: Query<(&Instance2D, &mut Inst), Changed<Instance2D>>) {
    instances.par_for_each_mut(1024, |(changed, mut inst)| {
        *inst = changed.to_matrix();
    });
}

/// Collects instances current visible by the camera and writes their data to the instance buffer.
fn collect_instances(
    instance_query: Query<(&Instance2D, &mut Inst)>,
    mut renderables: ResMut<Renderables>,
    render_state: Res<RenderState>,
    camera2d: Res<Camera2D>,
) {
    renderables.0.clear();

    for (inst, render_inst) in &instance_query {
        // Do a basic filter for where their position is within their maximum radius of the edge of the camera.
        // This only works correctly if a shape is defined with all vertices using normalized positions between (-1.0, 1.0)
        if inst.position.x - inst.scale.x < camera2d.position.x + camera2d.scale.x
            && inst.position.x + inst.scale.x > camera2d.position.x - camera2d.scale.x
            && inst.position.y - inst.scale.y < camera2d.position.y + camera2d.scale.y
            && inst.position.y + inst.scale.y > camera2d.position.y - camera2d.scale.y
        {
            renderables.0.push((*inst, *render_inst));
        }
    }
    // If sorting is enabled, sort the shapes by their shape ID.
    // When sorting is enabled, the number of draw calls will be equal to the number of discrete shapes visible to the
    // camera. This can be used to trade off CPU (list sorting) and GPU (draw calls).
    if render_state.sort_renderables {
        renderables.0.sort_by(|a, b| a.0.shape.cmp(&b.0.shape));
    }
    let shape2d_instances_data = renderables.0.iter().map(|(_a, b)| *b).collect::<Vec<_>>();

    render_state.queue.write_buffer(
        &render_state.buffers.instance_buffer,
        0,
        bytemuck::cast_slice(&shape2d_instances_data),
    );
}

pub(crate) struct RenderState {
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub camera_buffer: wgpu::Buffer,
    pub device: wgpu::Device,
    pub emitter_occluder_render_pipeline: wgpu::RenderPipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub instance_buffer: wgpu::Buffer,
    pub sort_renderables: bool,
    pub clear_color: wgpu::Color,
    pub sample_count: u32,
    pub multisample_framebuffer: wgpu::TextureView,
    pub recreate_framebuffer: bool,
    pub shaders: PrimShaderModules,
    pub bind_group_layouts: PrimBindGroupLayouts,
    pub pipelines: PrimPipelines,
    pub targets: PrimTargets,
    pub buffers: PrimBuffers,
    pub bind_groups: PrimBindGroups,
}

fn main_render_pass(
    mut render_state: ResMut<RenderState>,
    shape_registry: Res<ShapeRegistry>,
    renderables: Res<Renderables>,
    camera2d: Res<Camera2D>,
    mut font_registry: ResMut<FontRegistry>,
    mut text_sections: Query<&mut TextSection>,
    mut render_result: ResMut<RenderResult>,
) {
    let output = match render_state.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(err) => {
            *render_result = RenderResult(Err(err));
            return;
        }
    };

    if render_state.recreate_framebuffer {
        render_state.multisample_framebuffer = State::create_multisample_framebuffer(
            &render_state.device,
            &render_state.config,
            render_state.sample_count,
        );

        render_state.targets = PrimTargets::new(
            &render_state.device,
            &render_state.config,
            render_state.config.width,
            render_state.config.height,
        );

        render_state.bind_groups = PrimBindGroups::new(
            &render_state.device,
            &render_state.config,
            &render_state.bind_group_layouts,
            &render_state.targets,
            &render_state.buffers,
        );

        render_state.queue.write_buffer(
            &render_state.buffers.quad_buffer,
            0,
            bytemuck::cast_slice(&[Instance2D {
                scale: Vec2::new(
                    render_state.config.width as f32 * 2.0,
                    render_state.config.height as f32 * 2.0,
                ),
                shape: 2,
                ..Default::default()
            }
            .to_matrix()]),
        );

        render_state.recreate_framebuffer = false;
    }

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    // render_state.queue.write_buffer(
    //     &render_state.camera_buffer,
    //     0,
    //     bytemuck::cast_slice(&[camera2d.get_view()]),
    // );

    render_state.queue.write_buffer(
        &render_state.buffers.camera_buffer,
        0,
        bytemuck::cast_slice(&[
            camera2d.get_view(render_state.config.width, render_state.config.height)
        ]),
    );

    let mut encoder = render_state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Emitter Occluder Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_state.targets.emitter_occluder_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&render_state.pipelines.emitter_occluder_pipeline);
        render_pass.set_bind_group(0, &render_state.bind_groups.camera_bind_group, &[]);
        if let Some((first_renderable, _)) = renderables.0.first() {
            let mut s = first_renderable.shape;
            let mut start: u32 = 0;

            #[allow(clippy::cast_possible_truncation)]
            let total_len = renderables.0.len() as u32;

            render_pass.set_vertex_buffer(1, render_state.buffers.instance_buffer.slice(..));

            // Loop through the renderables and render all contiguous items of the same shape in one draw call.
            // Sorting the list by setting [`RenderState::sort_renderables`] will make sure this list is entirely unfragmented
            // and all visible shape types will have exactly one draw call. This may be disadvantageous in some senarios due to the
            // CPU requirements of sorting large numbers of renderables.
            for i in 0..total_len {
                if renderables.0[i as usize].0.shape == s && i != total_len - 1 {
                    continue;
                }

                let end = if i == total_len - 1 { total_len } else { i };
                render_pass
                    .draw_shape2d_instanced(shape_registry.get_shape(s), start as u32..end as u32);
                s = renderables.0[i as usize].0.shape;
                start = i;
            }
        }
    }
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Jump Seed Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_state.targets.jump_seed_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&render_state.pipelines.jump_seed_pipeline);
        render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &render_state.bind_groups.jump_seed_bind_group, &[]);
        render_pass.set_vertex_buffer(1, render_state.buffers.quad_buffer.slice(..));
        render_pass.draw_shape2d(shape_registry.get_shape(2));
    }

    let flood_passes = num_passes(&render_state.config) as usize;

    for i in 0..flood_passes {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Jump Flood Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: if i == flood_passes - 1 {
                    &view
                } else {
                    &render_state.targets.jump_flood_targets[i]
                },
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&render_state.pipelines.jump_flood_pipeline);
        render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &render_state.bind_groups.jump_flood_bind_groups[i], &[]);

        let offset = 2.0_f32.powf((flood_passes - i - 1) as f32);
        // let offset = 2.0_f32.powf((i + 1) as f32);

        render_state.queue.write_buffer(
            &render_state.buffers.jump_flood_params_buffer,
            0,
            bytemuck::cast_slice(&[JumpFloodParams {
                level: i as f32,
                max_steps: flood_passes as f32,
                offset,
            }]),
        );

        render_pass.set_vertex_buffer(1, render_state.buffers.quad_buffer.slice(..));
        render_pass.draw_shape2d(shape_registry.get_shape(2));
    }

    // {
    //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //         label: Some("Render Pass"),
    //         color_attachments: &[if render_state.sample_count == 1 {
    //             Some(wgpu::RenderPassColorAttachment {
    //                 view: &view,
    //                 resolve_target: None,
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(render_state.clear_color),
    //                     store: true,
    //                 },
    //             })
    //         } else {
    //             Some(wgpu::RenderPassColorAttachment {
    //                 view: &render_state.multisample_framebuffer,
    //                 resolve_target: Some(&view),
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(render_state.clear_color),
    //                     store: true,
    //                 },
    //             })
    //         }],
    //         depth_stencil_attachment: None,
    //     });

    //     render_pass.set_pipeline(&render_state.emitter_occluder_render_pipeline);
    //     render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);

    //     if let Some((first_renderable, _)) = renderables.0.first() {
    //         let mut s = first_renderable.shape;
    //         let mut start: u32 = 0;

    //         #[allow(clippy::cast_possible_truncation)]
    //         let total_len = renderables.0.len() as u32;

    //         render_pass.set_vertex_buffer(1, render_state.instance_buffer.slice(..));

    //         // Loop through the renderables and render all contiguous items of the same shape in one draw call.
    //         // Sorting the list by setting [`RenderState::sort_renderables`] will make sure this list is entirely unfragmented
    //         // and all visible shape types will have exactly one draw call. This may be disadvantageous in some senarios due to the
    //         // CPU requirements of sorting large numbers of renderables.
    //         for i in 0..total_len {
    //             if renderables.0[i as usize].0.shape == s && i != total_len - 1 {
    //                 continue;
    //             }

    //             let end = if i == total_len - 1 { total_len } else { i };
    //             render_pass
    //                 .draw_shape2d_instanced(shape_registry.get_shape(s), start as u32..end as u32);
    //             s = renderables.0[i as usize].0.shape;
    //             start = i;
    //         }
    //     }
    // }

    render_state.queue.submit(std::iter::once(encoder.finish()));

    for ts in &mut text_sections {
        font_registry.get_font_mut(ts.font_id).queue(&ts.section);
    }

    let buffers = font_registry
        .fonts_mut()
        .iter_mut()
        .map(|f| f.draw(&render_state.device, &view, &render_state.queue));
    render_state.queue.submit(buffers);
    output.present();
}

pub struct RenderResult(Result<(), wgpu::SurfaceError>);

pub struct FpsCounter {
    start: instant::Instant,
    frames: u16,
}

impl FpsCounter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            start: instant::Instant::now(),
            frames: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct FpsDisplay;

fn fps_counter(
    mut counter: ResMut<FpsCounter>,
    mut display_query: Query<&mut TextSection, With<FpsDisplay>>,
) {
    counter.frames += 1;
    let now = instant::Instant::now();
    let duration = now.duration_since(counter.start);
    if duration.as_secs_f32() >= 1.0 {
        if let Ok(mut display_section) = display_query.get_single_mut() {
            display_section.section.text[1] = OwnedText::default()
                .with_text(format!(
                    "{:.2}",
                    f32::from(counter.frames) / duration.as_secs_f32()
                ))
                .with_color(Vec4::new(0.75, 0.75, 0.75, 1.0));
        } else {
            info!(
                "FPS: {:.2}",
                f32::from(counter.frames) / duration.as_secs_f32()
            );
        }
        counter.start = now;
        counter.frames = 0;
    }
}

#[derive(Bundle)]
pub struct FpsDisplayBundle {
    fps_display: FpsDisplay,
    text_section: TextSection,
}

impl FpsDisplayBundle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for FpsDisplayBundle {
    fn default() -> Self {
        Self {
            fps_display: FpsDisplay,
            text_section: TextSection {
                font_id: 0,
                section: Section::default()
                    .with_text(vec![
                        Text::new("FPS: ").with_color(Vec4::ONE),
                        Text::new("").with_color(Vec4::ONE),
                    ])
                    .to_owned(),
            },
        }
    }
}
