pub mod camera;
pub mod instance;
pub mod object_registry;
pub mod shape;
pub mod shape_registry;
pub mod state;
pub mod time;
pub mod update;
pub mod vertex;

use instant::Instant;
use log::error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::state::State;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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

    let state = std::sync::Arc::new(std::sync::Mutex::new(State::new(&window)));
    let mut last_fps = Instant::now();
    let mut frames = 0;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));

    let logic_barrier = barrier.clone();
    let app_state = state.clone();

    std::thread::spawn(move || loop {
        {
            let mut unlocked_state = app_state.lock().unwrap();
            unlocked_state.update();
        }
        logic_barrier.wait();
    });
    error!("Starting event loop");
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => {
            let mut unlocked_state = state.lock().unwrap();
            if !unlocked_state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        unlocked_state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        unlocked_state.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            {
                let mut unlocked_state = state.lock().unwrap();
                let size = unlocked_state.size();
                match unlocked_state.render() {
                    Ok(_) => {
                        frames += 1;
                        let now = Instant::now();
                        if now.duration_since(last_fps).as_secs() >= 1 {
                            let fps = frames as f32;
                            error!("FPS: {}", fps);
                            last_fps = now;
                            frames = 0;
                        }
                    }
                    Err(wgpu::SurfaceError::Lost) => unlocked_state.resize(size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            barrier.wait();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
