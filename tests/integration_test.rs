// Integration tests for Suminagashi simulation
// These tests verify that the code compiles and basic structures work correctly

#[cfg(test)]
mod tests {
    use suminagasi::sim::params::{SimulationParams, InjectionBuffer, InjectionEvent, MAX_INJECTION_EVENTS};
    use bytemuck;

    #[test]
    fn test_simulation_params_size() {
        // Verify that SimulationParams can be serialized correctly
        let params = SimulationParams::default();
        let bytes = bytemuck::bytes_of(&params);
        assert_eq!(bytes.len(), std::mem::size_of::<SimulationParams>());
    }

    #[test]
    fn test_injection_buffer_size() {
        // Verify that InjectionBuffer can be serialized correctly
        // Expected size: 64 events * 64 bytes + 4 bytes (count) + 12 bytes (padding) + 32 bytes (extra_padding) + 32 bytes (extra_padding2) + 8 bytes (final_padding) + 128*6 + 120 = 5200 bytes
        let buffer = InjectionBuffer::default();
        let bytes = bytemuck::bytes_of(&buffer);
        let size = std::mem::size_of::<InjectionBuffer>();
        assert_eq!(bytes.len(), size);
        assert_eq!(size, 5200, "InjectionBuffer size should be 5200 bytes (64 * 64 + 4 + 12 + 32 + 32 + 8 + 128*6 + 120)");
    }
    
    #[test]
    fn test_injection_event_size() {
        // Verify that InjectionEvent is 64 bytes (aligned to 16 for WGSL array alignment)
        let size = std::mem::size_of::<InjectionEvent>();
        assert_eq!(size, 64, "InjectionEvent size should be 64 bytes (aligned to 16 for WGSL array alignment)");
    }

    #[test]
    fn test_injection_event_count() {
        // Verify that we can add events up to MAX_INJECTION_EVENTS
        let mut buffer = InjectionBuffer::default();
        for i in 0..MAX_INJECTION_EVENTS {
            let event = InjectionEvent {
                pos: [0.5, 0.5],
                radius: 0.01,
                surf_amp: 1.0,
                pigment_amp: 0.5,
                pigment_rgb: [0.0, 0.0, 0.0],
                event_type: 0,
                _padding: [0; 7],
            };
            buffer.events[i] = event;
        }
        buffer.count = MAX_INJECTION_EVENTS as u32;
        
        let bytes = bytemuck::bytes_of(&buffer);
        assert_eq!(bytes.len(), std::mem::size_of::<InjectionBuffer>());
    }

    #[test]
    fn test_default_params() {
        let params = SimulationParams::default();
        assert_eq!(params.n, 1024);
        assert_eq!(params.substeps, 2);
        assert_eq!(params.jacobi_surf, 15);
        assert_eq!(params.jacobi_pig, 10);
        assert_eq!(params.wave_enabled, 0); // Waves disabled by default
    }

    #[test]
    fn test_injection_event_serialization() {
        let event = InjectionEvent {
            pos: [0.3, 0.7],
            radius: 0.05,
            surf_amp: 2.0,
            pigment_amp: 1.0,
            pigment_rgb: [0.5, 0.3, 0.1],
            event_type: 1,
            _padding: [0; 7],
        };
        
        let bytes = bytemuck::bytes_of(&event);
        assert_eq!(bytes.len(), std::mem::size_of::<InjectionEvent>());
        
        // Verify we can cast it back
        let restored: &InjectionEvent = bytemuck::from_bytes(bytes);
        assert_eq!(restored.pos, event.pos);
        assert_eq!(restored.radius, event.radius);
        assert_eq!(restored.surf_amp, event.surf_amp);
    }
}
