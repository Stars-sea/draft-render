use crate::geometry::ray::Ray;
use glam::Vec3A;

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Vec3A,
    pub max: Vec3A,
}

impl BoundingBox {
    pub const fn empty() -> Self {
        Self {
            min: Vec3A::splat(f32::INFINITY),
            max: Vec3A::splat(f32::NEG_INFINITY),
        }
    }

    #[allow(dead_code)]
    pub fn from_points(points: &[Vec3A]) -> Self {
        let mut bbox = Self::empty();
        for &p in points {
            bbox.min = bbox.min.min(p);
            bbox.max = bbox.max.max(p);
        }
        bbox
    }

    #[allow(dead_code)]
    pub fn extend(&mut self, p: Vec3A) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn merge(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn extents(&self) -> Vec3A {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f32 {
        let e = self.extents();
        2.0 * (e.x * e.y + e.y * e.z + e.z * e.x)
    }

    pub fn intersect_range(&self, ray: &Ray) -> Option<(f32, f32)> {
        let t0 = (self.min - ray.origin) * ray.inv_direction;
        let t1 = (self.max - ray.origin) * ray.inv_direction;
        let near = t0.min(t1);
        let far = t0.max(t1);

        // When inv_direction is infinite (ray parallel to axis), min/max ops
        // produce NaN. Replace NaN with ±∞ so the [t_near, t_far] check passes.
        let nan = near.is_nan_mask();
        let near = Vec3A::select(nan, Vec3A::splat(f32::NEG_INFINITY), near);
        let far = Vec3A::select(nan, Vec3A::splat(f32::INFINITY), far);

        let t_near = near.max_element();
        let t_far = far.min_element();
        if t_near <= t_far {
            Some((t_near, t_far))
        } else {
            None
        }
    }

    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        self.intersect_range(ray)
            .is_some_and(|(tn, tf)| tn.max(t_min) <= tf.min(t_max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let b = BoundingBox::empty();
        assert!(b.min.x.is_infinite());
        assert!(b.max.x.is_infinite());
    }

    #[test]
    fn from_points() {
        let points = [
            Vec3A::new(0.0, 0.0, 0.0),
            Vec3A::new(1.0, 2.0, 3.0),
            Vec3A::new(-1.0, -1.0, -1.0),
        ];
        let b = BoundingBox::from_points(&points);
        assert_eq!(b.min, Vec3A::new(-1.0, -1.0, -1.0));
        assert_eq!(b.max, Vec3A::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn extend() {
        let mut b = BoundingBox::from_points(&[Vec3A::new(0.0, 0.0, 0.0)]);
        b.extend(Vec3A::new(2.0, -1.0, 0.5));
        assert_eq!(b.min, Vec3A::new(0.0, -1.0, 0.0));
        assert_eq!(b.max, Vec3A::new(2.0, 0.0, 0.5));
    }

    #[test]
    fn merge() {
        let a = BoundingBox {
            min: Vec3A::new(0.0, 0.0, 0.0),
            max: Vec3A::new(1.0, 1.0, 1.0),
        };
        let b = BoundingBox {
            min: Vec3A::new(0.5, -1.0, 0.5),
            max: Vec3A::new(2.0, 0.5, 2.0),
        };
        let m = a.merge(&b);
        assert_eq!(m.min, Vec3A::new(0.0, -1.0, 0.0));
        assert_eq!(m.max, Vec3A::new(2.0, 1.0, 2.0));
    }

    #[test]
    fn surface_area() {
        let b = BoundingBox {
            min: Vec3A::new(0.0, 0.0, 0.0),
            max: Vec3A::new(2.0, 3.0, 4.0),
        };
        assert!((b.surface_area() - 52.0).abs() < 1e-6);
    }

    #[test]
    fn ray_hit() {
        let b = BoundingBox {
            min: Vec3A::new(0.0, 0.0, 0.0),
            max: Vec3A::new(1.0, 1.0, 1.0),
        };
        let ray = Ray::new(Vec3A::new(0.5, 0.5, -1.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(b.intersect(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn ray_miss() {
        let b = BoundingBox {
            min: Vec3A::new(0.0, 0.0, 0.0),
            max: Vec3A::new(1.0, 1.0, 1.0),
        };
        let ray = Ray::new(Vec3A::new(2.0, 0.5, 0.5), Vec3A::new(1.0, 0.0, 0.0));
        assert!(!b.intersect(&ray, 0.0, f32::INFINITY));
    }
}
