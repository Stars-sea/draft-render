use glam::{Vec2, Vec3A};

pub struct Mesh {
    pub vertices: Vec<Vec3A>,
    pub indices: Vec<[usize; 3]>,
    pub uvs: Vec<Vec2>,
}

pub struct MeshBuilder {
    vertices: Vec<Vec3A>,
    uvs: Vec<Vec2>,
    indices: Vec<[usize; 3]>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex(mut self, v: Vec3A) -> Self {
        self.vertices.push(v);
        self
    }

    pub fn uv(mut self, u: f32, v: f32) -> Self {
        self.uvs.push(Vec2::new(u, v));
        self
    }

    pub fn triangle(mut self, i0: usize, i1: usize, i2: usize) -> Self {
        self.indices.push([i0, i1, i2]);
        self
    }

    pub fn build(self) -> Mesh {
        debug_assert!(
            self.uvs.is_empty() || self.uvs.len() == self.vertices.len(),
            "UV count ({}) must match vertex count ({}) or be zero",
            self.uvs.len(),
            self.vertices.len()
        );
        Mesh {
            vertices: self.vertices,
            indices: self.indices,
            uvs: self.uvs,
        }
    }
}
