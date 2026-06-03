use crate::pipeline::bvh::{Bvh, BvhBuilder};
use crate::scene::{Light, Material, Scene};
use glam::{Mat3, Mat4};
use std::sync::Arc;

pub(super) struct ObjTransform {
    pub(crate) model: Mat4,
    pub(crate) normal_mat: Mat3,
}

pub struct TraceScene {
    pub(crate) objects: Vec<Bvh>,
    pub(crate) materials: Vec<Material>,
    pub(crate) lights: Vec<Arc<dyn Light>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl TraceScene {
    pub fn from_scene(scene: &Scene, width: usize, height: usize) -> Self {
        let mut materials = Vec::new();
        let mut objects = Vec::new();

        for obj in &scene.objects {
            let normal_mat = obj.transform.normal_matrix();
            let mut builder = BvhBuilder::new();

            for sub in &obj.submeshes {
                if sub.material.texture.as_ref().is_some_and(|t| t.is_dark_effect()) {
                    continue;
                }
                let mat_id = materials.len() as u32;
                materials.push(sub.material.clone());
                builder.push(&sub.mesh, normal_mat, mat_id, sub.material.double_sided);
            }

            let bvh = builder.build();
            if !bvh.triangles.is_empty() {
                objects.push(bvh);
            }
        }

        TraceScene { objects, materials, lights: scene.lights.clone(), width, height }
    }
}
