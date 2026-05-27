use crate::color::Color;

/// 像素级 N 采样数据，AoS 布局对 CPU 缓存友好。
#[derive(Clone)]
pub struct Fragment<const N: usize> {
    pub color_buf: [Color; N],
    pub depth_buf: [f32; N],
}

impl<const N: usize> Fragment<N> {
    pub const fn new() -> Self {
        Self {
            color_buf: [Color::BLACK; N],
            depth_buf: [f32::INFINITY; N],
        }
    }
}
