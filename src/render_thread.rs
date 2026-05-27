use crate::color::Color;
use crate::linalg::{Mat4f, Vec3f, Vec4f};
use crate::pipeline::{BlinnPhongShader, Fragment, Rasterizer, RenderBuffer};
use crate::scene::{Light, Mesh};

use bytemuck::cast_slice;
use std::sync::Arc;
use std::sync::mpsc;

pub struct RenderJob {
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub objects: Vec<RenderObject>,
    pub width: usize,
    pub height: usize,
}

pub struct RenderObject {
    pub mesh: Arc<Mesh>,
    pub mvp: Mat4f,
    pub color: Color,
    pub face_normals: Vec<Vec3f>,
    pub world_positions: Vec<Vec3f>,
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
    let mut frame_buffer = RenderBuffer::new(1, 1, Color::BLACK);
    let mut frag_buf = RenderBuffer::new(1, 1, Fragment::<N>::new());

    while let Ok(job) = job_rx.recv() {
        if job.width != width || job.height != height {
            width = job.width;
            height = job.height;
            frame_buffer = RenderBuffer::new(width, height, Color::BLACK);
            frag_buf = RenderBuffer::new(width, height, Fragment::<N>::new());
        }

        frag_buf.clear(Fragment::new());

        for obj in &job.objects {
            let vertices = transform_vertices(&obj.mesh, &obj.mvp);
            let shader = BlinnPhongShader::new(obj.color, job.lights.clone());
            rasterizer.draw_mesh(
                &mut frag_buf,
                &vertices,
                &obj.world_positions,
                &obj.mesh.indices,
                &obj.face_normals,
                shader,
            );
        }

        rasterizer.resolve(&frag_buf, &mut frame_buffer);

        let data = cast_slice(frame_buffer.as_slice()).to_vec();
        if result_tx.send(RenderResult::FrameReady(data)).is_err() {
            break;
        }
    }
}

fn transform_vertices(mesh: &Mesh, mvp: &Mat4f) -> Vec<Vec4f> {
    mesh.vertices
        .iter()
        .map(|v| *mvp * Vec4f::from_vec3(*v, 1.0))
        .collect()
}
