use crate::color::Color;
use glam::Vec2;
use std::sync::Arc;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Color>,
    pub alpha: Vec<f32>,
}

impl Texture {
    pub fn new(width: usize, height: usize, data: Vec<Color>, alpha: Vec<f32>) -> Self {
        Self {
            width,
            height,
            data,
            alpha,
        }
    }

    pub fn checkerboard(width: usize, height: usize, size: usize, c1: Color, c2: Color) -> Self {
        let n = width * height;
        let mut data = vec![Color::BLACK; n];
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
            alpha: vec![1.0; n],
        }
    }

    /// Bilinear sample returning (color, alpha).
    pub fn sample(&self, uv: Vec2) -> (Color, f32) {
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

        let top_c = self.data[i00].lerp(&self.data[i10], fx);
        let bot_c = self.data[i01].lerp(&self.data[i11], fx);
        let color = top_c.lerp(&bot_c, fy);

        let top_a = lerp_f32(self.alpha[i00], self.alpha[i10], fx);
        let bot_a = lerp_f32(self.alpha[i01], self.alpha[i11], fx);
        let alpha = lerp_f32(top_a, bot_a, fy);

        (color, alpha)
    }

    /// Heuristic: returns `true` if this is likely an SPH / effect map
    /// (> 90% of pixels have both near-zero RGB and near-zero alpha).
    pub fn is_dark_effect(&self) -> bool {
        let n = self.data.len();
        let step = (n / 500).max(1);
        let (mut dark, mut total) = (0usize, 0usize);
        for i in (0..n).step_by(step) {
            if self.data[i].0.max_element() < 0.02 && self.alpha[i] < 0.1 {
                dark += 1;
            }
            total += 1;
        }
        dark as f32 / total as f32 > 0.9
    }
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[derive(Clone)]
pub struct Material {
    pub albedo: Color,
    #[allow(dead_code)]
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

    /// Evaluate the surface albedo and alpha at the given texture coordinate.
    pub fn albedo_alpha_at(&self, uv: Vec2) -> (Color, f32) {
        match &self.texture {
            Some(tex) => tex.sample(uv),
            None => (self.albedo, 1.0),
        }
    }

    #[allow(dead_code)]
    pub fn double_sided(&self) -> bool {
        self.double_sided
    }
}
