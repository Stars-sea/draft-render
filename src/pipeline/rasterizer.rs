use crate::color::Color;
use crate::pipeline::buffer::RenderBuffer;
use crate::pipeline::fragment::Fragment;
use crate::pipeline::shader::Shader;

use glam::{Mat4, Vec2, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles};
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
    u_grad: Vec2,
    v_grad: Vec2,
    uv0: Vec2,
    z_c: f32,
    dz_du: f32,
    dz_dv: f32,
}

impl Bounds {
    fn new(a: Vec3A, b: Vec3A, c: Vec3A) -> Self {
        let (min_x, max_x) = min_max(a.x, b.x, c.x);
        let (min_y, max_y) = min_max(a.y, b.y, c.y);
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

    fn min_center(&self) -> Vec2 {
        const OFFSET: Vec2 = Vec2::new(0.5, 0.5);
        Vec2::new(self.min_x as f32, self.min_y as f32) + OFFSET
    }
}

impl TopLeft {
    fn new(a: Vec3A, b: Vec3A, c: Vec3A) -> Self {
        Self {
            ca: is_top_left(c.x, c.y, a.x, a.y),
            cb: is_top_left(c.x, c.y, b.x, b.y),
            ab: is_top_left(a.x, a.y, b.x, b.y),
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

fn perspective_divide(v: Vec4) -> Option<Vec3A> {
    // w == 0 only when the vertex lies on the camera plane (z_view = 0).
    // Near-plane clipping (n = 0.1) guarantees |w| ≥ 0.1 for every visible
    // vertex, so exact comparison against 0.0 is sound — no rounding hazard.
    if v.w == 0.0 {
        return None;
    }
    let r = 1.0 / v.w;
    Some((v.xyz() * r).into())
}

impl TriSetup {
    fn new(sa: Vec3A, sb: Vec3A, sc: Vec3A) -> Self {
        let bounds = Bounds::new(sa, sb, sc);
        let top_left = TopLeft::new(sa, sb, sc);

        let (ea, eb) = (sa - sc, sb - sc);
        let inv = 1.0 / ea.xy().perp_dot(eb.xy());
        let u_grad = Vec2::new(eb.y, -eb.x) * inv;
        let v_grad = Vec2::new(-ea.y, ea.x) * inv;

        let p0 = bounds.min_center() - sc.xy();
        let uv0 = Vec2::new(p0.perp_dot(eb.xy()), ea.xy().perp_dot(p0)) * inv;

        Self {
            bounds,
            top_left,
            u_grad,
            v_grad,
            uv0,
            z_c: sc.z,
            dz_du: ea.z,
            dz_dv: eb.z,
        }
    }

    #[inline]
    fn row_start(&self, y: usize) -> (f32, f32) {
        let dy = (y - self.bounds.min_y) as f32;
        (
            self.uv0.x + dy * self.u_grad.y,
            self.uv0.y + dy * self.v_grad.y,
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
    samples: &[Vec2; N],
    normal: Vec3A,
    world_a: Vec3A,
    world_b: Vec3A,
    world_c: Vec3A,
    shader: &impl Shader,
) {
    let w_du = world_a - world_c;
    let w_dv = world_b - world_c;
    for y in setup.bounds.range_y(buf.height() - 1) {
        let (mut u, mut v) = setup.row_start(y);
        for x in setup.bounds.range_x(buf.width() - 1) {
            let frag = buf.get_mut(x, y);
            let mut shaded = None;
            for i in 0..N {
                let us = u + setup.u_grad.dot(samples[i]);
                let vs = v + setup.v_grad.dot(samples[i]);
                if !setup.is_inside(us, vs) {
                    continue;
                }
                let z = setup.z_c + us * setup.dz_du + vs * setup.dz_dv;
                if z >= frag.depth_buf[i] {
                    continue;
                }
                let color = *shaded.get_or_insert_with(|| {
                    let wp = world_c + w_du * u + w_dv * v;
                    shader.shade(Vec2::new(u, v), z, normal, wp)
                });
                frag.depth_buf[i] = z;
                frag.color_buf[i] = color;
            }
            u = u + setup.u_grad.x;
            v = v + setup.v_grad.x;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Rasterizer<N>
// ═══════════════════════════════════════════════════════════════════════════

pub struct Rasterizer<const N: usize> {
    pub samples: [Vec2; N],
}

impl Rasterizer<1> {
    pub const NON_MSAA: Self = Self {
        samples: [Vec2::ZERO],
    };
}

impl Rasterizer<4> {
    pub const MSAA_4X: Self = Self {
        samples: [
            Vec2::new(-0.125, -0.375),
            Vec2::new(0.375, -0.125),
            Vec2::new(-0.375, 0.125),
            Vec2::new(0.125, 0.375),
        ],
    };
}

impl<const N: usize> Rasterizer<N> {
    pub const fn new(samples: [Vec2; N]) -> Self {
        Self { samples }
    }

    fn viewport_matrix(w: f32, h: f32) -> Mat4 {
        Mat4::from_translation(Vec3A::new(w / 2.0, h / 2.0, 0.0).into())
            * Mat4::from_scale(Vec3A::new(w / 2.0, -h / 2.0, 1.0).into())
    }

    fn rasterize(
        &self,
        buf: &mut RenderBuffer<Fragment<N>>,
        vp: Mat4,
        a: Vec4,
        b: Vec4,
        c: Vec4,
        normal: Vec3A,
        world_a: Vec3A,
        world_b: Vec3A,
        world_c: Vec3A,
        shader: &impl Shader,
    ) {
        let sa = perspective_divide(vp * a).unwrap();
        let sb = perspective_divide(vp * b).unwrap();
        let sc = perspective_divide(vp * c).unwrap();

        let setup = TriSetup::new(sa, sb, sc);

        scan_triangle(
            buf,
            &setup,
            &self.samples,
            normal,
            world_a,
            world_b,
            world_c,
            shader,
        );
    }

    pub fn draw_mesh(
        &self,
        buf: &mut RenderBuffer<Fragment<N>>,
        vertices: &[Vec4],
        world_positions: &[Vec3A],
        indices: &[[usize; 3]],
        normals: &[Vec3A],
        shader: impl Shader,
    ) {
        let vp_matrix = Self::viewport_matrix(buf.width() as f32, buf.height() as f32);

        for (t, &[i0, i1, i2]) in indices.iter().enumerate() {
            let normal = normals[t];
            let (world_a, world_b, world_c) = (
                world_positions[i0],
                world_positions[i1],
                world_positions[i2],
            );

            // Back-face culling: outward normal must face toward camera (origin).
            let view_dir = -world_a;
            if normal.dot(view_dir) <= 0.0 {
                continue;
            }

            self.rasterize(
                buf,
                vp_matrix,
                vertices[i0],
                vertices[i1],
                vertices[i2],
                normal,
                world_a,
                world_b,
                world_c,
                &shader,
            );
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
