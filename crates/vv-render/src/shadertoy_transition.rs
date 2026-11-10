//! Shadertoy GPU Video Transition Engine.
//!
//! Compiles, caches, and executes two-input shader transitions matching the
//! 20 built-in presets (and custom shaders).

use std::collections::HashMap;
use std::sync::Mutex;
use vv_core::ShadertoyTransitionPreset;
use crate::compositor::OutputFrame;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransitionUniforms {
    pub resolution: [f32; 4],
    pub transition: [f32; 4],
    pub params0: [f32; 4],
    pub params1: [f32; 4],
    pub audio: [f32; 4],
    pub beat_params: [f32; 4],
}

impl TransitionUniforms {
    pub fn new(
        width: u32,
        height: u32,
        progress: f32,
        params: [f32; 8],
        output: &OutputFrame,
    ) -> Self {
        let (w, h) = (width.max(1) as f32, height.max(1) as f32);
        let aspect = w / h;
        Self {
            resolution: [w, h, aspect, output.frame_idx as f32],
            transition: [progress.clamp(0.0, 1.0), aspect, output.time_secs, output.fps],
            params0: [params[0], params[1], params[2], params[3]],
            params1: [params[4], params[5], params[6], params[7]],
            audio: output.audio,
            beat_params: [output.beat, 0.0, 0.0, 0.0],
        }
    }
}

pub struct ShadertoyTransitionRunner {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pipeline_cache: Mutex<HashMap<u32, wgpu::RenderPipeline>>,
    fallback_pipeline: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
}

impl ShadertoyTransitionRunner {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("vv-render shadertoy transition bind group layout"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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

        let fallback_source = concat!(
            include_str!("shaders/shadertoy_transition_runner.wgsl"),
            r#"
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let a = get_from_color(uv);
    let b = get_to_color(uv);
    return mix(a, b, progress);
}
"#
        );

        let fallback_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vv-render shadertoy transition fallback"),
            source: wgpu::ShaderSource::Wgsl(fallback_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vv-render shadertoy transition layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let fallback_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vv-render shadertoy transition fallback pipeline"),
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
            format,
        }
    }

    /// Preprocesses preset or custom code into a full WGSL module.
    pub fn preprocess_code(raw_code: &str) -> String {
        let runner = include_str!("shaders/shadertoy_transition_runner.wgsl");
        if raw_code.contains("@fragment") {
            return raw_code.to_string();
        }

        format!("{runner}\n{raw_code}")
    }

    /// Gets or compiles a render pipeline for the given preset.
    pub fn get_or_compile(
        &self,
        device: &wgpu::Device,
        preset: ShadertoyTransitionPreset,
    ) -> wgpu::RenderPipeline {
        let key = preset as u32;
        let mut cache = self.pipeline_cache.lock().unwrap();
        if let Some(pipeline) = cache.get(&key) {
            return pipeline.clone();
        }

        let full_wgsl = Self::preprocess_code(preset.wgsl_code());
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vv-render shadertoy transition preset layout"),
            bind_group_layouts: &[Some(&self.bind_group_layout)],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vv-render shadertoy transition preset module"),
            source: wgpu::ShaderSource::Wgsl(full_wgsl.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vv-render shadertoy transition preset pipeline"),
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

        cache.insert(key, pipeline.clone());
        pipeline
    }

    pub fn fallback_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.fallback_pipeline
    }

    /// Renders the transition pass from Input A (`channel0_view`) to Input B (`channel1_view`).
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        sampler: &wgpu::Sampler,
        target_view: &wgpu::TextureView,
        channel0_view: &wgpu::TextureView,
        channel1_view: &wgpu::TextureView,
        preset: ShadertoyTransitionPreset,
        custom_params: Option<[f32; 8]>,
        progress: f32,
        output: &OutputFrame,
    ) {
        use wgpu::util::DeviceExt;

        let pipeline = self.get_or_compile(device, preset);
        let params = custom_params.unwrap_or_else(|| preset.default_params());
        let uniforms = TransitionUniforms::new(output.width, output.height, progress, params, output);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vv-render transition uniform buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vv-render transition bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(channel0_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(channel1_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("vv-render shadertoy transition pass"),
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

#[cfg(test)]
#[path = "tests/shadertoy_transition.rs"]
mod tests;
