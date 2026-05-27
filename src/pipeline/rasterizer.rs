use crate::color::Color;
use crate::linalg::{Mat4, Vec4};
use crate::pipeline::buffer::RenderBuffer;
use crate::pipeline::fragment::Fragment;

use num_traits::ConstZero;
use num_traits::real::Real;
use std::cmp::min;

// ═══════════════════════════════════════════════════════════════════════════
// helpers
// ═══════════════════════════════════════════════════════════════════════════

#[inline]
fn is_inside<T: Real>(u: T, v: T, tl_ca: bool, tl_cb: bool, tl_ab: bool, eps: T) -> bool {
    let z = T::zero();
    let o = T::one();
    (u > z || (u > -eps && tl_ca))
        && (v > z || (v > -eps && tl_cb))
        && (u + v < o || (u + v < o + eps && tl_ab))
}

fn is_top_left<T: Real>(sx: T, sy: T, ex: T, ey: T, eps: T) -> bool {
    let dy = ey - sy;
    if dy.abs() < eps {
        sx > ex
    } else {
        dy < T::zero()
    }
}

fn min_max<T: Real>(a: T, b: T, c: T) -> (usize, usize) {
    (
        a.min(b).min(c).floor().to_usize().unwrap(),
        a.max(b).max(c).ceil().to_usize().unwrap(),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Rasterizer<T, N>
// ═══════════════════════════════════════════════════════════════════════════

pub struct Rasterizer<T: Real, const N: usize> {
    pub samples: [(T, T); N],
}

impl<T: Real + ConstZero> Rasterizer<T, 1> {
    #[allow(unused)]
    pub const NON_MSAA: Self = Self {
        samples: [(T::ZERO, T::ZERO)],
    };
}

macro_rules! impl_msaa_4x {
    ($T:ty) => {
        #[allow(unused)]
        impl Rasterizer<$T, 4> {
            pub const MSAA_4X: Self = Self {
                samples: [
                    (-0.125, -0.375),
                    (0.375, -0.125),
                    (-0.375, 0.125),
                    (0.125, 0.375),
                ],
            };
        }
    };
}

impl_msaa_4x!(f32);
impl_msaa_4x!(f64);

impl<T: Real, const N: usize> Rasterizer<T, N> {
    #[allow(unused)]
    pub const fn new(samples: [(T, T); N]) -> Self {
        Self { samples }
    }

    pub fn rasterize(
        &self,
        buf: &mut RenderBuffer<Fragment<T, N>>,
        a: Vec4<T>,
        b: Vec4<T>,
        c: Vec4<T>,
        color: Color,
    ) {
        let (o, i, two) = (T::zero(), T::one(), T::from(2.0).unwrap());
        let (w, h) = (
            T::from(buf.width()).unwrap(),
            T::from(buf.height()).unwrap(),
        );
        let vp = Mat4::from([
            [w / two, o, o, w / two],
            [o, -h / two, o, h / two],
            [o, o, i, o],
            [o, o, o, i],
        ]);

        let sa = (vp * a).perspective_divide().unwrap();
        let sb = (vp * b).perspective_divide().unwrap();
        let sc = (vp * c).perspective_divide().unwrap();
        let (ea, eb) = (sa - sc, sb - sc);

        let inv = i / (ea.x() * eb.y() - ea.y() * eb.x());
        let dux = eb.y() * inv;
        let duy = -eb.x() * inv;
        let dvx = -ea.y() * inv;
        let dvy = ea.x() * inv;

        let edge_eps = T::from(1e-5).unwrap();
        let top_left_eps = T::epsilon();

        let tl_ca = is_top_left(sc.x(), sc.y(), sa.x(), sa.y(), top_left_eps);
        let tl_cb = is_top_left(sc.x(), sc.y(), sb.x(), sb.y(), top_left_eps);
        let tl_ab = is_top_left(sa.x(), sa.y(), sb.x(), sb.y(), top_left_eps);

        let (min_x, max_x) = min_max(sa.x(), sb.x(), sc.x());
        let (min_y, max_y) = min_max(sa.y(), sb.y(), sc.y());

        let half = T::from(0.5).unwrap();
        let px0 = T::from(min_x).unwrap() + half - sc.x();
        let py0 = T::from(min_y).unwrap() + half - sc.y();
        let mut u0 = (eb.y() * px0 - eb.x() * py0) * inv;
        let mut v0 = (ea.x() * py0 - ea.y() * px0) * inv;

        for y in min_y..=min(max_y, buf.height() - 1) {
            let (mut u, mut v) = (u0, v0);
            for x in min_x..=min(max_x, buf.width() - 1) {
                let frag = buf.get_mut(x, y);
                for i in 0..N {
                    let (sx, sy) = self.samples[i];
                    let us = u + sx * dux + sy * duy;
                    let vs = v + sx * dvx + sy * dvy;
                    if is_inside(us, vs, tl_ca, tl_cb, tl_ab, edge_eps) {
                        let z = sc.z() + us * ea.z() + vs * eb.z();
                        if z < frag.depth_buf[i] {
                            frag.depth_buf[i] = z;
                            frag.color_buf[i] = color;
                        }
                    }
                }
                u = u + dux;
                v = v + dvx;
            }
            u0 = u0 + duy;
            v0 = v0 + dvy;
        }
    }

    /// 三角形迭代 —— 遍历索引并光栅化每个三角形。
    pub fn draw_mesh(
        &self,
        buf: &mut RenderBuffer<Fragment<T, N>>,
        vertices: &[Vec4<T>],
        indices: &[[usize; 3]],
        color: Color,
    ) {
        for &[i0, i1, i2] in indices {
            self.rasterize(buf, vertices[i0], vertices[i1], vertices[i2], color);
        }
    }

    pub fn resolve(&self, src: &RenderBuffer<Fragment<T, N>>, fb: &mut RenderBuffer<Color>) {
        for y in 0..src.height() {
            for x in 0..src.width() {
                fb[(x, y)] = Color::average(&src[(x, y)].color_buf);
            }
        }
    }
}
