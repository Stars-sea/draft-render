use crate::scene::{SubMesh, Transform};

pub struct SceneObject {
    pub submeshes: Vec<SubMesh>,
    pub transform: Transform,
}

impl SceneObject {
    pub fn new(submeshes: Vec<SubMesh>, transform: Transform) -> Self {
        Self {
            submeshes,
            transform,
        }
    }

    pub fn single(submesh: SubMesh, transform: Transform) -> Self {
        Self::new(vec![submesh], transform)
    }
}
