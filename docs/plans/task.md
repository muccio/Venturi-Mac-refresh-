| Task | Status | Details |
|---|---|---|
| Brainstorming & Design Document | Completed | Authored and committed design document for Audio Envelope, Shadertoy, and P5.js |
| Phase 1: Audio Envelope Generator | Completed | Implemented DSP analysis, ApplyAudioKeyframes command, AudioEnvelopeDialog UI, timeline & keyframe editor integration, full test coverage |
| Phase 2: Shadertoy Native GPU Shader Engine | Completed | Implemented Shadertoy filter in vv-core & vv-render with GLSL/WGSL runner, presets, live audio reactivity uniforms, and properties panel |
| Phase 3: P5.js Creative Coding Canvas | Completed | Implemented P5.js canvas bridge in vv-core & vv-render with presets (ASCII art, particle sampler, wave matrix), audio reactivity, and UI editor |
| Phase 4: Suite Interoperability & Build | Completed | Verified end-to-end chaining (Audio -> Shadertoy -> P5), 1,599 workspace unit tests passing, and release build Venturi.app + Venturi-arm64.dmg |
| Fix: Animated Shaders Playhead & Time Sync | Completed | Added source_frame to Shadertoy & P5 values, dynamic time/frame advancement in viewer & export, and real-time audio reactivity uniforms |
