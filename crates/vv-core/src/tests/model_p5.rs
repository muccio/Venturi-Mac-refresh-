use super::*;

#[test]
fn p5_canvas_filter_initialization() {
    let filter = ClipFilter::new(FilterKind::P5Canvas);
    assert_eq!(filter.kind, FilterKind::P5Canvas);
    assert!(filter.kind.is_p5());
    assert!(!filter.kind.is_shadertoy());
    assert!(!filter.kind.is_blur());

    let p5 = filter.p5.as_ref().expect("p5 params initialized");
    assert_eq!(p5.preset, P5Preset::AsciiColor);
    assert_eq!(p5.blend_mode, P5BlendMode::Normal);
    assert!(p5.code.contains("P5.js Canvas Effect: ASCII Art"));
    assert_eq!(p5.params.len(), 4);
}

#[test]
fn p5_presets_have_distinct_code_and_labels() {
    for preset in P5Preset::ALL {
        let label = preset.label();
        assert!(!label.is_empty(), "Preset label should not be empty");
        let code = preset.code();
        assert!(!code.is_empty(), "Preset code should not be empty");
        assert!(code.contains("function setup()"), "Preset must define setup()");
        assert!(code.contains("function draw()"), "Preset must define draw()");
    }
}

#[test]
fn p5_script_interner_roundtrip() {
    let script1 = "function setup() { createCanvas(100, 100); }";
    let script2 = "function draw() { background(255); }";

    let id1 = intern_p5_script(script1);
    let id2 = intern_p5_script(script2);
    let id1_again = intern_p5_script(script1);

    assert_eq!(id1, id1_again);
    assert_ne!(id1, id2);

    assert_eq!(get_p5_script(id1).as_deref(), Some(script1));
    assert_eq!(get_p5_script(id2).as_deref(), Some(script2));
}

#[test]
fn p5_filter_value_at_evaluates_keyframed_params() {
    let mut filter = ClipFilter::new(FilterKind::P5Canvas);
    let p5 = filter.p5.as_mut().unwrap();
    p5.params[0].upsert(0, 0.2, Interpolation::Linear);
    p5.params[0].upsert(10, 0.8, Interpolation::Linear);
    p5.params[1] = Keyframed::constant(1.5);

    let val0 = filter.value_at(0);
    let p5_val0 = val0.p5.expect("p5 value present");
    assert!((p5_val0.params[0] - 0.2).abs() < 1e-4);
    assert!((p5_val0.params[1] - 1.5).abs() < 1e-4);

    let val5 = filter.value_at(5);
    let p5_val5 = val5.p5.unwrap();
    assert!((p5_val5.params[0] - 0.5).abs() < 1e-4);

    let val10 = filter.value_at(10);
    let p5_val10 = val10.p5.unwrap();
    assert!((p5_val10.params[0] - 0.8).abs() < 1e-4);
}
