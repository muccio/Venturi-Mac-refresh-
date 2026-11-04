// Shadertoy runner template for vv-render.

struct ShadertoyUniforms {
    // x: width in pixels, y: height in pixels, z: aspect (w/h), w: timeline_frame
    resolution: vec4<f32>,
    // x: iTime in seconds, y: iTimeDelta, z: iFrame (as f32), w: iFrameRate
    time: vec4<f32>,
    // xy: mouse cursor position in pixels, zw: mouse click position in pixels
    mouse: vec4<f32>,
    // x: iAudioLevel (RMS), y: iBass, z: iMids, w: iTreble
    audio: vec4<f32>,
    // x: iBeat (transient beat pulse, 1.0 down to 0.0), y: param0, z: param1, w: param2
    beat_params: vec4<f32>,
    // x: param3, y, z, w: unused
    extra_params: vec4<f32>,
};

@group(0) @binding(0) var channel0_tex: texture_2d<f32>;
@group(0) @binding(1) var channel0_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: ShadertoyUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    var out: VertexOutput;
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

// Samples the input video/texture (iChannel0) with UV coordinates (0,0 bottom-left to 1,1 top-right).
fn sample_channel0(uv: vec2<f32>) -> vec4<f32> {
    // Invert Y because WebGPU texture coordinates have (0,0) at top-left, whereas Shadertoy UVs have (0,0) at bottom-left.
    let tex_uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return textureSample(channel0_tex, channel0_sampler, tex_uv);
}

// Fallback / standard entry point wrapper
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Compute frag_coord with (0,0) at bottom-left, matching Shadertoy standard.
    let frag_coord = vec2<f32>(in.uv.x * uniforms.resolution.x, (1.0 - in.uv.y) * uniforms.resolution.y);
    return main_image(frag_coord);
}
