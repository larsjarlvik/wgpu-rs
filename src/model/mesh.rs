use super::primitive;
use crate::camera;
use cgmath::*;

pub struct Mesh {
    pub name: String,
    pub primitives: Vec<primitive::Primitive>,
    pub bounding_box: camera::BoundingBox,
}

impl Mesh {
    pub fn new<'a>(mesh: gltf::Mesh<'_>, buffers: &Vec<gltf::buffer::Data>) -> Self {
        let mut primitives = vec![];
        let name = String::from(mesh.name().unwrap());
        let mut bounding_box = camera::BoundingBox {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0),
        };

        for gltf_primitive in mesh.primitives() {
            let primitive = primitive::Primitive::new(buffers, &gltf_primitive, &name);
            bounding_box = bounding_box.grow(&primitive.bounding_box);
            primitives.push(primitive);
        }

        Mesh {
            name,
            primitives,
            bounding_box,
        }
    }
}
