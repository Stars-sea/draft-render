use crate::buffer::RenderBuffer;
use crate::color::Color;
use crate::linalg::{Mat4f, Vec3f, Vec4f};
use crate::scene::Mesh;

use bytemuck::cast_slice;
use std::cmp::min;
use std::slice::Iter;
use std::sync::Arc;
use std::sync::mpsc;

pub struct RenderJob {
    pub objects: Vec<RenderObject>,
    pub width: usize,
    pub height: usize,
}

pub struct RenderObject {
    pub mesh: Arc<Mesh>,
    pub mvp: Mat4f,
    pub color: Color,
}

pub enum RenderResult {
    FrameReady(Vec<u32>),
}

pub fn render_loop(job_rx: mpsc::Receiver<RenderJob>, result_tx: mpsc::SyncSender<RenderResult>) {
    let mut width = 0;
    let mut height = 0;
    let mut frame_buffer = RenderBuffer::new(1, 1, Color::BLACK);
    let mut depth_buffer = RenderBuffer::new(1, 1, f32::INFINITY);

    while let Ok(job) = job_rx.recv() {
        if job.width != width || job.height != height {
            width = job.width;
            height = job.height;
            frame_buffer = RenderBuffer::new(width, height, Color::BLACK);
            depth_buffer = RenderBuffer::new(width, height, f32::INFINITY);
        }

        frame_buffer.clear(Color::BLACK);
        depth_buffer.clear(f32::INFINITY);

        for obj in &job.objects {
            let vertices: Vec<Vec3f> = obj
                .mesh
                .vertices
                .iter()
                .map(|v| {
                    (obj.mvp * Vec4f::from_vec3(*v, 1.0))
                        .perspective_divide()
                        .unwrap()
                })
                .collect();

            for &[i0, i1, i2] in &obj.mesh.indices {
                rasterize(
                    &mut frame_buffer,
                    &mut depth_buffer,
                    vertices[i0],
                    vertices[i1],
                    vertices[i2],
                    obj.color,
                );
            }
        }

        let data = cast_slice(frame_buffer.as_slice()).to_vec();
        if result_tx.send(RenderResult::FrameReady(data)).is_err() {
            break;
        }
    }
}

fn min_max(nums: Iter<f32>) -> (usize, usize) {
    let mut min_val = f32::MAX;
    let mut max_val = f32::MIN;
    for &num in nums {
        if num < min_val {
            min_val = num;
        }
        if num > max_val {
            max_val = num;
        }
    }
    (min_val.floor() as usize, max_val.ceil() as usize)
}

/// 判断从 start 到 end 的边是否为 top edge 或 left edge。
/// Top edge: 水平边且从右到左遍历 (start.x > end.x)。
/// Left edge: 非水平边且从下到上遍历 (start.y > end.y)。
/// 屏幕空间 Y 轴向下。
fn is_top_left(start: Vec3f, end: Vec3f) -> bool {
    let dy = end.y() - start.y();
    if dy.abs() < f32::EPSILON {
        start.x() > end.x()
    } else {
        dy < 0.0
    }
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

    let (ca, cb) = (screen_a - screen_c, screen_b - screen_c);
    let inv_denom = 1.0 / (ca.x() * cb.y() - ca.y() * cb.x());

    let du_dx = cb.y() * inv_denom;
    let du_dy = -cb.x() * inv_denom;
    let dv_dx = -ca.y() * inv_denom;
    let dv_dy = ca.x() * inv_denom;

    let dz_du = ca.z();
    let dz_dv = cb.z();

    // 重心坐标对应的三条边：
    //   u=0 边: screen_c → screen_a
    //   v=0 边: screen_c → screen_b
    //   u+v=1 边: screen_a → screen_b
    let tl_ca = is_top_left(screen_c, screen_a);
    let tl_cb = is_top_left(screen_c, screen_b);
    let tl_ab = is_top_left(screen_a, screen_b);

    let x_coords = [screen_a.x(), screen_b.x(), screen_c.x()];
    let y_coords = [screen_a.y(), screen_b.y(), screen_c.y()];

    let (min_x, max_x) = min_max(x_coords.iter());
    let (min_y, max_y) = min_max(y_coords.iter());

    let px0 = min_x as f32 + 0.5 - screen_c.x();
    let py0 = min_y as f32 + 0.5 - screen_c.y();
    let mut u_row = (cb.y() * px0 - cb.x() * py0) * inv_denom;
    let mut v_row = (ca.x() * py0 - ca.y() * px0) * inv_denom;

    for y in min_y..=min(max_y, frame_buffer.height() - 1) {
        let (mut u, mut v) = (u_row, v_row);
        for x in min_x..=min(max_x, frame_buffer.width() - 1) {
            let inside = (u > 0.0 || (u > -f32::EPSILON && tl_ca))
                && (v > 0.0 || (v > -f32::EPSILON && tl_cb))
                && (u + v < 1.0 || (u + v < 1.0 + f32::EPSILON && tl_ab));

            if inside {
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
