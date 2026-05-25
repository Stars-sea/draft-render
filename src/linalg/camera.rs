use crate::linalg::{Mat4, Vec3};
use num_traits::real::Real;

pub fn look_at<T: Real>(eye: Vec3<T>, center: Vec3<T>, up: Vec3<T>) -> Mat4<T> {
    let (o, i) = (T::zero(), T::one());
    let f = (center - eye).normalize();
    let s = f.cross(&up).normalize();
    let u = s.cross(&f);

    Mat4::from([
        [s[0], s[1], s[2], -(s * eye)],
        [u[0], u[1], u[2], -(u * eye)],
        [-f[0], -f[1], -f[2], f * eye],
        [o, o, o, i],
    ])
}
