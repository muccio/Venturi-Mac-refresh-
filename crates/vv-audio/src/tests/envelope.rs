use super::*;
use std::f32::consts::PI;
use vv_core::Rational;

fn make_sine(freq: f32, sample_rate: u32, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs).round() as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * PI * freq * t).sin()
        })
        .collect()
}

#[test]
fn rms_envelope_of_constant_amplitude_sine_is_constant() {
    let sample_rate = 48000;
    let fps = Rational::new(30, 1);
    let duration = 1.0;
    let samples = make_sine(440.0, sample_rate, duration, 0.8);
    let num_frames = 30;

    let settings = EnvelopeSettings {
        mode: AnalysisMode::Rms,
        attack_ms: 0.0,
        release_ms: 0.0,
        threshold: 0.0,
        sensitivity: 1.0,
        invert: false,
        min_out: 0.0,
        max_out: 1.0,
    };

    let env = compute_envelope(&samples, sample_rate, fps, num_frames, &settings);
    assert_eq!(env.len(), 30);
    // RMS of 0.8 amplitude sine is 0.8 / sqrt(2) ~= 0.5657
    for (i, &val) in env.iter().enumerate() {
        assert!(
            (val - 0.565).abs() < 0.05,
            "Frame {} has unexpected envelope value {}",
            i,
            val
        );
    }
}

#[test]
fn bandpass_filter_isolates_frequency_band() {
    let sample_rate = 48000;
    let fps = Rational::new(30, 1);
    let samples = make_sine(50.0, sample_rate, 1.0, 1.0);
    let num_frames = 30;

    let subbass_settings = EnvelopeSettings {
        mode: AnalysisMode::Band(20.0, 80.0),
        attack_ms: 0.0,
        release_ms: 0.0,
        threshold: 0.0,
        sensitivity: 1.0,
        invert: false,
        min_out: 0.0,
        max_out: 1.0,
    };

    let treble_settings = EnvelopeSettings {
        mode: AnalysisMode::Band(3000.0, 8000.0),
        attack_ms: 0.0,
        release_ms: 0.0,
        threshold: 0.0,
        sensitivity: 1.0,
        invert: false,
        min_out: 0.0,
        max_out: 1.0,
    };

    let subbass_env = compute_envelope(&samples, sample_rate, fps, num_frames, &subbass_settings);
    let treble_env = compute_envelope(&samples, sample_rate, fps, num_frames, &treble_settings);

    let avg_subbass: f32 = subbass_env.iter().sum::<f32>() / subbass_env.len() as f32;
    let avg_treble: f32 = treble_env.iter().sum::<f32>() / treble_env.len() as f32;

    assert!(
        avg_subbass > 0.4,
        "SubBass band should capture 50Hz tone, got {}",
        avg_subbass
    );
    assert!(
        avg_treble < 0.05,
        "Treble band should reject 50Hz tone, got {}",
        avg_treble
    );
}

#[test]
fn attack_and_release_smoothing() {
    let sample_rate = 48000;
    let fps = Rational::new(30, 1);
    let samples_per_frame = (sample_rate / 30) as usize;
    let mut samples = vec![0.0f32; samples_per_frame * 30];
    // Put burst at frames 5..10
    for i in (5 * samples_per_frame)..(10 * samples_per_frame) {
        samples[i] = 1.0;
    }

    let smooth_settings = EnvelopeSettings {
        mode: AnalysisMode::Rms,
        attack_ms: 100.0,
        release_ms: 200.0,
        threshold: 0.0,
        sensitivity: 1.0,
        invert: false,
        min_out: 0.0,
        max_out: 1.0,
    };

    let env = compute_envelope(&samples, sample_rate, fps, 30, &smooth_settings);
    // Frame 5 should start rising
    assert!(env[5] > 0.0 && env[5] < 0.9);
    // Frame 6 should be higher than Frame 5 (attack slope)
    assert!(env[6] > env[5]);
    // After frame 10 (release slope), frame 11 should decay
    assert!(env[11] < env[9]);
    assert!(env[12] < env[11]);
}

#[test]
fn transient_peak_detector_finds_onsets() {
    // 20 frames with spikes at index 5 and 15
    let mut env = vec![0.0f32; 20];
    env[5] = 1.0;
    env[6] = 0.8;
    env[15] = 0.95;

    let settings = PeakSettings {
        enabled: true,
        threshold: 0.5,
        min_distance_frames: 4,
        decay_frames: 2,
    };

    let peaks = detect_peaks(&env, &settings);
    assert_eq!(peaks.len(), 20);
    assert!(peaks[5] > 0.9, "Frame 5 should be detected as a peak");
    assert!(peaks[15] > 0.9, "Frame 15 should be detected as a peak");
    // Frame 6 should decay or not trigger a new onset because frame 5 just triggered
    assert!(peaks[0] == 0.0);
    assert!(peaks[1] == 0.0);
}

#[test]
fn simplify_keyframes_reduces_colinear_points() {
    // Linear ramp from 0.0 to 1.0 across 11 frames
    let values: Vec<f32> = (0..=10).map(|i| i as f32 / 10.0).collect();
    let simplified = simplify_keyframes(&values, 0.01);
    // A straight line only needs start and end
    assert_eq!(simplified.len(), 2);
    assert_eq!(simplified[0], (0, 0.0));
    assert_eq!(simplified[1], (10, 1.0));
}
