use crate::buffer::RenderBuffer;
use crate::color::Color;
use crate::linalg::{Mat4f, Vec2, Vec2f, Vec3f, Vec4f};
use crate::scene::Camera;
use crate::scene::object::SceneObject;

use bytemuck::cast_slice;
use num_traits::float::FloatCore;
use std::cmp::min;
use std::slice::Iter;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<SceneObject>,

    frame_buffer: RenderBuffer<Color>,
    depth_buffer: RenderBuffer<f32>,
}

impl Scene {
    pub fn new(camera: Camera, width: usize, height: usize) -> Self {
        Self {
            camera,
            objects: Vec::new(),
            frame_buffer: RenderBuffer::new(width, height, Color::BLACK),
            depth_buffer: RenderBuffer::new(width, height, f32::INFINITY),
        }
    }

    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    pub fn frame_data(&self) -> &[u32] {
        cast_slice(self.frame_buffer.as_slice())
    }

    pub fn render(&mut self) {
        self.frame_buffer.clear(Color::BLACK);
        self.depth_buffer.clear(f32::INFINITY);

        let aspect = self.frame_buffer.width() as f32 / self.frame_buffer.height() as f32;
        let vp = self.camera.vp_matrix(aspect);

        for object in &mut self.objects {
            let mvp = vp * object.transform.transform_matrix();

            let vertices: Vec<Vec3f> = object
                .mesh
                .vertices
                .iter()
                .map(|vec| {
                    (mvp * Vec4f::from_vec3(*vec, 1.0))
                        .perspective_divide()
                        .unwrap()
                })
                .collect();

            for &[i0, i1, i2] in &object.mesh.indices {
                rasterize(
                    &mut self.frame_buffer,
                    &mut self.depth_buffer,
                    vertices[i0],
                    vertices[i1],
                    vertices[i2],
                    object.color,
                );
            }
        }
    }
}

fn min_max(nums: Iter<f32>) -> (usize, usize) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for &num in nums {
        if num < min {
            min = num;
        }
        if num > max {
            max = num;
        }
    }

    (min.floor() as usize, max.ceil() as usize)
}

fn rasterize(
    frame_buffer: &mut RenderBuffer<Color>,
    depth_buffer: &mut RenderBuffer<f32>,
    a: Vec3f,
    b: Vec3f,
    c: Vec3f,
    color: Color,
) {
    let (w, h) = (frame_buffer.width() as f32, frame_buffer.height() as f32);
    let viewport = Mat4f::from([
        [w / 2.0, 0.0, 0.0, w / 2.0],
        [0.0, -h / 2.0, 0.0, h / 2.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    let screen_a = (viewport * Vec4f::from_vec3(a, 1.0)).xyz();
    let screen_b = (viewport * Vec4f::from_vec3(b, 1.0)).xyz();
    let screen_c = (viewport * Vec4f::from_vec3(c, 1.0)).xyz();

    let x_coords = [screen_a.x(), screen_b.x(), screen_c.x()];
    let y_coords = [screen_a.y(), screen_b.y(), screen_c.y()];

    let (min_x, max_x) = min_max(x_coords.iter());
    let (min_y, max_y) = min_max(y_coords.iter());

    // 边向量 + 重心坐标梯度（以 screen_c 为参考点）
    // u = (cb.y * cp.x - cb.x * cp.y) / denom,  v = (ca.x * cp.y - ca.y * cp.x) / denom
    let (ca, cb) = (screen_a - screen_c, screen_b - screen_c);
    let inv_denom = 1.0 / (ca.x() * cb.y() - ca.y() * cb.x());

    let du_dx = cb.y() * inv_denom;
    let du_dy = -cb.x() * inv_denom;
    let dv_dx = -ca.y() * inv_denom;
    let dv_dy = ca.x() * inv_denom;

    let dz_du = ca.z();
    let dz_dv = cb.z();

    // 第一行起始值
    let px0 = min_x as f32 + 0.5 - screen_c.x();
    let py0 = min_y as f32 + 0.5 - screen_c.y();
    let mut u_row = (cb.y() * px0 - cb.x() * py0) * inv_denom;
    let mut v_row = (ca.x() * py0 - ca.y() * px0) * inv_denom;

    for y in min_y..=min(max_y, frame_buffer.height() - 1) {
        let (mut u, mut v) = (u_row, v_row);
        for x in min_x..=min(max_x, frame_buffer.width() - 1) {
            let eps = -f32::EPSILON;
            if u >= eps && v >= eps && u + v <= 1.0 - eps {
                let z = screen_c.z() + u * dz_du + v * dz_dv;
                if z < depth_buffer[(x, y)] {
                    depth_buffer[(x, y)] = z;
                    frame_buffer[(x, y)] = color;
                }
            }
            u += du_dx;
            v += dv_dx;
        }
        u_row += du_dy;
        v_row += dv_dy;
    }
}
