use crate::linalg::{Mat4f, Quaternion, Rotator, Vec3f, transform};
use num_traits::ConstZero;

pub struct Transform {
    translation: Vec3f,
    rotation: Quaternion<f32>,
    scale: Vec3f,

    matrix_cache: Option<Mat4f>,
}

impl Transform {
    pub fn new(translation: Vec3f, rotation: Quaternion<f32>, scale: Vec3f) -> Self {
        Self {
            translation,
            rotation,
            scale,
            matrix_cache: None,
        }
    }

    pub fn translation(&self) -> Vec3f {
        self.translation
    }
    pub fn rotation(&self) -> Quaternion<f32> {
        self.rotation
    }
    pub fn scale(&self) -> Vec3f {
        self.scale
    }

    pub fn with_translation(&mut self, translation: Vec3f) -> &mut Self {
        self.translation = translation;
        self.matrix_cache = None;
        self
    }

    pub fn with_rotation(&mut self, rotation: Quaternion<f32>) -> &mut Self {
        self.rotation = rotation;
        self.matrix_cache = None;
        self
    }

    pub fn with_scale(&mut self, scale: Vec3f) -> &mut Self {
        self.scale = scale;
        self.matrix_cache = None;
        self
    }

    pub fn transform_matrix(&mut self) -> Mat4f {
        if let Some(m) = self.matrix_cache {
            return m;
        }

        let rotator: Rotator<f32> = self.rotation.into();
        let mt: Mat4f = transform::translate(self.translation);
        let mr: Mat4f = rotator.into();
        let ms: Mat4f = transform::scale(self.scale);

        let m = mt * mr * ms;
        self.matrix_cache = Some(m);
        m
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec3f::ZERO, Quaternion::identity(), Vec3f::ONE)
    }
}
