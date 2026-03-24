@group(0) @binding(0) var pigment: texture_2d<f32>;
@group(0) @binding(1) var sampler_linear: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Fullscreen triangle
    var out: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vertex_index & 2u) * 2.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(pigment, sampler_linear, in.uv).rgb;
    
    // Gamma correction
    let gamma = 1.0 / 2.2;
    let corrected = pow(color, vec3<f32>(gamma));
    
    return vec4<f32>(corrected, 1.0);
}
