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
@group(1) @binding(0) var eta: texture_2d<f32>;
@group(1) @binding(1) var uw: texture_2d<f32>;
@group(1) @binding(2) var eta_dst: texture_storage_2d<rgba16float, write>;
@group(1) @binding(3) var uw_dst: texture_storage_2d<rgba16float, write>;
@group(1) @binding(4) var vel_read: texture_2d<f32>;
@group(1) @binding(5) var vel_write: texture_storage_2d<rgba16float, write>;

fn sample_field(coord: vec2<i32>) -> vec4<f32> {
    if params.boundary_mode == 0u {
        let clamped = clamp(coord, vec2<i32>(0), vec2<i32>(i32(params.n) - 1));
        return textureLoad(eta, clamped, 0);
    } else {
        let wrapped = vec2<i32>(coord.x % i32(params.n), coord.y % i32(params.n));
        return textureLoad(eta, wrapped, 0);
    }
}

fn sample_uw(coord: vec2<i32>) -> vec4<f32> {
    if params.boundary_mode == 0u {
        let clamped = clamp(coord, vec2<i32>(0), vec2<i32>(i32(params.n) - 1));
        return textureLoad(uw, clamped, 0);
    } else {
        let wrapped = vec2<i32>(coord.x % i32(params.n), coord.y % i32(params.n));
        return textureLoad(uw, wrapped, 0);
    }
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.n || id.y >= params.n {
        return;
    }
    
    if params.wave_enabled == 0u {
        return;
    }
    
    let idx = vec2<i32>(id.xy);
    let dx = params.dx;
    
    // Step 1: Update uw from eta
    let eta_center = textureLoad(eta, idx, 0).x;
    let eta_left = sample_field(idx - vec2<i32>(1, 0)).x;
    let eta_right = sample_field(idx + vec2<i32>(1, 0)).x;
    let eta_down = sample_field(idx - vec2<i32>(0, 1)).x;
    let eta_up = sample_field(idx + vec2<i32>(0, 1)).x;
    
    let grad_eta_x = (eta_right - eta_left) / (2.0 * dx);
    let grad_eta_y = (eta_up - eta_down) / (2.0 * dx);
    
    let uw_old = textureLoad(uw, idx, 0).xy;
    let uw_new = uw_old + params.dt * (-params.wave_g * vec2<f32>(grad_eta_x, grad_eta_y) - params.wave_r * uw_old);
    
    textureStore(uw_dst, idx, vec4<f32>(uw_new, 0.0, 1.0));
    
    // Step 2: Update eta from uw divergence
    let uw_center = uw_new;
    let uw_left = sample_uw(idx - vec2<i32>(1, 0)).xy;
    let uw_right = sample_uw(idx + vec2<i32>(1, 0)).xy;
    let uw_down = sample_uw(idx - vec2<i32>(0, 1)).xy;
    let uw_up = sample_uw(idx + vec2<i32>(0, 1)).xy;
    
    let div_uw_x = (uw_right.x - uw_left.x) / (2.0 * dx);
    let div_uw_y = (uw_up.y - uw_down.y) / (2.0 * dx);
    let div_uw = div_uw_x + div_uw_y;
    
    let eta_new = eta_center + params.dt * (-params.wave_h * div_uw);
    
    textureStore(eta_dst, idx, vec4<f32>(eta_new, 0.0, 0.0, 1.0));
    
    // Step 3: Add to velocity
    let vel_old = textureLoad(vel_read, idx, 0);
    let vel_new = vel_old.xy + params.wave_beta * uw_new;
    textureStore(vel_write, idx, vec4<f32>(vel_new, 0.0, 1.0));
}
