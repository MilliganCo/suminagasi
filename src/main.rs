mod gpu;
mod sim;
mod app;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::Window,
};
use anyhow::Result;
use app::App;
use gpu::GpuContext;
use std::rc::Rc;

fn main() -> Result<()> {
    env_logger::init();
    
    let event_loop = EventLoop::new()?;
    let window = Rc::new(event_loop.create_window(
        Window::default_attributes()
            .with_title("Suminagashi Simulation")
            .with_inner_size(winit::dpi::PhysicalSize::new(1024, 1024))
    )?);
    
    let window_id = window.id();
    let mut gpu = pollster::block_on(GpuContext::new(&window))?;
    let mut app = App::new(&gpu, 1024);
    
    let mut mouse_pressed = false;
    let mut mouse_button = winit::event::MouseButton::Left;
    let mut mouse_pos = glam::Vec2::ZERO;
    
    // Clone Rc to keep window alive
    let window_clone = window.clone();
    
    event_loop.run(move |event, elwt| {
        let _window = &window_clone;
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent { ref event, window_id: id } if id == window_id => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        gpu.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        let frame = match gpu.get_current_texture() {
                            Ok(frame) => frame,
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                elwt.exit();
                                return;
                            }
                            Err(e) => {
                                eprintln!("Dropped frame: {e}");
                                return;
                            }
                        };
                        
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("render_encoder"),
                        });
                        
                        // Handle mouse input
                        if mouse_pressed {
                            let size = glam::Vec2::new(gpu.size.width as f32, gpu.size.height as f32);
                            let uv = mouse_pos / size;
                            app.add_injection(uv, mouse_button == winit::event::MouseButton::Right, &gpu);
                        }
                        
                        app.update(&mut encoder, &gpu);
                        app.render(&mut encoder, &view, &gpu);
                        
                        gpu.queue.submit(std::iter::once(encoder.finish()));
                        frame.present();
                        
                        window_clone.request_redraw();
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        mouse_pressed = *state == winit::event::ElementState::Pressed;
                        mouse_button = *button;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = glam::Vec2::new(position.x as f32, position.y as f32);
                    }
                    _ => {
                        app.handle_window_event(event, &gpu);
                    }
                }
            }
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}
