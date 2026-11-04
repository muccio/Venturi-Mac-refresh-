use super::*;
use vv_core::{Clip, ClipId, ClipSource, Project, Rational, Timeline, Track, TrackKind};

#[test]
fn dialog_default_ranges_match_parameter_nature() {
    assert_eq!(TargetParamChoice::ZoomUniform.default_range(), (1.0, 1.25));
    assert_eq!(TargetParamChoice::ZoomX.default_range(), (1.0, 1.25));
    assert_eq!(TargetParamChoice::ZoomY.default_range(), (1.0, 1.25));
    assert_eq!(TargetParamChoice::Opacity.default_range(), (0.0, 100.0));
    assert_eq!(TargetParamChoice::Rotation.default_range(), (-5.0, 5.0));
    assert_eq!(TargetParamChoice::PositionX.default_range(), (-50.0, 50.0));
    assert_eq!(TargetParamChoice::PositionY.default_range(), (-50.0, 50.0));
    assert_eq!(TargetParamChoice::BlurRadius.default_range(), (0.0, 25.0));
    assert_eq!(TargetParamChoice::Exposure.default_range(), (0.0, 2.5));
}

#[test]
fn dialog_envelope_calculation_generates_frame_accurate_curve() {
    let mut dialog = AudioEnvelopeDialog::new((0, ClipId(1)), 24);
    // Fill dummy audio: 1 second at 48kHz
    dialog.raw_samples = vec![0.5; 48000];
    dialog.sample_rate = 48000;
    dialog.num_frames = 24;

    dialog.recalculate(Rational::new(24, 1));
    assert_eq!(dialog.envelope_preview.len(), 24);
    for &val in &dialog.envelope_preview {
        assert!(val >= dialog.custom_min && val <= dialog.custom_max);
    }
}

#[test]
fn dialog_keyframe_baking_creates_commands() {
    let mut project = Project::default();
    let tl = project.timelines.insert(Timeline {
        name: "T".into(),
        fps: Rational::new(25, 1),
        resolution: (1920, 1080),
        tracks: vec![Track::new(TrackKind::Video)],
        markers: Vec::new(),
        master: Default::default(),
    });

    let clip_id = ClipId(10);
    project.timelines[tl].tracks[0].clips.push(Clip::from_source_range(
        clip_id,
        ClipSource::SolidColor,
        0,
        50,
        0,
        Rational::one(),
    ));

    let mut dialog = AudioEnvelopeDialog::new((0, clip_id), 50);
    dialog.raw_samples = vec![0.8; 48000 * 2];
    dialog.sample_rate = 48000;
    dialog.num_frames = 50;
    dialog.target_param = TargetParamChoice::ZoomX;
    dialog.recalculate(Rational::new(25, 1));

    let cmd = dialog.build_bake_command(tl, 0);
    assert!(cmd.is_some());
}
