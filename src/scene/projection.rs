use crate::linalg::{Mat4f, Vec3f, transform};
use std::f32::consts;

pub enum Projection {
    Perspective { fov: f32, near: f32, far: f32 },
    Orthographic { left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32 },
}

impl Projection {
    pub fn perspective(fov: f32, near: f32, far: f32) -> Self {
        Self::Perspective { fov, near, far }
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::Orthographic { left, right, bottom, top, near, far }
    }

    fn orthographic_from_size(width: f32, height: f32, near: f32, far: f32) -> Self {
        let hw = width / 2.0;
        let hh = height / 2.0;
        Self::orthographic(-hw, hw, -hh, hh, near, far)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4f {
        match self {
            Self::Perspective { fov, near: n, far: f } => {
                let squeeze = Mat4f::from([
                    [*n, 0.0, 0.0, 0.0],
                    [0.0, *n, 0.0, 0.0],
                    [0.0, 0.0, n + f, -n * f],
                    [0.0, 0.0, 1.0, 0.0],
                ]);
                let tan = n * (fov / 2.0).tan();
                let width = 2.0 * tan * aspect;
                let height = 2.0 * tan;
                let ortho = Self::orthographic_from_size(width, height, *n, *f);
                ortho.projection_matrix(aspect) * squeeze
            }
            Self::Orthographic { left, right, bottom, top, near, far } => {
                let (l, r, b, t, n, f) = (*left, *right, *bottom, *top, *near, *far);
                let center = Vec3f::new((r + l) / 2.0, (t + b) / 2.0, (n + f) / 2.0);
                let mt = transform::translate(-center);
                let ms = transform::scale(Vec3f::new(2.0 / (r - l), 2.0 / (t - b), 2.0 / (f - n)));
                ms * mt
            }
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self::perspective(consts::FRAC_PI_3, 0.1, 100.0)
    }
}
