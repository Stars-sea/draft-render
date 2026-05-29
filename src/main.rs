mod color;
mod pipeline;
mod pmx;
mod scene;

use crate::color::Color;
use crate::pipeline::{render_loop, Rasterizer, RenderJob, RenderResult};
use crate::scene::{
    Camera, DirectionalLight, Material, MeshBuilder, PointLight, Scene, SceneObject, SubMesh,
    Texture, Transform,
};

use anyhow::Result;
use glam::{Quat, Vec3A};
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

fn main() -> Result<()> {
    let (width, height) = (1920, 1080);

    let args: Vec<String> = env::args().collect();
    let pmx_path = args.get(1).filter(|p| p.ends_with(".pmx"));

    let mut scene = Scene::new(Camera::default());
    scene.add_light(directional_light());
    scene.add_light(point_light());

    if let Some(path) = pmx_path {
        let submeshes = pmx::load_pmx(path)?;
        let obj = SceneObject::new(
            submeshes,
            Transform::default()
                .with_translation(Vec3A::new(0.0, -1.5, 3.0))
                .with_scale(Vec3A::splat(0.2)),
        );
        scene.add_object(obj);
    } else {
        scene.add_object(cube());
        scene.add_object(textured_quad());
    }

    let (job_tx, job_rx) = mpsc::sync_channel::<RenderJob>(1);
    let (result_tx, result_rx) = mpsc::sync_channel::<RenderResult>(1);

    thread::spawn(move || render_loop(Rasterizer::<4>::MSAA_4X, job_rx, result_tx));

    let mut window = Window::new("cube", width, height, WindowOptions::default())?;
    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let angle = start.elapsed().as_secs_f32() * 0.8;
        scene.objects[0]
            .transform
            .set_rotation(Quat::from_axis_angle(Vec3A::Y.into(), angle));

        let job = RenderJob::from_scene(&scene, width, height);
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
        Vec3A::new(0.0, 0.0, -1.0),
        Color::WHITE,
        1.0,
    ))
}

fn point_light() -> Arc<PointLight> {
    Arc::new(PointLight::new(
        Vec3A::new(2.0, 3.0, 3.5),
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

    SceneObject::single(
        SubMesh::new(
            Arc::new(builder.build()),
            Material::solid(Color::rgb(200, 120, 60)),
        ),
        Transform::default().with_translation(Vec3A::new(0.0, 0.0, 3.0)),
    )
}

fn textured_quad() -> SceneObject {
    let texture = Arc::new(Texture::checkerboard(
        256,
        256,
        32,
        Color::WHITE,
        Color::rgb(50, 50, 160),
    ));
    let builder = MeshBuilder::new()
        .vertex(Vec3A::new(-0.5, -0.5, 0.0))
        .uv(0.0, 0.0)
        .vertex(Vec3A::new(0.5, -0.5, 0.0))
        .uv(1.0, 0.0)
        .vertex(Vec3A::new(0.5, 0.5, 0.0))
        .uv(1.0, 1.0)
        .vertex(Vec3A::new(-0.5, 0.5, 0.0))
        .uv(0.0, 1.0)
        .triangle(0, 2, 1)
        .triangle(0, 3, 2);

    SceneObject::single(
        SubMesh::new(Arc::new(builder.build()), Material::textured(texture)),
        Transform::default().with_translation(Vec3A::new(1.5, 0.0, 3.0)),
    )
}
