use winit::event::{WindowEvent, MouseButton, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::sim::params::{InjectionBuffer, InjectionEvent, MAX_INJECTION_EVENTS};
use crate::sim::Simulation;
use crate::gpu::GpuContext;
use crate::gpu::pipelines::SimulationPipelines;
use crate::gpu::texture::create_texture_view;
use glam::Vec2;
use wgpu::*;

pub struct App {
    pub simulation: Simulation,
    pub pipelines: SimulationPipelines,
    pub injection: InjectionBuffer,
    pub paused: bool,
    pub drop_radius: f32,
    pub current_color: [f32; 3],
    pub color_index: usize,
}

const COLORS: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0],        // Black
    [0.2, 0.1, 0.05],       // Brown
    [0.1, 0.1, 0.3],        // Dark blue
    [0.3, 0.1, 0.1],        // Dark red
    [0.1, 0.3, 0.1],        // Dark green
    [0.2, 0.1, 0.2],        // Purple
    [0.05, 0.15, 0.25],     // Navy
    [0.15, 0.15, 0.15],     // Dark gray
];

impl App {
    pub fn new(gpu: &GpuContext<'_>, size: u32) -> Self {
        let simulation = Simulation::new(&gpu.device, size);
        let pipelines = SimulationPipelines::new(&gpu.device, size, gpu.surface_config.format);
        
        // Initialize params
        pipelines.update_params(&gpu.queue, &simulation.params);
        
        Self {
            simulation,
            pipelines,
            injection: InjectionBuffer::default(),
            paused: false,
            drop_radius: 0.02,
            current_color: COLORS[0],
            color_index: 0,
        }
    }
    
    pub fn handle_window_event(&mut self, event: &WindowEvent, gpu: &GpuContext<'_>) {
        match event {
            WindowEvent::MouseInput { button, state, .. } => {
                if *state == ElementState::Pressed {
                    match button {
                        MouseButton::Left => {
                            // Will be handled in cursor_moved
                        }
                        MouseButton::Right => {
                            // Will be handled in cursor_moved
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { .. } => {
                // Mouse position is handled in main.rs
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.drop_radius = (self.drop_radius * (1.0 + y * 0.1)).max(0.005).min(0.1);
                    }
                    _ => {}
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Space => {
                                self.paused = !self.paused;
                            }
                            KeyCode::KeyR => {
                                // Reset simulation (TODO: implement)
                            }
                            KeyCode::Digit1 => {
                                self.simulation.params.boundary_mode = 0; // CLAMP
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::Digit2 => {
                                self.simulation.params.boundary_mode = 1; // WRAP
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::Digit3 => {
                                self.simulation.params.wave_enabled = 1 - self.simulation.params.wave_enabled;
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::Equal | KeyCode::NumpadAdd => {
                                self.simulation.params.alpha_e *= 1.1;
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::Minus | KeyCode::NumpadSubtract => {
                                self.simulation.params.alpha_e /= 1.1;
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::BracketLeft => {
                                self.simulation.params.jacobi_surf = (self.simulation.params.jacobi_surf - 1).max(1);
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::BracketRight => {
                                self.simulation.params.jacobi_surf += 1;
                                self.pipelines.update_params(&gpu.queue, &self.simulation.params);
                            }
                            KeyCode::KeyC => {
                                // Cycle colors
                                self.color_index = (self.color_index + 1) % COLORS.len();
                                self.current_color = COLORS[self.color_index];
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    pub fn add_injection(&mut self, pos: Vec2, is_dispersant: bool, _gpu: &GpuContext<'_>) {
        if self.injection.count as usize >= MAX_INJECTION_EVENTS {
            return;
        }
        
        let event = InjectionEvent {
            pos: [pos.x, pos.y],
            radius: self.drop_radius,
            surf_amp: if is_dispersant { -1.0 } else { 1.0 },
            pigment_amp: if is_dispersant { 0.0 } else { 0.5 },
            _pad0: [0; 3],
            pigment_rgb: self.current_color,
            event_type: if is_dispersant { 1 } else { 0 },
            _padding: [0; 4],
            _padding2: [0; 3],
            _pad1: 0,
        };
        
        self.injection.events[self.injection.count as usize] = event;
        self.injection.count += 1;
    }
    
    pub fn clear_injections(&mut self) {
        self.injection.count = 0;
    }
    
    pub fn update(&mut self, encoder: &mut CommandEncoder, gpu: &GpuContext<'_>) {
        if !self.paused {
            self.simulation.step(
                &gpu.device,
                &gpu.queue,
                encoder,
                &self.pipelines,
                &self.injection,
            );
        }
        
        // Clear injection events after processing
        self.clear_injections();
    }
    
    pub fn render(&self, encoder: &mut CommandEncoder, view: &TextureView, gpu: &GpuContext<'_>) {
        let pigment_view = create_texture_view(&self.simulation.pigment_ping);
        let sampler = if self.simulation.params.boundary_mode == 1 {
            &self.pipelines.wrap_sampler
        } else {
            &self.pipelines.clamp_sampler
        };
        
        let render_bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("render_bind_group"),
            layout: &self.pipelines.render.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&pigment_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.pipelines.render);
            render_pass.set_bind_group(0, &render_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
