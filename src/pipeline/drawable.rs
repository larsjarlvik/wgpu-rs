use crate::pipeline::*;
use crate::model;

pub struct Drawable {
    pub uniforms: uniform::UniformBuffer,
    pub texture_bind_group: wgpu::BindGroup,
    pub model: model::Model,
}

impl Drawable {
    pub fn new(device: &wgpu::Device, render_pipeline: &render::RenderPipeline, queue: &wgpu::Queue, sampler: &wgpu::Sampler, texture: &image::DynamicImage) -> Self {
        let u1 = uniform::UniformBuffer::new(&device, &render_pipeline.uniform_bind_group_layout);
        let t1 = texture::Texture::from_image(&device, &queue, &texture).unwrap();

        Drawable {
            model: model::Model::new(&device),
            uniforms: u1,
            texture_bind_group: t1.create_texture_bind_group(&device, &sampler, &render_pipeline.texture_bind_group_layout),
        }
    }
}