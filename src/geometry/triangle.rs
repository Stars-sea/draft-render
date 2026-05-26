use crate::geometry::Geometry;
use crate::linalg::{Mat4f, Vec3f, Vec4f};
use std::ops::Mul;

pub struct Triangle([Vec4f; 3]);

impl Triangle {
    pub fn new(points: [Vec3f; 3]) -> Triangle {
        Self(points.map(|point| Vec4f::from_vec3(point, 1.0)))
    }
}

impl Mul<Mat4f> for Triangle {
    type Output = Triangle;

    fn mul(self, rhs: Mat4f) -> Self::Output {
        Self(self.0.map(|point| rhs * point))
    }
}

impl Geometry for Triangle {
    fn vertices(&self) -> &[Vec4f] {
        &self.0
    }
}
