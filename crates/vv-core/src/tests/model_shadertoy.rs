use crate::command::{KeyframeTarget, target_track_mut};
use crate::model::{
    ClipFilter, FilterKind, Interpolation, Keyframed, ShadertoyPreset, get_shader, intern_shader,
};

#[test]
fn shadertoy_presets_contain_non_empty_code() {
    for preset in ShadertoyPreset::ALL {
        assert!(!preset.label().is_empty());
        let code = preset.code();
        assert!(!code.is_empty());
        assert!(code.contains("fn main_image"));
    }
}

#[test]
fn shader_interning_deduplicates_identical_code() {
    let code_a = "fn main_image(frag_coord: vec2<f32>) -> vec4<f32> { return vec4<f32>(1.0); }";
    let code_b = "fn main_image(frag_coord: vec2<f32>) -> vec4<f32> { return vec4<f32>(0.5); }";
    let id_a1 = intern_shader(code_a);
    let id_a2 = intern_shader(code_a);
    let id_b = intern_shader(code_b);

    assert_eq!(id_a1, id_a2);
    assert_ne!(id_a1, id_b);
    assert_eq!(get_shader(id_a1).as_deref(), Some(code_a));
    assert_eq!(get_shader(id_b).as_deref(), Some(code_b));
}

#[test]
fn clip_filter_shadertoy_evaluates_at_frame() {
    let mut filter = ClipFilter::new(FilterKind::Shadertoy);
    assert!(filter.kind.is_shadertoy());
    assert!(filter.shadertoy.is_some());

    let s = filter.shadertoy.as_mut().unwrap();
    let mut track = Keyframed::constant(0.0);
    track.upsert(0, 0.0, Interpolation::Linear);
    track.upsert(10, 1.0, Interpolation::Linear);
    s.params[0] = track;

    let val_0 = filter.value_at(0);
    let val_5 = filter.value_at(5);
    let val_10 = filter.value_at(10);

    let st_0 = val_0.shadertoy.expect("shadertoy value exists");
    let st_5 = val_5.shadertoy.expect("shadertoy value exists");
    let st_10 = val_10.shadertoy.expect("shadertoy value exists");

    assert_eq!(st_0.params[0], 0.0);
    assert!((st_5.params[0] - 0.5).abs() < 1e-4);
    assert_eq!(st_10.params[0], 1.0);
    assert_eq!(st_0.shader_id, st_5.shader_id);
}

#[test]
fn target_track_mut_accesses_shadertoy_params() {
    let mut effects = crate::model::EffectStack::default();
    effects.filters.push(ClipFilter::new(FilterKind::Shadertoy));

    let track = target_track_mut(&mut effects, KeyframeTarget::FilterParam(FilterKind::Shadertoy, 2));
    assert!(track.is_some());
    let track = track.unwrap();
    track.upsert(12, 0.75, Interpolation::EaseInOut);
    assert_eq!(track.value_at(12), 0.75);
}
