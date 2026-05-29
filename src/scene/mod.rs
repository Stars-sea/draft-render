mod camera;
mod light;
mod material;
mod mesh;
mod object;
mod projection;
mod transform;

use crate::render_thread::{DrawBatch, RenderJob};
use glam::{Mat3A, Vec3A};
use std::sync::Arc;

pub use camera::Camera;
pub use light::{DirectionalLight, Light, PointLight};
pub use material::{Material, Texture};
pub use mesh::{Mesh, MeshBuilder, SubMesh};
pub use object::SceneObject;
pub use projection::Projection;
pub use transform::Transform;

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

        let mut batches = Vec::new();

        for obj in &self.objects {
            let model = obj.transform.transform_matrix();
            let mvp = vp * model;
            let normal_matrix = Mat3A::from_mat4(model).inverse().transpose();

            for sub in &obj.submeshes {
                let m = &sub.mesh;
                let clip_vertices: Vec<_> = m
                    .vertices
                    .iter()
                    .map(|v| mvp * v.extend(1.0))
                    .collect();
                let world_positions: Vec<Vec3A> = m
                    .vertices
                    .iter()
                    .map(|v| model.transform_point3a(*v))
                    .collect();
                let vertex_normals: Vec<Vec3A> = m
                    .normals
                    .iter()
                    .map(|n| (normal_matrix * *n).normalize())
                    .collect();
                batches.push(DrawBatch {
                    indices: m.indices.clone(),
                    clip_vertices,
                    world_positions,
                    vertex_normals,
                    uvs: m.uvs.clone(),
                    material: sub.material.clone(),
                });
            }
        }

        RenderJob {
            lights: self.lights.clone(),
            batches,
            camera_pos: self.camera.position(),
            width,
            height,
        }
    }
}
