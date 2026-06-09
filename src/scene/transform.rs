use std::hash::{DefaultHasher, Hash, Hasher};
use crate::geometry::Ray;
use glam::{Mat3, Mat4, Quat, Vec3, Vec3A};

/// Pre-computed transform matrices for a scene object, used during ray tracing.
#[derive(Clone, Copy)]
pub struct ObjTransform {
    pub model: Mat4,
    pub normal_mat: Mat3,
}

pub struct Transform {
    translation: Vec3A,
    rotation: Quat,
    scale: Vec3A,
}

#[allow(unused)]
impl Transform {
    pub fn new(translation: Vec3A, rotation: Quat, scale: Vec3A) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn translation(&self) -> Vec3A {
        self.translation
    }
    pub fn rotation(&self) -> Quat {
        self.rotation
    }
    pub fn scale(&self) -> Vec3A {
        self.scale
    }

    pub fn set_translation(&mut self, translation: Vec3A) {
        self.translation = translation;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3A) {
        self.scale = scale;
    }

    #[must_use]
    pub fn with_translation(mut self, translation: Vec3A) -> Self {
        self.set_translation(translation);
        self
    }

    #[must_use]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.set_rotation(rotation);
        self
    }

    #[must_use]
    pub fn with_scale(mut self, scale: Vec3A) -> Self {
        self.set_scale(scale);
        self
    }

    pub fn transform_matrix(&self) -> Mat4 {
        let mt = Mat4::from_translation(Vec3::from(self.translation));
        let mr = Mat4::from_quat(self.rotation);
        let ms = Mat4::from_scale(Vec3::from(self.scale));
        mt * mr * ms
    }

    /// Inverse-transpose of the upper-left 3×3, for transforming normals
    /// correctly under non-uniform scaling.
    pub fn normal_matrix(&self) -> Mat3 {
        let r = Mat3::from_quat(self.rotation);
        let s_inv = Mat3::from_diagonal(1.0 / Vec3::from(self.scale));
        r * s_inv
    }

    pub(crate) fn state_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.translation.x.to_bits().hash(&mut h);
        self.translation.y.to_bits().hash(&mut h);
        self.translation.z.to_bits().hash(&mut h);
        self.rotation.x.to_bits().hash(&mut h);
        self.rotation.y.to_bits().hash(&mut h);
        self.rotation.z.to_bits().hash(&mut h);
        self.rotation.w.to_bits().hash(&mut h);
        self.scale.x.to_bits().hash(&mut h);
        self.scale.y.to_bits().hash(&mut h);
        self.scale.z.to_bits().hash(&mut h);
        h.finish()
    }

    pub fn to_obj_transform(&self) -> ObjTransform {
        ObjTransform {
            model: self.transform_matrix(),
            normal_mat: self.normal_matrix(),
        }
    }
}

impl ObjTransform {
    /// Transform a world-space ray to local space. Returns `(local_ray, dir_scale)`.
    pub fn ray_to_local(&self, ray: &Ray) -> (Ray, f32) {
        let inv = self.model.inverse();
        let origin = inv.transform_point3a(ray.origin);
        let dir_raw = inv.transform_vector3a(ray.direction);
        let s = dir_raw.length();
        (Ray::new(origin, dir_raw / s), s)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec3A::ZERO, Quat::IDENTITY, Vec3A::ONE)
    }
}
