use crate::linalg::Vec3f;

pub struct Mesh {
    pub vertices: Vec<Vec3f>,
    pub indices: Vec<[usize; 3]>,
}

pub struct MeshBuilder {
    vertices: Vec<Vec3f>,
    indices: Vec<[usize; 3]>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex(mut self, v: Vec3f) -> Self {
        self.vertices.push(v);
        self
    }

    pub fn triangle(mut self, i0: usize, i1: usize, i2: usize) -> Self {
        self.indices.push([i0, i1, i2]);
        self
    }

    pub fn build(mut self) -> Mesh {
        Mesh {
            vertices: self.vertices,
            indices: self.indices,
        }
    }
}
