use super::*;
use vv_core::{ClipFilter, FilterKind, ShadertoyPreset};

#[test]
fn all_presets_compile_successfully() {
    let compositor = crate::Compositor::new_headless();
    let runner = &compositor.shadertoy;
    let device = compositor.device();

    for (idx, preset) in ShadertoyPreset::ALL.iter().enumerate() {
        let code = preset.code();
        let pipeline = runner.get_or_compile(device, idx as u32 + 100, code);
        // If it compiles without panic, the pipeline was created on the GPU
        let _ = pipeline;
    }
}

#[test]
fn glsl_conversion_handles_standard_shadertoy_syntax() {
    let glsl = r#"
void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
    vec2 uv = fragCoord / iResolution.xy;
    vec3 col = 0.5 + 0.5 * cos(iTime + uv.xyx + vec3(0.0, 2.0, 4.0));
    fragColor = vec4(col, 1.0);
}
"#;
    let wgsl = ShadertoyRunner::preprocess_code(glsl);
    assert!(wgsl.contains("fn main_image(frag_coord: vec2<f32>) -> vec4<f32>"));
    assert!(wgsl.contains("uniforms.resolution.xy"));
    assert!(wgsl.contains("uniforms.time.x"));
    assert!(wgsl.contains("return vec4<f32>(col, 1.0);"));
}

#[test]
fn render_layer_with_shadertoy_produces_pixels() {
    let compositor = crate::Compositor::new_headless();
    let filter = ClipFilter::new(FilterKind::Shadertoy);
    let filter_val = filter.value_at(0);

    let layer = crate::Layer {
        content: crate::LayerContent::Solid(vv_core::Rgba::BLACK),
        transform: vv_core::Transform::default(),
        opacity: 1.0,
        filters: std::slice::from_ref(&filter_val),
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let output = crate::OutputFrame::exact(64, 64)
        .with_time(1.0, 24, 24.0)
        .with_audio([0.5, 0.8, 0.4, 0.2], 1.0);

    let pixels = compositor.render_layers(&[layer], output);
    assert_eq!(pixels.len(), 64 * 64 * 4);
    // Plasma shader produces colorful non-black pixels
    let non_zero = pixels.iter().any(|&b| b > 10);
    assert!(non_zero, "Shadertoy pass should render non-zero plasma pixels");
}
