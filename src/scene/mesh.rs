use crate::scene::Material;
use glam::{Vec2, Vec3A};
use std::sync::Arc;

pub struct Mesh {
    pub vertices: Vec<Vec3A>,
    pub indices: Vec<[usize; 3]>,
    pub uvs: Vec<Vec2>,
    /// Per-vertex normals. Empty means flat shading will be auto-computed.
    pub normals: Vec<Vec3A>,
}

pub struct SubMesh {
    pub mesh: Arc<Mesh>,
    pub material: Material,
}

impl SubMesh {
    pub fn new(mesh: Arc<Mesh>, material: Material) -> Self {
        Self { mesh, material }
    }
}

pub struct MeshBuilder {
    vertices: Vec<Vec3A>,
    uvs: Vec<Vec2>,
    normals: Vec<Vec3A>,
    indices: Vec<[usize; 3]>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
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

    #[allow(dead_code)]
    pub fn normal(mut self, x: f32, y: f32, z: f32) -> Self {
        self.normals.push(Vec3A::new(x, y, z));
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
        debug_assert!(
            self.normals.is_empty() || self.normals.len() == self.vertices.len(),
            "normal count ({}) must match vertex count ({}) or be zero",
            self.normals.len(),
            self.vertices.len()
        );
        Mesh {
            vertices: self.vertices,
            indices: self.indices,
            uvs: self.uvs,
            normals: self.normals,
        }
    }
}
