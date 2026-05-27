use crate::color::Color;
use num_traits::real::Real;

/// 像素级 N 采样数据，AoS 布局对 CPU 缓存友好。
#[derive(Clone)]
pub struct Fragment<T: Real, const N: usize> {
    pub color_buf: [Color; N],
    pub depth_buf: [T; N],
}

impl<T: Real, const N: usize> Fragment<T, N> {
    pub fn new() -> Self {
        Self {
            color_buf: [Color::BLACK; N],
            depth_buf: [T::max_value(); N],
        }
    }
}
