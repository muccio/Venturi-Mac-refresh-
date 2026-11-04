//! Built-in Shadertoy shader presets (generators and effects).

pub const PLASMA: &str = r#"fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = (frag_coord - 0.5 * uniforms.resolution.xy) / uniforms.resolution.y;
    let speed = uniforms.beat_params.y; // param0: speed
    let t = uniforms.time.x * select(speed, 1.0, speed == 0.0);
    let v1 = sin(uv.x * 10.0 + t);
    let v2 = sin(uv.y * 10.0 + t * 1.2);
    let v3 = sin((uv.x + uv.y) * 10.0 + t * 0.8);
    let r = sin(length(uv) * 12.0 + t * 2.0);
    let c = sin(v1 + v2 + v3 + r + uniforms.audio.x * 4.0);
    let col = vec3<f32>(
        0.5 + 0.5 * sin(c + 0.0),
        0.5 + 0.5 * sin(c + 2.0 + uniforms.audio.y * 2.0),
        0.5 + 0.5 * sin(c + 4.0 + uniforms.audio.w * 2.0)
    );
    return vec4<f32>(col, 1.0);
}
"#;

pub const CYBERPUNK_GRID: &str = r#"fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = (frag_coord - 0.5 * uniforms.resolution.xy) / uniforms.resolution.y;
    var col = vec3<f32>(0.05, 0.02, 0.1);

    // Sun on horizon
    let sun_pos = vec2<f32>(0.0, 0.1);
    let sun_dist = length(uv - sun_pos);
    if (sun_dist < 0.25) {
        let grad = (uv.y - sun_pos.y + 0.25) / 0.5;
        let sun_col = mix(vec3<f32>(1.0, 0.1, 0.5), vec3<f32>(1.0, 0.8, 0.1), grad);
        let stripes = sin(uv.y * 60.0);
        if (stripes > -0.2 || uv.y > sun_pos.y) {
            col = sun_col * (1.0 + uniforms.audio.y * 0.8);
        }
    }

    // Horizon grid
    if (uv.y < 0.0) {
        let p_z = -0.3 / uv.y;
        let p_x = uv.x * p_z;
        let grid_x = abs(fract(p_x * 2.0) - 0.5);
        let grid_z = abs(fract(p_z - uniforms.time.x * 3.0) - 0.5);
        let line = smoothstep(0.08, 0.0, min(grid_x, grid_z));
        let neon = vec3<f32>(0.1, 0.8, 1.0) * (1.0 + uniforms.audio.w);
        col = mix(col, neon, line * min(1.0, 2.0 / p_z));
    }
    return vec4<f32>(col, 1.0);
}
"#;

pub const RAYMARCHING: &str = r#"fn sdf_sphere(p: vec3<f32>, s: f32) -> f32 {
    return length(p) - s;
}

fn map_scene(p: vec3<f32>) -> f32 {
    let d1 = sdf_sphere(p, 0.8 + uniforms.audio.y * 0.3);
    let d2 = length(vec2<f32>(length(p.xz) - 1.2, p.y)) - 0.2;
    return min(d1, d2);
}

fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = (frag_coord - 0.5 * uniforms.resolution.xy) / uniforms.resolution.y;
    let ro = vec3<f32>(sin(uniforms.time.x * 0.5) * 3.0, 1.5, cos(uniforms.time.x * 0.5) * 3.0);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(ww, vec3<f32>(0.0, 1.0, 0.0)));
    let vv = normalize(cross(uu, ww));
    let rd = normalize(uv.x * uu + uv.y * vv + 1.5 * ww);

    var t = 0.0;
    var d = 0.0;
    for (var i = 0; i < 48; i = i + 1) {
        let p = ro + rd * t;
        d = map_scene(p);
        if (d < 0.002 || t > 20.0) { break; }
        t = t + d;
    }

    if (t < 20.0) {
        let p = ro + rd * t;
        let e = vec2<f32>(0.002, 0.0);
        let n = normalize(vec3<f32>(
            map_scene(p + e.xyy) - map_scene(p - e.xyy),
            map_scene(p + e.yxy) - map_scene(p - e.yxy),
            map_scene(p + e.yyx) - map_scene(p - e.yyx)
        ));
        let diff = max(dot(n, vec3<f32>(0.577, 0.577, 0.577)), 0.0);
        let col = mix(vec3<f32>(0.1, 0.4, 0.9), vec3<f32>(0.9, 0.2, 0.5), n.y * 0.5 + 0.5) * (diff + 0.2);
        return vec4<f32>(col, 1.0);
    }
    return vec4<f32>(0.02, 0.02, 0.05, 1.0);
}
"#;

pub const RIPPLE_WARP: &str = r#"fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = frag_coord / uniforms.resolution.xy;
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(uv - center);
    let wave = sin(dist * 30.0 - uniforms.time.x * 5.0) * (0.02 + uniforms.audio.x * 0.05);
    let dir = select(vec2<f32>(0.0, 0.0), normalize(uv - center), dist > 0.0001);
    let warped_uv = uv + dir * wave;
    return sample_channel0(warped_uv);
}
"#;

pub const CRT_SCANLINES: &str = r#"fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    var uv = frag_coord / uniforms.resolution.xy;
    // Barrel distortion
    let dc = uv - vec2<f32>(0.5, 0.5);
    uv = uv + dc * dot(dc, dc) * 0.15;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    // Chromatic aberration
    let r = sample_channel0(uv + vec2<f32>(0.003, 0.0)).r;
    let g = sample_channel0(uv).g;
    let b = sample_channel0(uv - vec2<f32>(0.003, 0.0)).b;
    var col = vec3<f32>(r, g, b);

    // Scanlines
    let scanline = sin(frag_coord.y * 2.5) * 0.12;
    col = col - scanline;

    // Vignette
    let vig = 16.0 * uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    col = col * clamp(pow(vig, 0.25), 0.0, 1.0);
    return vec4<f32>(col, 1.0);
}
"#;

pub const VHS_GLITCH: &str = r#"fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    var uv = frag_coord / uniforms.resolution.xy;
    let glitch_trigger = uniforms.beat_params.x; // iBeat
    let line_noise = sin(uv.y * 100.0 + uniforms.time.x * 20.0);
    if (abs(line_noise) > 0.95 || glitch_trigger > 0.5) {
        uv.x = uv.x + (sin(uv.y * 20.0 + uniforms.time.x * 50.0) * 0.03 * (1.0 + glitch_trigger));
    }
    let r = sample_channel0(uv + vec2<f32>(0.005, 0.0)).r;
    let g = sample_channel0(uv).g;
    let b = sample_channel0(uv - vec2<f32>(0.005, 0.0)).b;
    return vec4<f32>(r, g, b, 1.0);
}
"#;

pub const AUDIO_VISUALIZER: &str = r#"fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = frag_coord / uniforms.resolution.xy;
    var col = vec3<f32>(0.02, 0.02, 0.06);
    let bar_idx = i32(uv.x * 16.0);
    let bar_x = fract(uv.x * 16.0);

    let height = select(
        uniforms.audio.y,
        select(uniforms.audio.z, uniforms.audio.w, bar_idx > 8),
        bar_idx > 4
    ) * 0.8 + 0.05;

    if (bar_x > 0.15 && bar_x < 0.85 && uv.y < height) {
        let grad = uv.y / height;
        let bar_col = mix(vec3<f32>(0.1, 0.9, 0.4), vec3<f32>(1.0, 0.2, 0.3), grad);
        col = bar_col * (1.0 + uniforms.beat_params.x * 0.5);
    }
    return vec4<f32>(col, 1.0);
}
"#;
