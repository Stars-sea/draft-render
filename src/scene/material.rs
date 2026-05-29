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
pub enum Material {
    Solid {
        diffuse: Color,
        specular: Color,
        shininess: f32,
    },
    Textured {
        texture: Arc<Texture>,
        specular: Color,
        shininess: f32,
    },
}

impl Material {
    pub fn solid(diffuse: Color) -> Self {
        Self::Solid {
            diffuse,
            specular: Color::WHITE,
            shininess: 32.0,
        }
    }

    pub fn textured(texture: Arc<Texture>) -> Self {
        Self::Textured {
            texture,
            specular: Color::WHITE,
            shininess: 32.0,
        }
    }

    pub fn with_specular(mut self, specular: Color) -> Self {
        match &mut self {
            Self::Solid { specular: s, .. } | Self::Textured { specular: s, .. } => *s = specular,
        }
        self
    }

    pub fn with_shininess(mut self, shininess: f32) -> Self {
        match &mut self {
            Self::Solid { shininess: sh, .. } | Self::Textured { shininess: sh, .. } => {
                *sh = shininess
            }
        }
        self
    }

    pub fn diffuse(&self, tex_uv: Vec2) -> Color {
        match self {
            Self::Solid { diffuse, .. } => *diffuse,
            Self::Textured { texture, .. } => texture.sample(tex_uv),
        }
    }

    pub fn specular(&self) -> Color {
        match self {
            Self::Solid { specular, .. } | Self::Textured { specular, .. } => *specular,
        }
    }

    pub fn shininess(&self) -> f32 {
        match self {
            Self::Solid { shininess, .. } | Self::Textured { shininess, .. } => *shininess,
        }
    }
}
