use crate::linalg::{Mat3, Vec3};
use num_traits::real::Real;
use num_traits::{One, Zero};
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion<T: Real> {
    pub w: T,
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Real> Quaternion<T> {
    pub fn new(w: T, x: T, y: T, z: T) -> Self {
        Self { w, x, y, z }
    }

    pub fn identity() -> Self {
        Self {
            w: T::one(),
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    pub fn from_axis_angle(axis: Vec3<T>, angle: T) -> Self {
        let half = angle / (T::one() + T::one());
        let (s, c) = half.sin_cos();
        let axis = axis.normalize();
        Self {
            w: c,
            x: axis[0] * s,
            y: axis[1] * s,
            z: axis[2] * s,
        }
    }

    pub fn norm(&self) -> T {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n.is_zero() {
            return Self::identity();
        }
        Self {
            w: self.w / n,
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
        }
    }

    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn inverse(&self) -> Self {
        let n2 = self.norm();
        let n2 = n2 * n2;
        self.conjugate() * (T::one() / n2)
    }

    pub fn rotate_vector(&self, v: Vec3<T>) -> Vec3<T> {
        let qv = Quaternion {
            w: T::zero(),
            x: v[0],
            y: v[1],
            z: v[2],
        };
        let r = *self * qv * self.conjugate();
        Vec3::new(r.x, r.y, r.z)
    }
}

impl<T: Real> Add for Quaternion<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w + rhs.w,
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Real> Zero for Quaternion<T> {
    fn zero() -> Self {
        Self {
            w: T::zero(),
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.w.is_zero() && self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

impl<T: Real> One for Quaternion<T> {
    fn one() -> Self {
        Self {
            w: T::one(),
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }
}

impl<T: Real> Into<Mat3<T>> for Quaternion<T> {
    fn into(self) -> Mat3<T> {
        let Quaternion { w, x, y, z } = self;
        let i = T::one();
        let two = i + i;
        Mat3::from([
            [
                i - two * (y * y + z * z),
                two * (x * y - w * z),
                two * (x * z + w * y),
            ],
            [
                two * (x * y + w * z),
                i - two * (x * x + z * z),
                two * (y * z - w * x),
            ],
            [
                two * (x * z - w * y),
                two * (y * z + w * x),
                i - two * (x * x + y * y),
            ],
        ])
    }
}

// Hamilton product
impl<T: Real> Mul for Quaternion<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Quaternion {
            w: a1,
            x: b1,
            y: c1,
            z: d1,
        } = self;
        let Quaternion {
            w: a2,
            x: b2,
            y: c2,
            z: d2,
        } = rhs;
        Self {
            w: a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            x: a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
            y: a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
            z: a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
        }
    }
}

// Scalar multiplication
impl<T: Real> Mul<T> for Quaternion<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            w: self.w * rhs,
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
