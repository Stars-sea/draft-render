mod color;
mod geometry;
mod pipeline;
mod pmx;
mod scene;

use std::env;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use glam::{Quat, Vec3A};
use minifb::{Key, Window, WindowOptions};
use crate::color::Color;
use crate::pipeline::{Accumulator, TraceScene};
use crate::scene::{
    Camera, DirectionalLight, Material, MeshBuilder, PointLight, Scene, SceneObject, SubMesh,
    Transform,
};

fn main() -> Result<()> {
    let (width, height) = (800, 600);

    let args: Vec<String> = env::args().collect();
    let pmx_path = args.get(1).filter(|p| p.ends_with(".pmx"));

    let camera = Camera::default();
    let mut scene = Scene::new(camera);
    scene.add_light(key_light());
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
    }

    const SPP: u32 = 16;

    let ts = TraceScene::from_scene(&scene, width, height);

    let mut acc = Accumulator::new(width, height);
    let mut window = Window::new("path tracing", width, height, WindowOptions::default())?;
    let first_frame = Instant::now();
    let mut last_frame = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        scene.objects[0]
            .transform
            .set_rotation(Quat::from_axis_angle(
                Vec3A::Y.into(),
                first_frame.elapsed().as_secs_f32() * 0.8,
            ));

        acc.reset();
        acc.accumulate(&ts, &scene.camera, &scene, SPP);

        let elapsed = last_frame.elapsed().as_secs_f32();
        let fps = 1.0 / elapsed.max(0.001);
        last_frame = Instant::now();

        let image = acc.as_image();
        let data: Vec<u32> = image.iter().map(|c| c.to_u32()).collect();
        window.update_with_buffer(&data, width, height)?;
        let spp = acc.sample_count();
        window.set_title(&format!("path tracing — {spp} spp  {fps:.0} fps"));
    }

    Ok(())
}

fn key_light() -> Arc<DirectionalLight> {
    Arc::new(DirectionalLight::new(
        Vec3A::new(-0.4, -0.7, -0.8),
        Color::WHITE,
        1.6,
    ))
}

fn point_light() -> Arc<PointLight> {
    Arc::new(PointLight::new(
        Vec3A::new(2.0, 3.0, 3.5),
        Color::WHITE,
        25.0,
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
            Material::solid(Color::rgb(200, 120, 60)).with_roughness(0.4),
        ),
        Transform::default().with_translation(Vec3A::new(0.0, 0.0, 3.0)),
    )
}
