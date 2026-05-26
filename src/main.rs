mod buffer;
mod color;
mod linalg;
mod scene;

use crate::color::Color;
use crate::linalg::Vec3f;
use crate::scene::MeshBuilder;
use crate::scene::Transform;
use crate::scene::{Camera, Scene, SceneObject};

use anyhow::Result;
use minifb::{Key, Window, WindowOptions};
use std::sync::Arc;

fn main() -> Result<()> {
    let (width, height) = (800, 600);

    let mut scene = Scene::new(Camera::default(), width, height);
    scene.add_object(generate_triangle());
    scene.add_object(generate_square());

    let mut window = Window::new("renderer", width, height, WindowOptions::default())?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        scene.render();
        window.update_with_buffer(scene.frame_data(), width, height)?;
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
    let s = 0.4; // half size
    let builder = MeshBuilder::new()
        .vertex(Vec3f::new(-s, -s, 2.9))
        .vertex(Vec3f::new(s, -s, 2.9))
        .vertex(Vec3f::new(s, s, 3.1))
        .vertex(Vec3f::new(-s, s, 3.1))
        .triangle(0, 1, 2)
        .triangle(0, 2, 3);

    SceneObject::new(Arc::new(builder.build()), Transform::default(), Color::RED)
}
