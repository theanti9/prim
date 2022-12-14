//! Prim is an experimental 2D game library focused on basic geometric shapes.
//!
//! Prim uses basic predefined shapes for all rendered instances, allowing for efficient
//! GPU batching of simple geometry.
//!
//! Currently there is no support for texturing or lighting. Lighting is planned but
//! texturing is not. The idea of Prim is to keep the graphics relatively simple, and
//! focus on gameplay.
#![deny(clippy::pedantic)]
#![deny(missing_docs)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::inline_always)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_arguments)]

/// Includes utilities and functionality for basic animation.
pub mod animation {
    /// The implementation for cycling between shapes in a sprite-like manner.
    pub mod shape_cycle;
    /// The implementation for tweening values of a particular shape over time.
    pub mod tween;
}
/// Implementation of the engine's Camera mechanism, defining how to view the 2D world.
pub mod camera;
/// Implementation for a basic collision system between entities.
pub mod collision;
/// Implementation of Initializer commands, used to setup assets after basic engine initialization
/// but before game logic begins.
pub mod initialization;
/// Handling of user input mechanisms such as Keyboard and Mouse controls.
pub mod input;
/// Defines the basic units of renderable objects and logic necessary to place them in the world.
pub mod instance;
/// A cpu-based particle system implementation that works with Shapes provided to the engine.
pub mod particle_system {
    /// Components necessary for the particle system.
    pub mod components;
    /// ECS systems related to running particle systems.
    pub mod systems;
    /// Functionality and utilities for defining particle system values and ranges.
    pub mod values;
}
/// Definition and construction of resources related to the rendering pipeline.
pub mod pipeline;
/// Defines how Shapes are stored and rendered.
pub mod shape;
/// The registry which holds and allows access to shapes at runtime.
pub mod shape_registry;
/// The main engine and renderer runtime state.
pub mod state;
/// Constructs for dealing with and rendering Text within Prim games.
pub mod text;
/// Structs and methods for dealing with game time.
pub mod time;
/// Engine helpers.
pub mod util;
///
pub mod vertex;
/// Handling of application windows.
pub mod window;

pub use glam::{Vec2, Vec3, Vec4};

use log::{error, warn};
use window::PrimWindowOptions;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::state::State;

/// The main entrypoint to the engine.
///
/// The run function takes an initializer function which has one-time mutable access to the game state after it's been set up,
/// but before the game loop begins. This allows the caller to do set up [`bevy_ecs::world::World`] resources and [`bevy_ecs::schedule::Schedule`] systems.
///
/// [`State::add_initializer`] can also be used here to initialize and load resources such as Fonts.
///
/// After the initializer funciton is run, any queued initialization commands are run, and the game loop begins.
///
/// This function does not return, the application will quit from directly within the event loop.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run<F>(window_options: PrimWindowOptions, initializer: F)
where
    F: FnOnce(&mut State),
{
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "WARN");
                warn!("Defaulting logger to WARN level");
            }
            env_logger::init();
        }
    }
    let specified_size = window_options.window_size.unwrap_or((1024, 768));
    let logical_size = LogicalSize::new(specified_size.0, specified_size.1);

    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_decorations(window_options.window_decorations)
        .with_title(&window_options.window_title)
        .with_fullscreen(window_options.get_fullscreen())
        .with_inner_size(logical_size)
        .build(&event_loop)
    {
        Ok(window) => window,
        Err(err) => {
            error!("Error creating window: {:?}", err);
            return;
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(
        &window,
        window_options.vsync,
        window_options.clear_color,
        window_options.sample_count,
    );

    {
        initializer(&mut state);
    }

    state.run_initializer_queue();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let size = state.size();
            state.update();

            match state.render_result() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{e:?}"),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
