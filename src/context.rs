use std::{iter, mem};

use anyhow::{Ok, Result};
use futures::SinkExt;
use instant::Instant;
use wgpu::{util::DeviceExt, CommandEncoder, TextureView};

use crate::{
    camera::{Camera, CameraWithBuffers}, pipeline::Pipeline, sphere::{self, Sphere, SpheresWithBuffers, Spheres}, thread_context::ThreadContext, vertex::Vertex
};

use super::window::Window;

pub struct GraphicsContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub buffers: (wgpu::Buffer, wgpu::Buffer),
    pub thread: ThreadContext,

    pub pipeline: Pipeline,
    pub camera: CameraWithBuffers,
    pub spheres: SpheresWithBuffers,
}

impl GraphicsContext {
    /// Vertexes spanning screenspace
    const VERTICES: &'static [Vertex] = &[
        Vertex::xyz(1.0, 1.0, 0.0),
        Vertex::xyz(1.0, -1.0, 0.0),
        Vertex::xyz(-1.0, -1.0, 0.0),
        Vertex::xyz(-1.0, 1.0, 0.0),
    ];

    /// Indices for vertexes
    const INDICES: &'static [u16] = &[0, 3, 1, 1, 3, 2];

    /// Create a new GraphicsContext
    pub async fn new(window: &Window) -> Self {
        let size = window.raw.inner_size();

        // Create a new backend instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create a new surface to render to
        let surface = unsafe { instance.create_surface(&window.raw) }.unwrap();

        // Create a new device adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Get the queue and device from the adapter
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    #[cfg(target_arch = "wasm32")]
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    #[cfg(not(target_arch = "wasm32"))]
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // Get the surface capabilites and select a target format
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Create a config and configure the surface to use that config
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let buffers = GraphicsContext::create_buffers(&device);
        let camera = Camera::new(&device, [
            window.raw.inner_size().width as f32,
            window.raw.inner_size().height as f32,
        ]);


        let spheres = Spheres {
            spheres: vec![
                Sphere {
                    pos: [0.0, 0.0, -1.0],
                    radius: 0.5,
                }

            ],
        };

        let sphere_buffers = Sphere::new_sphere_buffers(spheres, &device);

        let pipeline = Pipeline::new(&device, &camera.layout, &sphere_buffers.layout).await;

        Self {
            surface,
            device,
            queue,
            config,
            buffers,
            thread: ThreadContext::default(),

            pipeline,
            camera,
            spheres: sphere_buffers,
        }
    }

    /// Create vertex and index buffers
    pub fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buf"),
            contents: bytemuck::cast_slice(GraphicsContext::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buf"),
            contents: bytemuck::cast_slice(GraphicsContext::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    /// Create the sampler used for all textures
    pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    /// Resize window callback
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            let dims = &mut self.camera.camera.screen_dimensions;
            dims[0] = width as f32;
            dims[1] = height as f32;
        }
    }


    /// Perform all render tasks per frame
    pub fn render(&mut self) -> Result<()> {
        self.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::bytes_of(&self.camera.camera),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Get current screen texture
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.spheres.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.buffers.0.slice(..));
            render_pass.set_index_buffer(self.buffers.1.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..GraphicsContext::INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}