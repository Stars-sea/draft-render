use crate::color::Color;
use crate::linalg::Vec3f;

pub trait Light {
    fn direction(&self, point: Vec3f) -> Vec3f;
    fn color(&self) -> Color;
    fn intensity(&self) -> f32;
    fn attenuation(&self, point: Vec3f) -> f32 {
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

/// 点光源 —— 位置固定，方向指向表面点，距离平方衰减。
pub struct PointLight {
    pub position: Vec3f,
    pub color: Color,
    pub intensity: f32,
}

impl PointLight {
    pub fn new(position: Vec3f, color: Color, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
        }
    }
}

impl Light for PointLight {
    fn direction(&self, point: Vec3f) -> Vec3f {
        (self.position - point).normalize()
    }
    fn color(&self) -> Color {
        self.color
    }
    fn intensity(&self) -> f32 {
        self.intensity
    }
    fn attenuation(&self, point: Vec3f) -> f32 {
        let distance = (self.position - point).norm();
        1.0 / (distance * distance)
    }
}
