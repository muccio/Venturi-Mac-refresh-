struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct P5Uniforms {
    resolution: vec2<f32>,
    time: f32,
    fps: f32,
    params: vec4<f32>,      // param0..param3
    audio: vec4<f32>,       // x: level, y: bass, z: mids, w: treble
    audio_beat: f32,
    preset: u32,            // 0: AsciiColor, 1: AsciiMono, 2: ParticleSampler, 3: WaveMatrix, 4: SpectrumRings, 5: GlitchPixelSort, 6: Custom
    blend_mode: u32,        // 0: Normal, 1: Add, 2: Screen, 3: Multiply, 4: Overlay
    _pad: u32,
};

@group(0) @binding(0) var video_in: texture_2d<f32>;
@group(0) @binding(1) var video_sampler: sampler;
@group(0) @binding(2) var<uniform> u: P5Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index & 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Procedural 3x5 font density mask for ASCII characters
// DENSITY = " .:-=+*#%@"
fn ascii_glyph_mask(char_idx: i32, local_uv: vec2<f32>) -> f32 {
    let p = local_uv;
    // Margins
    if (p.x < 0.1 || p.x > 0.9 || p.y < 0.1 || p.y > 0.9) {
        return 0.0;
    }
    let px = p.x;
    let py = p.y;
    let cx = abs(px - 0.5);
    let cy = abs(py - 0.5);

    switch char_idx {
        case 0: { // ' '
            return 0.0;
        }
        case 1: { // '.'
            return select(0.0, 1.0, px > 0.4 && px < 0.6 && py > 0.7 && py < 0.9);
        }
        case 2: { // ':'
            let dot_t = px > 0.4 && px < 0.6 && py > 0.25 && py < 0.45;
            let dot_b = px > 0.4 && px < 0.6 && py > 0.65 && py < 0.85;
            return select(0.0, 1.0, dot_t || dot_b);
        }
        case 3: { // '-'
            return select(0.0, 1.0, cx < 0.35 && cy < 0.08);
        }
        case 4: { // '='
            let bar1 = cx < 0.35 && abs(py - 0.38) < 0.07;
            let bar2 = cx < 0.35 && abs(py - 0.62) < 0.07;
            return select(0.0, 1.0, bar1 || bar2);
        }
        case 5: { // '+'
            let h = cx < 0.35 && cy < 0.08;
            let v = cy < 0.35 && cx < 0.08;
            return select(0.0, 1.0, h || v);
        }
        case 6: { // '*'
            let h = cx < 0.35 && cy < 0.06;
            let v = cy < 0.35 && cx < 0.06;
            let d1 = abs((px - 0.5) - (py - 0.5)) < 0.08 && cx < 0.28 && cy < 0.28;
            let d2 = abs((px - 0.5) + (py - 0.5)) < 0.08 && cx < 0.28 && cy < 0.28;
            return select(0.0, 1.0, h || v || d1 || d2);
        }
        case 7: { // '#'
            let v1 = abs(px - 0.35) < 0.07 && cy < 0.4;
            let v2 = abs(px - 0.65) < 0.07 && cy < 0.4;
            let h1 = abs(py - 0.35) < 0.07 && cx < 0.4;
            let h2 = abs(py - 0.65) < 0.07 && cx < 0.4;
            return select(0.0, 1.0, v1 || v2 || h1 || h2);
        }
        case 8: { // '%'
            let d = abs((px - 0.5) + (py - 0.5)) < 0.07 && cx < 0.38 && cy < 0.38;
            let c1 = length(p - vec2<f32>(0.3, 0.3)) < 0.14;
            let c2 = length(p - vec2<f32>(0.7, 0.7)) < 0.14;
            return select(0.0, 1.0, d || c1 || c2);
        }
        default: { // '@'
            let ring = abs(length(p - vec2<f32>(0.5, 0.5)) - 0.32) < 0.08;
            let center = length(p - vec2<f32>(0.5, 0.5)) < 0.15;
            return select(0.0, 1.0, ring || center);
        }
    }
}

// ITU-R BT.601 luminance
fn luminance(col: vec3<f32>) -> f32 {
    return 0.299 * col.r + 0.587 * col.g + 0.114 * col.b;
}

// Blend modes
fn apply_blend(base: vec4<f32>, overlay: vec4<f32>, mode: u32) -> vec4<f32> {
    switch mode {
        case 1u: { // Add
            return vec4<f32>(base.rgb + overlay.rgb * overlay.a, base.a);
        }
        case 2u: { // Screen
            let s = 1.0 - (1.0 - base.rgb) * (1.0 - overlay.rgb * overlay.a);
            return vec4<f32>(s, base.a);
        }
        case 3u: { // Multiply
            let m = mix(base.rgb, base.rgb * overlay.rgb, overlay.a);
            return vec4<f32>(m, base.a);
        }
        case 4u: { // Overlay
            let lo = 2.0 * base.rgb * overlay.rgb;
            let hi = 1.0 - 2.0 * (1.0 - base.rgb) * (1.0 - overlay.rgb);
            let ov = select(hi, lo, base.rgb < vec3<f32>(0.5));
            return vec4<f32>(mix(base.rgb, ov, overlay.a), base.a);
        }
        default: { // Normal
            return vec4<f32>(mix(base.rgb, overlay.rgb, overlay.a), base.a);
        }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let frag_coord = uv * u.resolution;
    let base_video = textureSample(video_in, video_sampler, uv);

    var canvas_out = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    switch u.preset {
        case 0u: { // AsciiColor (Full Color ASCII)
            var step = floor(mix(6.0, 28.0, clamp(u.params.x, 0.0, 1.0)));
            if (u.audio.x > 0.05) {
                step = max(4.0, step - u.audio.x * u.params.w * 8.0);
            }
            let cell_origin = floor(frag_coord / step) * step;
            let local_uv = (frag_coord - cell_origin) / step;
            let sample_uv = (cell_origin + 0.5 * step) / u.resolution;
            let cell_col = textureSample(video_in, video_sampler, sample_uv);

            var lum = luminance(cell_col.rgb);
            lum = pow(lum, 1.0 / max(0.2, u.params.y * 1.5));
            let char_idx = clamp(i32(lum * 9.0), 0, 9);
            let mask = ascii_glyph_mask(char_idx, local_uv);

            // Boost saturation and contrast
            let sat_boost = u.params.z * 1.4;
            let gray = vec3<f32>(lum);
            let final_rgb = mix(gray, cell_col.rgb, sat_boost);
            canvas_out = vec4<f32>(final_rgb * mask, 1.0);
        }

        case 1u: { // AsciiMono (Monochrome B&W ASCII)
            var step = floor(mix(6.0, 24.0, clamp(u.params.x, 0.0, 1.0)));
            var jitter = 0.0;
            if (u.audio_beat > 0.1) {
                jitter = (sin(frag_coord.y * 0.1 + u.time * 20.0) * u.audio_beat * u.params.w * 6.0);
            }
            let coord_j = vec2<f32>(frag_coord.x + jitter, frag_coord.y);
            let cell_origin = floor(coord_j / step) * step;
            let local_uv = (coord_j - cell_origin) / step;
            let sample_uv = (cell_origin + 0.5 * step) / u.resolution;
            let cell_col = textureSample(video_in, video_sampler, sample_uv);

            var lum = luminance(cell_col.rgb);
            let is_light = u.params.z > 0.5;
            if (is_light) {
                lum = 1.0 - lum;
            }
            lum = pow(lum, max(0.1, u.params.y * 1.6));
            let char_idx = clamp(i32(lum * 9.0), 0, 9);
            let mask = ascii_glyph_mask(char_idx, local_uv);

            if (is_light) {
                let text_col = vec3<f32>(1.0 - mask);
                canvas_out = vec4<f32>(text_col, 1.0);
            } else {
                let text_col = vec3<f32>(mask);
                canvas_out = vec4<f32>(text_col, 1.0);
            }
        }

        case 2u: { // ParticleSampler (Particle Video Sampler)
            let step = floor(mix(8.0, 32.0, clamp(u.params.x, 0.0, 1.0)));
            let max_r = mix(2.0, 24.0, clamp(u.params.y, 0.0, 1.0));
            let audio_boost = 1.0 + (u.audio.y * u.params.w * 2.5);

            let cell_origin = floor(frag_coord / step) * step;
            let cell_center = cell_origin + 0.5 * step;
            let sample_uv = cell_center / u.resolution;
            let cell_col = textureSample(video_in, video_sampler, sample_uv);

            let lum = luminance(cell_col.rgb);
            if (lum < u.params.z) {
                canvas_out = vec4<f32>(0.04, 0.05, 0.07, 1.0);
            } else {
                let r = max(1.0, lum * max_r * audio_boost);
                let dist = length(frag_coord - cell_center);
                let circle_mask = smoothstep(r + 0.5, r - 0.5, dist);
                let bg = vec3<f32>(0.04, 0.05, 0.07);
                canvas_out = vec4<f32>(mix(bg, cell_col.rgb, circle_mask), 1.0);
            }
        }

        case 3u: { // WaveMatrix (Generative Wave Matrix)
            canvas_out = base_video;
            let count = 16.0;
            let freq = mix(1.0, 20.0, u.params.x);
            let amp = mix(10.0, 120.0, u.params.y) + u.audio.y * u.params.w * 90.0;
            let t = u.time * 2.0 * u.params.z;

            var wave_acc = vec3<f32>(0.0);
            for (var i = 0.0; i < count; i += 1.0) {
                let y_base = (u.resolution.y / (count + 1.0)) * (i + 1.0);
                let wave_y = y_base + sin(uv.x * freq * 6.28318 + t + i * 0.4) * amp;
                let d = abs(frag_coord.y - wave_y);
                let glow = exp(-d * 0.4);
                let wave_col = vec3<f32>(
                    0.5 + 0.5 * sin(t + i * 0.3),
                    0.7 + 0.3 * cos(t + i * 0.2),
                    1.0
                );
                wave_acc += wave_col * glow * 1.5;
            }
            canvas_out = vec4<f32>(base_video.rgb + wave_acc, base_video.a);
        }

        case 4u: { // SpectrumRings (Audio Spectrum Rings)
            canvas_out = base_video;
            let center = u.resolution * 0.5;
            let delta = frag_coord - center;
            let r = length(delta);
            var angle = atan2(delta.y, delta.x);
            if (angle < 0.0) {
                angle += 6.2831853;
            }
            let rot = u.time * u.params.z * 1.5;
            let norm_angle = fract((angle + rot) / 6.2831853);

            let base_r = mix(50.0, 300.0, u.params.x);
            let bar_count = floor(mix(24.0, 96.0, u.params.y));
            let bar_idx = floor(norm_angle * bar_count);
            let bar_center = (bar_idx + 0.5) / bar_count;
            let angle_dist = abs(norm_angle - bar_center) * bar_count;

            let freq_bin = fract(bar_idx / bar_count);
            let audio_val = mix(u.audio.y, u.audio.w, freq_bin) * u.params.w * 2.0 + u.audio.x * 0.5;
            let bar_len = audio_val * 200.0;

            if (r >= base_r && r <= (base_r + bar_len) && angle_dist < 0.35) {
                let bar_col = vec3<f32>(
                    0.2 + 0.8 * freq_bin,
                    0.6 + 0.4 * sin(r * 0.05),
                    0.9
                );
                canvas_out = vec4<f32>(base_video.rgb + bar_col * 2.0, base_video.a);
            }
        }

        case 5u: { // GlitchPixelSort (Pixel Sorter Glitch)
            let thresh = mix(0.1, 0.85, u.params.x) - u.audio_beat * u.params.w * 0.2;
            let is_vert = u.params.y > 0.5;
            let streak = mix(10.0, 90.0, u.params.z);

            var sample_pt = frag_coord;
            let lum = luminance(base_video.rgb);
            if (lum > thresh) {
                let offset = (sin(frag_coord.x * 0.05 + u.time * 10.0) * streak);
                if (is_vert) {
                    sample_pt.y = clamp(sample_pt.y + offset, 0.0, u.resolution.y - 1.0);
                } else {
                    sample_pt.x = clamp(sample_pt.x + offset, 0.0, u.resolution.x - 1.0);
                }
            }
            let sort_col = textureSample(video_in, video_sampler, sample_pt / u.resolution);
            canvas_out = sort_col;
        }

        default: { // Custom / Fallback
            canvas_out = base_video;
        }
    }

    return apply_blend(base_video, canvas_out, u.blend_mode);
}
