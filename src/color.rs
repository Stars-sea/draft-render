use glam::Vec3A;
use std::ops::{Add, AddAssign, Mul, MulAssign};
use zerocopy::{self, FromBytes, Immutable, IntoBytes, KnownLayout};

/// Linear-RGB color backed by  Vec3A`.  HDR values (components > 1.0) are
/// supported; they are tone-mpped only when converting to display-ready u32.
///
/// Implements zerocopy traits — `&[Color]` ↔ `&[u8]` without copying.
#[derive(Debug, Clone, Copy, PartialEq, FromBytes, Immutable, IntoBytes, KnownLayout)]
#[repr(transparent)]
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

    /// Construct from linear RGB floats in `[0, 1]` (or HDR &gt; 1).
    /// Use this for data sources that already provide linear values
    /// (glTF baseColorFactor, spectral pre-integration).
    pub fn from_linear_rgb(r: f32, g: f32, b: f32) -> Color {
        Color(Vec3A::new(r, g, b))
    }

    /// alpha channel is ignored (kept for compatibility with PMX texture loading).
    pub fn argb(_a: u8, r: u8, g: u8, b: u8) -> Color {
        Color::rgb(r, g, b)
    }

    pub fn max_channel(&self) -> f32 {
        self.0.max_element()
    }

    /// Tone-mapped sRGB bytes `[R, G, B]`.
    pub fn to_srgb_bytes(self) -> [u8; 3] {
        let mapped = aces_tonemap(self.0);
        let gamma = mapped.powf(1.0 / 2.2);
        let c = (gamma.clamp(Vec3A::ZERO, Vec3A::ONE) * 255.0).round();
        [c.x as u8, c.y as u8, c.z as u8]
    }

    /// ACES filmic tone-map + sRGB gamma → ARGB `u32` for the framebuffer.
    pub fn to_u32(self) -> u32 {
        let [r, g, b] = self.to_srgb_bytes();
        0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
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
