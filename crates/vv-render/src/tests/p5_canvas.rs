use super::*;
use vv_core::{ClipFilter, FilterKind, P5BlendMode, P5Preset};

#[test]
fn p5_runner_pipeline_compiles() {
    let compositor = crate::Compositor::new_headless();
    let runner = &compositor.p5;
    assert_ne!(runner.format(), wgpu::TextureFormat::R8Unorm);
}

#[test]
fn p5_uniforms_pack_correctly() {
    let p5_val = vv_core::P5Value {
        preset: P5Preset::WaveMatrix,
        script_id: 42,
        params: [0.25, 0.5, 0.75, 1.0],
        blend_mode: P5BlendMode::Screen,
        source_frame: 0,
    };
    let uniforms = P5Uniforms::new(
        1920,
        1080,
        2.5,
        75,
        30.0,
        p5_val,
        [0.6, 0.8, 0.4, 0.2],
        1.0,
    );
    assert_eq!(uniforms.resolution, [1920.0, 1080.0]);
    assert_eq!(uniforms.time, 2.5);
    assert_eq!(uniforms.frame, 75);
    assert_eq!(uniforms.fps, 30.0);
    assert_eq!(uniforms.preset, 3); // WaveMatrix is 3
    assert_eq!(uniforms.blend_mode, 2); // Screen is 2
    assert_eq!(uniforms.audio, [0.6, 0.8, 0.4, 0.2]);
    assert_eq!(uniforms.audio_beat, 1.0);
}

#[test]
fn p5_canvas_renders_pixels_for_all_presets() {
    let compositor = crate::Compositor::new_headless();

    for preset in P5Preset::ALL {
        let mut filter = ClipFilter::new(FilterKind::P5Canvas);
        if let Some(p5) = filter.p5.as_mut() {
            p5.preset = preset;
        }
        let filter_val = filter.value_at(0);

        let layer = crate::Layer {
            content: crate::LayerContent::Solid(vv_core::Rgba {
                r: 0.5,
                g: 0.3,
                b: 0.2,
                a: 1.0,
            }),
            transform: vv_core::Transform::default(),
            opacity: 1.0,
            filters: std::slice::from_ref(&filter_val),
            blend: vv_core::BlendMode::Normal,
            masks: &[],
        };

        let output = crate::OutputFrame::exact(64, 64)
            .with_time(1.0, 24, 24.0)
            .with_audio([0.7, 0.9, 0.5, 0.3], 1.0);

        let pixels = compositor.render_layers(&[layer], output);
        assert_eq!(pixels.len(), 64 * 64 * 4);
        let has_content = pixels.iter().any(|&b| b > 0);
        assert!(
            has_content,
            "Preset {:?} should produce rendered pixels",
            preset
        );
    }
}

#[test]
fn p5_canvas_renders_distinct_pixels_with_zero_audio_for_all_presets() {
    let compositor = crate::Compositor::new_headless();

    // Baseline image without filters
    let base_layer = crate::Layer {
        content: crate::LayerContent::Solid(vv_core::Rgba {
            r: 0.6,
            g: 0.4,
            b: 0.2,
            a: 1.0,
        }),
        transform: vv_core::Transform::default(),
        opacity: 1.0,
        filters: &[],
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let zero_audio_output = crate::OutputFrame::exact(64, 64)
        .with_time(1.0, 24, 24.0)
        .with_audio([0.0, 0.0, 0.0, 0.0], 0.0);

    let base_pixels = compositor.render_layers(&[base_layer], zero_audio_output);

    for preset in P5Preset::ALL {
        let mut filter = ClipFilter::new(FilterKind::P5Canvas);
        if let Some(p5) = filter.p5.as_mut() {
            p5.preset = preset;
        }
        let filter_val = filter.value_at(0);

        let filtered_layer = crate::Layer {
            content: crate::LayerContent::Solid(vv_core::Rgba {
                r: 0.6,
                g: 0.4,
                b: 0.2,
                a: 1.0,
            }),
            transform: vv_core::Transform::default(),
            opacity: 1.0,
            filters: std::slice::from_ref(&filter_val),
            blend: vv_core::BlendMode::Normal,
            masks: &[],
        };

        let pixels = compositor.render_layers(&[filtered_layer], zero_audio_output);
        assert_eq!(pixels.len(), 64 * 64 * 4);
        let differs = pixels != base_pixels;
        assert!(
            differs,
            "Preset {:?} must produce distinct visual effect even with zero audio (idle / scrub / export)",
            preset
        );
    }
}

#[test]
fn p5_canvas_chains_into_shadertoy() {
    let compositor = crate::Compositor::new_headless();

    let mut p5_filter = ClipFilter::new(FilterKind::P5Canvas);
    if let Some(p5) = p5_filter.p5.as_mut() {
        p5.preset = P5Preset::AsciiColor;
    }
    let p5_val = p5_filter.value_at(0);

    let mut st_filter = ClipFilter::new(FilterKind::Shadertoy);
    if let Some(st) = st_filter.shadertoy.as_mut() {
        st.code = vv_core::ShadertoyPreset::CrtScanlines.code().to_string();
    }
    let st_val = st_filter.value_at(0);

    let filters = vec![p5_val, st_val];

    let layer = crate::Layer {
        content: crate::LayerContent::Solid(vv_core::Rgba {
            r: 0.8,
            g: 0.4,
            b: 0.1,
            a: 1.0,
        }),
        transform: vv_core::Transform::default(),
        opacity: 1.0,
        filters: &filters,
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let output = crate::OutputFrame::exact(64, 64)
        .with_time(1.0, 24, 24.0)
        .with_audio([0.8, 0.9, 0.6, 0.4], 1.0);

    let pixels = compositor.render_layers(&[layer], output);
    assert_eq!(pixels.len(), 64 * 64 * 4);
    let non_zero = pixels.iter().any(|&b| b > 0);
    assert!(non_zero, "Chained P5 -> Shadertoy should produce pixels");
}
