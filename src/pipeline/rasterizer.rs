use crate::color::Color;
use crate::linalg::{Mat4, Vec4};
use crate::pipeline::buffer::RenderBuffer;

use num_traits::ConstZero;
use num_traits::real::Real;
use std::cmp::min;

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

pub struct Rasterizer<T: Real, const N: usize> {
    pub samples: [(T, T); N],
}

impl<T: Real + ConstZero> Rasterizer<T, 1> {
    pub const NON_MSAA: Self = Self {
        samples: [(T::ZERO, T::ZERO)],
    };
}

macro_rules! impl_msaa_4x {
    ($T:ty) => {
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
    pub const fn new(samples: [(T, T); N]) -> Self {
        Self { samples }
    }

    pub fn rasterize(
        &self,
        cb: &mut RenderBuffer<Color, N>,
        zb: &mut RenderBuffer<T, N>,
        a: Vec4<T>,
        b: Vec4<T>,
        c: Vec4<T>,
        color: Color,
    ) {
        let (o, i, two) = (T::zero(), T::one(), T::from(2.0).unwrap());
        let (w, h) = (T::from(cb.width()).unwrap(), T::from(cb.height()).unwrap());
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

        for y in min_y..=min(max_y, cb.height() - 1) {
            let (mut u, mut v) = (u0, v0);
            for x in min_x..=min(max_x, cb.width() - 1) {
                let base = zb.idx(x, y, 0);
                for i in 0..N {
                    let (sx, sy) = self.samples[i];
                    let us = u + sx * dux + sy * duy;
                    let vs = v + sx * dvx + sy * dvy;
                    if is_inside(us, vs, tl_ca, tl_cb, tl_ab, edge_eps) {
                        let z = sc.z() + us * ea.z() + vs * eb.z();
                        let idx = base + i;
                        if z < *zb.get(idx) {
                            *zb.get_mut(idx) = z;
                            *cb.get_mut(idx) = color;
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

    pub fn resolve(&self, src: &RenderBuffer<Color, N>, fb: &mut RenderBuffer<Color>) {
        for y in 0..src.height() {
            for x in 0..src.width() {
                let base = src.idx(x, y, 0);
                fb[(x, y)] = Color::average(&src.as_slice()[base..base + N]);
            }
        }
    }
}
