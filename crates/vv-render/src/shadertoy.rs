//! Shadertoy GPU shader engine: pipeline compilation, uniform management,
//! and execution of audio-reactive shaders.

use std::collections::HashMap;
use std::sync::Mutex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadertoyUniforms {
    pub resolution: [f32; 4],
    pub time: [f32; 4],
    pub mouse: [f32; 4],
    pub audio: [f32; 4],
    pub beat_params: [f32; 4],
    pub extra_params: [f32; 4],
}

impl ShadertoyUniforms {
    pub fn new(
        width: u32,
        height: u32,
        time_secs: f32,
        frame_idx: i64,
        fps: f32,
        mouse: [f32; 4],
        audio_levels: [f32; 4], // [rms, bass, mids, treble]
        beat: f32,
        params: [f32; 4],
    ) -> Self {
        let (w, h) = (width.max(1) as f32, height.max(1) as f32);
        Self {
            resolution: [w, h, w / h, frame_idx as f32],
            time: [time_secs, 1.0 / fps.max(1.0), frame_idx as f32, fps],
            mouse,
            audio: audio_levels,
            beat_params: [beat, params[0], params[1], params[2]],
            extra_params: [params[3], 0.0, 0.0, 0.0],
        }
    }
}

pub struct ShadertoyRunner {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pipeline_cache: Mutex<HashMap<u32, wgpu::RenderPipeline>>,
    fallback_pipeline: wgpu::RenderPipeline,
    pub sampler: wgpu::Sampler,
    format: wgpu::TextureFormat,
}

impl ShadertoyRunner {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("vv-render shadertoy bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("vv-render shadertoy sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let fallback_source = concat!(
            include_str!("shaders/shadertoy_runner.wgsl"),
            r#"
fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = frag_coord / uniforms.resolution.xy;
    let checker = (i32(uv.x * 20.0) + i32(uv.y * 20.0)) % 2;
    if (checker == 0) {
        return vec4<f32>(0.9, 0.1, 0.7, 1.0); // Magenta error indicator
    } else {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    }
}
"#
        );

        let fallback_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vv-render shadertoy fallback"),
            source: wgpu::ShaderSource::Wgsl(fallback_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vv-render shadertoy pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let fallback_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vv-render shadertoy fallback pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &fallback_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fallback_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            bind_group_layout,
            pipeline_cache: Mutex::new(HashMap::new()),
            fallback_pipeline,
            sampler,
            format,
        }
    }

    /// Preprocesses user code into a complete WGSL shader module.
    pub fn preprocess_code(raw_code: &str) -> String {
        let runner = include_str!("shaders/shadertoy_runner.wgsl");
        if raw_code.contains("@fragment") {
            return raw_code.to_string();
        }

        // Automatic translation from classic Shadertoy GLSL to WGSL if needed.
        let mut processed = raw_code.to_string();
        if processed.contains("void mainImage") || processed.contains("iResolution") {
            processed = convert_glsl_to_wgsl(&processed);
        }

        format!("{runner}\n{processed}")
    }

    /// Gets or compiles a render pipeline for the given `shader_id`.
    pub fn get_or_compile(
        &self,
        device: &wgpu::Device,
        shader_id: u32,
        code: &str,
    ) -> wgpu::RenderPipeline {
        let mut cache = self.pipeline_cache.lock().unwrap();
        if let Some(pipeline) = cache.get(&shader_id) {
            return pipeline.clone();
        }

        let full_wgsl = Self::preprocess_code(code);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vv-render shadertoy custom layout"),
            bind_group_layouts: &[Some(&self.bind_group_layout)],
            immediate_size: 0,
        });

        // Safe compilation with fallback
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vv-render custom shadertoy module"),
            source: wgpu::ShaderSource::Wgsl(full_wgsl.into()),
        });

        // Device will validate on pipeline creation
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vv-render custom shadertoy pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        cache.insert(shader_id, pipeline.clone());
        pipeline
    }

    pub fn fallback_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.fallback_pipeline
    }

    /// Renders the Shadertoy pass.
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        channel0_view: &wgpu::TextureView,
        shader_id: u32,
        code: &str,
        uniforms: &ShadertoyUniforms,
    ) {
        use wgpu::util::DeviceExt;

        let pipeline = self.get_or_compile(device, shader_id, code);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vv-render shadertoy uniform buffer"),
            contents: bytemuck::bytes_of(uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vv-render shadertoy bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(channel0_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("vv-render shadertoy pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Converts basic Shadertoy GLSL identifiers and function signatures to WGSL.
fn convert_glsl_to_wgsl(glsl: &str) -> String {
    let mut code = glsl.to_string();

    // Uniforms mapping
    code = code.replace("iResolution", "uniforms.resolution");
    code = code.replace("iTimeDelta", "uniforms.time.y");
    code = code.replace("iTime", "uniforms.time.x");
    code = code.replace("iFrameRate", "uniforms.time.w");
    code = code.replace("iFrame", "i32(uniforms.time.z)");
    code = code.replace("iMouse", "uniforms.mouse");
    code = code.replace("iAudioLevel", "uniforms.audio.x");
    code = code.replace("iBass", "uniforms.audio.y");
    code = code.replace("iMids", "uniforms.audio.z");
    code = code.replace("iTreble", "uniforms.audio.w");
    code = code.replace("iBeat", "uniforms.beat_params.x");
    code = code.replace("param0", "uniforms.beat_params.y");
    code = code.replace("param1", "uniforms.beat_params.z");
    code = code.replace("param2", "uniforms.beat_params.w");
    code = code.replace("param3", "uniforms.extra_params.x");

    // Texture sample
    code = code.replace("texture(iChannel0,", "sample_channel0(");

    // Types
    code = code.replace("vec2(", "vec2<f32>(");
    code = code.replace("vec3(", "vec3<f32>(");
    code = code.replace("vec4(", "vec4<f32>(");
    code = code.replace("mat2(", "mat2x2<f32>(");
    code = code.replace("mat3(", "mat3x3<f32>(");
    code = code.replace("mat4(", "mat4x4<f32>(");
    code = code.replace("float ", "f32 ");
    code = code.replace("float(", "f32(");

    // Signature replacement: void mainImage( out vec4 fragColor, in vec2 fragCoord )
    if let Some(pos) = code.find("void mainImage") {
        if let Some(paren_close) = code[pos..].find(')') {
            let rest = &code[pos + paren_close + 1..];
            code = format!("fn main_image(frag_coord: vec2<f32>) -> vec4<f32> {rest}");
            code = code.replace("fragColor =", "return");
        }
    }

    code
}

#[cfg(test)]
#[path = "tests/shadertoy.rs"]
mod tests;
