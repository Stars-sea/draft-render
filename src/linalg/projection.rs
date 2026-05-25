use crate::linalg::Mat4;
use num_traits::real::Real;

pub fn perspective<T: Real>(fov_y: T, aspect: T, near: T, far: T) -> Mat4<T> {
    let (o, i) = (T::zero(), T::one());
    let two = i + i;
    let f = i / (fov_y / two).tan();
    Mat4::from([
        [f / aspect, o, o, o],
        [o, f, o, o],
        [
            o,
            o,
            -(far + near) / (far - near),
            -two * far * near / (far - near),
        ],
        [o, o, -i, o],
    ])
}

pub fn orthographic<T: Real>(left: T, right: T, bottom: T, top: T, near: T, far: T) -> Mat4<T> {
    let (o, i) = (T::zero(), T::one());
    let two = i + i;
    Mat4::from([
        [two / (right - left), o, o, -(right + left) / (right - left)],
        [o, two / (top - bottom), o, -(top + bottom) / (top - bottom)],
        [o, o, -two / (far - near), -(far + near) / (far - near)],
        [o, o, o, i],
    ])
}
