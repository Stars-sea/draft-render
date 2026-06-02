use glam::Vec3A;

// ---- Ray ----

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
    pub inv_direction: Vec3A,
}

impl Ray {
    pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
        let dir = direction.normalize();
        Self {
            origin,
            direction: dir,
            inv_direction: Vec3A::ONE / dir,
        }
    }

    pub fn at(&self, t: f32) -> Vec3A {
        self.origin + self.direction * t
    }
}

// ---- Triangle ----

/// World-space triangle with pre-computed edges.
///
/// Storing `e1` and `e2` avoids recomputing them on every ray intersection.
/// `material_id` survives BVH reordering so the hit shader can look up the
/// correct material.
#[derive(Clone, Copy)]
pub struct Triangle {
    pub v0: Vec3A,
    pub e1: Vec3A,
    pub e2: Vec3A,
    pub material_id: u32,
}

impl Triangle {
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A, material_id: u32) -> Self {
        Self {
            v0,
            e1: v1 - v0,
            e2: v2 - v0,
            material_id,
        }
    }

    pub fn v1(&self) -> Vec3A {
        self.v0 + self.e1
    }

    pub fn v2(&self) -> Vec3A {
        self.v0 + self.e2
    }

    pub fn centroid(&self) -> Vec3A {
        self.v0 + (self.e1 + self.e2) / 3.0
    }

    pub fn normal(&self) -> Vec3A {
        self.e1.cross(self.e2).normalize()
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let (v1, v2) = (self.v1(), self.v2());
        BoundingBox {
            min: self.v0.min(v1).min(v2),
            max: self.v0.max(v1).max(v2),
        }
    }
}

// ---- Ray-Triangle Intersection ----

/// Möller–Trumbore ray-triangle intersection.
/// Returns `(t, u, v)` — distance along ray and barycentric coords (w = 1 - u - v).
pub fn intersect(ray: &Ray, tri: &Triangle) -> Option<(f32, f32, f32)> {
    let h = ray.direction.cross(tri.e2);
    let a = tri.e1.dot(h);

    if a.abs() < 1e-7 {
        return None;
    }

    let f = 1.0 / a;
    let s = ray.origin - tri.v0;
    let u = f * s.dot(h);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(tri.e1);
    let v = f * ray.direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * tri.e2.dot(q);

    if t > 1e-6 { Some((t, u, v)) } else { None }
}

// Placed here (not at top) because `bounding_box` depends on `geometry::Ray`,
// and `Triangle::bounding_box` returns `BoundingBox`.  Moving this `use` up
// would create a circular dependency between `geometry` and `bounding_box`.
use crate::pipeline::bounding_box::BoundingBox;

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_hit() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
            0,
        );
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = intersect(&ray, &tri);
        assert!(hit.is_some());
        let (t, u, v) = hit.unwrap();
        assert!((t - 1.0).abs() < 1e-4);
        assert!((u - 0.25).abs() < 1e-4);
        assert!((v - 0.25).abs() < 1e-4);
    }

    #[test]
    fn intersect_miss() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
            0,
        );
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        assert!(intersect(&ray, &tri).is_none());
    }

    #[test]
    fn intersect_behind_ray() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, -1.0),
            Vec3A::new(1.0, 0.0, -1.0),
            Vec3A::new(0.0, 1.0, -1.0),
            0,
        );
        let ray = Ray::new(Vec3A::ZERO, Vec3A::Z);
        assert!(intersect(&ray, &tri).is_none());
    }

    #[test]
    fn centroid() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 0.0),
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            0,
        );
        let c = tri.centroid();
        assert!((c.x - 0.333333).abs() < 1e-4);
        assert!((c.y - 0.333333).abs() < 1e-4);
        assert!((c.z - 0.0).abs() < 1e-4);
    }

    #[test]
    fn normal_returns_unit_vector() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 0.0),
            Vec3A::new(2.0, 0.0, 0.0),
            Vec3A::new(0.0, 2.0, 0.0),
            0,
        );
        let n = tri.normal();
        assert!((n.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn edges_precomputed() {
        let tri = Triangle::new(
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(3.0, 0.0, 0.0),
            Vec3A::new(1.0, 2.0, 0.0),
            0,
        );
        assert_eq!(tri.e1, Vec3A::new(2.0, 0.0, 0.0));
        assert_eq!(tri.e2, Vec3A::new(0.0, 2.0, 0.0));
        assert_eq!(tri.v1(), Vec3A::new(3.0, 0.0, 0.0));
        assert_eq!(tri.v2(), Vec3A::new(1.0, 2.0, 0.0));
    }
}
