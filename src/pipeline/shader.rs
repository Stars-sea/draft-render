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
        let diffuse = material.diffuse(tex_uv);
        let specular = material.specular();
        let shininess = material.shininess();

        let mut diff_light = Color::BLACK;
        let mut spec_light = Color::BLACK;

        for light in &self.lights {
            let l = light.direction(world_pos);
            let n_dot_l = normal.dot(l);
            if n_dot_l <= 0.0 {
                continue;
            }

            let v = -world_pos.normalize();
            let h = (l + v).normalize();
            let n_dot_h = normal.dot(h);
            if n_dot_h <= 0.0 {
                continue;
            }
            let i = light.intensity();
            let attn = light.attenuation(world_pos);
            diff_light += light.color() * (i * attn * n_dot_l);
            spec_light += light.color() * (i * attn * n_dot_h.powf(shininess));
        }

        diffuse * 0.1 + diffuse * diff_light + specular * spec_light
    }
}
