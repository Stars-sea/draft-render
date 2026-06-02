use crate::pipeline::bvh::Bvh;
use crate::pipeline::geometry::Triangle;
use crate::scene::{Material, Scene};
use glam::Vec3A;

/// Convert a `Scene` into a ray-tracing-ready BVH with material table.
/// Returns `(bvh, materials)` where `bvh.material_ids[i]` indexes into `materials`.
pub fn build_trace_scene(scene: &Scene) -> (Bvh, Vec<Material>) {
    let mut triangles = Vec::new();
    let mut material_ids = Vec::new();
    let mut materials = Vec::new();

    for obj in &scene.objects {
        let model = obj.transform.transform_matrix();
        let world = |v: Vec3A| model.transform_point3a(v);
        for sub in &obj.submeshes {
            let mat_id = materials.len() as u32;
            materials.push(sub.material.clone());

            let verts: Vec<Vec3A> = sub.mesh.vertices.iter().map(|&v| world(v)).collect();
            for &[i0, i1, i2] in &sub.mesh.indices {
                triangles.push(Triangle::new(verts[i0], verts[i1], verts[i2], mat_id));
                material_ids.push(mat_id);
            }
        }
    }

    let bvh = Bvh::build(triangles, material_ids);
    (bvh, materials)
}
