use crate::render_thread::{RenderJob, RenderObject};
use crate::scene::object::SceneObject;
use crate::scene::{Camera, Light};
use glam::{Mat3A, Vec3A};
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

    pub fn build_render_job(&self, width: usize, height: usize) -> RenderJob {
        let aspect = width as f32 / height as f32;
        let vp = self.camera.vp_matrix(aspect);

        let objects: Vec<RenderObject> = self
            .objects
            .iter()
            .map(|obj| {
                let model = obj.transform.transform_matrix();
                let mvp = vp * model;
                let clip_vertices: Vec<_> = obj
                    .mesh
                    .vertices
                    .iter()
                    .map(|v| mvp * v.extend(1.0))
                    .collect();
                let world_positions: Vec<Vec3A> = obj
                    .mesh
                    .vertices
                    .iter()
                    .map(|v| model.transform_point3a(*v))
                    .collect();
                let normal_matrix = Mat3A::from_mat4(model).inverse().transpose();
                let vertex_normals: Vec<Vec3A> = obj
                    .mesh
                    .normals
                    .iter()
                    .map(|n| (normal_matrix * *n).normalize())
                    .collect();
                RenderObject {
                    indices: obj.mesh.indices.clone(),
                    clip_vertices,
                    world_positions,
                    vertex_normals,
                    uvs: obj.mesh.uvs.clone(),
                    material: obj.material.clone(),
                }
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
