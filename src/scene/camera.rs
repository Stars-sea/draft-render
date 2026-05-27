use num_traits::real::Real;
use crate::linalg::{Quaternion, Rotator, transform, Vec3, Mat4};
use crate::scene::Projection;
use num_traits::Zero;

pub struct Camera<T: Real> {
    pub position: Vec3<T>,
    pub rotator: Rotator<T>,
    pub proj: Projection<T>,
}

impl<T: Real> Camera<T> {
    pub fn new(position: Vec3<T>, rotator: Rotator<T>, proj: Projection<T>) -> Self {
        Self {
            position,
            rotator,
            proj,
        }
    }

    pub fn view_matrix(&self) -> Mat4<T> {
        let rotation: Quaternion<T> = self.rotator.into();
        let forward = rotation.rotate_vector(Vec3::unit_z());
        let up = rotation.rotate_vector(Vec3::unit_y());
        let right = up.cross(&forward).normalize();
        let corrected_up = forward.cross(&right);

        let (o, i) = (T::zero(), T::one());
        let r = Mat4::from([
            [right.x(), corrected_up.x(), forward.x(), o],
            [right.y(), corrected_up.y(), forward.y(), o],
            [right.z(), corrected_up.z(), forward.z(), o],
            [o, o, o, i],
        ]);

        let t = transform::translate(-self.position);

        r.inverse().unwrap() * t
    }

    pub fn vp_matrix(&self, aspect: T) -> Mat4<T> {
        self.proj.projection_matrix(aspect) * self.view_matrix()
    }
}

impl<T: Real> Default for Camera<T> {
    fn default() -> Self {
        Self::new(Vec3::zero(), Rotator::identity(), Projection::default())
    }
}
