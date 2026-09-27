@group(0) @binding(0) var texture: binding_array<texture_2d<f32>>;
@group(0) @binding(1) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) uv: vec2f,
}

@vertex
fn vert(
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
) -> VertexOutput {
    return VertexOutput(pos, uv * vec2f(1.0, -1.0));
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let sample = textureSample(texture[u32(in.uv.x)], texture_sampler, fract(in.uv)).x;
    return vec4f(vec3f(sample), 1.0);
}
