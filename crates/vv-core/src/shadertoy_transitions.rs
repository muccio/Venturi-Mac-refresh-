//! 20 Built-in Shadertoy GPU Video Transition Presets.
//!
//! Technical specifications:
//! - Input A: Source video stream (from / iChannel0).
//! - Input B: Destination video stream (to / iChannel1).
//! - UV coordinates: Normalized in [0.0, 1.0].
//! - Progress: float in [0.0, 1.0] where 0.0 is pure Input A and 1.0 is pure Input B.

use serde::{Deserialize, Serialize};

/// 20 Built-in GPU shader transition presets, grouped in 4 computational complexity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShadertoyTransitionPreset {
    // Level 1: Linear & 1D (2 texture taps)
    LinearCrossfade,
    LinearWipe,
    ColorFlashFade,
    RadialClockWipe,
    LumaMatteDissolve,

    // Level 2: 2D Geometric Deformations (2 texture taps, UV remapping)
    PushSlide,
    IrisCircleOpen,
    ZoomScaleInOut,
    MosaicPixelate,
    CheckerboardGridWipe,

    // Level 3: Continuous Deformations & Procedural Noise (2-6 texture taps)
    WaveSineDisplacement,
    SwirlTwirl,
    ChromaticAberrationSlide,
    PageCurl,
    SimplexNoiseBurn,

    // Level 4: Multi-Tap, Convolutions & Heavy Simulation (8-64 texture taps)
    DirectionalMotionBlurWipe,
    ZoomRadialBlur,
    DigitalGlitchSpectralDrift,
    GlassPrismRefraction,
    RaymarchedHeightfieldMorphing,
}

impl ShadertoyTransitionPreset {
    pub const ALL: [Self; 20] = [
        // Level 1
        Self::LinearCrossfade,
        Self::LinearWipe,
        Self::ColorFlashFade,
        Self::RadialClockWipe,
        Self::LumaMatteDissolve,
        // Level 2
        Self::PushSlide,
        Self::IrisCircleOpen,
        Self::ZoomScaleInOut,
        Self::MosaicPixelate,
        Self::CheckerboardGridWipe,
        // Level 3
        Self::WaveSineDisplacement,
        Self::SwirlTwirl,
        Self::ChromaticAberrationSlide,
        Self::PageCurl,
        Self::SimplexNoiseBurn,
        // Level 4
        Self::DirectionalMotionBlurWipe,
        Self::ZoomRadialBlur,
        Self::DigitalGlitchSpectralDrift,
        Self::GlassPrismRefraction,
        Self::RaymarchedHeightfieldMorphing,
    ];

    /// Unique identifier / slug.
    pub fn name(self) -> &'static str {
        match self {
            Self::LinearCrossfade => "linear_crossfade",
            Self::LinearWipe => "linear_wipe",
            Self::ColorFlashFade => "color_flash_fade",
            Self::RadialClockWipe => "radial_clock_wipe",
            Self::LumaMatteDissolve => "luma_matte_dissolve",
            Self::PushSlide => "push_slide",
            Self::IrisCircleOpen => "iris_circle_open",
            Self::ZoomScaleInOut => "zoom_scale_in_out",
            Self::MosaicPixelate => "mosaic_pixelate",
            Self::CheckerboardGridWipe => "checkerboard_grid_wipe",
            Self::WaveSineDisplacement => "wave_sine_displacement",
            Self::SwirlTwirl => "swirl_twirl",
            Self::ChromaticAberrationSlide => "chromatic_aberration_slide",
            Self::PageCurl => "page_curl",
            Self::SimplexNoiseBurn => "simplex_noise_burn",
            Self::DirectionalMotionBlurWipe => "directional_motion_blur_wipe",
            Self::ZoomRadialBlur => "zoom_radial_blur",
            Self::DigitalGlitchSpectralDrift => "digital_glitch_spectral_drift",
            Self::GlassPrismRefraction => "glass_prism_refraction",
            Self::RaymarchedHeightfieldMorphing => "raymarched_heightfield_morphing",
        }
    }

    /// User-facing display title.
    pub fn label(self) -> &'static str {
        match self {
            Self::LinearCrossfade => "Linear Crossfade",
            Self::LinearWipe => "Linear Wipe",
            Self::ColorFlashFade => "Color Flash Fade",
            Self::RadialClockWipe => "Radial / Clock Wipe",
            Self::LumaMatteDissolve => "Luma Matte Dissolve",
            Self::PushSlide => "Push / Slide Direzionale",
            Self::IrisCircleOpen => "Iris / Circle Open",
            Self::ZoomScaleInOut => "Zoom / Scale In-Out",
            Self::MosaicPixelate => "Mosaic / Pixelate Dissolve",
            Self::CheckerboardGridWipe => "Checkerboard / Grid Wipe",
            Self::WaveSineDisplacement => "Wave / Sine Displacement",
            Self::SwirlTwirl => "Swirl / Twirl",
            Self::ChromaticAberrationSlide => "Chromatic Aberration Slide",
            Self::PageCurl => "Page Curl 3D",
            Self::SimplexNoiseBurn => "Simplex Noise Burn",
            Self::DirectionalMotionBlurWipe => "Directional Motion Blur Wipe",
            Self::ZoomRadialBlur => "Zoom / Radial Blur",
            Self::DigitalGlitchSpectralDrift => "Digital Glitch & Spectral Drift",
            Self::GlassPrismRefraction => "Glass Prism & Refraction",
            Self::RaymarchedHeightfieldMorphing => "Raymarched Heightfield Morphing",
        }
    }

    /// Complexity tier (1..=4).
    pub fn level(self) -> u8 {
        match self {
            Self::LinearCrossfade
            | Self::LinearWipe
            | Self::ColorFlashFade
            | Self::RadialClockWipe
            | Self::LumaMatteDissolve => 1,

            Self::PushSlide
            | Self::IrisCircleOpen
            | Self::ZoomScaleInOut
            | Self::MosaicPixelate
            | Self::CheckerboardGridWipe => 2,

            Self::WaveSineDisplacement
            | Self::SwirlTwirl
            | Self::ChromaticAberrationSlide
            | Self::PageCurl
            | Self::SimplexNoiseBurn => 3,

            Self::DirectionalMotionBlurWipe
            | Self::ZoomRadialBlur
            | Self::DigitalGlitchSpectralDrift
            | Self::GlassPrismRefraction
            | Self::RaymarchedHeightfieldMorphing => 4,
        }
    }

    /// Brief description of the transition visual effect.
    pub fn description(self) -> &'static str {
        match self {
            Self::LinearCrossfade => "Dissolvenza lineare trasparente tra sorgente e destinazione.",
            Self::LinearWipe => "Taglio direzionale con angolo e feather regolabili.",
            Self::ColorFlashFade => "Convergenza verso flash cromatico a metà percorso.",
            Self::RadialClockWipe => "Apertura radiale a lancetta di orologio a 360 gradi.",
            Self::LumaMatteDissolve => "Erosione naturale basata sulla mappa di luminanza intrinseca Rec.709.",
            Self::PushSlide => "Scorrimento cartesiano del fotogramma con profili di decelerazione.",
            Self::IrisCircleOpen => "Cerchio che si espande dal centro svelando il video.",
            Self::ZoomScaleInOut => "Crash zoom non lineare con punto focale orientabile.",
            Self::MosaicPixelate => "Quantizzazione spaziale a macro-blocchi dinamici.",
            Self::CheckerboardGridWipe => "Reticolo a scacchiera con soglie di rivelazione pseudo-casuali.",
            Self::WaveSineDisplacement => "Deformazione fluida con onde sinusoidali orizzontali.",
            Self::SwirlTwirl => "Vortice polare ad alta velocità con raggio d'azione regolabile.",
            Self::ChromaticAberrationSlide => "Scomposizione spettrale RGB su asse di movimento cinematografico.",
            Self::PageCurl => "Voltapagina tridimensionale con curvatura cilindrica e ombre morbide.",
            Self::SimplexNoiseBurn => "Combustione organica con fronte luminescente ad alta energia.",
            Self::DirectionalMotionBlurWipe => "Sfocatura di movimento multi-campionamento proporzionale alla velocità.",
            Self::ZoomRadialBlur => "Tunnel prospettico convolutivo con scia radiale ponderata.",
            Self::DigitalGlitchSpectralDrift => "Corruzione digitale con sfasamento a bande, drift e sync drop.",
            Self::GlassPrismRefraction => "Rifrazione ottica frattale di Snell con dispersione cromatica.",
            Self::RaymarchedHeightfieldMorphing => "Raymarching volumetrico SDF estruso da altimetria con illuminazione 3D.",
        }
    }

    /// Default parameters [param0..3, param4..7] matching the technical specifications.
    pub fn default_params(self) -> [f32; 8] {
        match self {
            // [01] LinearCrossfade
            Self::LinearCrossfade => [0.0; 8],
            // [02] LinearWipe: angle = 0.0, feather = 0.05
            Self::LinearWipe => [0.0, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [03] ColorFlashFade: flashColor = (1.0, 1.0, 1.0), curveExponent = 2.0
            Self::ColorFlashFade => [1.0, 1.0, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0],
            // [04] RadialClockWipe: center = (0.5, 0.5), startAngle = 90.0, feather = 0.02
            Self::RadialClockWipe => [0.5, 0.5, 90.0, 0.02, 0.0, 0.0, 0.0, 0.0],
            // [05] LumaMatteDissolve: softness = 0.08, invert = 0.0
            Self::LumaMatteDissolve => [0.08, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [06] PushSlide: direction = (1.0, 0.0), easeCurve = 0 (Linear)
            Self::PushSlide => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [07] IrisCircleOpen: center = (0.5, 0.5), feather = 0.04
            Self::IrisCircleOpen => [0.5, 0.5, 0.04, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [08] ZoomScaleInOut: zoomIntensity = 3.0, center = (0.5, 0.5)
            Self::ZoomScaleInOut => [3.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [09] MosaicPixelate: maxBlockSize = 0.05
            Self::MosaicPixelate => [0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [10] CheckerboardGridWipe: gridDensity = (16.0, 9.0), randomness = 0.8
            Self::CheckerboardGridWipe => [16.0, 9.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [11] WaveSineDisplacement: frequency = 30.0, amplitude = 0.05, speed = 8.0
            Self::WaveSineDisplacement => [30.0, 0.05, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [12] SwirlTwirl: twistStrength = 5.0, radius = 0.8, center = (0.5, 0.5)
            Self::SwirlTwirl => [5.0, 0.8, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
            // [13] ChromaticAberrationSlide: separationStrength = 0.03, direction = (1.0, 0.0)
            Self::ChromaticAberrationSlide => [0.03, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [14] PageCurl: rollAngle = 45.0, cylinderRadius = 0.15, shadowIntensity = 0.6
            Self::PageCurl => [45.0, 0.15, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [15] SimplexNoiseBurn: noiseScale = 6.0, edgeWidth = 0.04, glowColor = (1.0, 0.35, 0.05)
            Self::SimplexNoiseBurn => [6.0, 0.04, 1.0, 0.35, 0.05, 0.0, 0.0, 0.0],
            // [16] DirectionalMotionBlurWipe: samples = 16, blurStrength = 0.08, direction = (1.0, 0.0)
            Self::DirectionalMotionBlurWipe => [16.0, 0.08, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [17] ZoomRadialBlur: samples = 16, blurAmount = 0.2, center = (0.5, 0.5)
            Self::ZoomRadialBlur => [16.0, 0.2, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
            // [18] DigitalGlitchSpectralDrift: blockRows = 40.0, glitchIntensity = 0.7, colorDrift = 0.04
            Self::DigitalGlitchSpectralDrift => [40.0, 0.7, 0.04, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [19] GlassPrismRefraction: glassFacetDensity = 12.0, refractiveIndex = 1.45, chromaticDispersion = 0.03
            Self::GlassPrismRefraction => [12.0, 1.45, 0.03, 0.0, 0.0, 0.0, 0.0, 0.0],
            // [20] RaymarchedHeightfieldMorphing: raySteps = 32, maxExtrusion = 0.2, lightPosition = (0.5, 0.5, -1.0), specularShininess = 16.0
            Self::RaymarchedHeightfieldMorphing => [32.0, 0.2, 0.5, 0.5, -1.0, 16.0, 0.0, 0.0],
        }
    }

    /// The exact WGSL fragment implementation for this preset.
    pub fn wgsl_code(self) -> &'static str {
        match self {
            Self::LinearCrossfade => LINEAR_CROSSFADE,
            Self::LinearWipe => LINEAR_WIPE,
            Self::ColorFlashFade => COLOR_FLASH_FADE,
            Self::RadialClockWipe => RADIAL_CLOCK_WIPE,
            Self::LumaMatteDissolve => LUMA_MATTE_DISSOLVE,
            Self::PushSlide => PUSH_SLIDE,
            Self::IrisCircleOpen => IRIS_CIRCLE_OPEN,
            Self::ZoomScaleInOut => ZOOM_SCALE_IN_OUT,
            Self::MosaicPixelate => MOSAIC_PIXELATE,
            Self::CheckerboardGridWipe => CHECKERBOARD_GRID_WIPE,
            Self::WaveSineDisplacement => WAVE_SINE_DISPLACEMENT,
            Self::SwirlTwirl => SWIRL_TWIRL,
            Self::ChromaticAberrationSlide => CHROMATIC_ABERRATION_SLIDE,
            Self::PageCurl => PAGE_CURL,
            Self::SimplexNoiseBurn => SIMPLEX_NOISE_BURN,
            Self::DirectionalMotionBlurWipe => DIRECTIONAL_MOTION_BLUR_WIPE,
            Self::ZoomRadialBlur => ZOOM_RADIAL_BLUR,
            Self::DigitalGlitchSpectralDrift => DIGITAL_GLITCH_SPECTRAL_DRIFT,
            Self::GlassPrismRefraction => GLASS_PRISM_REFRACTION,
            Self::RaymarchedHeightfieldMorphing => RAYMARCHED_HEIGHTFIELD_MORPHING,
        }
    }

    /// Lookup by slug name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|p| p.name() == name)
    }
}

// ============================================================================
// LEVEL 1: LINEAR & 1D
// ============================================================================

pub const LINEAR_CROSSFADE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let a = get_from_color(uv);
    let b = get_to_color(uv);
    return mix(a, b, progress);
}
"#;

pub const LINEAR_WIPE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let angle_deg = uniforms.params0.x;
    let feather = max(uniforms.params0.y, 0.0001);
    let rad = radians(angle_deg);
    let dir = vec2<f32>(cos(rad), sin(rad));

    let c_uv = uv - vec2<f32>(0.5, 0.5);
    let d = dot(c_uv, dir);
    let max_d = 0.5 * (abs(dir.x) + abs(dir.y));
    let norm_d = (d / max(max_d, 0.0001) + 1.0) * 0.5;

    let mask = smoothstep(progress - feather, progress + feather, norm_d);
    let a = get_from_color(uv);
    let b = get_to_color(uv);
    return mix(b, a, mask);
}
"#;

pub const COLOR_FLASH_FADE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    var flash_color = uniforms.params0.xyz;
    if (dot(flash_color, flash_color) == 0.0) {
        flash_color = vec3<f32>(1.0, 1.0, 1.0);
    }
    let exponent = select(uniforms.params0.w, 2.0, uniforms.params0.w == 0.0);
    let flash = vec4<f32>(flash_color, 1.0);

    if (progress < 0.5) {
        let t = pow(progress * 2.0, exponent);
        return mix(get_from_color(uv), flash, t);
    } else {
        let t = pow((1.0 - progress) * 2.0, exponent);
        return mix(get_to_color(uv), flash, t);
    }
}
"#;

pub const RADIAL_CLOCK_WIPE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    var center = uniforms.params0.xy;
    if (dot(center, center) == 0.0) {
        center = vec2<f32>(0.5, 0.5);
    }
    let start_angle = select(uniforms.params0.z, 90.0, uniforms.params0.z == 0.0);
    let feather = select(uniforms.params0.w, 0.02, uniforms.params0.w == 0.0);

    let d = uv - center;
    let pi = 3.14159265359;
    let angle = atan2(d.y, d.x);
    let start_rad = radians(start_angle);
    let norm_angle = fract((start_rad - angle) / (2.0 * pi) + 1.0);

    let mask = smoothstep(progress - feather, progress + feather, norm_angle);
    let a = get_from_color(uv);
    let b = get_to_color(uv);
    return mix(b, a, mask);
}
"#;

pub const LUMA_MATTE_DISSOLVE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let softness = select(uniforms.params0.x, 0.08, uniforms.params0.x == 0.0);
    let invert = uniforms.params0.y > 0.5;

    let a = get_from_color(uv);
    let b = get_to_color(uv);
    var luma = dot(a.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (invert) {
        luma = 1.0 - luma;
    }

    let threshold = progress * (1.0 + 2.0 * softness) - softness;
    let mask = smoothstep(threshold - softness, threshold + softness, luma);
    return mix(b, a, mask);
}
"#;

// ============================================================================
// LEVEL 2: 2D GEOMETRIC DEFORMATIONS
// ============================================================================

pub const PUSH_SLIDE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    var dir = uniforms.params0.xy;
    if (dot(dir, dir) == 0.0) {
        dir = vec2<f32>(1.0, 0.0);
    }
    let norm_dir = normalize(dir);

    var p = progress;
    let ease_mode = i32(uniforms.params0.z);
    if (ease_mode == 1) {
        p = p * p * (3.0 - 2.0 * p);
    } else if (ease_mode == 2) {
        p = select(8.0 * p * p * p * p, 1.0 - pow(-2.0 * p + 2.0, 4.0) / 2.0, p >= 0.5);
    }

    let uv_a = uv - norm_dir * p;
    let uv_b = uv + norm_dir * (1.0 - p);

    let in_a = uv_a.x >= 0.0 && uv_a.x <= 1.0 && uv_a.y >= 0.0 && uv_a.y <= 1.0;
    let in_b = uv_b.x >= 0.0 && uv_b.x <= 1.0 && uv_b.y >= 0.0 && uv_b.y <= 1.0;

    if (in_b) {
        return sample_channel1(uv_b);
    }
    if (in_a) {
        return sample_channel0(uv_a);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
"#;

pub const IRIS_CIRCLE_OPEN: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    var center = uniforms.params0.xy;
    if (dot(center, center) == 0.0) {
        center = vec2<f32>(0.5, 0.5);
    }
    let feather = select(uniforms.params0.z, 0.04, uniforms.params0.z == 0.0);
    let aspect = uniforms.resolution.x / uniforms.resolution.y;

    let d = length((uv - center) * vec2<f32>(aspect, 1.0));
    let corners = max(center, vec2<f32>(1.0, 1.0) - center);
    let r_max = length(corners * vec2<f32>(aspect, 1.0));
    let r = progress * (r_max + feather);

    let mask = smoothstep(r - feather, r, d);
    let a = get_from_color(uv);
    let b = get_to_color(uv);
    return mix(b, a, mask);
}
"#;

pub const ZOOM_SCALE_IN_OUT: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let zoom = select(uniforms.params0.x, 3.0, uniforms.params0.x == 0.0);
    var center = uniforms.params0.yz;
    if (dot(center, center) == 0.0) {
        center = vec2<f32>(0.5, 0.5);
    }

    let scale_a = 1.0 + progress * zoom;
    let scale_b = 1.0 + (1.0 - progress) * zoom;

    let uv_a = (uv - center) / scale_a + center;
    let uv_b = (uv - center) / scale_b + center;

    let a = get_from_color(uv_a);
    let b = get_to_color(uv_b);
    let t = smoothstep(0.4, 0.6, progress);
    return mix(a, b, t);
}
"#;

pub const MOSAIC_PIXELATE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let max_block = select(uniforms.params0.x, 0.05, uniforms.params0.x == 0.0);
    let s = sin(progress * 3.14159265359) * max_block;

    var uv_p = uv;
    if (s > 0.001) {
        uv_p = floor(uv / s) * s + s * 0.5;
    }

    let a = get_from_color(uv_p);
    let b = get_to_color(uv_p);
    return mix(a, b, progress);
}
"#;

pub const CHECKERBOARD_GRID_WIPE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    var grid = uniforms.params0.xy;
    if (dot(grid, grid) == 0.0) {
        grid = vec2<f32>(16.0, 9.0);
    }
    let randomness = select(uniforms.params0.z, 0.8, uniforms.params0.z == 0.0);

    let cell = floor(uv * grid);
    let r = fract(sin(dot(cell, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let spatial = (cell.x / grid.x + cell.y / grid.y) * 0.5;
    let threshold = mix(spatial, r, randomness);

    let edge = 0.05;
    let mask = smoothstep(threshold - edge, threshold + edge, progress);
    let a = get_from_color(uv);
    let b = get_to_color(uv);
    return mix(a, b, mask);
}
"#;

// ============================================================================
// LEVEL 3: CONTINUOUS DEFORMATIONS & PROCEDURAL NOISE
// ============================================================================

pub const WAVE_SINE_DISPLACEMENT: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let freq = select(uniforms.params0.x, 30.0, uniforms.params0.x == 0.0);
    let amp = select(uniforms.params0.y, 0.05, uniforms.params0.y == 0.0);
    let spd = select(uniforms.params0.z, 8.0, uniforms.params0.z == 0.0);

    let delta = sin(uv.y * freq + progress * spd) * amp * sin(progress * 3.14159265359);
    let uv_disp = uv + vec2<f32>(delta, 0.0);

    let a = get_from_color(uv_disp);
    let b = get_to_color(uv_disp);
    return mix(a, b, progress);
}
"#;

pub const SWIRL_TWIRL: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let twist = select(uniforms.params0.x, 5.0, uniforms.params0.x == 0.0);
    let radius = select(uniforms.params0.y, 0.8, uniforms.params0.y == 0.0);
    var center = uniforms.params0.zw;
    if (dot(center, center) == 0.0) {
        center = vec2<f32>(0.5, 0.5);
    }
    let aspect = uniforms.resolution.x / uniforms.resolution.y;

    let d = (uv - center) * vec2<f32>(aspect, 1.0);
    let r = length(d);
    var uv_swirled = uv;
    if (r < radius && r > 0.0001) {
        let theta = atan2(d.y, d.x);
        let factor = pow(1.0 - r / radius, 2.0) * twist * sin(progress * 3.14159265359);
        let theta_new = theta + factor;
        let d_new = vec2<f32>(cos(theta_new) / aspect, sin(theta_new)) * r;
        uv_swirled = center + d_new;
    }

    let a = get_from_color(uv_swirled);
    let b = get_to_color(uv_swirled);
    return mix(a, b, progress);
}
"#;

pub const CHROMATIC_ABERRATION_SLIDE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let sep = select(uniforms.params0.x, 0.03, uniforms.params0.x == 0.0);
    var dir = uniforms.params0.yz;
    if (dot(dir, dir) == 0.0) {
        dir = vec2<f32>(1.0, 0.0);
    }
    let norm_dir = normalize(dir);

    let delta = norm_dir * sep * sin(progress * 3.14159265359);
    let uv_a = uv - norm_dir * progress;
    let uv_b = uv + norm_dir * (1.0 - progress);

    let r = mix(get_from_color(uv_a - delta).r, get_to_color(uv_b - delta).r, progress);
    let g = mix(get_from_color(uv_a).g, get_to_color(uv_b).g, progress);
    let b = mix(get_from_color(uv_a + delta).b, get_to_color(uv_b + delta).b, progress);
    let a = mix(get_from_color(uv_a).a, get_to_color(uv_b).a, progress);

    return vec4<f32>(r, g, b, a);
}
"#;

pub const PAGE_CURL: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let angle_deg = select(uniforms.params0.x, 45.0, uniforms.params0.x == 0.0);
    let radius = select(uniforms.params0.y, 0.15, uniforms.params0.y == 0.0);
    let shadow_intensity = select(uniforms.params0.z, 0.6, uniforms.params0.z == 0.0);

    let rad = radians(angle_deg);
    let dir = vec2<f32>(cos(rad), sin(rad));

    let p = progress * (1.0 + 3.14159 * radius);
    let d = dot(uv, dir) - p;

    let col_b = get_to_color(uv);

    if (d > radius) {
        return get_from_color(uv);
    } else if (d > -radius) {
        let t = acos(clamp(d / radius, -1.0, 1.0));
        let uv_curl = uv - dir * (radius * (3.14159 - t) - d);
        let shadow = 1.0 - shadow_intensity * (1.0 - abs(d / radius));
        let curl_col = get_from_color(uv_curl) * shadow;
        return mix(col_b, curl_col, smoothstep(-radius, -radius * 0.5, d));
    } else {
        let shadow = clamp(-d / (radius * 2.0), 0.0, 1.0);
        let shadow_mult = 1.0 - (1.0 - shadow) * shadow_intensity;
        return col_b * shadow_mult;
    }
}
"#;

pub const SIMPLEX_NOISE_BURN: &str = r#"fn hash22_burn(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn simplex_noise_burn(p: vec2<f32>) -> f32 {
    let k1 = 0.366025404;
    let k2 = 0.211324865;
    let i = floor(p + (p.x + p.y) * k1);
    let a = p - i + (i.x + i.y) * k2;
    let o = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), a.x > a.y);
    let b = a - o + k2;
    let c = a - 1.0 + 2.0 * k2;
    let h = max(0.5 - vec3<f32>(dot(a, a), dot(b, b), dot(c, c)), vec3<f32>(0.0));
    let n = h * h * h * h * vec3<f32>(
        dot(a, hash22_burn(i) * 2.0 - 1.0),
        dot(b, hash22_burn(i + o) * 2.0 - 1.0),
        dot(c, hash22_burn(i + 1.0) * 2.0 - 1.0)
    );
    return dot(n, vec3<f32>(70.0)) * 0.5 + 0.5;
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let scale = select(uniforms.params0.x, 6.0, uniforms.params0.x == 0.0);
    let edge = select(uniforms.params0.y, 0.04, uniforms.params0.y == 0.0);
    var glow = vec3<f32>(uniforms.params0.z, uniforms.params0.w, uniforms.params1.x);
    if (dot(glow, glow) == 0.0) {
        glow = vec3<f32>(1.0, 0.35, 0.05);
    }

    let n = simplex_noise_burn(uv * scale);
    let t = progress * (1.0 + edge * 2.0) - edge;

    let a = get_from_color(uv);
    let b = get_to_color(uv);

    if (n < t - edge) {
        return b;
    } else if (n < t) {
        let burn_t = 1.0 - (t - n) / edge;
        let fire = glow * (2.0 + burn_t * 3.0);
        return vec4<f32>(mix(b.rgb, fire, burn_t), 1.0);
    } else {
        return a;
    }
}
"#;

// ============================================================================
// LEVEL 4: MULTI-TAP, CONVOLUTIONS & HEAVY SIMULATION
// ============================================================================

pub const DIRECTIONAL_MOTION_BLUR_WIPE: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let samples = i32(select(uniforms.params0.x, 16.0, uniforms.params0.x == 0.0));
    let strength = select(uniforms.params0.y, 0.08, uniforms.params0.y == 0.0);
    var dir = uniforms.params0.zw;
    if (dot(dir, dir) == 0.0) {
        dir = vec2<f32>(1.0, 0.0);
    }
    let norm_dir = normalize(dir);
    let blur_v = norm_dir * strength * sin(progress * 3.14159265359);

    let uv_a_base = uv - norm_dir * progress;
    let uv_b_base = uv + norm_dir * (1.0 - progress);

    var acc_a = vec4<f32>(0.0);
    var acc_b = vec4<f32>(0.0);
    let n = min(samples, 32);
    let f_n = f32(n);

    for (var i = 0; i < n; i = i + 1) {
        let offset = blur_v * ((f32(i) - f_n * 0.5) / f_n);
        acc_a = acc_a + get_from_color(uv_a_base + offset);
        acc_b = acc_b + get_to_color(uv_b_base + offset);
    }

    acc_a = acc_a / f_n;
    acc_b = acc_b / f_n;
    return mix(acc_a, acc_b, smoothstep(0.4, 0.6, progress));
}
"#;

pub const ZOOM_RADIAL_BLUR: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let samples = i32(select(uniforms.params0.x, 16.0, uniforms.params0.x == 0.0));
    let blur = select(uniforms.params0.y, 0.2, uniforms.params0.y == 0.0);
    var center = uniforms.params0.zw;
    if (dot(center, center) == 0.0) {
        center = vec2<f32>(0.5, 0.5);
    }

    let rad_vec = (uv - center) * sin(progress * 3.14159265359) * blur;

    var acc_a = vec4<f32>(0.0);
    var acc_b = vec4<f32>(0.0);
    var total_w = 0.0;
    let n = min(samples, 24);

    for (var i = 0; i < n; i = i + 1) {
        let t = f32(i) / f32(n);
        let w = exp(-t * 2.0);
        let sample_uv = uv - rad_vec * t;
        acc_a = acc_a + get_from_color(sample_uv) * w;
        acc_b = acc_b + get_to_color(sample_uv) * w;
        total_w = total_w + w;
    }

    acc_a = acc_a / total_w;
    acc_b = acc_b / total_w;
    return mix(acc_a, acc_b, progress);
}
"#;

pub const DIGITAL_GLITCH_SPECTRAL_DRIFT: &str = r#"fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let rows = select(uniforms.params0.x, 40.0, uniforms.params0.x == 0.0);
    let intensity = select(uniforms.params0.y, 0.7, uniforms.params0.y == 0.0);
    let drift = select(uniforms.params0.z, 0.04, uniforms.params0.z == 0.0);

    let active_intensity = intensity * sin(progress * 3.14159265359) * (1.0 + uniforms.beat_params.x * 0.5);

    let block_y = floor(uv.y * rows);
    let seed = sin(block_y * 12.9898 + floor(progress * 30.0) * 78.233) * 43758.5453;
    let rnd = fract(seed);

    var offset_x = 0.0;
    if (rnd < active_intensity) {
        offset_x = (fract(rnd * 13.37) - 0.5) * 0.3 * active_intensity;
    }

    let uv_shifted = vec2<f32>(uv.x + offset_x, uv.y);
    let d_rgb = vec2<f32>(drift * active_intensity, 0.0);

    let ra = get_from_color(uv_shifted - d_rgb).r;
    let ga = get_from_color(uv_shifted).g;
    let ba = get_from_color(uv_shifted + d_rgb).b;
    let col_a = vec4<f32>(ra, ga, ba, 1.0);

    let rb = get_to_color(uv_shifted - d_rgb).r;
    let gb = get_to_color(uv_shifted).g;
    let bb = get_to_color(uv_shifted + d_rgb).b;
    let col_b = vec4<f32>(rb, gb, bb, 1.0);

    let block_cut = fract(sin(block_y * 37.17) * 43758.5453);
    let t = smoothstep(0.0, 1.0, (progress - block_cut * 0.3) / 0.7);
    return mix(col_a, col_b, t);
}
"#;

pub const GLASS_PRISM_REFRACTION: &str = r#"fn hash22_glass(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn voronoi_glass(p: vec2<f32>) -> f32 {
    let n = floor(p);
    let f = fract(p);
    var md = 8.0;
    for (var j = -1; j <= 1; j = j + 1) {
        for (var i = -1; i <= 1; i = i + 1) {
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash22_glass(n + g);
            let r = g + o - f;
            md = min(md, dot(r, r));
        }
    }
    return sqrt(md);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let density = select(uniforms.params0.x, 12.0, uniforms.params0.x == 0.0);
    let eta_base = select(uniforms.params0.y, 1.45, uniforms.params0.y == 0.0);
    let dispersion = select(uniforms.params0.z, 0.03, uniforms.params0.z == 0.0);

    let depth = sin(progress * 3.14159265359) * 0.15;
    let eps = vec2<f32>(0.005, 0.0);

    let hx = voronoi_glass((uv + eps.xy) * density) - voronoi_glass((uv - eps.xy) * density);
    let hy = voronoi_glass((uv + eps.yx) * density) - voronoi_glass((uv - eps.yx) * density);
    let norm = normalize(vec3<f32>(hx * depth, hy * depth, 0.05));

    let eta_r = eta_base - dispersion;
    let eta_g = eta_base;
    let eta_b = eta_base + dispersion;

    let refr_r = refract(vec3<f32>(0.0, 0.0, 1.0), norm, 1.0 / eta_r).xy;
    let refr_g = refract(vec3<f32>(0.0, 0.0, 1.0), norm, 1.0 / eta_g).xy;
    let refr_b = refract(vec3<f32>(0.0, 0.0, 1.0), norm, 1.0 / eta_b).xy;

    let col_a = vec4<f32>(
        get_from_color(uv + refr_r).r,
        get_from_color(uv + refr_g).g,
        get_from_color(uv + refr_b).b,
        1.0
    );
    let col_b = vec4<f32>(
        get_to_color(uv + refr_r).r,
        get_to_color(uv + refr_g).g,
        get_to_color(uv + refr_b).b,
        1.0
    );

    return mix(col_a, col_b, progress);
}
"#;

pub const RAYMARCHED_HEIGHTFIELD_MORPHING: &str = r#"fn get_height(p: vec2<f32>, progress: f32, max_ext: f32) -> f32 {
    let la = dot(get_from_color(p).rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let lb = dot(get_to_color(p).rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(la, lb, progress) * max_ext * sin(progress * 3.14159265359);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let steps = i32(select(uniforms.params0.x, 32.0, uniforms.params0.x == 0.0));
    let max_ext = select(uniforms.params0.y, 0.2, uniforms.params0.y == 0.0);
    var light_pos = vec3<f32>(uniforms.params0.z, uniforms.params0.w, uniforms.params1.x);
    if (dot(light_pos, light_pos) == 0.0) {
        light_pos = vec3<f32>(0.5, 0.5, -1.0);
    }
    let shininess = select(uniforms.params1.y, 16.0, uniforms.params1.y == 0.0);

    let ro = vec3<f32>(uv, -0.5);
    let rd = vec3<f32>(0.0, 0.0, 1.0);

    var t = 0.0;
    var hit_uv = uv;
    let step_sz = 0.5 / f32(steps);
    for (var i = 0; i < steps; i = i + 1) {
        let p = ro + rd * t;
        let h = get_height(p.xy, progress, max_ext);
        if (p.z >= -h) {
            hit_uv = p.xy;
            break;
        }
        t = t + step_sz;
    }

    let eps = vec2<f32>(0.005, 0.0);
    let h0 = get_height(hit_uv, progress, max_ext);
    let hx = get_height(hit_uv + eps.xy, progress, max_ext) - h0;
    let hy = get_height(hit_uv + eps.yx, progress, max_ext) - h0;
    let norm = normalize(vec3<f32>(-hx, -hy, eps.x));

    let light_dir = normalize(light_pos - vec3<f32>(hit_uv, -h0));
    let view_dir = vec3<f32>(0.0, 0.0, -1.0);
    let half_dir = normalize(light_dir + view_dir);

    let diff = max(dot(norm, light_dir), 0.2);
    let spec = pow(max(dot(norm, half_dir), 0.0), shininess);

    let col = mix(get_from_color(hit_uv), get_to_color(hit_uv), progress);
    let lit_rgb = col.rgb * diff + vec3<f32>(spec * 0.4);
    return vec4<f32>(lit_rgb, col.a);
}
"#;

#[cfg(test)]
#[path = "tests/shadertoy_transitions.rs"]
mod tests;
