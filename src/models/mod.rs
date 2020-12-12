use std::collections::HashMap;
use wgpu::util::DeviceExt;
pub mod data;
mod mesh;
mod render_pipeline;

pub struct Model {
    pub primitives: Vec<render_pipeline::Primitive>,
    pub instances: data::InstanceBuffer,
}

pub struct Models {
    pub render_pipeline: render_pipeline::RenderPipeline,
    pub models: HashMap<String, Model>,
    pub sampler: wgpu::Sampler,
}

impl Models {
    pub fn new(device: &wgpu::Device) -> Self {
        let render_pipeline = render_pipeline::RenderPipeline::new(&device);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            sampler,
            render_pipeline,
            models: HashMap::new(),
        }
    }

    pub fn load_model(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, name: &str, path: &str) {
        let (gltf, buffers, images) = gltf::import(format!("./res/models/{}", path)).expect("Failed to import GLTF!");
        let mut primitives: Vec<render_pipeline::Primitive> = vec![];

        for gltf_mesh in gltf.meshes() {
            let meshes = mesh::Mesh::new(&device, &queue, gltf_mesh, &buffers, &images);
            for mesh in meshes {
                &primitives.push(mesh.to_primitive(&device, &self.sampler, &self.render_pipeline));
            }
        }

        let instances = data::InstanceBuffer::new(&device, Vec::<data::Instance>::new());
        self.models.insert(name.to_string(), Model { primitives, instances });
    }

    pub fn add_instance(&mut self, name: &str, instance: data::Instance) {
        let model = self.models.get_mut(&name.to_string()).expect("Model not found!");
        &model.instances.data.push(instance);
    }

    pub fn write_instance_buffers(&mut self, device: &wgpu::Device, name: &str) {
        let model = self.models.get_mut(name).expect("Model not found!");
        model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&model.instances.data),
            usage: wgpu::BufferUsage::VERTEX,
        });
    }

    pub fn set_uniforms(&mut self, queue: &wgpu::Queue, uniforms: data::Uniforms) {
        self.render_pipeline.uniforms.data = uniforms;
        queue.write_buffer(
            &self.render_pipeline.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.render_pipeline.uniforms.data]),
        );
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline.render_pipeline);
        render_pass.set_bind_group(1, &self.render_pipeline.uniforms.bind_group, &[]);

        for model in self.models.values() {
            for mesh in &model.primitives {
                render_pass.set_bind_group(0, &mesh.texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, model.instances.buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..));
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..model.instances.data.len() as _);
            }
        }
    }
}
