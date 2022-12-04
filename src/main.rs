pub mod event_loop;
pub mod window;
pub mod state;
#[macro_use]
pub mod shader;
pub mod vertex;
pub mod camera;
pub mod texture;
pub mod ecs;

use std::time::Instant;
use log::LevelFilter;
use winit::{
    event::*,
    event_loop::ControlFlow,
};

use state::State;

fn main() {
    env_logger::builder()
        .filter_module("wgpu_core::present", LevelFilter::Info)
        .filter_module("wgpu_core::device", LevelFilter::Info)
        .filter_module("wgpu_hal", LevelFilter::Info)
        .filter_module("naga::front", LevelFilter::Info)
        .init();

    pollster::block_on(run());
}

async fn run() {
    let event_loop = event_loop::create_event_loop();
    let window = window::create_window(&event_loop);

    let mut state = State::new(&window).await;
    let mut last_render_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                state.camera_controller.process_mouse(delta.0, delta.1)
            },

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            },

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }

            _ => {}
        }
    });
}

