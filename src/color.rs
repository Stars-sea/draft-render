use glam::{U8Vec4, UVec4, Vec4};
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(U8Vec4);

impl Color {
    pub const TRANSPARENT: Color = Color(U8Vec4::ZERO);
    pub const BLACK: Color = Color(U8Vec4::new(0, 0, 0, 0xFF));
    pub const WHITE: Color = Color(U8Vec4::new(0xFF, 0xFF, 0xFF, 0xFF));
    pub const RED: Color = Color(U8Vec4::new(0xFF, 0, 0, 0xFF));
    pub const GREEN: Color = Color(U8Vec4::new(0, 0xFF, 0, 0xFF));
    pub const BLUE: Color = Color(U8Vec4::new(0, 0, 0xFF, 0xFF));

    pub fn argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color(U8Vec4::new(r, g, b, a))
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::argb(0xFF, r, g, b)
    }

    pub fn to_u32(self) -> u32 {
        (self.0.w as u32) << 24 | (self.0.x as u32) << 16 | (self.0.y as u32) << 8 | self.0.z as u32
    }

    pub fn a(&self) -> u8 {
        self.0.w
    }
    pub fn r(&self) -> u8 {
        self.0.x
    }
    pub fn g(&self) -> u8 {
        self.0.y
    }
    pub fn b(&self) -> u8 {
        self.0.z
    }

    pub fn average(samples: &[Color]) -> Color {
        let n = samples.len() as u32;
        let sum = samples
            .iter()
            .map(|c| c.0.as_uvec4())
            .reduce(|a, b| a + b)
            .unwrap_or(UVec4::ZERO);
        Color((sum / n).as_u8vec4())
    }
}

// ---- Add ----

impl Add<&Color> for &Color {
    type Output = Color;

    fn add(self, rhs: &Color) -> Color {
        Color(self.0.saturating_add(rhs.0))
    }
}

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        &self + &rhs
    }
}

impl Add<Color> for &Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        self + &rhs
    }
}

impl Add<&Color> for Color {
    type Output = Color;
    fn add(self, rhs: &Color) -> Color {
        &self + rhs
    }
}

impl AddAssign<&Color> for Color {
    fn add_assign(&mut self, rhs: &Color) {
        *self = &*self + rhs;
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        *self = &*self + &rhs;
    }
}

// ---- Mul<f32> ----

impl Mul<f32> for &Color {
    type Output = Color;

    fn mul(self, factor: f32) -> Color {
        let v = (self.0.as_vec4() * factor).clamp(Vec4::ZERO, Vec4::splat(255.0));
        Color(v.as_u8vec4())
    }
}

impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, factor: f32) -> Color {
        &self * factor
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, factor: f32) {
        *self = &*self * factor;
    }
}

// ---- Mul<Color> (modulate) ----

impl Mul<&Color> for &Color {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Color {
        let r = self.0.as_u16vec4() * rhs.0.as_u16vec4();
        Color((r / 0xFF).as_u8vec4())
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        &self * &rhs
    }
}

impl Mul<Color> for &Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        self * &rhs
    }
}

impl Mul<&Color> for Color {
    type Output = Color;
    fn mul(self, rhs: &Color) -> Color {
        &self * rhs
    }
}

impl MulAssign<&Color> for Color {
    fn mul_assign(&mut self, rhs: &Color) {
        *self = &*self * rhs;
    }
}

impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        *self = &*self * &rhs;
    }
}
