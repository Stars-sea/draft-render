use crate::linalg::{Mat4, Vec3};
use num_traits::real::Real;
use num_traits::{One, Zero};

pub fn translate<T: Real>(v: Vec3<T>) -> Mat4<T> {
    let mut m = Mat4::one();
    m[(0, 3)] = v[0];
    m[(1, 3)] = v[1];
    m[(2, 3)] = v[2];
    m
}

pub fn scale<T: Real>(v: Vec3<T>) -> Mat4<T> {
    let mut m = Mat4::zero();
    m[(0, 0)] = v[0];
    m[(1, 1)] = v[1];
    m[(2, 2)] = v[2];
    m[(3, 3)] = T::one();
    m
}
