use crate::settings;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub light_dir: [f32; 3],
    pub ambient_strength: f32,
    pub light_color: [f32; 3],
    pub light_intensity: f32,
    pub shadow_matrix: [[[f32; 4]; 4]; settings::SHADOW_CASCADE_SPLITS.len()],
    pub shadow_split_depth: [[f32; 4]; settings::SHADOW_CASCADE_SPLITS.len()],
    pub time: f32,
}

pub struct UniformBuffer {
    pub data: Uniforms,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl UniformBuffer {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, data: Uniforms) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self { data, buffer, bind_group }
    }
}
