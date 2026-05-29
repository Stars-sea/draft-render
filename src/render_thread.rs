use crate::pipeline::{BlinnPhongShader, Fragment, Rasterizer, RenderBuffer};
use crate::scene::{Light, Material, Mesh};

use glam::{Mat4, Vec3A, Vec4};
use std::sync::{Arc, mpsc};

pub struct RenderJob {
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub objects: Vec<RenderObject>,
    pub width: usize,
    pub height: usize,
}

pub struct RenderObject {
    pub mesh: Arc<Mesh>,
    pub mvp: Mat4,
    pub material: Material,
    pub face_normals: Vec<Vec3A>,
    pub world_positions: Vec<Vec3A>,
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
            let vertices = transform_vertices(&obj.mesh, &obj.mvp);
            rasterizer.draw_mesh(
                &mut frag_buf,
                &vertices,
                &obj.world_positions,
                &obj.mesh.indices,
                &obj.face_normals,
                &shader,
                &obj.mesh.uvs,
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

fn transform_vertices(mesh: &Mesh, mvp: &Mat4) -> Vec<Vec4> {
    mesh.vertices.iter().map(|v| *mvp * v.extend(1.0)).collect()
}
