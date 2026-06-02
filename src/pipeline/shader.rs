use crate::color::Color;
use crate::scene::{Light, Material};
use glam::{Vec2, Vec3A};
use std::sync::Arc;

pub trait Shader {
    fn shade(
        &self,
        material: &Material,
        tex_uv: Vec2,
        z: f32,
        normal: Vec3A,
        world_pos: Vec3A,
    ) -> Color;
}

pub struct BlinnPhongShader {
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
}

impl BlinnPhongShader {
    pub fn new(lights: Vec<Arc<dyn Light + Send + Sync>>) -> Self {
        Self { lights }
    }
}

impl Shader for BlinnPhongShader {
    fn shade(
        &self,
        material: &Material,
        tex_uv: Vec2,
        _z: f32,
        normal: Vec3A,
        world_pos: Vec3A,
    ) -> Color {
        let albedo = material.albedo_at(tex_uv);
        let v = -world_pos.normalize();
        let normal = if material.double_sided() && normal.dot(v) < 0.0 {
            -normal
        } else {
            normal
        };

        let mut diff_light = Color::BLACK;

        for light in &self.lights {
            let l = light.direction(world_pos);
            let n_dot_l = normal.dot(l);
            if n_dot_l <= 0.0 {
                continue;
            }
            let i = light.intensity();
            let attn = light.attenuation(world_pos);
            diff_light += light.color() * (i * attn * n_dot_l);
        }

        albedo * 0.1 + albedo * diff_light
    }
}
