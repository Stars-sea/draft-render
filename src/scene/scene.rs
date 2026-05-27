use crate::render_thread::{RenderJob, RenderObject};
use crate::scene::Camera;
use crate::scene::object::SceneObject;

use num_traits::real::Real;
use std::sync::Arc;

pub struct Scene<T: Real> {
    pub camera: Camera<T>,
    pub objects: Vec<SceneObject<T>>,
}

impl<T: Real> Scene<T> {
    pub fn new(camera: Camera<T>) -> Self {
        Self {
            camera,
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: SceneObject<T>) {
        self.objects.push(object);
    }

    pub fn build_render_job(&mut self, width: usize, height: usize) -> RenderJob<T> {
        let aspect = T::from(width).unwrap() / T::from(height).unwrap();
        let vp = self.camera.vp_matrix(aspect);

        let objects: Vec<RenderObject<T>> = self
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
