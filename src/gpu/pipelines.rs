use wgpu::*;
use crate::sim::params::{SimulationParams, InjectionBuffer};
use crate::gpu::texture::{create_sampler, create_wrap_sampler};

pub struct SimulationPipelines {
    pub inject: ComputePipeline,
    pub velocity: ComputePipeline,
    pub advect_scalar: ComputePipeline,
    pub advect_color: ComputePipeline,
    pub diffuse_jacobi: ComputePipeline,
    pub diffuse_pigment: ComputePipeline,
    pub waves_step: ComputePipeline,
    pub render: RenderPipeline,
    
    pub params_bind_group: BindGroup,
    pub injection_bind_group: BindGroup,
    pub clamp_sampler: Sampler,
    pub wrap_sampler: Sampler,
    
    pub params_buffer: Buffer,
    pub injection_buffer: Buffer,
}

impl SimulationPipelines {
    pub fn new(device: &Device, _size: u32, surface_format: TextureFormat) -> Self {
        let shader_inject = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("inject"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/inject.wgsl").into()),
        });
        
        let shader_velocity = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("velocity"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/velocity_from_surf.wgsl").into()),
        });
        
        let shader_advect_scalar = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("advect_scalar"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/advect_scalar.wgsl").into()),
        });
        
        let shader_advect_color = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("advect_color"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/advect_color.wgsl").into()),
        });
        
        let shader_diffuse = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("diffuse"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/diffuse_jacobi.wgsl").into()),
        });
        
        let shader_diffuse_pigment = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("diffuse_pigment"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/diffuse_pigment.wgsl").into()),
        });
        
        let shader_waves = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("waves"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/waves_step.wgsl").into()),
        });
        
        let shader_render = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("render"),
            source: ShaderSource::Wgsl(include_str!("../../shaders/render.wgsl").into()),
        });
        
        // Create buffers
        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("params_buffer"),
            size: std::mem::size_of::<SimulationParams>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let injection_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("injection_buffer"),
            size: std::mem::size_of::<InjectionBuffer>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create samplers
        let clamp_sampler = create_sampler(device);
        let wrap_sampler = create_wrap_sampler(device);
        
        // Bind group layout for params
        let params_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("params_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let params_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("params_bind_group"),
            layout: &params_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });
        
        // Bind group layout for injection
        let injection_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("injection_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let injection_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("injection_bind_group"),
            layout: &injection_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: injection_buffer.as_entire_binding(),
            }],
        });
        
        // Bind group layout for inject textures (group 2)
        let inject_textures_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("inject_textures_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        // Compute pipeline layouts
        let inject_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("inject_layout"),
            bind_group_layouts: &[&params_bind_group_layout, &injection_bind_group_layout, &inject_textures_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Bind group layout for velocity (group 1)
        let velocity_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("velocity_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        // Bind group layout for advect (group 1)
        let advect_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("advect_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        // Bind group layout for diffuse (group 1)
        let diffuse_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("diffuse_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        let velocity_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("velocity_layout"),
            bind_group_layouts: &[&params_bind_group_layout, &velocity_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let advect_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("advect_layout"),
            bind_group_layouts: &[&params_bind_group_layout, &advect_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let diffuse_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("diffuse_layout"),
            bind_group_layouts: &[&params_bind_group_layout, &diffuse_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Bind group layout for waves (group 1)
        let waves_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("waves_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        let waves_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("waves_layout"),
            bind_group_layouts: &[&params_bind_group_layout, &waves_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create compute pipelines
        let inject = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("inject"),
            layout: Some(&inject_layout),
            module: &shader_inject,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        let velocity = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("velocity"),
            layout: Some(&velocity_layout),
            module: &shader_velocity,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        let advect_scalar = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("advect_scalar"),
            layout: Some(&advect_layout),
            module: &shader_advect_scalar,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        let advect_color = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("advect_color"),
            layout: Some(&advect_layout),
            module: &shader_advect_color,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        let diffuse_jacobi = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("diffuse_jacobi"),
            layout: Some(&diffuse_layout),
            module: &shader_diffuse,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        let diffuse_pigment = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("diffuse_pigment"),
            layout: Some(&diffuse_layout),
            module: &shader_diffuse_pigment,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        let waves_step = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("waves_step"),
            layout: Some(&waves_layout),
            module: &shader_waves,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
        });
        
        // Render pipeline
        let render_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("render_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        let render_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("render_layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render"),
            layout: Some(&render_layout),
            vertex: VertexState {
                module: &shader_render,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader_render,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Self {
            inject,
            velocity,
            advect_scalar,
            advect_color,
            diffuse_jacobi,
            diffuse_pigment,
            waves_step,
            render,
            params_bind_group,
            injection_bind_group,
            clamp_sampler,
            wrap_sampler,
            params_buffer,
            injection_buffer,
        }
    }
    
    pub fn update_params(&self, queue: &Queue, params: &SimulationParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
    
    pub fn update_injection(&self, queue: &Queue, injection: &InjectionBuffer) {
        queue.write_buffer(&self.injection_buffer, 0, bytemuck::bytes_of(injection));
    }
}
