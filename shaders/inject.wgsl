struct SimulationParams {
    dt: f32,
    dx: f32,
    n: u32,
    alpha_e: f32,
    u_max: f32,
    d_surf: f32,
    d_pig: f32,
    jacobi_surf: u32,
    jacobi_pig: u32,
    substeps: u32,
    wave_enabled: u32,
    wave_h: f32,
    wave_g: f32,
    wave_r: f32,
    wave_beta: f32,
    boundary_mode: u32,
    _padding: vec2<f32>,
}

struct InjectionEvent {
    pos: vec2<f32>,          // 8 bytes
    radius: f32,            // 4 bytes
    surf_amp: f32,          // 4 bytes
    pigment_amp: f32,       // 4 bytes
    pigment_rgb: vec3<f32>, // 12 bytes
    event_type: u32,        // 4 bytes
    _padding: vec4<u32>,    // 16 bytes padding
    _padding2: vec3<u32>,   // 12 bytes padding (total: 64 bytes for WGSL array alignment)
}

struct InjectionBuffer {
    events: array<InjectionEvent, 64>,
    count: u32,
    _padding: vec3<u32>,
    _extra_padding: vec4<u32>,
    _extra_padding2: vec4<u32>,
    _final_padding: vec2<u32>,
    _final_padding2: array<u32, 32>,
    _final_padding3: array<u32, 32>,
    _final_padding4: array<u32, 32>,
    _final_padding5: array<u32, 32>,
    _final_padding6: array<u32, 32>,
    _final_padding7: array<u32, 32>,
    _final_padding8: array<u32, 30>,
}

@group(0) @binding(0) var<uniform> params: SimulationParams;
@group(1) @binding(0) var<storage, read> injection: InjectionBuffer;

@group(2) @binding(0) var surf_src: texture_2d<f32>;
@group(2) @binding(1) var pigment_src: texture_2d<f32>;
@group(2) @binding(2) var surf_dst: texture_storage_2d<rgba16float, write>;
@group(2) @binding(3) var pigment_dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.n || id.y >= params.n {
        return;
    }
    
    let uv = vec2<f32>(id.xy) / f32(params.n);
    let texel_size = 1.0 / f32(params.n);
    
    // Read current values
    let surf_texel = textureLoad(surf_src, vec2<i32>(id.xy), 0);
    let pigment_texel = textureLoad(pigment_src, vec2<i32>(id.xy), 0);
    
    var surf_value = surf_texel.x;
    var pigment_value = pigment_texel.rgb;
    
    // Apply injection events
    for (var i = 0u; i < injection.count; i++) {
        let event = injection.events[i];
        let delta_pos = uv - event.pos;
        let r_sq = dot(delta_pos, delta_pos);
        let s_sq = event.radius * event.radius;
        let weight = exp(-r_sq / (2.0 * s_sq));
        
        if event.event_type == 0u {
            // Ink injection
            surf_value += event.surf_amp * weight;
            pigment_value += event.pigment_amp * weight * event.pigment_rgb;
        } else {
            // Dispersant (negative)
            surf_value -= event.surf_amp * weight;
        }
    }
    
    textureStore(surf_dst, vec2<i32>(id.xy), vec4<f32>(surf_value, 0.0, 0.0, 1.0));
    textureStore(pigment_dst, vec2<i32>(id.xy), vec4<f32>(pigment_value, 1.0));
}
