use crate::color::Color;
use crate::scene::mesh::Mesh;
use crate::scene::transform::Transform;

use std::sync::Arc;
use num_traits::real::Real;

pub struct SceneObject<T: Real> {
    pub mesh: Arc<Mesh<T>>,
    pub transform: Transform<T>,
    pub color: Color,
}

impl<T: Real> SceneObject<T> {
    pub fn new(mesh: Arc<Mesh<T>>, transform: Transform<T>, color: Color) -> Self {
        Self {
            mesh,
            transform,
            color,
        }
    }
}
