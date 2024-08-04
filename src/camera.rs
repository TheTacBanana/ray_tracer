use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct CameraWithBuffers {
    pub camera: Camera,
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub screen_dimensions: [f32; 2],
    pub fov: f32,
    pub pos: [f32; 3],
    pub up: [f32; 3],
    pub right: [f32; 3],
    pub _pad: [f32; 4],
}

impl Camera {
    pub fn new(device: &wgpu::Device, dimensions: [f32; 2]) -> CameraWithBuffers {
        let camera = Camera {
            screen_dimensions: dimensions,
            fov: 60.0,
            pos: [0.0, -3.0, 0.0],
            up: [0.0, 0.0, 1.0],
            right: [1.0, 0.0, 0.0],
            _pad: Default::default(),
        };

        // Create layout entrys
        let entries = (0..=6)
            .map(|i| wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

        // Create layout from entries
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("camera_binding"),
        });

        // Create buffer with intiial contents of default ImageDisplay
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buf"),
            contents: bytemuck::bytes_of(&camera),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group entries
        let entries = (0..=6)
            .map(|i| wgpu::BindGroupEntry {
                binding: i,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            })
            .collect::<Vec<wgpu::BindGroupEntry>>();

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some("camera_bind_group"),
        });

        CameraWithBuffers {
            camera,
            layout,
            buffer,
            bind_group,
        }
    }
}
