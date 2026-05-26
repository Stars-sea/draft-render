use crate::buffer::RenderBuffer;
use crate::color::Color;
use crate::linalg::{Mat4f, Vec3f, Vec4f};
use crate::pipeline::Rasterizer;
use crate::scene::Mesh;

use bytemuck::cast_slice;
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

    let rasterizer = Rasterizer::<4>::MSAA_4X;
    let mut color_buf = RenderBuffer::<_, 4>::new(1, 1, Color::BLACK);
    let mut depth_buf = RenderBuffer::<_, 4>::new(1, 1, f32::INFINITY);

    while let Ok(job) = job_rx.recv() {
        if job.width != width || job.height != height {
            width = job.width;
            height = job.height;
            frame_buffer = RenderBuffer::new(width, height, Color::BLACK);
            color_buf = RenderBuffer::<_, 4>::new(width, height, Color::BLACK);
            depth_buf = RenderBuffer::<_, 4>::new(width, height, f32::INFINITY);
        }

        color_buf.clear(Color::BLACK);
        depth_buf.clear(f32::INFINITY);

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
                rasterizer.rasterize(
                    &mut color_buf,
                    &mut depth_buf,
                    vertices[i0],
                    vertices[i1],
                    vertices[i2],
                    obj.color,
                );
            }
        }

        rasterizer.resolve(&color_buf, &mut frame_buffer);

        let data = cast_slice(frame_buffer.as_slice()).to_vec();
        if result_tx.send(RenderResult::FrameReady(data)).is_err() {
            break;
        }
    }
}
