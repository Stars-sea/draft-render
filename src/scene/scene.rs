use crate::linalg::{Mat4f, Vec3f, Vec4f};
use crate::render_thread::{RenderJob, RenderObject};
use crate::scene::Mesh;
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
            .map(|obj| {
                let model = obj.transform.transform_matrix();
                let mvp = vp * model;
                let world_positions: Vec<Vec3f> = obj
                    .mesh
                    .vertices
                    .iter()
                    .map(|v| (model * Vec4f::from_vec3(*v, 1.0)).xyz())
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

/// 从 mesh 顶点计算对象空间面法线，再用模型矩阵的上 3x3 变换到世界空间。
fn compute_face_normals(mesh: &Mesh, model: &Mat4f) -> Vec<Vec3f> {
    mesh.indices
        .iter()
        .map(|&[i0, i1, i2]| {
            let v0 = mesh.vertices[i0];
            let v1 = mesh.vertices[i1];
            let v2 = mesh.vertices[i2];
            let n = (v1 - v0).cross(&(v2 - v0)).normalize();

            Vec3f::new(
                model.row(0).xyz() * n,
                model.row(1).xyz() * n,
                model.row(2).xyz() * n,
            )
            .normalize()
        })
        .collect()
}
