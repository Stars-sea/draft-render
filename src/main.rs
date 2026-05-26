mod buffer;
mod color;
mod linalg;
mod pipeline;
mod render_thread;
mod scene;

use crate::color::Color;
use crate::linalg::Vec3f;
use crate::render_thread::{RenderJob, RenderResult};
use crate::scene::MeshBuilder;
use crate::scene::Transform;
use crate::scene::{Camera, Scene, SceneObject};

use anyhow::Result;
use minifb::{Key, Window, WindowOptions};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

fn main() -> Result<()> {
    let (width, height) = (1920, 1080);

    let mut scene = Scene::new(Camera::default());
    scene.add_object(generate_triangle());
    scene.add_object(generate_square());

    let (job_tx, job_rx) = mpsc::sync_channel::<RenderJob>(1);
    let (result_tx, result_rx) = mpsc::sync_channel::<RenderResult>(1);

    thread::spawn(move || render_thread::render_loop(job_rx, result_tx));

    let mut window = Window::new("renderer", width, height, WindowOptions::default())?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let job = scene.build_render_job(width, height);

        if job_tx.send(job).is_err() {
            break;
        }

        match result_rx.recv() {
            Ok(RenderResult::FrameReady(data)) => {
                window.update_with_buffer(&data, width, height)?;
            }
            Err(_) => break,
        }
    }

    Ok(())
}

fn generate_triangle() -> SceneObject {
    let builder = MeshBuilder::new()
        .vertex(Vec3f::new(-0.5, -0.5, 3.0))
        .vertex(Vec3f::new(0.5, -0.5, 3.0))
        .vertex(Vec3f::new(0.0, 0.5, 3.0))
        .triangle(0, 1, 2);

    SceneObject::new(
        Arc::new(builder.build()),
        Transform::default(),
        Color::GREEN,
    )
}

fn generate_square() -> SceneObject {
    let s = 0.4;
    let builder = MeshBuilder::new()
        .vertex(Vec3f::new(-s, -s, 2.9))
        .vertex(Vec3f::new(s, -s, 2.9))
        .vertex(Vec3f::new(s, s, 3.1))
        .vertex(Vec3f::new(-s, s, 3.1))
        .triangle(0, 1, 2)
        .triangle(0, 2, 3);

    SceneObject::new(Arc::new(builder.build()), Transform::default(), Color::RED)
}
