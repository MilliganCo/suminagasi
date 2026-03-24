use wgpu::*;

pub fn create_simulation_texture(
    device: &Device,
    size: u32,
    label: Option<&str>,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label,
        size: Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

pub fn create_texture_view(texture: &Texture) -> TextureView {
    texture.create_view(&TextureViewDescriptor::default())
}

pub fn create_sampler(device: &Device) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..Default::default()
    })
}

pub fn create_wrap_sampler(device: &Device) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..Default::default()
    })
}
