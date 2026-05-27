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

pub type Mat2<T> = Matrix<T, 2, 2>;
pub type Mat3<T> = Matrix<T, 3, 3>;
pub type Mat4<T> = Matrix<T, 4, 4>;
