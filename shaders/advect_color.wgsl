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

@group(0) @binding(0) var<uniform> params: SimulationParams;
@group(1) @binding(0) var src: texture_2d<f32>;
@group(1) @binding(1) var vel: texture_2d<f32>;
@group(1) @binding(2) var sampler_linear: sampler;
@group(1) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

fn wrap_uv(uv: vec2<f32>) -> vec2<f32> {
    if params.boundary_mode == 1u {
        return fract(uv);
    } else {
        return clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    }
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.n || id.y >= params.n {
        return;
    }
    
    let idx = vec2<i32>(id.xy);
    let uv = vec2<f32>(id.xy) / f32(params.n);
    
    // Read velocity at current position
    let vel_texel = textureLoad(vel, idx, 0);
    let u = vel_texel.xy;
    
    // Backtrace: x0 = x - dt * u
    let uv_back = uv - params.dt * u;
    let uv_back_wrapped = wrap_uv(uv_back);
    
    // Sample using bilinear interpolation
    let color = textureSampleLevel(src, sampler_linear, uv_back_wrapped, 0.0).rgb;
    
    textureStore(dst, idx, vec4<f32>(color, 1.0));
}
