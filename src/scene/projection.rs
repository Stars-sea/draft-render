use crate::linalg::{Mat4f, Vec3f, transform};
use std::f32::consts;

pub trait Projection {
    fn projection_matrix(&self) -> Mat4f;
}

#[derive(Clone, Debug)]
pub struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Orthographic {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    pub fn from_size(width: f32, height: f32, near: f32, far: f32) -> Self {
        let hw = width / 2.0;
        let hh = height / 2.0;
        Self::new(-hw, hw, -hh, hh, near, far)
    }
}

impl Default for Orthographic {
    fn default() -> Self {
        Self::from_size(2.0, 2.0, 0.1, 100.0)
    }
}

impl Projection for Orthographic {
    fn projection_matrix(&self) -> Mat4f {
        let center = Vec3f::new(
            (self.right + self.left) / 2.0,
            (self.top + self.bottom) / 2.0,
            (self.near + self.far) / 2.0,
        );

        let mt = transform::translate(-center);

        let scale = Vec3f::new(
            2.0 / (self.right - self.left),
            2.0 / (self.top - self.bottom),
            2.0 / (self.far - self.near),
        );

        let ms = transform::scale(scale);

        ms * mt
    }
}

#[derive(Clone, Debug)]
pub struct Perspective {
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Perspective {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            aspect,
            near,
            far,
        }
    }
}

impl Default for Perspective {
    fn default() -> Self {
        Self::new(consts::FRAC_PI_3, 16.0 / 9.0, 0.1, 100.0)
    }
}

impl Projection for Perspective {
    fn projection_matrix(&self) -> Mat4f {
        let (n, f) = (self.near, self.far);

        // Step 1: perspective squeeze (frustum → rectangular box in homogeneous coords)
        // After division by w, x/w and y/w are perspective-projected,
        // and z/w maps [n, f] to [n, f] linearly.
        let squeeze = Mat4f::from([
            [n, 0.0, 0.0, 0.0],
            [0.0, n, 0.0, 0.0],
            [0.0, 0.0, n + f, -n * f],
            [0.0, 0.0, 1.0, 0.0],
        ]);

        // Step 2: orthographic projection of the resulting box to NDC
        // After squeeze + perspective divide, the frustum becomes a box:
        //   x ∈ [-n·tan(fov/2), n·tan(fov/2)]
        //   y ∈ [-n·tan(fov/2), n·tan(fov/2)]
        //   z ∈ [n, f]
        let tan = n * (self.fov / 2.0).tan();
        let width = 2.0 * tan * self.aspect;
        let height = 2.0 * tan;
        let ortho = Orthographic::from_size(width, height, n, f);

        ortho.projection_matrix() * squeeze
    }
}
