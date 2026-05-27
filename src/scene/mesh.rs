use num_traits::real::Real;
use crate::linalg::Vec3;

pub struct Mesh<T: Real> {
    pub vertices: Vec<Vec3<T>>,
    pub indices: Vec<[usize; 3]>,
}

pub struct MeshBuilder<T: Real> {
    vertices: Vec<Vec3<T>>,
    indices: Vec<[usize; 3]>,
}

impl<T: Real> MeshBuilder<T> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex(mut self, v: Vec3<T>) -> Self {
        self.vertices.push(v);
        self
    }

    pub fn triangle(mut self, i0: usize, i1: usize, i2: usize) -> Self {
        self.indices.push([i0, i1, i2]);
        self
    }

    pub fn build(mut self) -> Mesh<T> {
        Mesh {
            vertices: self.vertices,
            indices: self.indices,
        }
    }
}
