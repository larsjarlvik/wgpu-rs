use crate::camera;
use cgmath::*;

pub struct Primitive {
    pub bounding_box: camera::BoundingBox,
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub material_index: usize,
}

impl Primitive {
    pub fn new(buffers: &Vec<gltf::buffer::Data>, primitive: &gltf::Primitive, mesh_name: &String) -> Self {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let indices = reader.read_indices().unwrap().into_u32().collect::<Vec<u32>>();
        let positions = reader.read_positions().unwrap().collect::<Vec<[f32; 3]>>();
        let normals = reader.read_normals().unwrap().collect::<Vec<[f32; 3]>>();
        let tangents = reader.read_tangents().unwrap().collect::<Vec<[f32; 4]>>();
        let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect::<Vec<[f32; 2]>>();
        let material_index = primitive.material().index().expect(&format!("No material found! {}", mesh_name));

        let bounding_box = camera::BoundingBox {
            min: Point3::from(primitive.bounding_box().min),
            max: Point3::from(primitive.bounding_box().max),
        };

        Self {
            indices,
            positions,
            normals,
            tangents,
            tex_coords,
            material_index,
            bounding_box,
        }
    }
}
