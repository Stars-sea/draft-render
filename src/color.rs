use glam::Vec3A;
use std::ops::{Add, AddAssign, Mul, MulAssign};

/// Linear-RGB colour backed by `Vec3A`.  HDR values (components > 1.0) are
/// supported; they are tone-mapped only when converting to display-ready u32.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub Vec3A);

impl Color {
    pub const BLACK: Color = Color(Vec3A::ZERO);
    pub const WHITE: Color = Color(Vec3A::ONE);

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color(Vec3A::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        ))
    }

    /// alpha channel is ignored (kept for compatibility with PMX texture loading).
    pub fn argb(_a: u8, r: u8, g: u8, b: u8) -> Color {
        Color::rgb(r, g, b)
    }

    #[allow(dead_code)]
    pub fn r(&self) -> f32 {
        self.0.x
    }
    #[allow(dead_code)]
    pub fn g(&self) -> f32 {
        self.0.y
    }
    #[allow(dead_code)]
    pub fn b(&self) -> f32 {
        self.0.z
    }

    /// ACES filmic tone-map + sRGB gamma → ARGB `u32` for the framebuffer.
    pub fn to_u32(self) -> u32 {
        let mapped = aces_tonemap(self.0);
        let gamma = mapped.powf(1.0 / 2.2);
        let c = (gamma.clamp(Vec3A::ZERO, Vec3A::ONE) * 255.0).round();
        0xFF00_0000 | ((c.x as u32) << 16) | ((c.y as u32) << 8) | (c.z as u32)
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        Color(self.0.lerp(other.0, t))
    }
}

// ---- Add ----

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        Color(self.0 + rhs.0)
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.0 += rhs.0;
    }
}

// ---- Mul<f32> ----

impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, factor: f32) -> Color {
        Color(self.0 * factor)
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, factor: f32) {
        self.0 *= factor;
    }
}

// ---- Mul<Color> (modulate) ----

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        Color(self.0 * rhs.0)
    }
}

impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.0 *= rhs.0;
    }
}

/// ACES filmic tone-mapping (Narkowicz 2015 fit).
fn aces_tonemap(x: Vec3A) -> Vec3A {
    let a = Vec3A::splat(2.51);
    let b = Vec3A::splat(0.03);
    let c = Vec3A::splat(2.43);
    let d = Vec3A::splat(0.59);
    let e = Vec3A::splat(0.14);
    (x * (a * x + b)) / (x * (c * x + d) + e)
}
