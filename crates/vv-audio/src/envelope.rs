//! Audio envelope extraction, customizable frequency band filtering, and
//! transient/beat peak detection for driving animation keyframes.

use vv_core::Rational;

/// Common frequency band presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandPreset {
    SubBass,
    Bass,
    LowMids,
    HighMids,
    Highs,
}

impl BandPreset {
    pub const ALL: [Self; 5] = [
        Self::SubBass,
        Self::Bass,
        Self::LowMids,
        Self::HighMids,
        Self::Highs,
    ];

    pub fn frequency_range(self) -> (f32, f32) {
        match self {
            Self::SubBass => (20.0, 60.0),
            Self::Bass => (60.0, 250.0),
            Self::LowMids => (250.0, 1000.0),
            Self::HighMids => (1000.0, 4000.0),
            Self::Highs => (4000.0, 20000.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SubBass => "Sub-Bass (20-60 Hz)",
            Self::Bass => "Bass (60-250 Hz)",
            Self::LowMids => "Low-Mids (250-1000 Hz)",
            Self::HighMids => "High-Mids (1-4 kHz)",
            Self::Highs => "Highs (4-20 kHz)",
        }
    }
}

/// Analysis frequency scope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisMode {
    Rms,
    Band(f32, f32),
}

/// Configuration for audio envelope calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopeSettings {
    pub mode: AnalysisMode,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub threshold: f32,
    pub sensitivity: f32,
    pub invert: bool,
    pub min_out: f32,
    pub max_out: f32,
}

impl Default for EnvelopeSettings {
    fn default() -> Self {
        Self {
            mode: AnalysisMode::Rms,
            attack_ms: 10.0,
            release_ms: 50.0,
            threshold: 0.0,
            sensitivity: 1.0,
            invert: false,
            min_out: 0.0,
            max_out: 1.0,
        }
    }
}

/// Configuration for transient peak and beat detection.
#[derive(Debug, Clone, PartialEq)]
pub struct PeakSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub min_distance_frames: usize,
    pub decay_frames: usize,
}

impl Default for PeakSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.5,
            min_distance_frames: 4,
            decay_frames: 3,
        }
    }
}

/// Computes an envelope curve sampled at `fps` across `num_frames`.
pub fn compute_envelope(
    samples: &[f32],
    sample_rate: u32,
    fps: Rational,
    num_frames: usize,
    settings: &EnvelopeSettings,
) -> Vec<f32> {
    if num_frames == 0 || samples.is_empty() {
        return vec![settings.min_out; num_frames];
    }

    let mut raw_energy = Vec::with_capacity(num_frames);
    let fps_f64 = fps.as_f64().max(0.001);
    let sr_f64 = sample_rate as f64;

    for frame in 0..num_frames {
        let start = ((frame as f64 * sr_f64) / fps_f64).round() as usize;
        let end = (((frame + 1) as f64 * sr_f64) / fps_f64).round() as usize;
        let start_clamped = start.min(samples.len());
        let end_clamped = end.min(samples.len());

        let window = if start_clamped < end_clamped {
            &samples[start_clamped..end_clamped]
        } else {
            &[]
        };

        let energy = match settings.mode {
            AnalysisMode::Rms => compute_rms(window),
            AnalysisMode::Band(low, high) => compute_band_energy(window, sample_rate, low, high),
        };
        raw_energy.push(energy);
    }

    let dt = 1.0 / fps_f64 as f32;
    let alpha_att = if settings.attack_ms > 0.0 {
        (-dt / (settings.attack_ms * 0.001)).exp()
    } else {
        0.0
    };
    let alpha_rel = if settings.release_ms > 0.0 {
        (-dt / (settings.release_ms * 0.001)).exp()
    } else {
        0.0
    };

    let mut smoothed = Vec::with_capacity(num_frames);
    let mut prev = 0.0f32;

    for &raw in &raw_energy {
        let amplified = raw * settings.sensitivity;
        let gated = if amplified <= settings.threshold {
            0.0
        } else if settings.threshold < 1.0 {
            ((amplified - settings.threshold) / (1.0 - settings.threshold)).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let current = if gated > prev {
            alpha_att * prev + (1.0 - alpha_att) * gated
        } else {
            alpha_rel * prev + (1.0 - alpha_rel) * gated
        };
        prev = current;

        let normalized = if settings.invert {
            1.0 - current
        } else {
            current
        };
        let mapped = settings.min_out + normalized * (settings.max_out - settings.min_out);
        smoothed.push(mapped);
    }

    smoothed
}

fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn compute_band_energy(samples: &[f32], sample_rate: u32, low_hz: f32, high_hz: f32) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let n = samples.len().next_power_of_two().max(256);
    let mut re = vec![0.0f32; n];
    let mut im = vec![0.0f32; n];

    for (i, &s) in samples.iter().enumerate() {
        let hann = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / samples.len() as f32).cos();
        re[i] = s * hann;
    }

    crate::spectrum::fft(&mut re, &mut im);

    let bin_hz = sample_rate as f32 / n as f32;
    let min_bin = ((low_hz / bin_hz).floor() as usize).min(n / 2);
    let max_bin = ((high_hz / bin_hz).ceil() as usize).min(n / 2);

    let mut band_sum = 0.0f32;
    for k in min_bin..=max_bin {
        band_sum += re[k] * re[k] + im[k] * im[k];
    }

    // Normalization factor for windowed FFT
    let scale = 2.0 / samples.len() as f32;
    (band_sum.sqrt() * scale).min(1.0)
}

/// Detects transient peaks and onsets, returning pulse values in 0.0..=1.0 per frame.
pub fn detect_peaks(envelope: &[f32], settings: &PeakSettings) -> Vec<f32> {
    let mut peaks = vec![0.0f32; envelope.len()];
    if !settings.enabled || envelope.is_empty() {
        return peaks;
    }

    let mut last_peak_frame: Option<usize> = None;

    for i in 0..envelope.len() {
        let val = envelope[i];
        let prev = if i > 0 { envelope[i - 1] } else { 0.0 };
        let is_rising = val > prev && val >= settings.threshold;

        let distance_ok = match last_peak_frame {
            Some(last) => i.saturating_sub(last) >= settings.min_distance_frames,
            None => true,
        };

        if is_rising && distance_ok {
            peaks[i] = 1.0;
            last_peak_frame = Some(i);
        } else if let Some(last) = last_peak_frame {
            let elapsed = i.saturating_sub(last);
            if elapsed < settings.decay_frames && settings.decay_frames > 0 {
                peaks[i] = 1.0 - (elapsed as f32 / settings.decay_frames as f32);
            }
        }
    }

    peaks
}

/// Simplifies a sequence of values using Ramer-Douglas-Peucker to remove redundant colinear keyframes.
pub fn simplify_keyframes(values: &[f32], tolerance: f32) -> Vec<(usize, f32)> {
    if values.is_empty() {
        return Vec::new();
    }
    if values.len() <= 2 {
        return values.iter().copied().enumerate().collect();
    }

    let mut keep = vec![false; values.len()];
    keep[0] = true;
    keep[values.len() - 1] = true;

    rdp_step(values, 0, values.len() - 1, tolerance, &mut keep);

    values
        .iter()
        .copied()
        .enumerate()
        .filter(|&(idx, _)| keep[idx])
        .collect()
}

fn rdp_step(values: &[f32], start: usize, end: usize, tolerance: f32, keep: &mut [bool]) {
    if end <= start + 1 {
        return;
    }

    let mut max_dist = 0.0f32;
    let mut index = 0;

    let x1 = start as f32;
    let y1 = values[start];
    let x2 = end as f32;
    let y2 = values[end];
    let dx = x2 - x1;
    let dy = y2 - y1;
    let line_len_sq = dx * dx + dy * dy;

    for i in (start + 1)..end {
        let x0 = i as f32;
        let y0 = values[i];

        let dist = if line_len_sq < 1e-6 {
            let ddx = x0 - x1;
            let ddy = y0 - y1;
            (ddx * ddx + ddy * ddy).sqrt()
        } else {
            let num = ((y2 - y1) * x0 - (x2 - x1) * y0 + x2 * y1 - y2 * x1).abs();
            num / line_len_sq.sqrt()
        };

        if dist > max_dist {
            max_dist = dist;
            index = i;
        }
    }

    if max_dist > tolerance {
        keep[index] = true;
        rdp_step(values, start, index, tolerance, keep);
        rdp_step(values, index, end, tolerance, keep);
    }
}

#[cfg(test)]
#[path = "tests/envelope.rs"]
mod tests;
