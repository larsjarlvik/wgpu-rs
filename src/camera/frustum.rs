extern crate cgmath;
use cgmath::*;
use std::mem;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrustumCuller {
    nx_x: f32,
    nx_y: f32,
    nx_z: f32,
    nx_w: f32,
    px_x: f32,
    px_y: f32,
    px_z: f32,
    px_w: f32,
    ny_x: f32,
    ny_y: f32,
    ny_z: f32,
    ny_w: f32,
    py_x: f32,
    py_y: f32,
    py_z: f32,
    py_w: f32,
    nz_x: f32,
    nz_y: f32,
    nz_z: f32,
    nz_w: f32,
    pz_x: f32,
    pz_y: f32,
    pz_z: f32,
    pz_w: f32,
}
#[derive(Clone)]
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBox {
    pub fn transform(&self, transform: [[f32; 4]; 4]) -> Self {
        let b1: Point3<f32> = cgmath::Matrix4::from(transform).transform_point(self.min);
        let b2: Point3<f32> = cgmath::Matrix4::from(transform).transform_point(self.max);

        Self {
            min: Point3::new(
                if b1.x < b2.x { b1.x } else { b2.x },
                if b1.y < b2.y { b1.y } else { b2.y },
                if b1.z < b2.z { b1.z } else { b2.z },
            ),
            max: Point3::new(
                if b1.x > b2.x { b1.x } else { b2.x },
                if b1.y > b2.y { b1.y } else { b2.y },
                if b1.z > b2.z { b1.z } else { b2.z },
            ),
        }
    }

    pub fn grow(&self, bounding_box: &Self) -> Self {
        Self {
            min: Point3::new(
                self.min.x.min(bounding_box.min.x),
                self.min.y.min(bounding_box.min.y),
                self.min.z.min(bounding_box.min.z),
            ),
            max: Point3::new(
                self.max.x.max(bounding_box.max.x),
                self.max.y.max(bounding_box.max.y),
                self.max.z.max(bounding_box.max.z),
            ),
        }
    }
}

pub enum Intersection {
    Inside,
    Partial,
    Outside,
}

impl FrustumCuller {
    pub fn new() -> Self {
        Self::from_matrix(Matrix4::identity())
    }

    pub fn from_matrix(m: Matrix4<f32>) -> Self {
        let mut culler: Self = unsafe { mem::zeroed() };

        culler.nx_x = m.x.w + m.x.x;
        culler.nx_y = m.y.w + m.y.x;
        culler.nx_z = m.z.w + m.z.x;
        culler.nx_w = m.w.w + m.w.x;

        let invl = (culler.nx_x * culler.nx_x + culler.nx_y * culler.nx_y + culler.nx_z * culler.nx_z)
            .sqrt()
            .recip();
        culler.nx_x *= invl;
        culler.nx_y *= invl;
        culler.nx_z *= invl;
        culler.nx_w *= invl;

        culler.px_x = m.x.w - m.x.x;
        culler.px_y = m.y.w - m.y.x;
        culler.px_z = m.z.w - m.z.x;
        culler.px_w = m.w.w - m.w.x;

        let invl = (culler.px_x * culler.px_x + culler.px_y * culler.px_y + culler.px_z * culler.px_z)
            .sqrt()
            .recip();
        culler.px_x *= invl;
        culler.px_y *= invl;
        culler.px_z *= invl;
        culler.px_w *= invl;

        culler.ny_x = m.x.w + m.x.y;
        culler.ny_y = m.y.w + m.y.y;
        culler.ny_z = m.z.w + m.z.y;
        culler.ny_w = m.w.w + m.w.y;

        let invl = (culler.ny_x * culler.ny_x + culler.ny_y * culler.ny_y + culler.ny_z * culler.ny_z)
            .sqrt()
            .recip();
        culler.ny_x *= invl;
        culler.ny_y *= invl;
        culler.ny_z *= invl;
        culler.ny_w *= invl;

        culler.py_x = m.x.w - m.x.y;
        culler.py_y = m.y.w - m.y.y;
        culler.py_z = m.z.w - m.z.y;
        culler.py_w = m.w.w - m.w.y;

        let invl = (culler.py_x * culler.py_x + culler.py_y * culler.py_y + culler.py_z * culler.py_z)
            .sqrt()
            .recip();
        culler.py_x *= invl;
        culler.py_y *= invl;
        culler.py_z *= invl;
        culler.py_w *= invl;

        culler.nz_x = m.x.w + m.x.z;
        culler.nz_y = m.y.w + m.y.z;
        culler.nz_z = m.z.w + m.z.z;
        culler.nz_w = m.w.w + m.w.z;

        let invl = (culler.nz_x * culler.nz_x + culler.nz_y * culler.nz_y + culler.nz_z * culler.nz_z)
            .sqrt()
            .recip();
        culler.nz_x *= invl;
        culler.nz_y *= invl;
        culler.nz_z *= invl;
        culler.nz_w *= invl;

        culler.pz_x = m.x.w - m.x.z;
        culler.pz_y = m.y.w - m.y.z;
        culler.pz_z = m.z.w - m.z.z;
        culler.pz_w = m.w.w - m.w.z;

        let invl = (culler.pz_x * culler.pz_x + culler.pz_y * culler.pz_y + culler.pz_z * culler.pz_z)
            .sqrt()
            .recip();
        culler.pz_x *= invl;
        culler.pz_y *= invl;
        culler.pz_z *= invl;
        culler.pz_w *= invl;

        culler
    }

    pub fn test_bounding_box(&self, aab: &BoundingBox) -> Intersection {
        let mut inside = true;
        if self.nx_x * if self.nx_x < 0.0 { aab.min.x } else { aab.max.x }
            + self.nx_y * if self.nx_y < 0.0 { aab.min.y } else { aab.max.y }
            + self.nx_z * if self.nx_z < 0.0 { aab.min.z } else { aab.max.z }
            >= -self.nx_w
        {
            inside &= self.nx_x * if self.nx_x < 0.0 { aab.max.x } else { aab.min.x }
                + self.nx_y * if self.nx_y < 0.0 { aab.max.y } else { aab.min.y }
                + self.nx_z * if self.nx_z < 0.0 { aab.max.z } else { aab.min.z }
                >= -self.nx_w;
            if self.px_x * if self.px_x < 0.0 { aab.min.x } else { aab.max.x }
                + self.px_y * if self.px_y < 0.0 { aab.min.y } else { aab.max.y }
                + self.px_z * if self.px_z < 0.0 { aab.min.z } else { aab.max.z }
                >= -self.px_w
            {
                inside &= self.px_x * if self.px_x < 0.0 { aab.max.x } else { aab.min.x }
                    + self.px_y * if self.px_y < 0.0 { aab.max.y } else { aab.min.y }
                    + self.px_z * if self.px_z < 0.0 { aab.max.z } else { aab.min.z }
                    >= -self.px_w;
                if self.ny_x * if self.ny_x < 0.0 { aab.min.x } else { aab.max.x }
                    + self.ny_y * if self.ny_y < 0.0 { aab.min.y } else { aab.max.y }
                    + self.ny_z * if self.ny_z < 0.0 { aab.min.z } else { aab.max.z }
                    >= -self.ny_w
                {
                    inside &= self.ny_x * if self.ny_x < 0.0 { aab.max.x } else { aab.min.x }
                        + self.ny_y * if self.ny_y < 0.0 { aab.max.y } else { aab.min.y }
                        + self.ny_z * if self.ny_z < 0.0 { aab.max.z } else { aab.min.z }
                        >= -self.ny_w;
                    if self.py_x * if self.py_x < 0.0 { aab.min.x } else { aab.max.x }
                        + self.py_y * if self.py_y < 0.0 { aab.min.y } else { aab.max.y }
                        + self.py_z * if self.py_z < 0.0 { aab.min.z } else { aab.max.z }
                        >= -self.py_w
                    {
                        inside &= self.py_x * if self.py_x < 0.0 { aab.max.x } else { aab.min.x }
                            + self.py_y * if self.py_y < 0.0 { aab.max.y } else { aab.min.y }
                            + self.py_z * if self.py_z < 0.0 { aab.max.z } else { aab.min.z }
                            >= -self.py_w;
                        if self.nz_x * if self.nz_x < 0.0 { aab.min.x } else { aab.max.x }
                            + self.nz_y * if self.nz_y < 0.0 { aab.min.y } else { aab.max.y }
                            + self.nz_z * if self.nz_z < 0.0 { aab.min.z } else { aab.max.z }
                            >= -self.nz_w
                        {
                            inside &= self.nz_x * if self.nz_x < 0.0 { aab.max.x } else { aab.min.x }
                                + self.nz_y * if self.nz_y < 0.0 { aab.max.y } else { aab.min.y }
                                + self.nz_z * if self.nz_z < 0.0 { aab.max.z } else { aab.min.z }
                                >= -self.nz_w;
                            if self.pz_x * if self.pz_x < 0.0 { aab.min.x } else { aab.max.x }
                                + self.pz_y * if self.pz_y < 0.0 { aab.min.y } else { aab.max.y }
                                + self.pz_z * if self.pz_z < 0.0 { aab.min.z } else { aab.max.z }
                                >= -self.pz_w
                            {
                                inside &= self.pz_x * if self.pz_x < 0.0 { aab.max.x } else { aab.min.x }
                                    + self.pz_y * if self.pz_y < 0.0 { aab.max.y } else { aab.min.y }
                                    + self.pz_z * if self.pz_z < 0.0 { aab.max.z } else { aab.min.z }
                                    >= -self.pz_w;
                                return if inside { Intersection::Inside } else { Intersection::Partial };
                            }
                        }
                    }
                }
            }
        }

        Intersection::Outside
    }
}
