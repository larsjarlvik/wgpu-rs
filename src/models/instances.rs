use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct Instance {
    buffer: wgpu::Buffer,
}

pub struct Instances {
    pub model_instances: HashMap<String, Instance>,
}

impl Instances {
    pub fn new(device: &wgpu::Device, models: &super::Models) -> Self {
        let mut model_instances = HashMap::new();

        for (key, _) in models.models.iter() {
            let instances: Vec<pipelines::model::Instance> = vec![];
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsage::VERTEX,
            });
            model_instances.insert(key.clone(), Instance { buffer });
        }

        Self { model_instances }
    }
}
