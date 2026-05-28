use crate::color::Color;
use crate::linalg::{Vec2f, Vec3f};
use crate::scene::Light;

use num_traits::Signed;
use std::sync::Arc;

pub trait Shader {
    fn shade(&self, uv: Vec2f, z: f32, normal: Vec3f, world_pos: Vec3f) -> Color;
}

pub struct BlinnPhongShader {
    pub color: Color,
    pub ambient: Color,
    pub specular: Color,
    pub shininess: f32,
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
}

impl BlinnPhongShader {
    pub fn new(color: Color, lights: Vec<Arc<dyn Light + Send + Sync>>) -> Self {
        Self {
            color,
            ambient: color * 0.1,
            specular: Color::WHITE,
            shininess: 32.0,
            lights,
        }
    }
}

impl Shader for BlinnPhongShader {
    fn shade(&self, _uv: Vec2f, _z: f32, normal: Vec3f, world_pos: Vec3f) -> Color {
        let mut diff_light = Color::BLACK;
        let mut spec_light = Color::BLACK;

        for light in &self.lights {
            let l = light.direction(world_pos);
            let n_dot_l = normal.dot(&l).max(0.0);
            if n_dot_l.is_negative() {
                continue;
            }

            let h = (l - Vec3f::unit_z()).normalize();
            let n_dot_h = normal.dot(&h).max(0.0);
            if n_dot_h <= 0.0 {
                continue;
            }
            let i = light.intensity();
            let attn = light.attenuation(world_pos);
            diff_light = diff_light + light.color() * (i * attn * n_dot_l);
            spec_light = spec_light + light.color() * (i * attn * n_dot_h.powf(self.shininess));
        }

        self.ambient + self.color * diff_light + self.specular * spec_light
    }
}
