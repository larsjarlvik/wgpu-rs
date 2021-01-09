use super::{data, render_pipeline};
use crate::camera::frustum;
use cgmath::*;
use std::collections::HashMap;

pub struct Model {
    pub primitives: Vec<render_pipeline::PrimitiveBuffers>,
    pub instances: data::InstanceBuffer,
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

    pub fn add_instances(&mut self, instances: HashMap<String, data::Instance>) {
        self.instances.data.extend(instances);
    }

    pub fn remove_instances(&mut self, instances: &Vec<String>) {
        for instance in instances {
            self.instances.data.remove(instance);
        }
    }
}
