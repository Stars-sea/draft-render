mod vector;
mod matrix;

pub use vector::Vector;
pub use matrix::Matrix;

pub type Vec2<T> = Vector<T, 2>;
pub type Vec3<T> = Vector<T, 3>;
pub type Vec4<T> = Vector<T, 4>;

pub type Vec2i = Vector<i32, 2>;
pub type Vec3i = Vector<i32, 3>;
pub type Vec4i = Vector<i32, 4>;

pub type Vec2f = Vector<f32, 2>;
pub type Vec3f = Vector<f32, 3>;
pub type Vec4f = Vector<f32, 4>;

pub type Vec2d = Vector<f64, 2>;
pub type Vec3d = Vector<f64, 3>;
pub type Vec4d = Vector<f64, 4>;

pub type Mat2<T> = Matrix<T, 2, 2>;
pub type Mat3<T> = Matrix<T, 3, 3>;
pub type Mat4<T> = Matrix<T, 4, 4>;

pub type Mat2i = Matrix<i32, 2, 2>;
pub type Mat3i = Matrix<i32, 3, 3>;
pub type Mat4i = Matrix<i32, 4, 4>;

pub type Mat2f = Matrix<f32, 2, 2>;
pub type Mat3f = Matrix<f32, 3, 3>;
pub type Mat4f = Matrix<f32, 4, 4>;

pub type Mat2d = Matrix<f64, 2, 2>;
pub type Mat3d = Matrix<f64, 3, 3>;
pub type Mat4d = Matrix<f64, 4, 4>;
