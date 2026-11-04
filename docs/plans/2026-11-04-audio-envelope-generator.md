# Audio Envelope & Transient Keyframe Generator Implementation Plan

> **For Antigravity:** REQUIRED WORKFLOW: Use `.agent/workflows/execute-plan.md` to execute this plan in single-flow mode.

**Goal:** Build a native Audio Envelope & Transient/Beat Keyframe Generator in Venturi that extracts audio envelope or frequency-band transients and injects frame-accurate keyframes into any animatable clip parameter with full Undo/Redo.

**Architecture:** 
- Core DSP algorithms implemented in `crates/vv-audio/src/envelope.rs` using existing Hann FFT routines and exponential attack/release smoothing.
- Reversible keyframe injection command implemented in `crates/vv-core/src/command.rs`.
- Timeline audio extraction and interactive Egui dialog with real-time waveform/envelope visualizer implemented in `crates/vv-app/src/audio_envelope_dialog.rs`.

**Tech Stack:** Rust, `vv-audio` (FFT/DSP), `vv-core` (Keyframed/History/Command), `vv-session` (offline audio render), `egui` (GUI and painter).

---

### Task 1: Core DSP Envelope & Transient Detection in `vv-audio`

**Files:**
- Create: `crates/vv-audio/src/envelope.rs`
- Create: `crates/vv-audio/src/tests/envelope.rs`
- Modify: `crates/vv-audio/src/lib.rs`

**Step 1: Write the failing unit tests**
In `crates/vv-audio/src/tests/envelope.rs`:
- Test 1: `rms_envelope_of_constant_amplitude_sine_is_constant`
- Test 2: `bandpass_filter_isolates_frequency_band` (verifying 50 Hz tone is captured by SubBass 20-60 Hz and rejected by Treble 2000-8000 Hz)
- Test 3: `attack_and_release_smoothing` (verifying slow attack ramps up gradually and slow release decays gradually)
- Test 4: `transient_peak_detector_finds_onsets` (verifying isolated spikes trigger peak events with minimum distance guard)
- Test 5: `simplify_keyframes_reduces_colinear_points`

**Step 2: Run test to verify it fails**
Run: `cargo test -p vv-audio --test envelope`
Expected: FAIL with module or types not found.

**Step 3: Implement minimal DSP in `crates/vv-audio/src/envelope.rs`**
- `BandPreset` enum (`SubBass`, `Bass`, `LowMids`, `HighMids`, `Highs`, `Custom`).
- `AnalysisMode` enum (`Rms`, `Band(f32, f32)`).
- `EnvelopeSettings` struct (mode, attack_ms, release_ms, threshold, sensitivity, invert, min_out, max_out).
- `PeakSettings` struct (enabled, threshold, min_distance_frames, decay_frames).
- `compute_envelope(samples: &[f32], sample_rate: u32, fps: vv_core::Rational, num_frames: usize, settings: &EnvelopeSettings) -> Vec<f32>`
- `detect_peaks(envelope: &[f32], settings: &PeakSettings) -> Vec<f32>`
- `simplify_keyframes(values: &[f32], tolerance: f32) -> Vec<(usize, f32)>`

**Step 4: Run tests to verify they pass**
Run: `cargo test -p vv-audio envelope`
Expected: PASS with all tests passing.

**Step 5: Commit**
```bash
git add crates/vv-audio/src/envelope.rs crates/vv-audio/src/tests/envelope.rs crates/vv-audio/src/lib.rs
git commit -m "feat(audio): implement audio envelope and transient detection DSP"
```

---

### Task 2: Reversible Keyframe Batch Command in `vv-core`

**Files:**
- Modify: `crates/vv-core/src/command.rs`
- Modify: `crates/vv-core/src/tests/command.rs`

**Step 1: Write the failing unit test**
In `crates/vv-core/src/tests/command.rs`:
- Test: `apply_audio_keyframes_modifies_param_and_undoes_cleanly`
  - Applies keyframes to `TransformParam::ZoomX`.
  - Verifies keyframes exist at target frames.
  - Calls `history.undo(&mut project)` and verifies original constant or keyframe state is restored.
  - Calls `history.redo(&mut project)` and verifies keyframes are re-applied.

**Step 2: Run test to verify it fails**
Run: `cargo test -p vv-core command::tests::apply_audio_keyframes`
Expected: FAIL with command not defined.

**Step 3: Implement `ApplyAudioKeyframes` command**
In `crates/vv-core/src/command.rs`:
- Define `KeyframeTargetParam` enum (`Transform(TransformParam)`, `FilterRadius(usize)`, `FilterAmount(usize)`, `FilterOffset(usize, Axis)`).
- Implement `ApplyAudioKeyframes` struct holding `clip_id`, `target`, `new_keyframes`, and `saved_state` for undo.
- Implement `Command` trait for `ApplyAudioKeyframes`.

**Step 4: Run tests to verify they pass**
Run: `cargo test -p vv-core command::tests::apply_audio_keyframes`
Expected: PASS.

**Step 5: Commit**
```bash
git add crates/vv-core/src/command.rs crates/vv-core/src/tests/command.rs
git commit -m "feat(core): implement ApplyAudioKeyframes reversible command"
```

---

### Task 3: Timeline Audio Extraction in `vv-app`

**Files:**
- Create: `crates/vv-app/src/audio_envelope_dialog.rs`
- Create: `crates/vv-app/src/tests/audio_envelope_dialog.rs`

**Step 1: Write the failing unit test**
In `crates/vv-app/src/tests/audio_envelope_dialog.rs`:
- Test: `extract_audio_samples_for_timeline_range`
  - Creates a timeline with an audio clip.
  - Extracts the audio segment corresponding to a video clip's timeframe.
  - Verifies sample count matches `(duration_secs * 48000)`.

**Step 2: Run test to verify it fails**
Run: `cargo test -p vv-app audio_envelope_dialog`
Expected: FAIL.

**Step 3: Implement audio extraction helper**
In `crates/vv-app/src/audio_envelope_dialog.rs`:
- Add `extract_timeline_audio(project: &Project, timeline_id: TimelineId, track_idx: Option<usize>, range: Range<FrameIdx>, sample_rate: u32) -> Result<Vec<f32>, String>`
- Downmix stereo/multichannel to mono.

**Step 4: Run tests to verify they pass**
Run: `cargo test -p vv-app audio_envelope_dialog`
Expected: PASS.

**Step 5: Commit**
```bash
git add crates/vv-app/src/audio_envelope_dialog.rs crates/vv-app/src/tests/audio_envelope_dialog.rs
git commit -m "feat(app): add audio extraction for timeline envelope analysis"
```

---

### Task 4: Interactive "Audio to Keyframes" Dialog & UI Integration

**Files:**
- Modify: `crates/vv-app/src/audio_envelope_dialog.rs`
- Modify: `crates/vv-app/src/main.rs`
- Modify: `crates/vv-app/src/keyframe_editor.rs`
- Modify: `crates/vv-app/src/timeline_ui.rs`
- Modify: `crates/vv-app/locales/en.json` (and `it.json`)

**Step 1: Write UI tests**
In `crates/vv-app/src/tests/audio_envelope_dialog.rs`:
- Test: `dialog_state_defaults_and_param_mapping`
  - Verifies default min/max ranges for Zoom (1.00..1.25), Opacity (0.0..100.0), Blur (0.0..15.0).
  - Verifies curve painter allocates preview rect without panicking.

**Step 2: Run test to verify it fails**
Run: `cargo test -p vv-app audio_envelope_dialog`
Expected: FAIL.

**Step 3: Implement Dialog UI**
In `crates/vv-app/src/audio_envelope_dialog.rs`:
- `AudioEnvelopeDialog` struct holding UI state, parameters, cached waveform, and calculated envelope curve.
- Egui window `show(&mut self, ctx: &egui::Context, project: &mut Project, history: &mut History, timeline_id: TimelineId, clip_id: ClipId)`
- Interactive waveform & envelope preview painter with threshold line.
- Frequency range controls with slider / log scale.
- Transient mode toggle with peak markers.
- Parameter selection dropdown.
- "Bake Keyframes" button creating `ApplyAudioKeyframes` command.
In `crates/vv-app/src/timeline_ui.rs` & `keyframe_editor.rs`:
- Add "Audio to Keyframes..." to clip context menu and Keyframe Editor header bar.

**Step 4: Run tests to verify they pass**
Run: `cargo test -p vv-app`
Expected: PASS all tests.

**Step 5: Commit**
```bash
git add crates/vv-app/src/audio_envelope_dialog.rs crates/vv-app/src/main.rs crates/vv-app/src/keyframe_editor.rs crates/vv-app/src/timeline_ui.rs crates/vv-app/locales/
git commit -m "feat(app): integrate interactive Audio to Keyframes dialog"
```

---

### Task 5: End-to-End Verification & Documentation Update

**Files:**
- Modify: `docs/plans/task.md`

**Step 1: Run comprehensive tests across workspace**
- `cargo test -p vv-audio`
- `cargo test -p vv-core`
- `cargo test -p vv-app`

**Step 2: Verify binary compilation**
- Run `./scripts/build-macos.sh`

**Step 3: Update `docs/plans/task.md` with verification evidence**
- Document test outcomes and verification status.

**Step 4: Commit**
```bash
git add docs/plans/task.md
git commit -m "docs: record Phase 1 verification evidence"
```
