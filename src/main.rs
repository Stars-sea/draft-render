mod buffer;
mod color;
mod linalg;
mod scene;

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

    let mut window = Window::new("renderer", width, height, WindowOptions::default())?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        scene.render();
        window.update_with_buffer(scene.frame_data(), width, height)?;
    }

    Ok(())
}

fn generate_triangle() -> SceneObject {
    let mut builder = MeshBuilder::new();
    builder
        .vertex(Vec3f::new(-0.5, -0.5, -3.0))
        .vertex(Vec3f::new(0.5, -0.5, -3.0))
        .vertex(Vec3f::new(0.0, 0.5, -3.0))
        .triangle(0, 1, 2);
    let mesh = builder.build();

    SceneObject::new(Arc::new(mesh), Transform::default())
}
