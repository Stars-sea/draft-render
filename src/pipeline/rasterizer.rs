use crate::buffer::RenderBuffer;
use crate::color::Color;
use crate::linalg::{Mat4f, Vec3f, Vec4f};

use std::cmp::min;

#[inline]
fn is_inside(u: f32, v: f32, tl_ca: bool, tl_cb: bool, tl_ab: bool) -> bool {
    const EPS: f32 = 1e-5;
    (u > 0.0 || (u > -EPS && tl_ca))
        && (v > 0.0 || (v > -EPS && tl_cb))
        && (u + v < 1.0 || (u + v < 1.0 + EPS && tl_ab))
}

fn is_top_left(start: Vec3f, end: Vec3f) -> bool {
    let dy = end.y() - start.y();
    if dy.abs() < f32::EPSILON {
        start.x() > end.x()
    } else {
        dy < 0.0
    }
}

fn min_max(a: f32, b: f32, c: f32) -> (usize, usize) {
    (
        a.min(b).min(c).floor() as usize,
        a.max(b).max(c).ceil() as usize,
    )
}

pub struct Rasterizer<const N: usize> {
    pub samples: [(f32, f32); N],
}

impl Rasterizer<1> {
    #[allow(unused)]
    pub const NON_MSAA: Self = Self {
        samples: [(0.0, 0.0)],
    };
}

impl Rasterizer<4> {
    #[allow(unused)]
    pub const MSAA_4X: Self = Self {
        samples: [
            (-0.125, -0.375),
            (0.375, -0.125),
            (-0.375, 0.125),
            (0.125, 0.375),
        ],
    };
}

impl<const N: usize> Rasterizer<N> {
    #[allow(unused)]
    pub const fn new(samples: [(f32, f32); N]) -> Self {
        Self { samples }
    }

    pub fn rasterize(
        &self,
        cb: &mut RenderBuffer<Color, N>,
        zb: &mut RenderBuffer<f32, N>,
        a: Vec3f,
        b: Vec3f,
        c: Vec3f,
        color: Color,
    ) {
        let (w, h) = (cb.width() as f32, cb.height() as f32);
        let vp = Mat4f::from([
            [w / 2.0, 0.0, 0.0, w / 2.0],
            [0.0, -h / 2.0, 0.0, h / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let sa = (vp * Vec4f::from_vec3(a, 1.0)).xyz();
        let sb = (vp * Vec4f::from_vec3(b, 1.0)).xyz();
        let sc = (vp * Vec4f::from_vec3(c, 1.0)).xyz();
        let (ea, eb) = (sa - sc, sb - sc);

        let inv = 1.0 / (ea.x() * eb.y() - ea.y() * eb.x());
        let dux = eb.y() * inv;
        let duy = -eb.x() * inv;
        let dvx = -ea.y() * inv;
        let dvy = ea.x() * inv;

        let tl_ca = is_top_left(sc, sa);
        let tl_cb = is_top_left(sc, sb);
        let tl_ab = is_top_left(sa, sb);

        let (min_x, max_x) = min_max(sa.x(), sb.x(), sc.x());
        let (min_y, max_y) = min_max(sa.y(), sb.y(), sc.y());

        let px0 = min_x as f32 + 0.5 - sc.x();
        let py0 = min_y as f32 + 0.5 - sc.y();
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
                    if is_inside(us, vs, tl_ca, tl_cb, tl_ab) {
                        let z = sc.z() + us * ea.z() + vs * eb.z();
                        let idx = base + i;
                        if z < *zb.get(idx) {
                            *zb.get_mut(idx) = z;
                            *cb.get_mut(idx) = color;
                        }
                    }
                }
                u += dux;
                v += dvx;
            }
            u0 += duy;
            v0 += dvy;
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
