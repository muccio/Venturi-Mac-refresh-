//! Audio envelope and transient keyframe generator dialog.

use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use vv_audio::envelope::{
    AnalysisMode, BandPreset, EnvelopeSettings, PeakSettings, compute_envelope, detect_peaks,
    simplify_keyframes,
};
use vv_core::command::{ApplyAudioKeyframes, Command, CompositeCommand, KeyframeTarget};
use vv_core::{
    ClipId, FilterKind, FrameIdx, Interpolation, Project, Rational, Timeline, TimelineId,
    TrackKind, TransformParam,
};

use rust_i18n::t;
use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AudioSourceChoice {
    TimelineMaster,
    TimelineTrack(usize),
    ExternalFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetParamChoice {
    ZoomUniform,
    ZoomX,
    ZoomY,
    PositionX,
    PositionY,
    Rotation,
    Opacity,
    CropLeft,
    CropRight,
    CropTop,
    CropBottom,
    BlurRadius,
    Exposure,
    ShadertoyParam0,
    ShadertoyParam1,
    ShadertoyParam2,
    ShadertoyParam3,
    P5Param0,
    P5Param1,
    P5Param2,
    P5Param3,
}

impl TargetParamChoice {
    pub const ALL: [Self; 21] = [
        Self::ZoomUniform,
        Self::ZoomX,
        Self::ZoomY,
        Self::PositionX,
        Self::PositionY,
        Self::Rotation,
        Self::Opacity,
        Self::CropLeft,
        Self::CropRight,
        Self::CropTop,
        Self::CropBottom,
        Self::BlurRadius,
        Self::Exposure,
        Self::ShadertoyParam0,
        Self::ShadertoyParam1,
        Self::ShadertoyParam2,
        Self::ShadertoyParam3,
        Self::P5Param0,
        Self::P5Param1,
        Self::P5Param2,
        Self::P5Param3,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ZoomUniform => "Zoom (Uniform X+Y)",
            Self::ZoomX => "Zoom X",
            Self::ZoomY => "Zoom Y",
            Self::PositionX => "Position X",
            Self::PositionY => "Position Y",
            Self::Rotation => "Rotation",
            Self::Opacity => "Opacity",
            Self::CropLeft => "Crop Left",
            Self::CropRight => "Crop Right",
            Self::CropTop => "Crop Top",
            Self::CropBottom => "Crop Bottom",
            Self::BlurRadius => "Blur Radius",
            Self::Exposure => "Exposure Amount",
            Self::ShadertoyParam0 => "Shadertoy Param 0",
            Self::ShadertoyParam1 => "Shadertoy Param 1",
            Self::ShadertoyParam2 => "Shadertoy Param 2",
            Self::ShadertoyParam3 => "Shadertoy Param 3",
            Self::P5Param0 => "P5.js Param 0",
            Self::P5Param1 => "P5.js Param 1",
            Self::P5Param2 => "P5.js Param 2",
            Self::P5Param3 => "P5.js Param 3",
        }
    }

    pub fn default_range(self) -> (f32, f32) {
        match self {
            Self::ZoomUniform | Self::ZoomX | Self::ZoomY => (1.0, 1.25),
            Self::PositionX | Self::PositionY => (-50.0, 50.0),
            Self::Rotation => (-5.0, 5.0),
            Self::Opacity => (0.0, 100.0),
            Self::CropLeft | Self::CropRight | Self::CropTop | Self::CropBottom => (0.0, 150.0),
            Self::BlurRadius => (0.0, 25.0),
            Self::Exposure => (0.0, 2.5),
            Self::ShadertoyParam0
            | Self::ShadertoyParam1
            | Self::ShadertoyParam2
            | Self::ShadertoyParam3
            | Self::P5Param0
            | Self::P5Param1
            | Self::P5Param2
            | Self::P5Param3 => (0.0, 1.0),
        }
    }

    pub fn to_keyframe_targets(self) -> Vec<KeyframeTarget> {
        match self {
            Self::ZoomUniform => vec![
                KeyframeTarget::TransformParam(TransformParam::ZoomX),
                KeyframeTarget::TransformParam(TransformParam::ZoomY),
            ],
            Self::ZoomX => vec![KeyframeTarget::TransformParam(TransformParam::ZoomX)],
            Self::ZoomY => vec![KeyframeTarget::TransformParam(TransformParam::ZoomY)],
            Self::PositionX => vec![KeyframeTarget::TransformParam(TransformParam::PositionX)],
            Self::PositionY => vec![KeyframeTarget::TransformParam(TransformParam::PositionY)],
            Self::Rotation => vec![KeyframeTarget::TransformParam(TransformParam::Rotation)],
            Self::Opacity => vec![KeyframeTarget::TransformParam(TransformParam::Opacity)],
            Self::CropLeft => vec![KeyframeTarget::TransformParam(TransformParam::CropLeft)],
            Self::CropRight => vec![KeyframeTarget::TransformParam(TransformParam::CropRight)],
            Self::CropTop => vec![KeyframeTarget::TransformParam(TransformParam::CropTop)],
            Self::CropBottom => vec![KeyframeTarget::TransformParam(TransformParam::CropBottom)],
            Self::BlurRadius => vec![KeyframeTarget::FilterRadius(FilterKind::GaussianBlur)],
            Self::Exposure => vec![KeyframeTarget::FilterAmount(FilterKind::Exposure)],
            Self::ShadertoyParam0 => vec![KeyframeTarget::FilterParam(FilterKind::Shadertoy, 0)],
            Self::ShadertoyParam1 => vec![KeyframeTarget::FilterParam(FilterKind::Shadertoy, 1)],
            Self::ShadertoyParam2 => vec![KeyframeTarget::FilterParam(FilterKind::Shadertoy, 2)],
            Self::ShadertoyParam3 => vec![KeyframeTarget::FilterParam(FilterKind::Shadertoy, 3)],
            Self::P5Param0 => vec![KeyframeTarget::FilterParam(FilterKind::P5Canvas, 0)],
            Self::P5Param1 => vec![KeyframeTarget::FilterParam(FilterKind::P5Canvas, 1)],
            Self::P5Param2 => vec![KeyframeTarget::FilterParam(FilterKind::P5Canvas, 2)],
            Self::P5Param3 => vec![KeyframeTarget::FilterParam(FilterKind::P5Canvas, 3)],
        }
    }
}

pub(crate) enum DialogOutcome {
    None,
    Bake(Box<dyn Command>),
    Cancel,
}

pub(crate) struct AudioEnvelopeDialog {
    pub target_clip: (usize, ClipId),
    pub source_choice: AudioSourceChoice,
    pub external_file: Option<PathBuf>,
    pub use_band_filter: bool,
    pub band_preset: Option<BandPreset>,
    pub custom_low_hz: f32,
    pub custom_high_hz: f32,
    pub peak_mode: bool,
    pub envelope_settings: EnvelopeSettings,
    pub peak_settings: PeakSettings,
    pub target_param: TargetParamChoice,
    pub custom_min: f32,
    pub custom_max: f32,
    pub simplify: bool,
    pub simplify_tolerance: f32,

    pub raw_samples: Vec<f32>,
    pub sample_rate: u32,
    pub envelope_preview: Vec<f32>,
    pub peaks_preview: Vec<f32>,
    pub num_frames: usize,
    pub status_text: Option<String>,
}

impl AudioEnvelopeDialog {
    pub fn new(target_clip: (usize, ClipId), num_frames: usize) -> Self {
        let default_param = TargetParamChoice::ZoomUniform;
        let (def_min, def_max) = default_param.default_range();

        Self {
            target_clip,
            source_choice: AudioSourceChoice::TimelineMaster,
            external_file: None,
            use_band_filter: false,
            band_preset: Some(BandPreset::Bass),
            custom_low_hz: 60.0,
            custom_high_hz: 250.0,
            peak_mode: false,
            envelope_settings: EnvelopeSettings {
                mode: AnalysisMode::Rms,
                attack_ms: 10.0,
                release_ms: 60.0,
                threshold: 0.0,
                sensitivity: 1.0,
                invert: false,
                min_out: def_min,
                max_out: def_max,
            },
            peak_settings: PeakSettings {
                enabled: false,
                threshold: 0.4,
                min_distance_frames: 6,
                decay_frames: 4,
            },
            target_param: default_param,
            custom_min: def_min,
            custom_max: def_max,
            simplify: true,
            simplify_tolerance: 0.01,

            raw_samples: Vec::new(),
            sample_rate: 48000,
            envelope_preview: Vec::new(),
            peaks_preview: Vec::new(),
            num_frames,
            status_text: None,
        }
    }

    pub fn load_timeline_audio(
        &mut self,
        project: &Project,
        timeline: &Timeline,
        timeline_start: FrameIdx,
        timeline_len: FrameIdx,
    ) {
        let fps = timeline.fps.as_f64().max(0.001);
        let start_sec = timeline_start as f64 / fps;
        let end_sec = (timeline_start + timeline_len) as f64 / fps;

        let sample_rate = 48000u64;
        let start_sample = (start_sec * sample_rate as f64).round() as u64;
        let end_sample = (end_sec * sample_rate as f64).round() as u64;

        if end_sample <= start_sample {
            return;
        }

        let mut mono = Vec::new();
        let mut level_cache = vv_session::audio_render::LevelCache::default();

        let track_idx = match self.source_choice {
            AudioSourceChoice::TimelineTrack(idx) => Some(idx),
            _ => None,
        };

        let res = if let Some(solo_idx) = track_idx {
            let mut solo_tl = timeline.clone();
            for (i, t) in solo_tl.tracks.iter_mut().enumerate() {
                if i != solo_idx && t.kind == TrackKind::Audio {
                    t.muted = true;
                }
            }
            vv_session::audio_render::render_offline(
                project,
                &solo_tl,
                start_sample..end_sample,
                &mut level_cache,
                |chunk| {
                    for frame in chunk.chunks_exact(2) {
                        mono.push((frame[0] + frame[1]) * 0.5);
                    }
                    ControlFlow::Continue(())
                },
            )
        } else {
            vv_session::audio_render::render_offline(
                project,
                timeline,
                start_sample..end_sample,
                &mut level_cache,
                |chunk| {
                    for frame in chunk.chunks_exact(2) {
                        mono.push((frame[0] + frame[1]) * 0.5);
                    }
                    ControlFlow::Continue(())
                },
            )
        };

        if let Err(e) = res {
            self.status_text = Some(format!("Audio render error: {:?}", e));
        } else {
            self.raw_samples = mono;
            self.sample_rate = 48000;
            self.recalculate(timeline.fps);
        }
    }

    pub fn load_file_audio(&mut self, path: &Path, timeline_fps: Rational) {
        match vv_media::decode_audio_track(path, 0) {
            Ok(Some(buf)) => {
                let channels = buf.channels.max(1) as usize;
                let mono: Vec<f32> = buf
                    .samples
                    .chunks_exact(channels)
                    .map(|c| c.iter().sum::<f32>() / channels as f32)
                    .collect();
                self.sample_rate = buf.sample_rate;
                self.raw_samples = mono;
                self.external_file = Some(path.to_path_buf());
                self.source_choice = AudioSourceChoice::ExternalFile;
                self.recalculate(timeline_fps);
            }
            Ok(None) => {
                self.status_text = Some("No audio stream found in file".into());
            }
            Err(e) => {
                self.status_text = Some(format!("Error loading audio file: {}", e));
            }
        }
    }

    pub fn recalculate(&mut self, fps: Rational) {
        let mode = if self.use_band_filter {
            AnalysisMode::Band(self.custom_low_hz, self.custom_high_hz)
        } else {
            AnalysisMode::Rms
        };

        self.envelope_settings.mode = mode;
        self.envelope_settings.min_out = self.custom_min;
        self.envelope_settings.max_out = self.custom_max;

        self.envelope_preview = compute_envelope(
            &self.raw_samples,
            self.sample_rate,
            fps,
            self.num_frames,
            &self.envelope_settings,
        );

        if self.peak_mode {
            self.peak_settings.enabled = true;
            self.peaks_preview = detect_peaks(&self.envelope_preview, &self.peak_settings);
        } else {
            self.peak_settings.enabled = false;
            self.peaks_preview.clear();
        }
    }

    pub fn build_bake_command(
        &self,
        timeline: TimelineId,
        source_offset: FrameIdx,
    ) -> Option<Box<dyn Command>> {
        let curve = if self.peak_mode && !self.peaks_preview.is_empty() {
            // Remap 0..1 peaks to min..max
            self.peaks_preview
                .iter()
                .map(|&p| self.custom_min + p * (self.custom_max - self.custom_min))
                .collect::<Vec<f32>>()
        } else {
            self.envelope_preview.clone()
        };

        if curve.is_empty() {
            return None;
        }

        let keyframe_pairs: Vec<(FrameIdx, f32)> = if self.simplify {
            let tol = (self.custom_max - self.custom_min).abs() * self.simplify_tolerance;
            simplify_keyframes(&curve, tol)
                .into_iter()
                .map(|(idx, val)| (source_offset + idx as FrameIdx, val))
                .collect()
        } else {
            curve
                .into_iter()
                .enumerate()
                .map(|(idx, val)| (source_offset + idx as FrameIdx, val))
                .collect()
        };

        let keyframes: Vec<(FrameIdx, f32, Interpolation)> = keyframe_pairs
            .into_iter()
            .map(|(f, v)| (f, v, Interpolation::Linear))
            .collect();

        let targets = self.target_param.to_keyframe_targets();
        let (track_idx, clip_id) = self.target_clip;

        if targets.len() == 1 {
            Some(Box::new(ApplyAudioKeyframes::new(
                timeline,
                track_idx,
                clip_id,
                targets[0],
                keyframes,
            )))
        } else {
            let commands: Vec<Box<dyn Command>> = targets
                .into_iter()
                .map(|target| {
                    Box::new(ApplyAudioKeyframes::new(
                        timeline,
                        track_idx,
                        clip_id,
                        target,
                        keyframes.clone(),
                    )) as Box<dyn Command>
                })
                .collect();
            Some(Box::new(CompositeCommand::new(
                vv_core::command::CommandLabel::SetKeyframe,
                commands,
            )))
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        timeline_id: TimelineId,
    ) -> DialogOutcome {
        let mut outcome = DialogOutcome::None;
        let mut open = true;

        let Some(timeline) = project.timelines.get(timeline_id) else {
            return DialogOutcome::Cancel;
        };

        egui::Window::new(t!("audio_envelope.title"))
            .id(egui::Id::new("audio_envelope_dialog"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .show(ctx, |ui| {
                let mut dirty = false;

                // Audio source selection
                ui.horizontal(|ui| {
                    ui.label(t!("audio_envelope.source"));
                    let current_label = match self.source_choice {
                        AudioSourceChoice::TimelineMaster => t!("audio_envelope.source_master").into_owned(),
                        AudioSourceChoice::TimelineTrack(idx) => {
                            format!("{}: A{}", t!("audio_envelope.source_track"), idx + 1)
                        }
                        AudioSourceChoice::ExternalFile => {
                            if let Some(path) = &self.external_file {
                                path.file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("File")
                                    .to_string()
                            } else {
                                t!("audio_envelope.source_file").into_owned()
                            }
                        }
                    };

                    egui::ComboBox::from_id_salt("audio_source_combo")
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    self.source_choice == AudioSourceChoice::TimelineMaster,
                                    t!("audio_envelope.source_master"),
                                )
                                .clicked()
                            {
                                self.source_choice = AudioSourceChoice::TimelineMaster;
                                dirty = true;
                            }
                            for (idx, track) in timeline.tracks.iter().enumerate() {
                                if track.kind == TrackKind::Audio {
                                    let label = format!(
                                        "{}: A{}",
                                        t!("audio_envelope.source_track"),
                                        idx + 1
                                    );
                                    if ui
                                        .selectable_label(
                                            self.source_choice
                                                == AudioSourceChoice::TimelineTrack(idx),
                                            label,
                                        )
                                        .clicked()
                                    {
                                        self.source_choice = AudioSourceChoice::TimelineTrack(idx);
                                        dirty = true;
                                    }
                                }
                            }
                            if ui
                                .selectable_label(
                                    self.source_choice == AudioSourceChoice::ExternalFile,
                                    t!("audio_envelope.source_file"),
                                )
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Audio/Video",
                                        &["wav", "mp3", "aac", "m4a", "flac", "ogg", "mp4", "mov"],
                                    )
                                    .pick_file()
                                {
                                    self.load_file_audio(&path, timeline.fps);
                                }
                            }
                        });
                });

                ui.separator();

                // Analysis mode & Band selection
                ui.horizontal(|ui| {
                    ui.label(t!("audio_envelope.analysis_mode"));
                    if ui
                        .selectable_label(!self.use_band_filter, t!("audio_envelope.mode_rms"))
                        .clicked()
                    {
                        self.use_band_filter = false;
                        dirty = true;
                    }
                    if ui
                        .selectable_label(self.use_band_filter, t!("audio_envelope.mode_band"))
                        .clicked()
                    {
                        self.use_band_filter = true;
                        dirty = true;
                    }
                });

                if self.use_band_filter {
                    ui.horizontal(|ui| {
                        ui.label(t!("audio_envelope.band_preset"));
                        let preset_label = self
                            .band_preset
                            .map(|p| p.label().to_string())
                            .unwrap_or_else(|| t!("audio_envelope.custom_band").into_owned());

                        egui::ComboBox::from_id_salt("band_preset_combo")
                            .selected_text(preset_label)
                            .show_ui(ui, |ui| {
                                for preset in BandPreset::ALL {
                                    if ui
                                        .selectable_label(
                                            self.band_preset == Some(preset),
                                            preset.label(),
                                        )
                                        .clicked()
                                    {
                                        self.band_preset = Some(preset);
                                        let (low, high) = preset.frequency_range();
                                        self.custom_low_hz = low;
                                        self.custom_high_hz = high;
                                        dirty = true;
                                    }
                                }
                                if ui
                                    .selectable_label(
                                        self.band_preset.is_none(),
                                        t!("audio_envelope.custom_band"),
                                    )
                                    .clicked()
                                {
                                    self.band_preset = None;
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label(t!("audio_envelope.freq_min"));
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.custom_low_hz)
                                    .range(20.0..=self.custom_high_hz)
                                    .speed(5.0)
                                    .suffix(" Hz"),
                            )
                            .changed()
                        {
                            self.band_preset = None;
                            dirty = true;
                        }

                        ui.label(t!("audio_envelope.freq_max"));
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.custom_high_hz)
                                    .range(self.custom_low_hz..=20000.0)
                                    .speed(10.0)
                                    .suffix(" Hz"),
                            )
                            .changed()
                        {
                            self.band_preset = None;
                            dirty = true;
                        }
                    });
                }

                ui.separator();

                // Modulation mode: Smooth Envelope vs Transient Peaks
                ui.horizontal(|ui| {
                    ui.label(t!("audio_envelope.detection_mode"));
                    if ui
                        .selectable_label(!self.peak_mode, t!("audio_envelope.type_envelope"))
                        .clicked()
                    {
                        self.peak_mode = false;
                        dirty = true;
                    }
                    if ui
                        .selectable_label(self.peak_mode, t!("audio_envelope.type_peaks"))
                        .clicked()
                    {
                        self.peak_mode = true;
                        dirty = true;
                    }
                });

                if !self.peak_mode {
                    ui.horizontal(|ui| {
                        ui.label(t!("audio_envelope.attack_ms"));
                        if ui
                            .add(
                                egui::Slider::new(&mut self.envelope_settings.attack_ms, 0.0..=500.0)
                                    .suffix(" ms"),
                            )
                            .changed()
                        {
                            dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("audio_envelope.release_ms"));
                        if ui
                            .add(
                                egui::Slider::new(&mut self.envelope_settings.release_ms, 5.0..=1000.0)
                                    .suffix(" ms"),
                            )
                            .changed()
                        {
                            dirty = true;
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(t!("audio_envelope.min_peak_dist"));
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.peak_settings.min_distance_frames)
                                    .range(1..=60),
                            )
                            .changed()
                        {
                            dirty = true;
                        }

                        ui.label(t!("audio_envelope.pulse_decay"));
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.peak_settings.decay_frames)
                                    .range(1..=30),
                            )
                            .changed()
                        {
                            dirty = true;
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.label(t!("audio_envelope.sensitivity"));
                    if ui
                        .add(
                            egui::Slider::new(&mut self.envelope_settings.sensitivity, 0.1..=5.0)
                                .logarithmic(true),
                        )
                        .changed()
                    {
                        dirty = true;
                    }

                    ui.label(t!("audio_envelope.threshold"));
                    let thresh = if self.peak_mode {
                        &mut self.peak_settings.threshold
                    } else {
                        &mut self.envelope_settings.threshold
                    };
                    if ui.add(egui::Slider::new(thresh, 0.0..=1.0)).changed() {
                        dirty = true;
                    }
                });

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut self.envelope_settings.invert, t!("audio_envelope.invert"))
                        .changed()
                    {
                        dirty = true;
                    }
                    ui.checkbox(&mut self.simplify, t!("audio_envelope.simplify"));
                });

                ui.separator();

                // Target parameter selection
                ui.horizontal(|ui| {
                    ui.label(t!("audio_envelope.target_param"));
                    egui::ComboBox::from_id_salt("target_param_combo")
                        .selected_text(self.target_param.label())
                        .show_ui(ui, |ui| {
                            for param in TargetParamChoice::ALL {
                                if ui
                                    .selectable_label(self.target_param == param, param.label())
                                    .clicked()
                                {
                                    self.target_param = param;
                                    let (min_val, max_val) = param.default_range();
                                    self.custom_min = min_val;
                                    self.custom_max = max_val;
                                    dirty = true;
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(t!("audio_envelope.min_val"));
                    if ui
                        .add(egui::DragValue::new(&mut self.custom_min).speed(0.1))
                        .changed()
                    {
                        dirty = true;
                    }

                    ui.label(t!("audio_envelope.max_val"));
                    if ui
                        .add(egui::DragValue::new(&mut self.custom_max).speed(0.1))
                        .changed()
                    {
                        dirty = true;
                    }
                });

                ui.separator();

                // Interactive Waveform & Envelope Painter
                ui.label(t!("audio_envelope.preview"));
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 100.0),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(rect);

                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 20, 24));
                painter.rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                    egui::StrokeKind::Inside,
                );

                if !self.raw_samples.is_empty() {
                    let w = rect.width();
                    let h = rect.height();
                    let mid_y = rect.min.y + h * 0.5;

                    // Draw waveform in muted slate
                    let samples_per_pixel = (self.raw_samples.len() as f32 / w).max(1.0);
                    for pixel in 0..(w as usize) {
                        let x = rect.min.x + pixel as f32;
                        let start = (pixel as f32 * samples_per_pixel) as usize;
                        let end = (((pixel + 1) as f32 * samples_per_pixel) as usize)
                            .min(self.raw_samples.len());
                        if start < end {
                            let mut min_s = 0.0f32;
                            let mut max_s = 0.0f32;
                            for &s in &self.raw_samples[start..end] {
                                if s < min_s {
                                    min_s = s;
                                }
                                if s > max_s {
                                    max_s = s;
                                }
                            }
                            let y1 = mid_y - (max_s.min(1.0) * h * 0.45);
                            let y2 = mid_y - (min_s.max(-1.0) * h * 0.45);
                            painter.line_segment(
                                [egui::pos2(x, y1), egui::pos2(x, y2)],
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 75, 90)),
                            );
                        }
                    }

                    // Draw envelope curve or peaks
                    let curve = if self.peak_mode && !self.peaks_preview.is_empty() {
                        &self.peaks_preview
                    } else {
                        &self.envelope_preview
                    };

                    if curve.len() > 1 {
                        let range = (self.custom_max - self.custom_min).abs().max(1e-4);
                        let points: Vec<egui::Pos2> = curve
                            .iter()
                            .enumerate()
                            .map(|(i, &v)| {
                                let norm = if self.peak_mode {
                                    v.clamp(0.0, 1.0)
                                } else {
                                    ((v - self.custom_min) / range).clamp(0.0, 1.0)
                                };
                                let x = rect.min.x + (i as f32 / (curve.len() - 1) as f32) * w;
                                let y = rect.max.y - (norm * h * 0.9 + h * 0.05);
                                egui::pos2(x, y)
                            })
                            .collect();

                        painter.add(egui::Shape::line(
                            points,
                            egui::Stroke::new(2.0, theme::ACCENT),
                        ));
                    }

                    // Draw threshold line
                    let thresh = if self.peak_mode {
                        self.peak_settings.threshold
                    } else {
                        self.envelope_settings.threshold
                    };
                    let thresh_y = rect.max.y - (thresh * h * 0.9 + h * 0.05);
                    painter.line_segment(
                        [egui::pos2(rect.min.x, thresh_y), egui::pos2(rect.max.x, thresh_y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 120, 50)),
                    );
                }

                if let Some(err) = &self.status_text {
                    ui.colored_label(egui::Color32::from_rgb(230, 90, 80), err);
                }

                ui.separator();

                // Buttons
                ui.horizontal(|ui| {
                    if ui.button(t!("audio_envelope.bake_button")).clicked() {
                        let (track_idx, clip_id) = self.target_clip;
                        if let Some(clip) = timeline.clip(track_idx, clip_id) {
                            if let Some(cmd) = self.build_bake_command(timeline_id, clip.source_offset) {
                                outcome = DialogOutcome::Bake(cmd);
                            }
                        }
                    }
                    if ui.button(t!("audio_envelope.cancel_button")).clicked() {
                        outcome = DialogOutcome::Cancel;
                    }
                });

                if dirty {
                    if self.source_choice != AudioSourceChoice::ExternalFile {
                        let (track_idx, clip_id) = self.target_clip;
                        if let Some(clip) = timeline.clip(track_idx, clip_id) {
                            self.load_timeline_audio(
                                project,
                                timeline,
                                clip.timeline_start,
                                clip.timeline_len,
                            );
                        }
                    } else {
                        self.recalculate(timeline.fps);
                    }
                }
            });

        if !open {
            outcome = DialogOutcome::Cancel;
        }

        outcome
    }
}

impl super::VenturiApp {
    pub(crate) fn open_audio_envelope_dialog(&mut self, track_idx: usize, clip_id: ClipId) {
        let Some(timeline_id) = self.timeline_id else {
            return;
        };
        let tl = &self.session.project.timelines[timeline_id];
        let Some(clip) = tl.clip(track_idx, clip_id) else {
            return;
        };
        let mut dialog = AudioEnvelopeDialog::new((track_idx, clip_id), clip.timeline_len as usize);
        dialog.load_timeline_audio(&self.session.project, tl, clip.timeline_start, clip.timeline_len);
        self.audio_envelope_dialog = Some(dialog);
    }

    pub(crate) fn show_audio_envelope_dialog(&mut self, ctx: &egui::Context) {
        let Some(timeline_id) = self.timeline_id else {
            return;
        };
        let mut close = false;
        if let Some(dialog) = &mut self.audio_envelope_dialog {
            match dialog.show(ctx, &self.session.project, timeline_id) {
                DialogOutcome::Bake(cmd) => {
                    self.session.history.do_command(&mut self.session.project, cmd);
                    close = true;
                }
                DialogOutcome::Cancel => {
                    close = true;
                }
                DialogOutcome::None => {}
            }
        }
        if close {
            self.audio_envelope_dialog = None;
        }
    }
}

#[cfg(test)]
#[path = "tests/audio_envelope_dialog.rs"]
mod tests;
