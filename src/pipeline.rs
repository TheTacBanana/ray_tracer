use crate::{load_bytes, vertex::Vertex};

pub struct Pipeline {
    // layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub async fn new(
        device: &wgpu::Device,
        camera_layout: &wgpu::BindGroupLayout,
        spheres_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = Pipeline::load_shader(device, "./src/raytrace.wgsl").await;

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[camera_layout, spheres_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("raytrace"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Pipeline { pipeline }
    }

    async fn load_shader(device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(String::from_utf8_lossy(
                &load_bytes(path).await.unwrap(),
            )),
        })
    }
}
