use crate::geometry::Triangle;
use crate::pipeline::bvh::Bvh;
use crate::scene::{Light, Material, Scene, SceneObject, SubMesh};
use glam::{Vec2, Vec3A};
use std::sync::Arc;

/// Pre-built ray-tracing data: BVH, material table, and lights.
pub struct TraceScene {
    pub(crate) bvh: Bvh,
    pub(crate) materials: Vec<Material>,
    pub(crate) lights: Vec<Arc<dyn Light>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl TraceScene {
    pub fn from_scene(scene: &Scene, width: usize, height: usize) -> Self {
        let mut builder = SceneBuilder::new();
        for obj in &scene.objects {
            builder.push_object(obj);
        }
        builder.build(scene.lights.clone(), width, height)
    }
}

struct SceneBuilder {
    triangles: Vec<Triangle>,
    material_ids: Vec<u32>,
    materials: Vec<Material>,
    tri_uvs: Vec<[Vec2; 3]>,
    tri_normals: Vec<[Vec3A; 3]>,
}

impl SceneBuilder {
    fn new() -> Self {
        Self {
            triangles: Vec::new(),
            material_ids: Vec::new(),
            materials: Vec::new(),
            tri_uvs: Vec::new(),
            tri_normals: Vec::new(),
        }
    }

    fn push_object(&mut self, obj: &SceneObject) {
        let model = obj.transform.transform_matrix();
        for sub in &obj.submeshes {
            if sub
                .material
                .texture
                .as_ref()
                .is_some_and(|t| t.is_dark_effect())
            {
                continue;
            }
            self.push_submesh(sub, &model);
        }
    }

    fn push_submesh(&mut self, sub: &SubMesh, model: &glam::Mat4) {
        let mat_id = self.materials.len() as u32;
        self.materials.push(sub.material.clone());

        let verts: Vec<Vec3A> = sub
            .mesh
            .vertices
            .iter()
            .map(|&v| model.transform_point3a(v))
            .collect();
        let mesh_uvs = &sub.mesh.uvs;
        let mesh_normals = &sub.mesh.normals;
        for &[i0, i1, i2] in &sub.mesh.indices {
            self.triangles
                .push(Triangle::new(verts[i0], verts[i1], verts[i2]));
            self.material_ids.push(mat_id);
            let uv = if mesh_uvs.is_empty() {
                [Vec2::ZERO; 3]
            } else {
                [mesh_uvs[i0], mesh_uvs[i1], mesh_uvs[i2]]
            };
            self.tri_uvs.push(uv);
            let n = if mesh_normals.is_empty() {
                [Vec3A::ZERO; 3]
            } else {
                let n0 = model.transform_vector3a(mesh_normals[i0]).normalize();
                let n1 = model.transform_vector3a(mesh_normals[i1]).normalize();
                let n2 = model.transform_vector3a(mesh_normals[i2]).normalize();
                [n0, n1, n2]
            };
            self.tri_normals.push(n);
        }
    }

    fn build(
        self,
        lights: Vec<Arc<dyn Light>>,
        width: usize,
        height: usize,
    ) -> TraceScene {
        TraceScene {
            bvh: Bvh::build(self.triangles, self.material_ids, self.tri_uvs, self.tri_normals),
            materials: self.materials,
            lights,
            width,
            height,
        }
    }
}
