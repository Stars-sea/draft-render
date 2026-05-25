use num_traits::Zero;
use num_traits::real::Real;
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vector<T: Real, const N: usize>([T; N]);

impl<T: Real, const N: usize> Vector<T, N> {
    pub fn dot(&self, other: &Self) -> T {
        let mut sum = T::zero();
        for i in 0..N {
            sum = sum + self[i] * other[i];
        }
        sum
    }

    pub fn norm(&self) -> T {
        self.dot(self).sqrt()
    }

    pub fn normalize(&self) -> Self {
        if self.is_zero() {
            return Self::zero();
        }
        let norm = self.norm();
        let mut result = *self;
        for i in 0..N {
            result[i] = result[i] / norm;
        }
        result
    }
}

// Constructors

impl<T: Real, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(arr: [T; N]) -> Self {
        Self(arr)
    }
}

impl<T: Real, const N: usize> From<Vector<T, N>> for [T; N] {
    fn from(v: Vector<T, N>) -> Self {
        v.0
    }
}

impl<T: Real, const N: usize> Zero for Vector<T, N> {
    fn zero() -> Self {
        Self([T::zero(); N])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|x| x.is_zero())
    }
}

// Accessors

impl<T: Real, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Real, const N: usize> IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

// Dot product via *
impl<T: Real, const N: usize> Mul for Vector<T, N> {
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        self.dot(&rhs)
    }
}

// Scalar multiplication
impl<T: Real, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut result = self;
        for i in 0..N {
            result[i] = result[i] * rhs;
        }
        result
    }
}

// Element-wise operations
impl<T: Real, const N: usize> Add for Vector<T, N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        for i in 0..N {
            result[i] = result[i] + rhs[i];
        }
        result
    }
}

impl<T: Real, const N: usize> Sub for Vector<T, N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        for i in 0..N {
            result[i] = result[i] - rhs[i];
        }
        result
    }
}

impl<T: Real, const N: usize> Neg for Vector<T, N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut result = self;
        for i in 0..N {
            result[i] = -result[i];
        }
        result
    }
}

// Named constructors and accessors

impl<T: Real> Vector<T, 2> {
    pub fn new(x: T, y: T) -> Self {
        Self([x, y])
    }

    pub fn x(&self) -> T {
        self[0]
    }
    pub fn y(&self) -> T {
        self[1]
    }
}

impl<T: Real> Vector<T, 3> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self([x, y, z])
    }

    pub fn x(&self) -> T {
        self[0]
    }
    pub fn y(&self) -> T {
        self[1]
    }
    pub fn z(&self) -> T {
        self[2]
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self([
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        ])
    }
}

impl<T: Real> Vector<T, 4> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self([x, y, z, w])
    }

    pub fn x(&self) -> T {
        self[0]
    }
    pub fn y(&self) -> T {
        self[1]
    }
    pub fn z(&self) -> T {
        self[2]
    }
    pub fn w(&self) -> T {
        self[3]
    }

    pub fn perspective_divide(&self) -> Option<Vector<T, 3>> {
        let w = self.w();
        if w.is_zero() {
            return None;
        }
        Some(Vector([self.x() / w, self.y() / w, self.z() / w]))
    }
}

// Unit vectors

macro_rules! impl_unit_vectors {
    ($T:ty, $O:expr, $I:expr) => {
        impl Vector<$T, 2> {
            pub const UNIT_X: Self = Self([$I, $O]);
            pub const UNIT_Y: Self = Self([$O, $I]);
        }
        impl Vector<$T, 3> {
            pub const UNIT_X: Self = Self([$I, $O, $O]);
            pub const UNIT_Y: Self = Self([$O, $I, $O]);
            pub const UNIT_Z: Self = Self([$O, $O, $I]);
        }
        impl Vector<$T, 4> {
            pub const UNIT_X: Self = Self([$I, $O, $O, $O]);
            pub const UNIT_Y: Self = Self([$O, $I, $O, $O]);
            pub const UNIT_Z: Self = Self([$O, $O, $I, $O]);
            pub const UNIT_W: Self = Self([$O, $O, $O, $I]);
        }
    };
}

impl_unit_vectors!(f32, 0.0, 1.0);
impl_unit_vectors!(f64, 0.0, 1.0);

impl<T: Real> Vector<T, 2> {
    pub fn unit_x() -> Self {
        Self([T::one(), T::zero()])
    }
    pub fn unit_y() -> Self {
        Self([T::zero(), T::one()])
    }
}

impl<T: Real> Vector<T, 3> {
    pub fn unit_x() -> Self {
        Self([T::one(), T::zero(), T::zero()])
    }
    pub fn unit_y() -> Self {
        Self([T::zero(), T::one(), T::zero()])
    }
    pub fn unit_z() -> Self {
        Self([T::zero(), T::zero(), T::one()])
    }
}

impl<T: Real> Vector<T, 4> {
    pub fn unit_x() -> Self {
        Self([T::one(), T::zero(), T::zero(), T::zero()])
    }
    pub fn unit_y() -> Self {
        Self([T::zero(), T::one(), T::zero(), T::zero()])
    }
    pub fn unit_z() -> Self {
        Self([T::zero(), T::zero(), T::one(), T::zero()])
    }
    pub fn unit_w() -> Self {
        Self([T::zero(), T::zero(), T::zero(), T::one()])
    }
}
