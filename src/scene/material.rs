use crate::color::Color;
use glam::Vec2;
use std::sync::Arc;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Color>,
}

impl Texture {
    pub fn new(width: usize, height: usize, data: Vec<Color>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn checkerboard(width: usize, height: usize, size: usize, c1: Color, c2: Color) -> Self {
        let mut data = vec![Color::BLACK; width * height];
        for y in 0..height {
            for x in 0..width {
                let cx = (x / size).is_multiple_of(2);
                let cy = (y / size).is_multiple_of(2);
                data[y * width + x] = if cx == cy { c1 } else { c2 };
            }
        }
        Self {
            width,
            height,
            data,
        }
    }

    pub fn sample(&self, uv: Vec2) -> Color {
        let uv = uv.fract();
        let (w, h) = (self.width, self.height);
        let tx = w as f32 * uv.x;
        let ty = h as f32 * uv.y;

        let x0 = tx as usize;
        let y0 = ty as usize;
        let x1 = if x0 + 1 < w { x0 + 1 } else { x0 };
        let y1 = if y0 + 1 < h { y0 + 1 } else { y0 };
        let fx = tx.fract();
        let fy = ty.fract();

        let i00 = y0 * w + x0;
        let i10 = y0 * w + x1;
        let i01 = y1 * w + x0;
        let i11 = y1 * w + x1;

        let top = self.data[i00].lerp(&self.data[i10], fx);
        let bot = self.data[i01].lerp(&self.data[i11], fx);
        top.lerp(&bot, fy)
    }
}

#[derive(Clone)]
pub struct Material {
    pub albedo: Color,
    pub emission: Color,
    pub double_sided: bool,
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn solid(albedo: Color) -> Self {
        Self {
            albedo,
            emission: Color::BLACK,
            double_sided: false,
            texture: None,
        }
    }

    pub fn textured(texture: Arc<Texture>) -> Self {
        Self {
            albedo: Color::WHITE,
            emission: Color::BLACK,
            double_sided: false,
            texture: Some(texture),
        }
    }

    pub fn with_double_sided(mut self) -> Self {
        self.double_sided = true;
        self
    }

    /// Evaluate the surface albedo at the given texture coordinate.
    pub fn albedo_at(&self, uv: Vec2) -> Color {
        match &self.texture {
            Some(tex) => tex.sample(uv),
            None => self.albedo,
        }
    }

    pub fn double_sided(&self) -> bool {
        self.double_sided
    }
}
