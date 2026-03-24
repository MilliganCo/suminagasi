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
@group(1) @binding(1) var dst: texture_storage_2d<rgba16float, write>;

fn sample_field(coord: vec2<i32>, is_scalar: bool) -> vec4<f32> {
    if params.boundary_mode == 0u {
        // CLAMP
        let clamped = clamp(coord, vec2<i32>(0), vec2<i32>(i32(params.n) - 1));
        return textureLoad(src, clamped, 0);
    } else {
        // WRAP
        let wrapped = vec2<i32>(coord.x % i32(params.n), coord.y % i32(params.n));
        return textureLoad(src, wrapped, 0);
    }
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.n || id.y >= params.n {
        return;
    }
    
    let idx = vec2<i32>(id.xy);
    let dx = params.dx;
    
    // Determine diffusion coefficient based on which field we're diffusing
    // This is a bit hacky - we'll use a push constant or separate shader variants
    // For now, assume we're diffusing surfactant (larger D)
    let d = params.d_surf;
    let a = params.dt * d / (dx * dx);
    
    // Read neighbors
    let center = textureLoad(src, idx, 0);
    let left = sample_field(idx - vec2<i32>(1, 0), true);
    let right = sample_field(idx + vec2<i32>(1, 0), true);
    let down = sample_field(idx - vec2<i32>(0, 1), true);
    let up = sample_field(idx + vec2<i32>(0, 1), true);
    
    // Jacobi iteration: f_new = (f_adv + a*sum_neighbors) / (1 + 4a)
    // For scalar field (surfactant)
    let sum_neighbors = left.x + right.x + down.x + up.x;
    let new_value = (center.x + a * sum_neighbors) / (1.0 + 4.0 * a);
    
    textureStore(dst, idx, vec4<f32>(new_value, 0.0, 0.0, 1.0));
}
