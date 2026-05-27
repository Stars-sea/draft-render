use crate::color::Color;
use crate::linalg::{Mat4, Vec4};
use crate::pipeline::{Fragment, Rasterizer, RenderBuffer};
use crate::scene::Mesh;

use bytemuck::cast_slice;
use num_traits::real::Real;
use std::sync::Arc;
use std::sync::mpsc;

pub struct RenderJob<T: Real> {
    pub objects: Vec<RenderObject<T>>,
    pub width: usize,
    pub height: usize,
}

pub struct RenderObject<T: Real> {
    pub mesh: Arc<Mesh<T>>,
    pub mvp: Mat4<T>,
    pub color: Color,
}

pub enum RenderResult {
    FrameReady(Vec<u32>),
}

pub fn render_loop<T: Real, const N: usize>(
    rasterizer: Rasterizer<T, N>,
    job_rx: mpsc::Receiver<RenderJob<T>>,
    result_tx: mpsc::SyncSender<RenderResult>,
) {
    let mut width = 0;
    let mut height = 0;
    let mut frame_buffer = RenderBuffer::new(1, 1, Color::BLACK);
    let mut frag_buf = RenderBuffer::new(1, 1, Fragment::<T, N>::new());

    while let Ok(job) = job_rx.recv() {
        if job.width != width || job.height != height {
            width = job.width;
            height = job.height;
            frame_buffer = RenderBuffer::new(width, height, Color::BLACK);
            frag_buf = RenderBuffer::new(width, height, Fragment::<T, N>::new());
        }

        frag_buf.clear(Fragment::new());

        for obj in &job.objects {
            let vertices: Vec<Vec4<T>> = obj
                .mesh
                .vertices
                .iter()
                .map(|v| obj.mvp * Vec4::from_vec3(*v, T::one()))
                .collect();

            for &[i0, i1, i2] in &obj.mesh.indices {
                rasterizer.rasterize(
                    &mut frag_buf,
                    vertices[i0],
                    vertices[i1],
                    vertices[i2],
                    obj.color,
                );
            }
        }

        rasterizer.resolve(&frag_buf, &mut frame_buffer);

        let data = cast_slice(frame_buffer.as_slice()).to_vec();
        if result_tx.send(RenderResult::FrameReady(data)).is_err() {
            break;
        }
    }
}
