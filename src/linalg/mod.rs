#![allow(unused)]

mod matrix;
mod quaternion;
mod rotator;
pub mod transform;
mod vector;

pub use matrix::Matrix;
pub use quaternion::Quaternion;
pub use rotator::Rotator;
pub use vector::Vector;

pub type Vec2<T> = Vector<T, 2>;
pub type Vec3<T> = Vector<T, 3>;
pub type Vec4<T> = Vector<T, 4>;

pub type Vec2f = Vec2<f32>;
pub type Vec3f = Vec3<f32>;
pub type Vec4f = Vec4<f32>;

pub type Mat2<T> = Matrix<T, 2, 2>;
pub type Mat3<T> = Matrix<T, 3, 3>;
pub type Mat4<T> = Matrix<T, 4, 4>;

pub type Mat2f = Mat2<f32>;
pub type Mat3f = Mat3<f32>;
pub type Mat4f = Mat4<f32>;
