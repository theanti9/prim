#![deny(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::inline_always)]
#![allow(clippy::module_name_repetitions)]

pub mod camera;
pub mod collision;
pub mod input;
pub mod instance;
pub mod particle_system {
    pub mod components;
    pub mod particles;
    pub mod systems;
    pub mod values;
}
pub mod shape;
pub mod shape_registry;
pub mod state;
pub mod text;
pub mod time;
pub mod vertex;

use log::error;
use winit::{
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
pub fn run<F>(initializer: F)
where
    F: FnOnce(&mut State),
{
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new().build(&event_loop) {
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

    let mut state = State::new(&window);
    {
        initializer(&mut state);
    }

    state.run_initializer_queue();

    error!("Starting event loop");
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
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
