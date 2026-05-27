use crate::color::Color;
use crate::linalg::{transform, Vec2f, Vec3f, Vec4f};
use crate::pipeline::buffer::RenderBuffer;
use crate::pipeline::fragment::Fragment;
use crate::pipeline::shader::Shader;

use num_traits::ConstZero;
use std::cmp::min;
use std::ops::RangeInclusive;

// ═══════════════════════════════════════════════════════════════════════════
// triangle setup
// ═══════════════════════════════════════════════════════════════════════════

struct Bounds {
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
}

struct TopLeft {
    ca: bool,
    cb: bool,
    ab: bool,
}

struct TriSetup {
    bounds: Bounds,
    top_left: TopLeft,
    u_grad: Vec2f,
    v_grad: Vec2f,
    uv0: Vec2f,
    z_c: f32,
    dz_du: f32,
    dz_dv: f32,
}

impl Bounds {
    fn new(a: Vec3f, b: Vec3f, c: Vec3f) -> Self {
        let (min_x, max_x) = min_max(a.x(), b.x(), c.x());
        let (min_y, max_y) = min_max(a.y(), b.y(), c.y());
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    fn range_y(&self, max: usize) -> RangeInclusive<usize> {
        self.min_y..=min(self.max_y, max)
    }
    fn range_x(&self, max: usize) -> RangeInclusive<usize> {
        self.min_x..=min(self.max_x, max)
    }
}

impl TopLeft {
    fn new(a: Vec3f, b: Vec3f, c: Vec3f) -> Self {
        Self {
            ca: is_top_left(c.x(), c.y(), a.x(), a.y()),
            cb: is_top_left(c.x(), c.y(), b.x(), b.y()),
            ab: is_top_left(a.x(), a.y(), b.x(), b.y()),
        }
    }
}

fn min_max(a: f32, b: f32, c: f32) -> (usize, usize) {
    (
        a.min(b).min(c).floor() as usize,
        a.max(b).max(c).ceil() as usize,
    )
}

fn is_top_left(sx: f32, sy: f32, ex: f32, ey: f32) -> bool {
    let dy = ey - sy;
    if dy.abs() < f32::EPSILON {
        sx > ex
    } else {
        dy < 0.0
    }
}

impl TriSetup {
    fn new(sa: Vec3f, sb: Vec3f, sc: Vec3f) -> Self {
        let bounds = Bounds::new(sa, sb, sc);
        let top_left = TopLeft::new(sa, sb, sc);
        let (ea, eb) = (sa - sc, sb - sc);
        let inv = 1.0 / (ea.x() * eb.y() - ea.y() * eb.x());
        let dux = eb.y() * inv;
        let duy = -eb.x() * inv;
        let dvx = -ea.y() * inv;
        let dvy = ea.x() * inv;

        let px0 = bounds.min_x as f32 + 0.5 - sc.x();
        let py0 = bounds.min_y as f32 + 0.5 - sc.y();
        let u0 = (eb.y() * px0 - eb.x() * py0) * inv;
        let v0 = (ea.x() * py0 - ea.y() * px0) * inv;

        Self {
            bounds,
            top_left,
            u_grad: Vec2f::new(dux, duy),
            v_grad: Vec2f::new(dvx, dvy),
            uv0: Vec2f::new(u0, v0),
            z_c: sc.z(),
            dz_du: ea.z(),
            dz_dv: eb.z(),
        }
    }

    #[inline]
    fn row_start(&self, y: usize) -> (f32, f32) {
        let dy = (y - self.bounds.min_y) as f32;
        (
            self.uv0.x() + dy * self.u_grad.y(),
            self.uv0.y() + dy * self.v_grad.y(),
        )
    }

    #[inline]
    fn is_inside(&self, u: f32, v: f32) -> bool {
        const EPS: f32 = 1e-5;
        let tl = &self.top_left;
        (u > 0.0 || (u > -EPS && tl.ca))
            && (v > 0.0 || (v > -EPS && tl.cb))
            && (u + v < 1.0 || (u + v < 1.0 + EPS && tl.ab))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// scanline
// ═══════════════════════════════════════════════════════════════════════════

fn scan_triangle<const N: usize>(
    buf: &mut RenderBuffer<Fragment<N>>,
    setup: &TriSetup,
    samples: &[Vec2f; N],
    normal: Vec3f,
    shader: &impl Shader,
) {
    for y in setup.bounds.range_y(buf.height() - 1) {
        let (mut u, mut v) = setup.row_start(y);
        for x in setup.bounds.range_x(buf.width() - 1) {
            let frag = buf.get_mut(x, y);
            for i in 0..N {
                let us = u + setup.u_grad * samples[i];
                let vs = v + setup.v_grad * samples[i];
                if setup.is_inside(us, vs) {
                    let z = setup.z_c + us * setup.dz_du + vs * setup.dz_dv;
                    if z < frag.depth_buf[i] {
                        frag.depth_buf[i] = z;
                        frag.color_buf[i] = shader.shade(us, vs, z, normal);
                    }
                }
            }
            u = u + setup.u_grad.x();
            v = v + setup.v_grad.x();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Rasterizer<N>
// ═══════════════════════════════════════════════════════════════════════════

pub struct Rasterizer<const N: usize> {
    pub samples: [Vec2f; N],
}

impl Rasterizer<1> {
    #[allow(unused)]
    pub const NON_MSAA: Self = Self {
        samples: [Vec2f::ZERO],
    };
}

impl Rasterizer<4> {
    #[allow(unused)]
    pub const MSAA_4X: Self = Self {
        samples: [
            Vec2f::new(-0.125, -0.375),
            Vec2f::new(0.375, -0.125),
            Vec2f::new(-0.375, 0.125),
            Vec2f::new(0.125, 0.375),
        ],
    };
}

impl<const N: usize> Rasterizer<N> {
    #[allow(unused)]
    pub const fn new(samples: [Vec2f; N]) -> Self {
        Self { samples }
    }

    pub fn rasterize(
        &self,
        buf: &mut RenderBuffer<Fragment<N>>,
        a: Vec4f,
        b: Vec4f,
        c: Vec4f,
        normal: Vec3f,
        shader: &impl Shader,
    ) {
        let (w, h) = (buf.width() as f32, buf.height() as f32);
        let vp = transform::translate(Vec3f::new(w / 2.0, h / 2.0, 0.0))
            * transform::scale(Vec3f::new(w / 2.0, -h / 2.0, 1.0));

        let sa = (vp * a).perspective_divide().unwrap();
        let sb = (vp * b).perspective_divide().unwrap();
        let sc = (vp * c).perspective_divide().unwrap();

        let setup = TriSetup::new(sa, sb, sc);

        scan_triangle(buf, &setup, &self.samples, normal, shader);
    }

    pub fn draw_mesh(
        &self,
        buf: &mut RenderBuffer<Fragment<N>>,
        vertices: &[Vec4f],
        indices: &[[usize; 3]],
        normals: &[Vec3f],
        shader: impl Shader,
    ) {
        for (t, &[i0, i1, i2]) in indices.iter().enumerate() {
            self.rasterize(buf, vertices[i0], vertices[i1], vertices[i2], normals[t], &shader);
        }
    }

    pub fn resolve(&self, src: &RenderBuffer<Fragment<N>>, fb: &mut RenderBuffer<Color>) {
        for y in 0..src.height() {
            for x in 0..src.width() {
                fb[(x, y)] = Color::average(&src[(x, y)].color_buf);
            }
        }
    }
}
