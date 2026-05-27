use crate::linalg::{Mat4, Vec3, transform};
use num_traits::real::Real;
use std::f32::consts;

pub enum Projection<T: Real> {
    Perspective {
        fov: T,
        near: T,
        far: T,
    },
    Orthographic {
        left: T,
        right: T,
        bottom: T,
        top: T,
        near: T,
        far: T,
    },
}

impl<T: Real> Projection<T> {
    pub fn perspective(fov: T, near: T, far: T) -> Self {
        Self::Perspective { fov, near, far }
    }

    pub fn orthographic(left: T, right: T, bottom: T, top: T, near: T, far: T) -> Self {
        Self::Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    pub fn orthographic_from_size(width: T, height: T, near: T, far: T) -> Self {
        let two = T::from(2.0).unwrap();
        let hw = width / two;
        let hh = height / two;
        Self::orthographic(-hw, hw, -hh, hh, near, far)
    }

    pub fn projection_matrix(&self, aspect: T) -> Mat4<T> {
        let (o, i, two) = (T::zero(), T::one(), T::from(2.0).unwrap());

        match self {
            Self::Perspective { fov, near, far } => {
                let (fov, n, f) = (*fov, *near, *far);
                let squeeze = Mat4::from([
                    [n, o, o, o],
                    [o, n, o, o],
                    [o, o, n + f, -n * f],
                    [o, o, i, o],
                ]);
                let tan = n * (fov / two).tan();
                let width = two * tan * aspect;
                let height = two * tan;
                let ortho = Self::orthographic_from_size(width, height, n, f);
                ortho.projection_matrix(aspect) * squeeze
            }
            Self::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => {
                let (l, r, b, t, n, f) = (*left, *right, *bottom, *top, *near, *far);
                let center = Vec3::new((r + l) / two, (t + b) / two, (n + f) / two);
                let mt = transform::translate(-center);
                let ms = transform::scale(Vec3::new(two / (r - l), two / (t - b), two / (f - n)));
                ms * mt
            }
        }
    }
}

impl<T: Real> Default for Projection<T> {
    fn default() -> Self {
        Self::perspective(
            T::from(consts::FRAC_PI_3).unwrap(),
            T::from(0.1).unwrap(),
            T::from(100.0).unwrap(),
        )
    }
}
