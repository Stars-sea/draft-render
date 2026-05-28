use glam::{Mat4, Quat, Vec3, Vec3A};

pub struct Transform {
    translation: Vec3A,
    rotation: Quat,
    scale: Vec3A,

    matrix_cache: Option<Mat4>,
}

impl Transform {
    pub fn new(translation: Vec3A, rotation: Quat, scale: Vec3A) -> Self {
        Self {
            translation,
            rotation,
            scale,
            matrix_cache: None,
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
        self.matrix_cache = None;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.matrix_cache = None;
    }

    pub fn set_scale(&mut self, scale: Vec3A) {
        self.scale = scale;
        self.matrix_cache = None;
    }

    pub fn with_translation(mut self, translation: Vec3A) -> Self {
        self.set_translation(translation);
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.set_rotation(rotation);
        self
    }

    pub fn with_scale(mut self, scale: Vec3A) -> Self {
        self.set_scale(scale);
        self
    }

    pub fn transform_matrix(&mut self) -> Mat4 {
        if let Some(m) = self.matrix_cache {
            return m;
        }

        let mt = Mat4::from_translation(Vec3::from(self.translation));
        let mr = Mat4::from_quat(self.rotation);
        let ms = Mat4::from_scale(Vec3::from(self.scale));

        let m = mt * mr * ms;
        self.matrix_cache = Some(m);
        m
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec3A::ZERO, Quat::IDENTITY, Vec3A::ONE)
    }
}
