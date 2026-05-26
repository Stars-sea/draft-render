use crate::linalg::{Mat4, Quaternion, Rotator, Vec3, transform};
use num_traits::Zero;
use num_traits::real::Real;

pub struct Transform<T: Real> {
    translation: Vec3<T>,
    rotation: Quaternion<T>,
    scale: Vec3<T>,

    matrix_cache: Option<Mat4<T>>,
}

impl<T: Real> Transform<T> {
    pub fn new(translation: Vec3<T>, rotation: Quaternion<T>, scale: Vec3<T>) -> Self {
        Self {
            translation,
            rotation,
            scale,
            matrix_cache: None,
        }
    }

    pub fn with_translation(&mut self, translation: Vec3<T>) -> &mut Self {
        self.translation = translation;
        self.matrix_cache = None;
        self
    }

    pub fn with_rotation(&mut self, rotation: Quaternion<T>) -> &mut Self {
        self.rotation = rotation;
        self.matrix_cache = None;
        self
    }

    pub fn with_scale(&mut self, scale: Vec3<T>) -> &mut Self {
        self.scale = scale;
        self.matrix_cache = None;
        self
    }

    pub fn transform_matrix(&mut self) -> Mat4<T> {
        if let Some(matrix_cache) = self.matrix_cache {
            matrix_cache
        } else {
            let rotator: Rotator<T> = self.rotation.into();

            let mt: Mat4<T> = transform::translate(self.translation);
            let mr: Mat4<T> = rotator.into();
            let ms: Mat4<T> = transform::scale(self.scale);

            let matrix = mt * mr * ms;
            self.matrix_cache = Some(matrix);
            matrix
        }
    }
}

impl<T: Real> Default for Transform<T> {
    fn default() -> Self {
        Self::new(
            Vec3::zero(),
            Quaternion::identity(),
            Vec3::new(T::one(), T::one(), T::one()),
        )
    }
}
