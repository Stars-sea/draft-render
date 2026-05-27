use crate::color::Color;
use crate::linalg::Vec3f;
use crate::scene::{Mesh, Transform};

use std::sync::Arc;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub transform: Transform,
    pub color: Color,
}

impl SceneObject {
    pub fn new(mesh: Arc<Mesh>, transform: Transform, color: Color) -> Self {
        Self {
            mesh,
            transform,
            color,
        }
    }
}
