mod build;
mod node;
mod visit;

use crate::geometry::{Ray, Triangle};
use build::BvhBuilder;
use node::BvhNode;
use visit::{AnyHitVisitor, ClosestHitVisitor, Visitor};

use glam::{Vec2, Vec3A};

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
    pub triangles: Vec<Triangle>,
    pub material_ids: Vec<u32>,
    pub uvs: Vec<[Vec2; 3]>,
    pub normals: Vec<[Vec3A; 3]>,
    pub cull_backface: Vec<bool>,
}

impl Bvh {
    pub fn build(
        mut triangles: Vec<Triangle>,
        mut material_ids: Vec<u32>,
        mut uvs: Vec<[Vec2; 3]>,
        mut normals: Vec<[Vec3A; 3]>,
        mut cull_backface: Vec<bool>,
    ) -> Self {
        assert_eq!(triangles.len(), material_ids.len());
        assert_eq!(triangles.len(), uvs.len());
        assert_eq!(triangles.len(), normals.len());
        assert_eq!(triangles.len(), cull_backface.len());
        let len = triangles.len();
        let mut nodes = Vec::with_capacity(len * 2);
        let root = BvhBuilder {
            nodes: &mut nodes,
            triangles: &mut triangles,
            material_ids: &mut material_ids,
            uvs: &mut uvs,
            normals: &mut normals,
            cull_backface: &mut cull_backface,
        }
        .build();
        nodes.shrink_to_fit();
        Bvh {
            nodes,
            root,
            triangles,
            material_ids,
            uvs,
            normals,
            cull_backface,
        }
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
    use glam::Vec2;

    #[test]
    fn build_empty() {
        let bvh = Bvh::build(vec![], vec![], vec![], vec![], vec![]);
        assert!(
            bvh.intersect(&Ray::new(Vec3A::ZERO, Vec3A::Z), 0.0, 100.0)
                .is_none()
        );
    }

    #[test]
    fn intersect_single() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
        );
        let bvh = Bvh::build(
            vec![tri],
            vec![0],
            vec![[Vec2::ZERO; 3]],
            vec![[Vec3A::ZERO; 3]],
            vec![false],
        );
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, 100.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().tri_idx, 0);
    }

    #[test]
    fn intersect_any_shadow() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
        );
        let bvh = Bvh::build(
            vec![tri],
            vec![0],
            vec![[Vec2::ZERO; 3]],
            vec![[Vec3A::ZERO; 3]],
            vec![false],
        );
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));
        let miss = Ray::new(Vec3A::new(2.0, 2.0, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(!bvh.intersect_any(&miss, 0.0, 100.0));
    }

    #[test]
    fn many_triangles_closest_hit() {
        let mut tris = Vec::new();
        let mut ids = Vec::new();
        let mut uvs = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                let x = col as f32;
                let y = row as f32;
                let z = 1.0 + (row * 5 + col) as f32;
                tris.push(Triangle::new(
                    Vec3A::new(x, y, z),
                    Vec3A::new(x + 0.8, y, z),
                    Vec3A::new(x, y + 0.8, z),
                ));
                ids.push((row * 5 + col) as u32);
                uvs.push([Vec2::ZERO; 3]);
            }
        }
        let normals = vec![[Vec3A::ZERO; 3]; 25];
        let cull = vec![false; 25];
        let bvh = Bvh::build(tris, ids, uvs, normals, cull);
        let ray = Ray::new(Vec3A::new(2.2, 3.2, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, 100.0);
        assert!(hit.is_some());
        let hr = hit.unwrap();
        assert_eq!(hr.tri_idx, 17);
        assert!((hr.t - 18.0).abs() < 0.01);
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));
        let miss = Ray::new(Vec3A::new(-0.5, -0.5, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(bvh.intersect(&miss, 0.0, 100.0).is_none());
    }
}
