use crate::linalg::{Mat4f, Quaternion, Rotator, Vec3f, transform};
use crate::scene::Projection;
use num_traits::{ConstZero, Zero};

pub struct Camera {
    pub position: Vec3f,
    pub rotator: Rotator<f32>,
    pub proj: Projection,
}

impl Camera {
    pub fn new(position: Vec3f, rotator: Rotator<f32>, proj: Projection) -> Self {
        Self {
            position,
            rotator,
            proj,
        }
    }

    pub fn view_matrix(&self) -> Mat4f {
        let rotation: Quaternion<f32> = self.rotator.into();
        let forward = rotation.rotate_vector(Vec3f::unit_z());
        let up = rotation.rotate_vector(Vec3f::unit_y());
        let right = up.cross(&forward).normalize();
        let corrected_up = forward.cross(&right);

        let r = Mat4f::from([
            [right.x(), corrected_up.x(), forward.x(), 0.0],
            [right.y(), corrected_up.y(), forward.y(), 0.0],
            [right.z(), corrected_up.z(), forward.z(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let t = transform::translate(-self.position);
        r.inverse().unwrap() * t
    }

    pub fn vp_matrix(&self, aspect: f32) -> Mat4f {
        self.proj.projection_matrix(aspect) * self.view_matrix()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3f::ZERO, Rotator::identity(), Projection::default())
    }
}
