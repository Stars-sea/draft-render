use crate::scene::{Material, Mesh, Transform};

use std::sync::Arc;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub transform: Transform,
    pub material: Material,
}

impl SceneObject {
    pub fn new(mesh: Arc<Mesh>, transform: Transform, material: Material) -> Self {
        Self {
            mesh,
            transform,
            material,
        }
    }
}
