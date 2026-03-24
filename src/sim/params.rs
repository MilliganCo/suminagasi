use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SimulationParams {
    pub dt: f32,
    pub dx: f32,
    pub n: u32,
    pub alpha_e: f32,        // αE combined Marangoni scale
    pub u_max: f32,          // velocity clamp
    pub d_surf: f32,         // surfactant diffusion
    pub d_pig: f32,          // pigment diffusion
    pub jacobi_surf: u32,    // Jacobi iterations for surfactant
    pub jacobi_pig: u32,     // Jacobi iterations for pigment
    pub substeps: u32,       // M substeps per frame
    
    // Wave parameters (optional)
    pub wave_enabled: u32,   // 0 or 1
    pub wave_h: f32,         // depth
    pub wave_g: f32,         // gravity
    pub wave_r: f32,         // damping
    pub wave_beta: f32,      // coupling strength
    
    // Boundary conditions: 0 = CLAMP, 1 = WRAP
    pub boundary_mode: u32,
    
    // Padding to align to 16 bytes
    pub _padding: [f32; 2],
}

impl Default for SimulationParams {
    fn default() -> Self {
        let n = 1024;
        Self {
            dt: 1.0 / 60.0,
            dx: 1.0 / n as f32,
            n,
            alpha_e: 0.25,
            u_max: 1.0,
            d_surf: 0.0005,
            d_pig: 0.0001,
            jacobi_surf: 15,
            jacobi_pig: 10,
            substeps: 2,
            wave_enabled: 0,
            wave_h: 0.02,
            wave_g: 9.8,
            wave_r: 1.0,
            wave_beta: 0.05,
            boundary_mode: 0, // CLAMP by default
            _padding: [0.0; 2],
        }
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InjectionEvent {
    pub pos: [f32; 2],           // normalized [0,1] - 8 bytes
    pub radius: f32,             // normalized radius - 4 bytes
    pub surf_amp: f32,           // surfactant amplitude - 4 bytes
    pub pigment_amp: f32,        // pigment amplitude - 4 bytes
    pub _pad0: [u32; 3],         // padding to align pigment_rgb to 16 bytes
    pub pigment_rgb: [f32; 3],   // RGB color - 12 bytes
    pub event_type: u32,         // 0 = ink, 1 = dispersant - 4 bytes
    pub _padding: [u32; 4],      // 16 bytes
    pub _padding2: [u32; 3],     // 12 bytes
    pub _pad1: u32,              // struct size to 80 bytes (WGSL host-shareable)
}

impl Default for InjectionEvent {
    fn default() -> Self {
        Self {
            pos: [0.5, 0.5],
            radius: 0.02,
            surf_amp: 1.0,
            pigment_amp: 0.5,
            _pad0: [0; 3],
            pigment_rgb: [0.0, 0.0, 0.0],
            event_type: 0,
            _padding: [0; 4],
            _padding2: [0; 3],
            _pad1: 0,
        }
    }
}

pub const MAX_INJECTION_EVENTS: usize = 64;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InjectionBuffer {
    pub events: [InjectionEvent; MAX_INJECTION_EVENTS],  // 64 * 80 = 5120 bytes
    pub count: u32,                                       // 4 bytes
    pub _pad0: [u32; 3],                                 // 12 bytes (align to 16)
    pub _padding: [u32; 4],                              // 16 bytes (WGSL vec3 + implicit padding)
    pub _extra_padding: [u32; 4],                        // 16 bytes
    pub _extra_padding2: [u32; 4],                       // 16 bytes
    pub _final_padding: [u32; 2],                        // 8 bytes
    pub _final_padding2: [u32; 32],                      // 128 bytes
    pub _final_padding3: [u32; 32],                      // 128 bytes
    pub _final_padding4: [u32; 32],                      // 128 bytes
    pub _final_padding5: [u32; 32],                      // 128 bytes
    pub _final_padding6: [u32; 32],                      // 128 bytes
    pub _final_padding7: [u32; 32],                      // 128 bytes
    pub _final_padding8: [u32; 30],                      // 120 bytes (total: 6080 bytes)
}

impl Default for InjectionBuffer {
    fn default() -> Self {
        Self {
            events: [InjectionEvent::default(); MAX_INJECTION_EVENTS],
            count: 0,
            _pad0: [0; 3],
            _padding: [0; 4],
            _extra_padding: [0; 4],
            _extra_padding2: [0; 4],
            _final_padding: [0; 2],
            _final_padding2: [0; 32],
            _final_padding3: [0; 32],
            _final_padding4: [0; 32],
            _final_padding5: [0; 32],
            _final_padding6: [0; 32],
            _final_padding7: [0; 32],
            _final_padding8: [0; 30],
        }
    }
}
