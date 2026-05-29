use crate::scene::{Light, Material, Scene, SceneObject, SubMesh};
use glam::{Mat3A, Mat4, Vec2, Vec3A, Vec4};
use std::sync::Arc;

pub struct DrawBatch {
    pub indices: Vec<[usize; 3]>,
    pub clip_vertices: Vec<Vec4>,
    pub world_positions: Vec<Vec3A>,
    pub vertex_normals: Vec<Vec3A>,
    pub uvs: Vec<Vec2>,
    pub material: Material,
}

pub struct RenderJob {
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub batches: Vec<DrawBatch>,
    pub width: usize,
    pub height: usize,
}

impl RenderJob {
    pub fn from_scene(scene: &Scene, width: usize, height: usize) -> Self {
        let vp = scene.camera.vp_matrix(width as f32 / height as f32);
        let camera_pos = scene.camera.position();
        Self {
            lights: scene.lights.clone(),
            batches: Self::build_batches(&scene.objects, vp, camera_pos),
            width,
            height,
        }
    }

    fn build_batches(objects: &[SceneObject], vp: Mat4, camera_pos: Vec3A) -> Vec<DrawBatch> {
        objects
            .iter()
            .flat_map(|obj| {
                let model = obj.transform.transform_matrix();
                let mvp = vp * model;
                let nm = Mat3A::from_mat4(model).inverse().transpose();
                obj.submeshes.iter().filter_map(move |sub| {
                    DrawBatch::from_submesh(sub, &model, &mvp, &nm, camera_pos)
                })
            })
            .collect::<Vec<_>>()
    }
}

impl DrawBatch {
    fn visible_indices(
        indices: &[[usize; 3]],
        world: &[Vec3A],
        camera_pos: Vec3A,
        double_sided: bool,
    ) -> Vec<[usize; 3]> {
        if double_sided {
            return indices.to_vec();
        }
        indices
            .iter()
            .copied()
            .filter(|idx| Self::is_visible(world, camera_pos, idx))
            .collect()
    }

    fn is_visible(world: &[Vec3A], camera_pos: Vec3A, idx: &[usize; 3]) -> bool {
        let (w0, w1, w2) = (world[idx[0]], world[idx[1]], world[idx[2]]);
        (w1 - w0)
            .cross(w2 - w0)
            .dot(camera_pos - (w0 + w1 + w2) / 3.0)
            > 0.0
    }

    fn from_submesh(
        sub: &SubMesh,
        model: &Mat4,
        mvp: &Mat4,
        nm: &Mat3A,
        camera_pos: Vec3A,
    ) -> Option<Self> {
        let m = &sub.mesh;
        let world: Vec<Vec3A> = m
            .vertices
            .iter()
            .map(|v| model.transform_point3a(*v))
            .collect();

        let indices =
            Self::visible_indices(&m.indices, &world, camera_pos, sub.material.double_sided());
        if indices.is_empty() {
            return None;
        }

        Some(DrawBatch {
            clip_vertices: m.vertices.iter().map(|v| *mvp * v.extend(1.0)).collect(),
            vertex_normals: m.normals.iter().map(|n| (*nm * *n).normalize()).collect(),
            uvs: m.uvs.clone(),
            material: sub.material.clone(),
            indices,
            world_positions: world,
        })
    }
}
