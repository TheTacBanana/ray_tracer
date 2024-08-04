use wgpu::util::DeviceExt;

pub struct SpheresWithBuffers {
    pub spheres: Spheres,
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub struct Spheres {
    pub spheres: Vec<Sphere>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pub pos: [f32; 3],
    pub radius: f32,
}

impl Sphere {
    pub fn new_sphere_buffers(spheres: Spheres, device: &wgpu::Device) -> SpheresWithBuffers {
        // Create layout from entries
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("spheres_binding"),
        });

        // Create buffer with intiial contents of default ImageDisplay
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("spheres_buf"),
            contents: bytemuck::cast_slice(&spheres.spheres),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
            label: Some("spheres_group"),
        });

        SpheresWithBuffers {
            spheres,
            layout,
            buffer,
            bind_group,
        }
    }
}
