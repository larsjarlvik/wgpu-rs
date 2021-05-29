use super::primitive;
use crate::camera;
use cgmath::*;
use std::collections::HashMap;

pub struct Mesh {
    pub name: String,
    pub primitives: Vec<primitive::Primitive>,
    pub bounding_box: camera::BoundingBox,
    pub extras: HashMap<String, f32>,
}

impl Mesh {
    pub fn new<'a>(mesh: gltf::Mesh<'_>, buffers: &Vec<gltf::buffer::Data>) -> Self {
        let mut primitives = vec![];
        let name = String::from(mesh.name().unwrap());
        let mut bounding_box = camera::BoundingBox {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0),
        };

        let mut extras: HashMap<String, f32> = HashMap::new();
        if let Some(json) = mesh.extras() {
            extras = serde_json::from_str(json.get()).unwrap();
        }

        for gltf_primitive in mesh.primitives() {
            let primitive = primitive::Primitive::new(buffers, &gltf_primitive, &name);
            bounding_box = bounding_box.grow(&primitive.bounding_box);
            primitives.push(primitive);
        }

        Mesh {
            name,
            primitives,
            bounding_box,
            extras,
        }
    }

    pub fn get_extra_f32(&self, name: &str) -> f32 {
        *self.extras.get(name).unwrap_or(&0.0)
    }

    pub fn get_extra_f32_or(&self, name: &str, default: f32) -> f32 {
        *self.extras.get(name).unwrap_or(&default)
    }

    pub fn get_extra_range(&self, name: &str) -> [f32; 2] {
        [
            *self.extras.get(format!("{}_min", name).as_str()).unwrap_or(&0.0),
            *self.extras.get(format!("{}_max", name).as_str()).unwrap_or(&0.0),
        ]
    }

    pub fn get_extra_bool(&self, name: &str) -> bool {
        *self.extras.get(name).unwrap_or(&0.0) == 1.0
    }
}
