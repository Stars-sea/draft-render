mod color;
mod pipeline;
mod render_thread;
mod scene;

use crate::color::Color;
use crate::pipeline::Rasterizer;
use crate::render_thread::{RenderJob, RenderResult};
use crate::scene::{
    Camera, DirectionalLight, MeshBuilder, PointLight, Scene, SceneObject, Transform,
};

use anyhow::Result;
use glam::{Quat, Vec3A};
use minifb::{Key, Window, WindowOptions};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn main() -> Result<()> {
    let (width, height) = (800, 600);

    let mut scene = Scene::new(Camera::default());
    scene.add_object(cube());
    scene.add_light(directional_light());
    scene.add_light(point_light());

    let (job_tx, job_rx) = mpsc::sync_channel::<RenderJob>(1);
    let (result_tx, result_rx) = mpsc::sync_channel::<RenderResult>(1);

    thread::spawn(move || render_thread::render_loop(Rasterizer::<4>::MSAA_4X, job_rx, result_tx));

    let mut window = Window::new("cube", width, height, WindowOptions::default())?;
    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let angle = start.elapsed().as_secs_f32() * 0.8;
        scene.objects[0]
            .transform
            .set_rotation(Quat::from_axis_angle(Vec3A::Y.into(), angle));

        let job = scene.build_render_job(width, height);
        if job_tx.send(job).is_err() {
            break;
        }
        if let Ok(RenderResult::FrameReady(data)) = result_rx.recv() {
            window.update_with_buffer(&data, width, height)?;
        } else {
            break;
        }
    }

    Ok(())
}

fn directional_light() -> Arc<DirectionalLight> {
    Arc::new(DirectionalLight::new(
        Vec3A::new(0.0, -1.0, -1.0),
        Color::WHITE,
        1.0,
    ))
}

fn point_light() -> Arc<PointLight> {
    Arc::new(PointLight::new(
        Vec3A::new(2.0, 2.0, 3.5),
        Color::WHITE,
        8.0,
    ))
}

fn cube() -> SceneObject {
    const S: f32 = 0.5;
    let builder = MeshBuilder::new()
        .vertex(Vec3A::new(-S, -S, S))
        .vertex(Vec3A::new(S, -S, S))
        .vertex(Vec3A::new(S, S, S))
        .vertex(Vec3A::new(-S, S, S))
        .vertex(Vec3A::new(-S, -S, -S))
        .vertex(Vec3A::new(S, -S, -S))
        .vertex(Vec3A::new(S, S, -S))
        .vertex(Vec3A::new(-S, S, -S))
        .triangle(0, 1, 2)
        .triangle(0, 2, 3)
        .triangle(5, 4, 7)
        .triangle(5, 7, 6)
        .triangle(1, 5, 6)
        .triangle(1, 6, 2)
        .triangle(4, 0, 3)
        .triangle(4, 3, 7)
        .triangle(3, 2, 6)
        .triangle(3, 6, 7)
        .triangle(4, 5, 1)
        .triangle(4, 1, 0);

    let transform = Transform::default().with_translation(Vec3A::new(0.0, 0.0, 3.0));
    SceneObject::new(
        Arc::new(builder.build()),
        transform,
        Color::rgb(200, 120, 60),
    )
}
