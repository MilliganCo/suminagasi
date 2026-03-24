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

fn sample_field(coord: vec2<i32>) -> vec4<f32> {
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
    
    let d = params.d_pig;
    let a = params.dt * d / (dx * dx);
    
    // Read neighbors
    let center = textureLoad(src, idx, 0);
    let left = sample_field(idx - vec2<i32>(1, 0));
    let right = sample_field(idx + vec2<i32>(1, 0));
    let down = sample_field(idx - vec2<i32>(0, 1));
    let up = sample_field(idx + vec2<i32>(0, 1));
    
    // Jacobi iteration for RGB channels separately
    let sum_neighbors_r = left.r + right.r + down.r + up.r;
    let sum_neighbors_g = left.g + right.g + down.g + up.g;
    let sum_neighbors_b = left.b + right.b + down.b + up.b;
    
    let new_r = (center.r + a * sum_neighbors_r) / (1.0 + 4.0 * a);
    let new_g = (center.g + a * sum_neighbors_g) / (1.0 + 4.0 * a);
    let new_b = (center.b + a * sum_neighbors_b) / (1.0 + 4.0 * a);
    
    textureStore(dst, idx, vec4<f32>(new_r, new_g, new_b, 1.0));
}
