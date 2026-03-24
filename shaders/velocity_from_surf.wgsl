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
@group(1) @binding(0) var surf: texture_2d<f32>;
@group(1) @binding(1) var vel_dst: texture_storage_2d<rgba16float, write>;

fn sample_surf(coord: vec2<i32>) -> f32 {
    if params.boundary_mode == 0u {
        // CLAMP
        let clamped = clamp(coord, vec2<i32>(0), vec2<i32>(i32(params.n) - 1));
        return textureLoad(surf, clamped, 0).x;
    } else {
        // WRAP
        let wrapped = vec2<i32>(coord.x % i32(params.n), coord.y % i32(params.n));
        return textureLoad(surf, wrapped, 0).x;
    }
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.n || id.y >= params.n {
        return;
    }
    
    let idx = vec2<i32>(id.xy);
    let dx = params.dx;
    
    // Central differences for gradient
    let gamma_l = sample_surf(idx - vec2<i32>(1, 0));
    let gamma_r = sample_surf(idx + vec2<i32>(1, 0));
    let gamma_d = sample_surf(idx - vec2<i32>(0, 1));
    let gamma_u = sample_surf(idx + vec2<i32>(0, 1));
    
    let grad_x = (gamma_r - gamma_l) / (2.0 * dx);
    let grad_y = (gamma_u - gamma_d) / (2.0 * dx);
    
    // Marangoni velocity: u_mar = -αE * ∇Γ
    var u_mar = -params.alpha_e * vec2<f32>(grad_x, grad_y);
    
    // Clamp velocity magnitude
    let speed = length(u_mar);
    if speed > params.u_max {
        u_mar *= params.u_max / speed;
    }
    
    textureStore(vel_dst, idx, vec4<f32>(u_mar, 0.0, 1.0));
}
