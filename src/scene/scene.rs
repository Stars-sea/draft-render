use crate::linalg::Vec4f;
use crate::render_thread::{RenderJob, RenderObject};
use crate::scene::Camera;
use crate::scene::object::SceneObject;

use std::sync::Arc;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    pub fn build_render_job(&mut self, width: usize, height: usize) -> RenderJob {
        let aspect = width as f32 / height as f32;
        let vp = self.camera.vp_matrix(aspect);

        let objects: Vec<RenderObject> = self
            .objects
            .iter_mut()
            .map(|obj| RenderObject {
                mesh: Arc::clone(&obj.mesh),
                mvp: vp * obj.transform.transform_matrix(),
                color: obj.color,
            })
            .collect();

        RenderJob {
            objects,
            width,
            height,
        }
    }
}
