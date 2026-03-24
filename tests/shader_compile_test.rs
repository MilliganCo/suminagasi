// Tests to verify shader compilation
// Note: These tests require a GPU context, so they may not run in CI
// but are useful for local development

#[cfg(test)]
mod tests {
    use wgpu::*;
    use pollster;

    fn compile_shader(device: &Device, shader_source: &str, label: &str) -> ShaderModule {
        device.create_shader_module(ShaderModuleDescriptor {
            label: Some(label),
            source: ShaderSource::Wgsl(shader_source.into()),
        })
    }

    #[test]
    #[ignore] // Requires GPU, run with: cargo test -- --ignored
    fn test_shader_compilation() {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = pollster::block_on(
            instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
        ).expect("Failed to get adapter");

        let (device, _queue) = pollster::block_on(
            adapter.request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
        ).expect("Failed to get device");

        // Test that we can create a simple shader
        let simple_shader = r#"
            @compute @workgroup_size(1, 1, 1)
            fn main() {
            }
        "#;

        let _shader = compile_shader(&device, simple_shader, "test_shader");
        // If we get here, the shader compiled successfully
    }
}
