use crate::pipeline::{BlinnPhongShader, Fragment, Rasterizer, RenderBuffer};
use crate::scene::{Light, Material};

use glam::{Vec3A, Vec4};
use std::sync::{Arc, mpsc};

pub struct RenderJob {
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub objects: Vec<RenderObject>,
    pub width: usize,
    pub height: usize,
}

pub struct RenderObject {
    pub indices: Vec<[usize; 3]>,
    pub clip_vertices: Vec<Vec4>,
    pub world_positions: Vec<Vec3A>,
    pub vertex_normals: Vec<Vec3A>,
    pub uvs: Vec<glam::Vec2>,
    pub material: Material,
}

pub enum RenderResult {
    FrameReady(Vec<u32>),
}

pub fn render_loop<const N: usize>(
    rasterizer: Rasterizer<N>,
    job_rx: mpsc::Receiver<RenderJob>,
    result_tx: mpsc::SyncSender<RenderResult>,
) {
    let mut width = 0;
    let mut height = 0;
    let mut frame_buffer = RenderBuffer::new(1, 1, 0u32);
    let mut frag_buf = RenderBuffer::new(1, 1, Fragment::<N>::new());

    while let Ok(job) = job_rx.recv() {
        if job.width != width || job.height != height {
            width = job.width;
            height = job.height;
            frame_buffer = RenderBuffer::new(width, height, 0u32);
            frag_buf = RenderBuffer::new(width, height, Fragment::<N>::new());
        }

        frag_buf.clear(Fragment::new());

        let shader = BlinnPhongShader::new(job.lights.clone());

        for obj in &job.objects {
            rasterizer.draw_mesh(
                &mut frag_buf,
                &obj.clip_vertices,
                &obj.world_positions,
                &obj.vertex_normals,
                &obj.uvs,
                &obj.indices,
                &shader,
                &obj.material,
            );
        }

        rasterizer.resolve(&frag_buf, &mut frame_buffer);

        let data = frame_buffer.as_slice().to_vec();
        if result_tx.send(RenderResult::FrameReady(data)).is_err() {
            break;
        }
    }
}
