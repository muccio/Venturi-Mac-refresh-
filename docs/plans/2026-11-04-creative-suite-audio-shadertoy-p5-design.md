# Design Document: Creative Suite Integration (Audio Envelope, Shadertoy GPU, P5.js Canvas)

**Date:** 2026-11-04  
**Status:** Approved  
**Scope:** Porting DaVinci Resolve custom creative tools into native Venturi video editor subsystems with deep mutual interoperability.

---

## 1. Executive Summary

This project integrates three complementary creative features into Venturi:
1. **Audio Envelope & Transient Keyframe Generator:** Analyzes timeline audio mixes or external audio files, extracts custom frequency bands (FFT) and transient peak onsets, and injects frame-accurate keyframes into any animatable clip parameter.
2. **Shadertoy Native GPU Shader Engine:** A high-performance WGSL/GLSL shader generator and filter running on `wgpu` (Metal on macOS), providing Shadertoy-compatible uniforms (`iTime`, `iResolution`, `iChannel0..3`, audio uniforms), live code editing, and preset library.
3. **P5.js Creative Coding Canvas:** A creative JavaScript canvas generator and filter running an offscreen web rendering bridge, exposing `videoIn` texture sampling and `audioIn` reactive data, with ASCII Art and generative video filters.
4. **Universal Interoperability Pipeline:** A unified audio analysis engine powering live audio feeds for all three tools, cross-feature parameter keyframe targeting, and seamless GPU texture chaining between P5.js and Shadertoy.

---

## 2. Architecture & Interoperability

```mermaid
flowchart TD
    subgraph AudioPipeline["1. Core Audio Analysis Engine (vv-audio / vv-session)"]
        AudioSrc[Timeline Audio Mix / Audio File] --> FFTFilter[FFT Band Filter (Custom Low/High Hz)]
        AudioSrc --> TransientDet[Spectral Flux Transient / Peak Detector]
        FFTFilter --> SmoothEnv[Attack / Release Envelope Generator]
        TransientDet --> PeakPulse[Pulse / Beat Trigger Generator]
    end

    subgraph EnvelopeUI["Audio Envelope Dialog (vv-app)"]
        SmoothEnv --> WaveformPreview[Waveform & Envelope Visualizer]
        PeakPulse --> WaveformPreview
        WaveformPreview --> KeyframeBake[History Command: Bake Keyframes]
    end

    subgraph ModulationTargets["2. Keyframe Target Parameters (vv-core)"]
        KeyframeBake -->|Keyframed f32| TargetTransform[Clip Transform & Crop]
        KeyframeBake -->|Keyframed f32| TargetFilters[Clip Filters: Blur, Exposure, Shadow]
        KeyframeBake -->|Keyframed f32| TargetShadertoy[Shadertoy Custom Uniforms]
        KeyframeBake -->|Keyframed f32| TargetP5[P5.js Custom Sketch Parameters]
    end

    subgraph LiveAudioFeed["3. Live Real-Time Audio Modulation"]
        FFTFilter -->|Real-time spectrum / bands| ShaderAudioUniforms[Shadertoy iAudio Uniforms]
        FFTFilter -->|Real-time FFT & levels| P5AudioIn[P5.js audioIn API]
    end

    subgraph GPUChaining["4. GPU Texture Pipeline (vv-render)"]
        SourceVideo[Video Track / Clip] -->|videoIn| P5Canvas[P5.js Offscreen Canvas]
        P5Canvas -->|Rendered Texture / iChannel0| ShadertoyPass[Shadertoy WGSL Pass]
        ShadertoyPass --> FinalComposite[Compositor Output]
    end
```

---

## 3. Subsystem Detailed Designs

### 3.1 Subsystem 1: Audio Envelope & Transient Keyframe Generator

#### Location & Modules
- `crates/vv-audio/src/envelope.rs`: DSP algorithms for envelope extraction, custom FFT bandpass filtering, and transient onset detection.
- `crates/vv-core/src/command.rs`: Command for batch keyframe replacement/insertion with undo/redo support.
- `crates/vv-app/src/audio_envelope_dialog.rs`: Interactive Egui modal dialog with live curve preview and controls.

#### Core Capabilities
1. **Audio Sources:**
   - Any audio track in the current timeline (e.g. A1, A2, or the combined master mix).
   - Any audio/video media file on disk via `vv_media::decode`.
2. **Frequency Analysis:**
   - *Full Spectrum RMS:* Moving window root-mean-square.
   - *Preset Bands:* Sub-bass (20–60 Hz), Bass (60–250 Hz), Low-Mids (250–1000 Hz), High-Mids (1–4 kHz), Highs (4–20 kHz).
   - *Custom Bandpass:* User-definable `freq_low` and `freq_high` (20 Hz – 20,000 Hz) with logarithmic frequency sliders.
3. **Transient / Beat Detector:**
   - Spectral flux calculation (positive frame-to-frame difference of sub-band energy).
   - Configurable peak threshold and minimum peak distance (preventing double-triggers).
   - Peak pulse decay shaping (sharp spike vs. exponential falloff).
4. **Envelope Shaper:**
   - Attack time (ms) and Release time (ms) one-pole asymmetrical smoothing filter.
   - Noise gate threshold and sensitivity multiplier.
   - Inversion toggle ($1.0 - x$).
   - Output range mapping: linear remap to target parameter bounds $[Min, Max]$.
   - Keyframe decimation: removes redundant colinear keyframes to keep curves clean.
5. **Target Parameters:**
   - Transform: `ZoomX`, `ZoomY`, `ZoomUniform`, `PositionX`, `PositionY`, `Rotation`, `Opacity`, `CropLeft`, `CropRight`, `CropTop`, `CropBottom`.
   - Filters: `BlurRadius`, `ExposureAmount`, `ShadowOffset`, `ShadowGrow`.
   - Shadertoy custom parameters (`Param0..Param3`).
   - P5.js custom parameters.

---

### 3.2 Subsystem 2: Shadertoy Native GPU Shader Engine

#### Location & Modules
- `crates/vv-core/src/model.rs`: New `FilterKind::Shadertoy` and `Generator::Shadertoy` definitions.
- `crates/vv-render/src/shaders/shadertoy_runner.wgsl`: Shading template providing Shadertoy uniforms and GLSL compatibility wrappers.
- `crates/vv-render/src/compositor.rs`: Wgpu pipeline compiling and executing dynamic WGSL shaders with cached render pipelines.
- `crates/vv-app/src/shadertoy_editor.rs`: Multi-line code editor in the properties panel with live compilation status, error diagnostic mapping, and preset selector.

#### Core Capabilities
1. **Shadertoy Standard Uniforms:**
   - `iResolution: vec3<f32>` (viewport dimensions + aspect ratio).
   - `iTime: f32` (timeline position in seconds).
   - `iTimeDelta: f32` (frame duration).
   - `iFrame: i32` (timeline frame number).
   - `iFrameRate: f32` (timeline fps).
   - `iMouse: vec4<f32>` (interactive cursor coordinates).
   - `iDate: vec4<f32>` (year, month, day, time in seconds).
   - `iChannel0..3`: Texture samplers (incoming video layer, companion tracks, or procedural dummy).
   - `iChannelResolution: array<vec3<f32>, 4>`.
2. **Audio-Reactive Uniforms (Interoperability):**
   - `iAudioLevel: f32`: Current master RMS level.
   - `iBass: f32`, `iMids: f32`, `iTreble: f32`: Filtered frequency energy.
   - `iBeat: f32`: Transient pulse signal (1.0 on beat, decaying to 0.0).
3. **Preset Library:**
   - *Geometric Plasma 2D*
   - *Raymarching 3D SDF* (spheres, torus, Phong shading)
   - *Cyberpunk Synthwave Grid*
   - *Fractal Pyramid*
   - *Texture Ripple Warp* (effect filter on incoming clip)
   - *CRT Scanlines & Chromatic Aberration* (effect filter)
   - *VHS Glitch & Tracking Noise* (effect filter)

---

### 3.3 Subsystem 3: P5.js Creative Coding Canvas

#### Location & Modules
- `crates/vv-core/src/model.rs`: New `FilterKind::P5Canvas` and `Generator::P5Canvas`.
- `crates/vv-render/src/p5_bridge.rs`: Offscreen WebKit / headless rendering bridge capturing canvas frames into GPU memory.
- `crates/vv-app/src/p5_editor.rs`: Inspector controls, JavaScript editor, preset selector, and error console.

#### Core Capabilities
1. **APIs Available in Sketches:**
   - Full standard P5.js 2D and WebGL canvas drawing (`setup()`, `draw()`).
   - `videoIn`: Access to the incoming video frame via `image(videoIn, 0, 0)`, `videoIn.loadPixels()`, `videoIn.get(x, y)`.
   - `audioIn`: Access to audio analysis via `audioIn.getLevel()`, `audioIn.bass`, `audioIn.mid`, `audioIn.treble`, `audioIn.fft()`.
2. **Built-in Presets:**
   - *ASCII Art (Full Color)*
   - *ASCII Art (Monochrome B&W)*
   - *Particle Video Sampler*
   - *Generative Wave Matrix*

---

## 4. Implementation Phasing Strategy

To ensure strict single-flow execution and verify each milestone thoroughly:
- **Phase 1: Audio Envelope & Transient Keyframe Generator**
  - Implement DSP module in `vv-audio`.
  - Add batch keyframe commands in `vv-core`.
  - Build UI dialog with waveform visualizer in `vv-app`.
  - Verify unit tests and live keyframe generation on audio clips.
- **Phase 2: Shadertoy Native GPU Shader Engine**
  - Implement Shadertoy model types and uniforms in `vv-core`.
  - Add wgpu pipeline pass in `vv-render`.
  - Connect audio uniforms from the Phase 1 audio analysis engine.
  - Implement inspector code editor and presets in `vv-app`.
- **Phase 3: P5.js Creative Coding Canvas**
  - Implement headless canvas bridge and texture integration.
  - Expose `videoIn` and link Phase 1 `audioIn`.
  - Implement inspector editor and ASCII art presets.
- **Phase 4: Unified Interoperability & Polishing**
  - End-to-end texture chaining (P5.js -> Shadertoy -> Compositor).
  - Cross-module keyframe modulation testing.
  - Verification across unit tests and full app bundle build.
