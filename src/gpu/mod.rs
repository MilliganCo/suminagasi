pub mod texture;
pub mod pipelines;

use wgpu::*;
use winit::window::Window;
use anyhow::Result;

pub struct GpuContext<'window> {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'window>,
    pub surface_config: SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> GpuContext<'window> {
    pub async fn new(window: &'window Window) -> Result<Self> {
        let size = window.inner_size();
        
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window)?;
        
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to get adapter"))?;
        
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await?;
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &surface_config);
        
        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            size,
        })
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
    
    pub fn get_current_texture(&mut self) -> Result<SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}
