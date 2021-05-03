use crate::{
    camera, models,
    pipelines::model,
    settings, world,
    world::node::{Node, NodeData},
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct Instance {
    buffer: wgpu::Buffer,
    length: u32,
}

pub struct ModelInstances {
    pub model_instances: HashMap<String, Instance>,
}

impl ModelInstances {
    pub fn new(device: &wgpu::Device, models: &models::Models) -> Self {
        let mut model_instances = HashMap::new();

        for (key, _) in models.meshes.iter() {
            let instances: Vec<model::Instance> = vec![];
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsage::VERTEX,
            });
            model_instances.insert(key.clone(), Instance { buffer, length: 0 });
        }

        Self { model_instances }
    }
}

pub fn get_models_bundle(
    device: &wgpu::Device,
    camera: &camera::Instance,
    world_data: &world::WorldData,
    model_instances: &mut ModelInstances,
    nodes: &Vec<(&Node, &NodeData)>,
) -> wgpu::RenderBundle {
    update_instance_buffer(device, model_instances, nodes);

    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });

    encoder.set_pipeline(&world_data.model.render_pipeline);
    encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

    for (key, model_instances) in model_instances.model_instances.iter_mut() {
        let model = world_data.models.meshes.get(key).unwrap();
        for mesh in &model.primitives {
            encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
            encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, model_instances.buffer.slice(..));
            encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..mesh.num_elements, 0, 0..model_instances.length as _);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") })
}

pub fn get_models_shadow_bundle(
    device: &wgpu::Device,
    camera: &camera::Instance,
    world_data: &world::WorldData,
    model_instances: &mut ModelInstances,
    nodes: &Vec<(&Node, &NodeData)>,
) -> wgpu::RenderBundle {
    update_instance_buffer(device, model_instances, nodes);

    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });

    encoder.set_pipeline(&world_data.model.shadow_pipeline);
    encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

    for (key, model_instances) in model_instances.model_instances.iter_mut() {
        let model = world_data.models.meshes.get(key).unwrap();
        for mesh in &model.primitives {
            encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
            encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, model_instances.buffer.slice(..));
            encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..mesh.num_elements, 0, 0..model_instances.length as _);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor {
        label: Some("models_shadow"),
    })
}

fn update_instance_buffer(device: &wgpu::Device, model_instances: &mut ModelInstances, nodes: &Vec<(&Node, &NodeData)>) {
    model_instances.model_instances.par_iter_mut().for_each(|(key, instance)| {
        let mut instances: Vec<model::Instance> = vec![];
        for (_, data) in nodes {
            let node_instances = data.model_instances.get(key);
            match node_instances {
                Some(ni) => {
                    instances.extend(ni);
                }
                None => (),
            }
        }

        instance.length = instances.len() as u32;
        instance.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsage::VERTEX,
        });
    });
}
