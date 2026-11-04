//! P5.js creative coding canvas engine: GPU accelerated shader pipelines,
//! uniform management, and execution of audio-reactive creative canvas sketches.

use vv_core::{P5BlendMode, P5Preset, P5Value};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct P5Uniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub fps: f32,
    pub params: [f32; 4],
    pub audio: [f32; 4], // x: level, y: bass, z: mids, w: treble
    pub audio_beat: f32,
    pub preset: u32,
    pub blend_mode: u32,
    pub _pad: u32,
}

impl P5Uniforms {
    pub fn new(
        width: u32,
        height: u32,
        time_secs: f32,
        fps: f32,
        p5: P5Value,
        audio_levels: [f32; 4],
        beat: f32,
    ) -> Self {
        let preset_idx = match p5.preset {
            P5Preset::AsciiColor => 0u32,
            P5Preset::AsciiMono => 1u32,
            P5Preset::ParticleSampler => 2u32,
            P5Preset::WaveMatrix => 3u32,
            P5Preset::SpectrumRings => 4u32,
            P5Preset::GlitchPixelSort => 5u32,
            P5Preset::Custom => 6u32,
        };
        let blend_idx = match p5.blend_mode {
            P5BlendMode::Normal => 0u32,
            P5BlendMode::Add => 1u32,
            P5BlendMode::Screen => 2u32,
            P5BlendMode::Multiply => 3u32,
            P5BlendMode::Overlay => 4u32,
        };

        Self {
            resolution: [width.max(1) as f32, height.max(1) as f32],
            time: time_secs,
            fps: fps.max(1.0),
            params: p5.params,
            audio: audio_levels,
            audio_beat: beat,
            preset: preset_idx,
            blend_mode: blend_idx,
            _pad: 0,
        }
    }
}

pub struct P5Runner {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub sampler: wgpu::Sampler,
    format: wgpu::TextureFormat,
}

impl P5Runner {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("vv-render p5 bind group layout"),
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
            label: Some("vv-render p5 sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vv-render p5 canvas shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/p5_canvas.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vv-render p5 pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vv-render p5 render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            bind_group_layout,
            pipeline,
            sampler,
            format,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        time_secs: f32,
        fps: f32,
        p5: P5Value,
        audio_levels: [f32; 4],
        beat: f32,
    ) {
        let uniforms = P5Uniforms::new(width, height, time_secs, fps, p5, audio_levels, beat);
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vv-render p5 uniform buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vv-render p5 bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buf.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("vv-render p5 canvas pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
#[path = "tests/p5_canvas.rs"]
mod tests;
