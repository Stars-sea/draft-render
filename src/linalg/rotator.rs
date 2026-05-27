use crate::linalg::{Mat3, Mat4, Quaternion, Vec3};
use num_traits::real::Real;
use num_traits::{ConstOne, ConstZero, FloatConst};

/// Rotation stored as Euler angles (yaw: Y, pitch: X, roll: Z)
#[derive(Debug, Clone, Copy)]
pub struct Rotator<T: Real> {
    pub yaw: T,
    pub pitch: T,
    pub roll: T,
}

impl<T: Real> Rotator<T> {
    pub fn new(yaw: T, pitch: T, roll: T) -> Self {
        Self { yaw, pitch, roll }
    }

    pub fn identity() -> Self {
        Self {
            yaw: T::zero(),
            pitch: T::zero(),
            roll: T::zero(),
        }
    }

    pub fn from_axis_angle(axis: Vec3<T>, angle: T) -> Self {
        Self::from(Quaternion::from_axis_angle(axis, angle))
    }

    pub fn from_rotation_x(angle: T) -> Self {
        Self {
            yaw: T::zero(),
            pitch: angle,
            roll: T::zero(),
        }
    }

    pub fn from_rotation_y(angle: T) -> Self {
        Self {
            yaw: angle,
            pitch: T::zero(),
            roll: T::zero(),
        }
    }

    pub fn from_rotation_z(angle: T) -> Self {
        Self {
            yaw: T::zero(),
            pitch: T::zero(),
            roll: angle,
        }
    }

    pub fn with_yaw(mut self, yaw: T) -> Self {
        self.yaw = yaw;
        self
    }

    pub fn with_pitch(mut self, pitch: T) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn with_roll(mut self, roll: T) -> Self {
        self.roll = roll;
        self
    }
}

impl<T: Real> From<Quaternion<T>> for Rotator<T> {
    fn from(q: Quaternion<T>) -> Self {
        let m: Mat3<T> = q.into();
        let pitch = -m[(1, 2)].asin();
        let cos_pitch = pitch.cos();

        if cos_pitch.abs() > T::epsilon() {
            let yaw = m[(0, 2)].atan2(m[(2, 2)]);
            let roll = m[(1, 0)].atan2(m[(1, 1)]);
            Self { yaw, pitch, roll }
        } else {
            let yaw = (-m[(2, 0)]).atan2(m[(0, 0)]);
            Self {
                yaw,
                pitch,
                roll: T::zero(),
            }
        }
    }
}

impl<T: Real + ConstZero + ConstOne> Into<Quaternion<T>> for Rotator<T> {
    fn into(self) -> Quaternion<T> {
        let qy = Quaternion::from_axis_angle(Vec3::unit_y(), self.yaw);
        let qp = Quaternion::from_axis_angle(Vec3::unit_x(), self.pitch);
        let qr = Quaternion::from_axis_angle(Vec3::unit_z(), self.roll);
        qy * qp * qr
    }
}

impl<T: Real + ConstZero + ConstOne> Into<Mat4<T>> for Rotator<T> {
    fn into(self) -> Mat4<T> {
        let (sy, cy) = self.yaw.sin_cos();
        let (sx, cx) = self.pitch.sin_cos();
        let (sz, cz) = self.roll.sin_cos();
        let (o, i) = (T::zero(), T::one());

        Mat4::from([
            [cy * cz + sy * sx * sz, -cy * sz + sy * sx * cz, sy * cx, o],
            [cx * sz, cx * cz, -sx, o],
            [-sy * cz + cy * sx * sz, sy * sz + cy * sx * cz, cy * cx, o],
            [o, o, o, i],
        ])
    }
}
