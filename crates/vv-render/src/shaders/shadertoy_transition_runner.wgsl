// Shadertoy transition runner template for vv-render.

struct TransitionUniforms {
    // x: width in pixels, y: height in pixels, z: aspect (w/h), w: timeline_frame
    resolution: vec4<f32>,
    // x: progress [0.0, 1.0], y: aspect (w/h), z: iTime (seconds), w: fps
    transition: vec4<f32>,
    // params0: param0, param1, param2, param3
    params0: vec4<f32>,
    // params1: param4, param5, param6, param7
    params1: vec4<f32>,
    // audio: x: RMS, y: Bass, z: Mids, w: Treble
    audio: vec4<f32>,
    // beat_params: x: iBeat (transient beat pulse, 1.0 down to 0.0), y, z, w: unused
    beat_params: vec4<f32>,
};

@group(0) @binding(0) var channel0_tex: texture_2d<f32>;
@group(0) @binding(1) var channel1_tex: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;
@group(0) @binding(3) var<uniform> uniforms: TransitionUniforms;

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

// Samples channel0 (Input A / from) in normalized Shadertoy UV (0,0 bottom-left to 1,1 top-right).
fn sample_channel0(uv: vec2<f32>) -> vec4<f32> {
    let tex_uv = vec2<f32>(clamp(uv.x, 0.0, 1.0), clamp(1.0 - uv.y, 0.0, 1.0));
    return textureSample(channel0_tex, image_sampler, tex_uv);
}

// Samples channel1 (Input B / to) in normalized Shadertoy UV (0,0 bottom-left to 1,1 top-right).
fn sample_channel1(uv: vec2<f32>) -> vec4<f32> {
    let tex_uv = vec2<f32>(clamp(uv.x, 0.0, 1.0), clamp(1.0 - uv.y, 0.0, 1.0));
    return textureSample(channel1_tex, image_sampler, tex_uv);
}

// Bounds-checked sampling for Input A: returns transparent black outside [0, 1] range.
fn get_from_color(uv: vec2<f32>) -> vec4<f32> {
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return sample_channel0(uv);
}

// Bounds-checked sampling for Input B: returns transparent black outside [0, 1] range.
fn get_to_color(uv: vec2<f32>) -> vec4<f32> {
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return sample_channel1(uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Compute frag_coord with (0,0) at bottom-left, matching Shadertoy standard.
    let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let progress = clamp(uniforms.transition.x, 0.0, 1.0);
    return transition(uv, progress);
}
