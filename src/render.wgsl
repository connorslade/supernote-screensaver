var<push_constant> ctx: Uniform;

@group(0) @binding(0) var texture: binding_array<texture_2d<f32>>;
@group(0) @binding(1) var texture_sampler: sampler;

struct Uniform {
    // all in px
    viewport: vec2f,
    position: vec2f,
    size: vec2f,

    // texture array index
    n: u32
}

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) uv: vec2f,
}

@vertex
fn vert(
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
) -> VertexOutput {
    let scale = ctx.size / ctx.viewport;
    let translate = ctx.position / ctx.viewport - vec2f(1.0);
    let clip = vec4f(pos.xy * scale + translate, pos.z, pos.w);
    return VertexOutput(clip, uv * vec2f(1.0, -1.0));
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let sample = textureSample(texture[ctx.n], texture_sampler, in.uv).x;
    return vec4f(vec3f(sample), 1.0);
}
