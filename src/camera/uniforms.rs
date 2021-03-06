use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub eye_pos: [f32; 3],
    pub z_near: f32,
    pub look_at: [f32; 3],
    pub z_far: f32,
    pub clip: [f32; 4],
    pub viewport_size: [f32; 2],
}
pub struct UniformBuffer {
    pub data: Uniforms,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl UniformBuffer {
    pub fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, data: Uniforms) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { data, buffer, bind_group }
    }
}
