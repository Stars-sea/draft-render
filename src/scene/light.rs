use crate::color::Color;
use crate::linalg::Vec3f;

pub trait Light {
    fn direction(&self, point: Vec3f) -> Vec3f;
    fn color(&self) -> Color;
    fn intensity(&self) -> f32;
    fn attenuation(&self, _distance: f32) -> f32 {
        1.0
    }
}

pub struct DirectionalLight {
    pub direction: Vec3f,
    pub color: Color,
    pub intensity: f32,
}

impl DirectionalLight {
    pub fn new(direction: Vec3f, color: Color, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
        }
    }
}

impl Light for DirectionalLight {
    fn direction(&self, _point: Vec3f) -> Vec3f {
        self.direction
    }
    fn color(&self) -> Color {
        self.color
    }
    fn intensity(&self) -> f32 {
        self.intensity
    }
}
