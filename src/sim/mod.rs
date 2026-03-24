pub mod params;

use wgpu::*;
use crate::sim::params::{SimulationParams, InjectionBuffer};
use crate::gpu::texture::{create_simulation_texture, create_texture_view};
use crate::gpu::pipelines::SimulationPipelines;

pub struct Simulation {
    pub params: SimulationParams,
    
    // Textures (ping-pong pairs)
    pub surf_ping: Texture,
    pub surf_pong: Texture,
    pub pigment_ping: Texture,
    pub pigment_pong: Texture,
    pub vel: Texture,
    
    // Optional wave textures
    pub eta_ping: Texture,
    pub eta_pong: Texture,
    pub uw_ping: Texture,
    pub uw_pong: Texture,
    
    pub size: u32,
    pub workgroup_size: u32,
}

impl Simulation {
    pub fn new(device: &Device, size: u32) -> Self {
        let workgroup_size = 16;
        
        let surf_ping = create_simulation_texture(device, size, Some("surf_ping"));
        let surf_pong = create_simulation_texture(device, size, Some("surf_pong"));
        let pigment_ping = create_simulation_texture(device, size, Some("pigment_ping"));
        let pigment_pong = create_simulation_texture(device, size, Some("pigment_pong"));
        let vel = create_simulation_texture(device, size, Some("vel"));
        
        let eta_ping = create_simulation_texture(device, size, Some("eta_ping"));
        let eta_pong = create_simulation_texture(device, size, Some("eta_pong"));
        let uw_ping = create_simulation_texture(device, size, Some("uw_ping"));
        let uw_pong = create_simulation_texture(device, size, Some("uw_pong"));
        
        Self {
            params: SimulationParams::default(),
            surf_ping,
            surf_pong,
            pigment_ping,
            pigment_pong,
            vel,
            eta_ping,
            eta_pong,
            uw_ping,
            uw_pong,
            size,
            workgroup_size,
        }
    }
    
    pub fn reset(&self, _queue: &Queue) {
        // Clear all textures to zero - simplified version
        // In a full implementation, we'd clear each texture properly
    }
    
    pub fn step(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        pipelines: &SimulationPipelines,
        injection: &InjectionBuffer,
    ) {
        let dt_sub = self.params.dt / self.params.substeps as f32;
        let mut params_sub = self.params;
        params_sub.dt = dt_sub;
        pipelines.update_params(queue, &params_sub);
        pipelines.update_injection(queue, injection);
        
        let dispatch_size = (self.size + self.workgroup_size - 1) / self.workgroup_size;
        
        for _ in 0..self.params.substeps {
            // 1. Inject
            {
                let surf_src_view = create_texture_view(&self.surf_ping);
                let pigment_src_view = create_texture_view(&self.pigment_ping);
                let surf_dst_view = create_texture_view(&self.surf_pong);
                let pigment_dst_view = create_texture_view(&self.pigment_pong);
                
                let inject_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("inject_bind_group"),
                    layout: &pipelines.inject.get_bind_group_layout(2),
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&surf_src_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&pigment_src_view),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::TextureView(&surf_dst_view),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: BindingResource::TextureView(&pigment_dst_view),
                        },
                    ],
                });
                
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("inject_pass"),
                    timestamp_writes: None,
                });
                
                pass.set_pipeline(&pipelines.inject);
                pass.set_bind_group(0, &pipelines.params_bind_group, &[]);
                pass.set_bind_group(1, &pipelines.injection_bind_group, &[]);
                pass.set_bind_group(2, &inject_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);
            }
            
            // Swap ping-pong
            std::mem::swap(&mut self.surf_ping, &mut self.surf_pong);
            std::mem::swap(&mut self.pigment_ping, &mut self.pigment_pong);
            
            // 2. Compute velocity
            {
                let surf_view = create_texture_view(&self.surf_ping);
                let vel_view = create_texture_view(&self.vel);
                
                let velocity_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("velocity_bind_group"),
                    layout: &pipelines.velocity.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&surf_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&vel_view),
                        },
                    ],
                });
                
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("velocity_pass"),
                    timestamp_writes: None,
                });
                
                pass.set_pipeline(&pipelines.velocity);
                pass.set_bind_group(0, &pipelines.params_bind_group, &[]);
                pass.set_bind_group(1, &velocity_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);
            }
            
            // 3. Optional waves (simplified - would need proper implementation)
            if self.params.wave_enabled == 1 {
                // TODO: Implement wave step
            }
            
            // 4. Advect surfactant
            {
                let surf_src_view = create_texture_view(&self.surf_ping);
                let vel_view = create_texture_view(&self.vel);
                let surf_dst_view = create_texture_view(&self.surf_pong);
                
                let sampler = if self.params.boundary_mode == 1 {
                    &pipelines.wrap_sampler
                } else {
                    &pipelines.clamp_sampler
                };
                
                let advect_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("advect_surf_bind_group"),
                    layout: &pipelines.advect_scalar.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&surf_src_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&vel_view),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::Sampler(sampler),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: BindingResource::TextureView(&surf_dst_view),
                        },
                    ],
                });
                
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("advect_surf_pass"),
                    timestamp_writes: None,
                });
                
                pass.set_pipeline(&pipelines.advect_scalar);
                pass.set_bind_group(0, &pipelines.params_bind_group, &[]);
                pass.set_bind_group(1, &advect_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);
            }
            
            std::mem::swap(&mut self.surf_ping, &mut self.surf_pong);
            
            // 5. Diffuse surfactant (Jacobi iterations)
            for _ in 0..self.params.jacobi_surf {
                let surf_src_view = create_texture_view(&self.surf_ping);
                let surf_dst_view = create_texture_view(&self.surf_pong);
                
                let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("diffuse_surf_bind_group"),
                    layout: &pipelines.diffuse_jacobi.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&surf_src_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&surf_dst_view),
                        },
                    ],
                });
                
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("diffuse_surf_pass"),
                    timestamp_writes: None,
                });
                
                pass.set_pipeline(&pipelines.diffuse_jacobi);
                pass.set_bind_group(0, &pipelines.params_bind_group, &[]);
                pass.set_bind_group(1, &diffuse_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);
                
                std::mem::swap(&mut self.surf_ping, &mut self.surf_pong);
            }
            
            // 6. Advect pigment
            {
                let pigment_src_view = create_texture_view(&self.pigment_ping);
                let vel_view = create_texture_view(&self.vel);
                let pigment_dst_view = create_texture_view(&self.pigment_pong);
                
                let sampler = if self.params.boundary_mode == 1 {
                    &pipelines.wrap_sampler
                } else {
                    &pipelines.clamp_sampler
                };
                
                let advect_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("advect_pigment_bind_group"),
                    layout: &pipelines.advect_color.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&pigment_src_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&vel_view),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::Sampler(sampler),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: BindingResource::TextureView(&pigment_dst_view),
                        },
                    ],
                });
                
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("advect_pigment_pass"),
                    timestamp_writes: None,
                });
                
                pass.set_pipeline(&pipelines.advect_color);
                pass.set_bind_group(0, &pipelines.params_bind_group, &[]);
                pass.set_bind_group(1, &advect_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);
            }
            
            std::mem::swap(&mut self.pigment_ping, &mut self.pigment_pong);
            
            // 7. Diffuse pigment (Jacobi iterations)
            for _ in 0..self.params.jacobi_pig {
                let pigment_src_view = create_texture_view(&self.pigment_ping);
                let pigment_dst_view = create_texture_view(&self.pigment_pong);
                
                let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("diffuse_pigment_bind_group"),
                    layout: &pipelines.diffuse_pigment.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&pigment_src_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&pigment_dst_view),
                        },
                    ],
                });
                
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("diffuse_pigment_pass"),
                    timestamp_writes: None,
                });
                
                pass.set_pipeline(&pipelines.diffuse_pigment);
                pass.set_bind_group(0, &pipelines.params_bind_group, &[]);
                pass.set_bind_group(1, &diffuse_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_size, dispatch_size, 1);
                
                std::mem::swap(&mut self.pigment_ping, &mut self.pigment_pong);
            }
        }
    }
}
