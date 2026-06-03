use glam::{Mat3, Mat4, Quat, Vec3, Vec3A};

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
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec3A::ZERO, Quat::IDENTITY, Vec3A::ONE)
    }
}
