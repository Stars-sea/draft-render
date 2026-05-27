use crate::color::Color;
use crate::linalg::Vec3f;
use crate::scene::Light;

use num_traits::ConstZero;
use std::sync::Arc;

pub trait Shader {
    fn shade(&self, u: f32, v: f32, z: f32, normal: Vec3f) -> Color;
}

pub struct BlinnPhongShader {
    pub color: Color,
    pub specular: Color,
    pub shininess: f32,
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
}

impl BlinnPhongShader {
    pub fn new(color: Color, lights: Vec<Arc<dyn Light + Send + Sync>>) -> Self {
        Self {
            color,
            specular: Color::WHITE,
            shininess: 32.0,
            lights,
        }
    }
}

impl Shader for BlinnPhongShader {
    fn shade(&self, _u: f32, _v: f32, _z: f32, normal: Vec3f) -> Color {
        let v = -Vec3f::unit_z();
        let origin = Vec3f::ZERO;

        let mut diff_light = Color::BLACK;
        let mut spec_light = Color::BLACK;

        for light in &self.lights {
            let l = light.direction(origin).normalize();
            let n_dot_l = normal.dot(&l).max(0.0);
            if n_dot_l > 0.0 {
                let h = (l + v).normalize();
                let n_dot_h = normal.dot(&h).max(0.0);
                let i = light.intensity();
                diff_light = diff_light + light.color() * (i * n_dot_l);
                spec_light = spec_light + light.color() * (i * n_dot_h.powf(self.shininess));
            }
        }

        self.color * diff_light + self.specular * spec_light
    }
}
