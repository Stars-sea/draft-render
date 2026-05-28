use crate::render_thread::{RenderJob, RenderObject};
use crate::scene::object::SceneObject;
use crate::scene::{Camera, Light, Mesh};
use glam::{Mat4, Vec3A};
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

        let objects: Vec<RenderObject> = self
            .objects
            .iter_mut()
            .map(|obj| {
                let model = obj.transform.transform_matrix();
                let mvp = vp * model;
                let world_positions: Vec<Vec3A> = obj
                    .mesh
                    .vertices
                    .iter()
                    .map(|v| model.transform_point3a(*v))
                    .collect();
                RenderObject {
                    mesh: Arc::clone(&obj.mesh),
                    mvp,
                    color: obj.color,
                    face_normals: compute_face_normals(&obj.mesh, &model),
                    world_positions,
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

fn compute_face_normals(mesh: &Mesh, model: &Mat4) -> Vec<Vec3A> {
    mesh.indices
        .iter()
        .map(|&[i0, i1, i2]| {
            let v0 = mesh.vertices[i0];
            let v1 = mesh.vertices[i1];
            let v2 = mesh.vertices[i2];
            let n = (v1 - v0).cross(v2 - v0).normalize();
            model.transform_vector3a(n).normalize()
        })
        .collect()
}
