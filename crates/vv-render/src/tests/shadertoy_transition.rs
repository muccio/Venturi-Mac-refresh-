use vv_core::{Rgba, ShadertoyTransitionPreset, Transform};
use crate::{Compositor, Layer, LayerContent, OutputFrame};

#[test]
fn all_twenty_transition_presets_compile_on_gpu() {
    let compositor = Compositor::new_headless();
    let runner = &compositor.shadertoy_transition;
    let device = compositor.device();

    for preset in ShadertoyTransitionPreset::ALL {
        let pipeline = runner.get_or_compile(device, preset);
        let _ = pipeline;
    }
}

#[test]
fn linear_crossfade_blends_from_and_to_textures() {
    let compositor = Compositor::new_headless();
    let from_layer = Layer {
        content: LayerContent::Solid(Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }), // Red
        transform: Transform::default(),
        opacity: 1.0,
        filters: &[],
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let to_layer = Layer {
        content: LayerContent::Solid(Rgba { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }), // Blue
        transform: Transform::default(),
        opacity: 1.0,
        filters: &[],
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let output = OutputFrame::exact(32, 32);

    // Test at progress 0.0 (Pure Input A / Red)
    let tex_0 = compositor.render_shadertoy_transition_to_owned_texture(
        &from_layer,
        &to_layer,
        ShadertoyTransitionPreset::LinearCrossfade,
        None,
        0.0,
        output,
    );
    let pixels_0 = compositor.render_layers(
        &[Layer::new(
            LayerContent::Texture {
                texture: &tex_0,
                source_size: (32, 32),
            },
            Transform::default(),
        )],
        output,
    );
    assert!(pixels_0[0] > 200, "Red channel should dominate at progress 0.0 (got {})", pixels_0[0]);
    assert!(pixels_0[2] < 50, "Blue channel should be near 0 at progress 0.0 (got {})", pixels_0[2]);

    // Test at progress 1.0 (Pure Input B / Blue)
    let tex_1 = compositor.render_shadertoy_transition_to_owned_texture(
        &from_layer,
        &to_layer,
        ShadertoyTransitionPreset::LinearCrossfade,
        None,
        1.0,
        output,
    );
    let pixels_1 = compositor.render_layers(
        &[Layer::new(
            LayerContent::Texture {
                texture: &tex_1,
                source_size: (32, 32),
            },
            Transform::default(),
        )],
        output,
    );
    assert!(pixels_1[0] < 50, "Red channel should be near 0 at progress 1.0 (got {})", pixels_1[0]);
    assert!(pixels_1[2] > 200, "Blue channel should dominate at progress 1.0 (got {})", pixels_1[2]);

    // Test at progress 0.5 (Blend)
    let tex_half = compositor.render_shadertoy_transition_to_owned_texture(
        &from_layer,
        &to_layer,
        ShadertoyTransitionPreset::LinearCrossfade,
        None,
        0.5,
        output,
    );
    let pixels_half = compositor.render_layers(
        &[Layer::new(
            LayerContent::Texture {
                texture: &tex_half,
                source_size: (32, 32),
            },
            Transform::default(),
        )],
        output,
    );
    assert!(pixels_half[0] > 80 && pixels_half[0] < 180, "Red channel should be mixed (got {})", pixels_half[0]);
    assert!(pixels_half[2] > 80 && pixels_half[2] < 180, "Blue channel should be mixed (got {})", pixels_half[2]);
}

#[test]
fn complex_presets_render_without_error() {
    let compositor = Compositor::new_headless();
    let from_layer = Layer {
        content: LayerContent::Solid(Rgba { r: 0.8, g: 0.2, b: 0.1, a: 1.0 }),
        transform: Transform::default(),
        opacity: 1.0,
        filters: &[],
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let to_layer = Layer {
        content: LayerContent::Solid(Rgba { r: 0.1, g: 0.9, b: 0.3, a: 1.0 }),
        transform: Transform::default(),
        opacity: 1.0,
        filters: &[],
        blend: vv_core::BlendMode::Normal,
        masks: &[],
    };

    let output = OutputFrame::exact(32, 32)
        .with_time(0.5, 12, 24.0)
        .with_audio([0.3, 0.6, 0.4, 0.2], 0.8);

    // Test level 3 (SimplexNoiseBurn)
    let tex_burn = compositor.render_shadertoy_transition_to_owned_texture(
        &from_layer,
        &to_layer,
        ShadertoyTransitionPreset::SimplexNoiseBurn,
        None,
        0.5,
        output,
    );
    let pix_burn = compositor.render_layers(
        &[Layer::new(
            LayerContent::Texture {
                texture: &tex_burn,
                source_size: (32, 32),
            },
            Transform::default(),
        )],
        output,
    );
    assert_eq!(pix_burn.len(), 32 * 32 * 4);

    // Test level 4 (RaymarchedHeightfieldMorphing)
    let tex_rm = compositor.render_shadertoy_transition_to_owned_texture(
        &from_layer,
        &to_layer,
        ShadertoyTransitionPreset::RaymarchedHeightfieldMorphing,
        None,
        0.5,
        output,
    );
    let pix_rm = compositor.render_layers(
        &[Layer::new(
            LayerContent::Texture {
                texture: &tex_rm,
                source_size: (32, 32),
            },
            Transform::default(),
        )],
        output,
    );
    assert_eq!(pix_rm.len(), 32 * 32 * 4);
}
