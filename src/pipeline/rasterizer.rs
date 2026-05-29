use crate::color::Color;
use crate::pipeline::buffer::RenderBuffer;
use crate::pipeline::fragment::Fragment;
use crate::pipeline::shader::Shader;
use crate::scene::Material;

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
    bc0: Vec2,
    z_c: f32,
    dz_du: f32,
    dz_dv: f32,
}

impl Bounds {
    fn min_max(a: f32, b: f32, c: f32) -> (usize, usize) {
        (
            a.min(b).min(c).floor() as usize,
            a.max(b).max(c).ceil() as usize,
        )
    }

    fn new(a: Vec3A, b: Vec3A, c: Vec3A) -> Self {
        let (min_x, max_x) = Self::min_max(a.x, b.x, c.x);
        let (min_y, max_y) = Self::min_max(a.y, b.y, c.y);
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
    fn is_top_left(sx: f32, sy: f32, ex: f32, ey: f32) -> bool {
        let dy = ey - sy;
        if dy.abs() < f32::EPSILON {
            sx > ex
        } else {
            dy < 0.0
        }
    }

    fn new(a: Vec3A, b: Vec3A, c: Vec3A) -> Self {
        Self {
            ca: Self::is_top_left(c.x, c.y, a.x, a.y),
            cb: Self::is_top_left(c.x, c.y, b.x, b.y),
            ab: Self::is_top_left(a.x, a.y, b.x, b.y),
        }
    }
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
        let bc0 = Vec2::new(p0.perp_dot(eb.xy()), ea.xy().perp_dot(p0)) * inv;

        Self {
            bounds,
            top_left,
            u_grad,
            v_grad,
            bc0,
            z_c: sc.z,
            dz_du: ea.z,
            dz_dv: eb.z,
        }
    }

    #[inline]
    fn row_start(&self, y: usize) -> (f32, f32) {
        let dy = (y - self.bounds.min_y) as f32;
        (
            self.bc0.x + dy * self.u_grad.y,
            self.bc0.y + dy * self.v_grad.y,
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
// triangle
// ═══════════════════════════════════════════════════════════════════════════

struct Triangle {
    clip: [Vec4; 3],
    world: [Vec3A; 3],
    uv: [Vec2; 3],
    normals: [Vec3A; 3],
}

impl Triangle {
    fn from_slices(
        vertices: &[Vec4],
        world_positions: &[Vec3A],
        vertex_normals: &[Vec3A],
        uvs: &[Vec2],
        indexes: &[usize; 3],
    ) -> Self {
        let clip = indexes.map(|i| vertices[i]);
        let world = indexes.map(|i| world_positions[i]);
        let uv = if uvs.is_empty() {
            [Vec2::ZERO; 3]
        } else {
            indexes.map(|i| uvs[i])
        };
        let normals = if vertex_normals.len() == world_positions.len() {
            indexes.map(|i| vertex_normals[i])
        } else {
            let fnorm = (world[1] - world[0]).cross(world[2] - world[0]).normalize();
            [fnorm; 3]
        };
        Self { clip, world, uv, normals }
    }

    fn is_backface(&self, camera_pos: Vec3A) -> bool {
        let face_n = (self.world[1] - self.world[0])
            .cross(self.world[2] - self.world[0]);
        let centroid = (self.world[0] + self.world[1] + self.world[2]) / 3.0;
        face_n.dot(camera_pos - centroid) <= 0.0
    }

    fn perspective_divide(v: Vec4) -> Option<Vec3A> {
        if v.w <= 0.0 {
            return None;
        }
        let r = 1.0 / v.w;
        Some((v.xyz() * r).into())
    }

    fn setup(&self, vp: Mat4) -> Option<TriSetup> {
        let sa = Self::perspective_divide(vp * self.clip[0])?;
        let sb = Self::perspective_divide(vp * self.clip[1])?;
        let sc = Self::perspective_divide(vp * self.clip[2])?;
        Some(TriSetup::new(sa, sb, sc))
    }

    fn interpolator(&self) -> TriInterp {
        TriInterp {
            w_c: self.world[2],
            w_du: self.world[0] - self.world[2],
            w_dv: self.world[1] - self.world[2],
            uv_c: self.uv[2],
            uv_du: self.uv[0] - self.uv[2],
            uv_dv: self.uv[1] - self.uv[2],
            n_c: self.normals[2],
            n_du: self.normals[0] - self.normals[2],
            n_dv: self.normals[1] - self.normals[2],
        }
    }
}

struct TriInterp {
    w_c: Vec3A,
    w_du: Vec3A,
    w_dv: Vec3A,
    uv_c: Vec2,
    uv_du: Vec2,
    uv_dv: Vec2,
    n_c: Vec3A,
    n_du: Vec3A,
    n_dv: Vec3A,
}

impl TriInterp {
    #[inline]
    fn world_pos(&self, u: f32, v: f32) -> Vec3A {
        self.w_c + self.w_du * u + self.w_dv * v
    }

    #[inline]
    fn tex_uv(&self, u: f32, v: f32) -> Vec2 {
        self.uv_c + self.uv_du * u + self.uv_dv * v
    }

    #[inline]
    fn normal(&self, u: f32, v: f32) -> Vec3A {
        (self.n_c + self.n_du * u + self.n_dv * v).normalize()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Rasterizer<N>
// ═══════════════════════════════════════════════════════════════════════════

pub struct Rasterizer<const N: usize> {
    pub samples: [Vec2; N],
}

impl Rasterizer<1> {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub const fn new(samples: [Vec2; N]) -> Self {
        Self { samples }
    }

    fn viewport_matrix(w: f32, h: f32) -> Mat4 {
        Mat4::from_translation(Vec3A::new(w / 2.0, h / 2.0, 0.0).into())
            * Mat4::from_scale(Vec3A::new(w / 2.0, -h / 2.0, 1.0).into())
    }

    #[allow(clippy::too_many_arguments)]
    fn shade_fragment(
        &self,
        frag: &mut Fragment<N>,
        setup: &TriSetup,
        interp: &TriInterp,
        shader: &impl Shader,
        material: &Material,
        u: f32,
        v: f32,
    ) {
        for i in 0..N {
            let us = u + setup.u_grad.dot(self.samples[i]);
            let vs = v + setup.v_grad.dot(self.samples[i]);
            if !setup.is_inside(us, vs) {
                continue;
            }
            let z = setup.z_c + us * setup.dz_du + vs * setup.dz_dv;
            if z >= frag.depth_buf[i] {
                continue;
            }
            let normal = interp.normal(us, vs);
            let color = shader.shade(
                material,
                interp.tex_uv(us, vs),
                z,
                normal,
                interp.world_pos(us, vs),
            );
            frag.depth_buf[i] = z;
            frag.color_buf[i] = color;
        }
    }

    fn rasterize(
        &self,
        buf: &mut RenderBuffer<Fragment<N>>,
        vp: Mat4,
        tri: &Triangle,
        shader: &impl Shader,
        material: &Material,
    ) {
        let setup = match tri.setup(vp) {
            Some(s) => s,
            None => return,
        };
        let interp = tri.interpolator();

        for y in setup.bounds.range_y(buf.height() - 1) {
            let (mut u, mut v) = setup.row_start(y);
            for x in setup.bounds.range_x(buf.width() - 1) {
                self.shade_fragment(
                    buf.get_mut(x, y),
                    &setup,
                    &interp,
                    shader,
                    material,
                    u,
                    v,
                );
                u += setup.u_grad.x;
                v += setup.v_grad.x;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_mesh(
        &self,
        buf: &mut RenderBuffer<Fragment<N>>,
        vertices: &[Vec4],
        world_positions: &[Vec3A],
        vertex_normals: &[Vec3A],
        uvs: &[Vec2],
        indices: &[[usize; 3]],
        shader: &impl Shader,
        material: &Material,
        camera_pos: Vec3A,
    ) {
        let vp = Self::viewport_matrix(buf.width() as f32, buf.height() as f32);

        for indexes in indices {
            let tri = Triangle::from_slices(vertices, world_positions, vertex_normals, uvs, indexes);
            if tri.is_backface(camera_pos) {
                continue;
            }
            self.rasterize(buf, vp, &tri, shader, material);
        }
    }

    pub fn resolve(&self, src: &RenderBuffer<Fragment<N>>, fb: &mut RenderBuffer<u32>) {
        for y in 0..src.height() {
            for x in 0..src.width() {
                fb[(x, y)] = Color::average(&src[(x, y)].color_buf).to_u32();
            }
        }
    }
}
