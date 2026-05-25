use super::vector::Vector;
use num_traits::real::Real;
use num_traits::{One, Zero};
use std::ops::{Add, Index, IndexMut, Mul, Sub};

#[derive(Debug, Clone)]
pub struct Matrix<T: Real, const R: usize, const C: usize>([[T; C]; R]);

// Constructors

impl<T: Real, const R: usize, const C: usize> Zero for Matrix<T, R, C> {
    fn zero() -> Self {
        Self([[T::zero(); C]; R])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|row| row.iter().all(|x| x.is_zero()))
    }
}

impl<T: Real, const R: usize, const C: usize> From<[[T; C]; R]> for Matrix<T, R, C> {
    fn from(rows: [[T; C]; R]) -> Self {
        Self(rows)
    }
}

impl<T: Real, const R: usize> From<Vector<T, R>> for Matrix<T, R, 1> {
    fn from(vec: Vector<T, R>) -> Self {
        let mut result = Self::zero();
        for i in 0..R {
            result[i][0] = vec[i];
        }
        result
    }
}

// Accessors

impl<T: Real, const R: usize, const C: usize> Index<usize> for Matrix<T, R, C> {
    type Output = [T; C];

    fn index(&self, row: usize) -> &Self::Output {
        &self.0[row]
    }
}

impl<T: Real, const R: usize, const C: usize> Index<(usize, usize)> for Matrix<T, R, C> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.0[row][col]
    }
}

impl<T: Real, const R: usize, const C: usize> IndexMut<usize> for Matrix<T, R, C> {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        &mut self.0[row]
    }
}

impl<T: Real, const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<T, R, C> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.0[row][col]
    }
}

// General operations (any R, C)

impl<T: Real, const R: usize, const C: usize> Matrix<T, R, C> {
    pub fn transpose(&self) -> Matrix<T, C, R> {
        let mut result = Matrix::<T, C, R>::zero();
        for i in 0..R {
            for j in 0..C {
                result[j][i] = self[i][j];
            }
        }
        result
    }
}

// Square matrix operations (R == C)

impl<T: Real, const N: usize> One for Matrix<T, N, N> {
    fn one() -> Self {
        let mut m = Self::zero();
        for i in 0..N {
            m[i][i] = T::one();
        }
        m
    }
}

impl<T: Real, const N: usize> Matrix<T, N, N> {
    pub fn det(&self) -> T {
        let mut a = self.clone();
        let mut det = T::one();

        for i in 0..N {
            if a[i][i].is_zero() {
                match (i + 1..N).find(|&r| !a[r][i].is_zero()) {
                    Some(r) => {
                        a.0.swap(i, r);
                        det = -det;
                    }
                    None => return T::zero(),
                }
            }
            det = det * a[i][i];
            for r in (i + 1)..N {
                let factor = a[r][i] / a[i][i];
                for c in i..N {
                    a[r][c] = a[r][c] - factor * a[i][c];
                }
            }
        }
        det
    }

    pub fn inverse(&self) -> Option<Self> {
        let mut a = self.clone();
        let mut b = Self::one();

        for i in 0..N {
            match (i..N).find(|&r| !a[r][i].is_zero()) {
                Some(r) => {
                    if r != i {
                        a.0.swap(i, r);
                        b.0.swap(i, r);
                    }
                }
                None => return None,
            }

            let pivot = a[i][i];
            for j in 0..N {
                a[i][j] = a[i][j] / pivot;
                b[i][j] = b[i][j] / pivot;
            }

            for r in 0..N {
                if r != i {
                    let factor = a[r][i];
                    for c in 0..N {
                        a[r][c] = a[r][c] - factor * a[i][c];
                        b[r][c] = b[r][c] - factor * b[i][c];
                    }
                }
            }
        }
        Some(b)
    }
}

// Element-wise addition (same dimensions)

impl<T: Real, const R: usize, const C: usize> Add for Matrix<T, R, C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        for i in 0..R {
            for j in 0..C {
                result[i][j] = self[i][j] + rhs[i][j];
            }
        }
        result
    }
}

impl<T: Real, const R: usize, const C: usize> Sub for Matrix<T, R, C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        for i in 0..R {
            for j in 0..C {
                result[i][j] = self[i][j] - rhs[i][j];
            }
        }
        result
    }
}

// Matrix multiplication: R x K * K x C = R x C

impl<T: Real, const R: usize, const K: usize, const C: usize> Mul<Matrix<T, K, C>>
    for Matrix<T, R, K>
{
    type Output = Matrix<T, R, C>;

    fn mul(self, rhs: Matrix<T, K, C>) -> Self::Output {
        let mut result = Matrix::<T, R, C>::zero();
        for i in 0..R {
            for j in 0..C {
                let mut sum = T::zero();
                for k in 0..K {
                    sum = sum + self[i][k] * rhs[k][j];
                }
                result[i][j] = sum;
            }
        }
        result
    }
}

// Scalar multiplication

impl<T: Real, const R: usize, const C: usize> Mul<T> for Matrix<T, R, C> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut result = Self::zero();
        for i in 0..R {
            for j in 0..C {
                result[i][j] = self[i][j] * rhs;
            }
        }
        result
    }
}

// Generic matrix-vector multiplication: R x C * Vector<C> = Vector<R>

impl<T: Real, const R: usize, const C: usize> Mul<Vector<T, C>> for Matrix<T, R, C> {
    type Output = Vector<T, R>;

    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        let mut result = Vector::zero();
        for i in 0..R {
            let mut sum = T::zero();
            for j in 0..C {
                sum = sum + self[i][j] * rhs[j];
            }
            result[i] = sum;
        }
        result
    }
}
