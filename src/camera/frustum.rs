extern crate cgmath;
use cgmath::*;
use std::mem;

use super::bounding_box;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrustumCuller {
    f: [Vector4<f32>; 6],
}

impl FrustumCuller {
    pub fn new() -> Self {
        Self::from_matrix(Matrix4::identity())
    }

    pub fn from_matrix(m: Matrix4<f32>) -> Self {
        let mut culler: Self = unsafe { mem::zeroed() };

        culler.f[0] = vec4(m.x.w + m.x.x, m.y.w + m.y.x, m.z.w + m.z.x, m.w.w + m.w.x).normalize();
        culler.f[1] = vec4(m.x.w - m.x.x, m.y.w - m.y.x, m.z.w - m.z.x, m.w.w - m.w.x).normalize();
        culler.f[2] = vec4(m.x.w + m.x.y, m.y.w + m.y.y, m.z.w + m.z.y, m.w.w + m.w.y).normalize();
        culler.f[3] = vec4(m.x.w - m.x.y, m.y.w - m.y.y, m.z.w - m.z.y, m.w.w - m.w.y).normalize();
        culler.f[4] = vec4(m.x.w + m.x.z, m.y.w + m.y.z, m.z.w + m.z.z, m.w.w + m.w.z).normalize();
        culler.f[5] = vec4(m.x.w - m.x.z, m.y.w - m.y.z, m.z.w - m.z.z, m.w.w - m.w.z).normalize();
        culler
    }

    pub fn test_bounding_box(&self, aab: &bounding_box::BoundingBox) -> bool {
        let v1 = aab.min;
        let v2 = aab.max;

        for p in 0..6 {
            if self.f[p].x * v1.x + self.f[p].y * v1.y + self.f[p].z * v1.z + self.f[p].w > -1.0
                || self.f[p].x * v2.x + self.f[p].y * v1.y + self.f[p].z * v1.z + self.f[p].w > -1.0
                || self.f[p].x * v1.x + self.f[p].y * v2.y + self.f[p].z * v1.z + self.f[p].w > -1.0
                || self.f[p].x * v2.x + self.f[p].y * v2.y + self.f[p].z * v1.z + self.f[p].w > -1.0
                || self.f[p].x * v1.x + self.f[p].y * v1.y + self.f[p].z * v2.z + self.f[p].w > -1.0
                || self.f[p].x * v2.x + self.f[p].y * v1.y + self.f[p].z * v2.z + self.f[p].w > -1.0
                || self.f[p].x * v1.x + self.f[p].y * v2.y + self.f[p].z * v2.z + self.f[p].w > -1.0
                || self.f[p].x * v2.x + self.f[p].y * v2.y + self.f[p].z * v2.z + self.f[p].w > -1.0
            {
                continue;
            }

            return false;
        }

        true
    }
}
