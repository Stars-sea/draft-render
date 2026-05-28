use crate::color::Color;
use glam::Vec3A;

pub trait Light {
    fn direction(&self, point: Vec3A) -> Vec3A;
    fn color(&self) -> Color;
    fn intensity(&self) -> f32;
    fn attenuation(&self, _point: Vec3A) -> f32 {
        1.0
    }
}

pub struct DirectionalLight {
    pub direction: Vec3A,
    pub color: Color,
    pub intensity: f32,
}

impl DirectionalLight {
    pub fn new(direction: Vec3A, color: Color, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
        }
    }
}

impl Light for DirectionalLight {
    fn direction(&self, _point: Vec3A) -> Vec3A {
        self.direction
    }
    fn color(&self) -> Color {
        self.color
    }
    fn intensity(&self) -> f32 {
        self.intensity
    }
}

pub struct PointLight {
    pub position: Vec3A,
    pub color: Color,
    pub intensity: f32,
}

impl PointLight {
    pub fn new(position: Vec3A, color: Color, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
        }
    }
}

impl Light for PointLight {
    fn direction(&self, point: Vec3A) -> Vec3A {
        (self.position - point).normalize()
    }
    fn color(&self) -> Color {
        self.color
    }
    fn intensity(&self) -> f32 {
        self.intensity
    }
    fn attenuation(&self, point: Vec3A) -> f32 {
        let d = self.position.distance(point);
        1.0 / (d * d)
    }
}
