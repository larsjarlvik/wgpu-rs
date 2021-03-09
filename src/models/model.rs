use std::collections::HashMap;

use crate::{camera::frustum, pipelines};
use cgmath::*;

pub struct PrimitiveBuffers {
    pub texture_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub struct Model {
    pub primitives: Vec<PrimitiveBuffers>,
    pub instances: pipelines::model::data::InstanceBuffer,
    pub bounding_box: frustum::BoundingBox,
}

impl Model {
    pub fn transformed_bounding_box(&self, transform: Matrix4<f32>) -> frustum::BoundingBox {
        let b1: Point3<f32> = cgmath::Matrix4::from(transform).transform_point(self.bounding_box.min);
        let b2: Point3<f32> = cgmath::Matrix4::from(transform).transform_point(self.bounding_box.max);

        let min = Point3::new(
            if b1.x < b2.x { b1.x } else { b2.x },
            if b1.y < b2.y { b1.y } else { b2.y },
            if b1.z < b2.z { b1.z } else { b2.z },
        );
        let max = Point3::new(
            if b1.x > b2.x { b1.x } else { b2.x },
            if b1.y > b2.y { b1.y } else { b2.y },
            if b1.z > b2.z { b1.z } else { b2.z },
        );

        frustum::BoundingBox { min, max }
    }

    pub fn add_instances(&mut self, instances: HashMap<String, pipelines::model::data::Instance>) {
        self.instances.data.extend(instances);
    }

    pub fn remove_instances(&mut self, instances: &Vec<String>) {
        for instance in instances {
            self.instances.data.remove(instance);
        }
    }
}
