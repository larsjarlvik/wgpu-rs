use crate::{
    camera, plane, settings,
    world::{
        self,
        node::{Node, NodeData},
    },
};
use cgmath::*;

pub fn get_terrain_bundle(
    device: &wgpu::Device,
    camera: &camera::Instance,
    world_data: &world::WorldData,
    nodes: &Vec<(&Node, &NodeData)>,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some("terrain_bundle"),
        color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&world_data.terrain.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &world_data.terrain.texture_bind_group, &[]);
    encoder.set_bind_group(2, &world_data.terrain.noise_bindings.bind_group, &[]);
    encoder.set_bind_group(4, &world_data.map.bind_group, &[]);
    encoder.set_vertex_buffer(0, world_data.terrain.vertex_buffer.slice(..));

    let direction = camera.uniforms.data.clip[1];
    let plane = camera.uniforms.data.clip[3];
    let eye = vec3(
        camera.uniforms.data.eye_pos[0],
        camera.uniforms.data.eye_pos[1],
        camera.uniforms.data.eye_pos[2],
    );

    for (node, data) in nodes {
        for lod in 0..=settings::LODS.len() {
            let terrain_lod = world_data.lods.get(lod).expect("Could not get LOD!");

            if check_clip(direction, plane, &node.bounding_box)
                && plane::get_lod(eye, vec3(node.x, 0.0, node.z), camera.uniforms.data.z_far) == lod as u32
            {
                let ct = plane::get_connect_type(eye, vec3(node.x, 0.0, node.z), lod as u32, camera.uniforms.data.z_far);
                let lod_buffer = terrain_lod.get(&ct).unwrap();

                encoder.set_bind_group(3, &data.uniforms.bind_group, &[]);
                encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
            }
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn check_clip(direction: f32, plane: f32, bounding_box: &camera::BoundingBox) -> bool {
    direction == 0.0 || (direction > 0.0 && bounding_box.max.y >= plane) || (direction <= 0.0 && bounding_box.min.y < plane)
}
