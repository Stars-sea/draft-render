mod build;
mod node;
mod visit;

use crate::geometry::{Ray, Triangle};
use node::BvhNode;
use visit::{AnyHitVisitor, ClosestHitVisitor, Visitor};

use glam::{Vec2, Vec3A};

pub use build::BvhBuilder;

pub struct Hit {
    pub tri_idx: usize,
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

impl Hit {
    #[inline]
    pub fn point(&self, ray: &Ray) -> Vec3A {
        ray.at(self.t)
    }

    #[inline]
    pub fn interpolate_uv(&self, uvs: &[[Vec2; 3]]) -> Vec2 {
        let uv = &uvs[self.tri_idx];
        let w = 1.0 - self.u - self.v;
        uv[0] * w + uv[1] * self.u + uv[2] * self.v
    }

    #[inline]
    pub fn interpolate_normal(&self, normals: &[[Vec3A; 3]]) -> Vec3A {
        let n = &normals[self.tri_idx];
        let w = 1.0 - self.u - self.v;
        (n[0] * w + n[1] * self.u + n[2] * self.v).normalize()
    }
}

pub struct Bvh {
    nodes: Vec<BvhNode>,
    root: u32,
    pub(super) triangles: Vec<Triangle>,
    pub(super) material_ids: Vec<u32>,
    pub(super) uvs: Vec<[Vec2; 3]>,
    pub(super) normals: Vec<[Vec3A; 3]>,
    pub(super) cull_backface: Vec<bool>,
}

impl Bvh {
    #[inline]
    pub fn tri_data(&self, tri_idx: usize) -> (&Triangle, u32, &[Vec2; 3], &[Vec3A; 3]) {
        (
            &self.triangles[tri_idx],
            self.material_ids[tri_idx],
            &self.uvs[tri_idx],
            &self.normals[tri_idx],
        )
    }

    #[inline]
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        ClosestHitVisitor {
            t_closest: t_max,
            best: None,
            cull_backface: &self.cull_backface,
        }
        .traverse(ray, &self.triangles, &self.nodes, self.root, t_min)
        .map(|(tri_idx, t, u, v)| Hit { tri_idx, t, u, v })
    }

    #[inline]
    pub fn intersect_any(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        AnyHitVisitor {
            found: false,
            t_max,
            cull_backface: &self.cull_backface,
        }
        .traverse(ray, &self.triangles, &self.nodes, self.root, t_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Mesh;
    use glam::Mat3;

    fn mesh_from_tris(tris: &[(Vec3A, Vec3A, Vec3A)]) -> Mesh {
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        for &(v0, v1, v2) in tris {
            let i = verts.len();
            verts.push(v0);
            verts.push(v1);
            verts.push(v2);
            indices.push([i, i + 1, i + 2]);
        }
        Mesh {
            vertices: verts,
            indices,
            uvs: vec![],
            normals: vec![],
        }
    }

    #[test]
    fn build_empty() {
        let bvh = BvhBuilder::new().build();
        assert!(
            bvh.intersect(&Ray::new(Vec3A::ZERO, Vec3A::Z), 0.0, 100.0)
                .is_none()
        );
    }

    #[test]
    fn intersect_single() {
        let mesh = mesh_from_tris(&[(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
        )]);
        let mut b = BvhBuilder::new();
        b.push(&mesh, Mat3::IDENTITY, 0, true);
        let bvh = b.build();
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, 100.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().tri_idx, 0);
    }

    #[test]
    fn intersect_any_shadow() {
        let mesh = mesh_from_tris(&[(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
        )]);
        let mut b = BvhBuilder::new();
        b.push(&mesh, Mat3::IDENTITY, 0, true);
        let bvh = b.build();
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));
        let miss = Ray::new(Vec3A::new(2.0, 2.0, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(!bvh.intersect_any(&miss, 0.0, 100.0));
    }

    #[test]
    fn many_triangles_closest_hit() {
        let mut tris = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                let x = col as f32;
                let y = row as f32;
                let z = 1.0 + (row * 5 + col) as f32;
                tris.push((
                    Vec3A::new(x, y, z),
                    Vec3A::new(x + 0.8, y, z),
                    Vec3A::new(x, y + 0.8, z),
                ));
            }
        }
        let mesh = mesh_from_tris(&tris);
        let mut b = BvhBuilder::new();
        b.push(&mesh, Mat3::IDENTITY, 0, true);
        let bvh = b.build();
        let ray = Ray::new(Vec3A::new(2.2, 3.2, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, 100.0);
        assert!(hit.is_some());
        assert!(hit.unwrap().t < 20.0);
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));
    }
}
