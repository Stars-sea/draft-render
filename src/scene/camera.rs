use crate::geometry::Ray;
use glam::{Quat, Vec2, Vec3A};

#[derive(Clone, Copy)]
pub struct Camera {
    position: Vec3A,
    rotation: Quat,
    fov: f32,
}

impl Camera {
    pub fn new(position: Vec3A, rotation: Quat, fov: f32) -> Self {
        Self {
            position,
            rotation,
            fov,
        }
    }

    #[allow(dead_code)]
    pub fn with_position(mut self, position: Vec3A) -> Self {
        self.position = position;
        self
    }

    pub fn primary_ray(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        jitter: Vec2,
    ) -> Ray {
        let aspect = width as f32 / height as f32;
        let half_h = (self.fov / 2.0).tan();
        let half_w = half_h * aspect;

        let nx = (x as f32 + jitter.x) / width as f32;
        let ny = (y as f32 + jitter.y) / height as f32;

        let sx = (2.0 * nx - 1.0) * half_w;
        let sy = (1.0 - 2.0 * ny) * half_h;

        let dir = self.rotation * Vec3A::new(sx, sy, 1.0);
        Ray::new(self.position, dir)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3A::ZERO, Quat::IDENTITY, std::f32::consts::FRAC_PI_3)
    }
}
