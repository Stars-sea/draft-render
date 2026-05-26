use crate::scene::mesh::Mesh;
use crate::scene::transform::Transform;
use std::sync::Arc;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub transform: Transform<f32>,
}

impl SceneObject {
    pub fn new(mesh: Arc<Mesh>, transform: Transform<f32>) -> Self {
        Self { mesh, transform }
    }
}
