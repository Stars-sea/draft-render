use crate::render_thread::{RenderJob, RenderObject};
use crate::scene::object::SceneObject;
use crate::scene::{Camera, Light};
use std::sync::Arc;

pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn add_light(&mut self, light: Arc<dyn Light + Send + Sync>) {
        self.lights.push(light);
    }

    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    pub fn build_render_job(&mut self, width: usize, height: usize) -> RenderJob {
        let aspect = width as f32 / height as f32;
        let vp = self.camera.vp_matrix(aspect);

        let objects = self
            .objects
            .iter_mut()
            .map(|obj| RenderObject {
                mesh: Arc::clone(&obj.mesh),
                mvp: vp * obj.transform.transform_matrix(),
                color: obj.color,
            })
            .collect();

        RenderJob {
            lights: self.lights.clone(),
            objects,
            width,
            height,
        }
    }
}
