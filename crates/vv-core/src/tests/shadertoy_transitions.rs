use std::collections::HashSet;
use crate::shadertoy_transitions::ShadertoyTransitionPreset;

#[test]
fn all_twenty_presets_are_defined_and_unique() {
    assert_eq!(ShadertoyTransitionPreset::ALL.len(), 20);
    let mut names = HashSet::new();
    let mut labels = HashSet::new();
    for preset in ShadertoyTransitionPreset::ALL {
        assert!(names.insert(preset.name()), "Duplicate name: {}", preset.name());
        assert!(labels.insert(preset.label()), "Duplicate label: {}", preset.label());
        assert!((1..=4).contains(&preset.level()), "Invalid level: {}", preset.level());
        assert!(!preset.description().is_empty(), "Empty description for {}", preset.name());
        assert!(preset.wgsl_code().contains("fn transition("), "Missing transition() in {}", preset.name());
        assert_eq!(ShadertoyTransitionPreset::from_name(preset.name()), Some(preset));
    }
}

#[test]
fn presets_levels_distribution() {
    let mut count_per_level = [0usize; 5];
    for preset in ShadertoyTransitionPreset::ALL {
        count_per_level[preset.level() as usize] += 1;
    }
    assert_eq!(count_per_level[1], 5, "Level 1 should have 5 presets");
    assert_eq!(count_per_level[2], 5, "Level 2 should have 5 presets");
    assert_eq!(count_per_level[3], 5, "Level 3 should have 5 presets");
    assert_eq!(count_per_level[4], 5, "Level 4 should have 5 presets");
}
