use crate::linalg::{Mat4f, Vec3f, Vec4f};
use num_traits::Zero;
use std::ops::Mul;

mod triangle;

pub trait Geometry: Mul<Mat4f, Output = Self> {
    fn vertices(&self) -> &[Vec4f];

    fn points_edges(&self) -> (Vec<Vec3f>, Vec<Vec3f>) {
        let vertices: Vec<_> = self
            .vertices()
            .iter()
            .filter_map(|v| v.perspective_divide())
            .collect();

        let len = vertices.len();
        let mut edges = vec![Vec3f::zero(); len];
        for i in 0..len {
            edges[i] = vertices[(i + 1) % len] - vertices[i];
        }
        (vertices, edges)
    }

    fn contains(&self, point: Vec3f) -> bool {
        let (vertices, edges) = self.points_edges();
        if vertices.is_empty() {
            return false;
        }
        let to_point: Vec<_> = vertices.iter().map(|v| point - *v).collect();

        let first = to_point[0].cross(&edges[0]);
        to_point
            .iter()
            .zip(edges.iter())
            .all(|(v, e)| v.cross(e) * first >= 0.0)
    }
}
