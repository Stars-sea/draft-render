mod accumulator;
mod bsdf;
mod integrator;
mod sampling;
mod scene;

pub use accumulator::Accumulator;
pub use scene::TraceScene;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::scene::{
        Camera, Material, MeshBuilder, PointLight, Scene, SceneObject, SubMesh, Transform,
    };
    use glam::{Vec2, Vec3A};
    use std::sync::Arc;

    fn make_scene() -> Scene {
        let camera = Camera::default().with_position(Vec3A::ZERO);
        let mut scene = Scene::new(camera);

        scene.add_light(Arc::new(PointLight::new(
            Vec3A::new(0.0, 2.0, 0.0),
            Color::WHITE,
            31.4,
        )));

        let mesh = MeshBuilder::new()
            .vertex(Vec3A::new(-1.0, -1.0, 2.0))
            .vertex(Vec3A::new(1.0, -1.0, 2.0))
            .vertex(Vec3A::new(1.0, 1.0, 2.0))
            .vertex(Vec3A::new(-1.0, 1.0, 2.0))
            .triangle(0, 2, 1)
            .triangle(0, 3, 2)
            .build();

        scene.add_object(SceneObject::single(
            SubMesh::new(Arc::new(mesh), Material::solid(Color::WHITE)),
            Transform::default(),
        ));

        scene
    }

    #[test]
    fn trace_pixel_center_hit_lit() {
        let scene = make_scene();
        let ts = TraceScene::from_scene(&scene, 64, 64);
        let c = ts.trace_pixel(&scene.camera, 32, 32, 0);
        assert!(c.r() > 0.0 || c.g() > 0.0 || c.b() > 0.0);
    }

    #[test]
    fn trace_pixel_corner_miss_sky() {
        let scene = make_scene();
        let ts = TraceScene::from_scene(&scene, 64, 64);
        let c = ts.trace_pixel(&scene.camera, 0, 0, 0);
        assert!(c.b() > 0.0);
    }

    #[test]
    fn accumulator_converges() {
        let scene = make_scene();
        let ts = TraceScene::from_scene(&scene, 64, 64);
        let mut acc = Accumulator::new(64, 64);
        for _ in 0..16 {
            acc.accumulate(&ts, &scene.camera);
        }
        let img = acc.as_image();
        let center = img[32 * 64 + 32];
        assert!(center.r() > 0.0 || center.g() > 0.0 || center.b() > 0.0);
    }

    #[test]
    fn primary_ray_center_goes_forward() {
        let camera = Camera::default();
        let ray = camera.primary_ray(32, 32, 64, 64, Vec2::splat(0.5));
        assert!((ray.origin.x - 0.0).abs() < 1e-6);
        assert!((ray.origin.y - 0.0).abs() < 1e-6);
        assert!((ray.origin.z - 0.0).abs() < 1e-6);
        assert!(ray.direction.z > 0.9);
    }
}
