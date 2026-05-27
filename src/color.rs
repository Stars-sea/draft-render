#![allow(unused)]

use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
pub struct Color(pub u32);

impl Color {
    pub const TRANSPARENT: Color = Color(0);
    pub const BLACK: Color = Color(0xFF000000);
    pub const WHITE: Color = Color(0xFFFFFFFF);
    pub const RED: Color = Color(0xFFFF0000);
    pub const GREEN: Color = Color(0xFF00FF00);
    pub const BLUE: Color = Color(0xFF0000FF);

    pub fn argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        let hex = (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32) << 0;
        Color(hex)
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::argb(0xFF, r, g, b)
    }

    pub fn hex(&self) -> u32 {
        self.0
    }

    pub fn a(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    pub fn r(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.0 >> 0) & 0xFF) as u8
    }

    /// Average color
    pub fn average(samples: &[Color]) -> Color {
        let n = samples.len() as u32;
        let (mut r, mut g, mut b, mut a) = (0u32, 0u32, 0u32, 0u32);
        for c in samples {
            r += c.r() as u32;
            g += c.g() as u32;
            b += c.b() as u32;
            a += c.a() as u32;
        }
        Color::argb((a / n) as u8, (r / n) as u8, (g / n) as u8, (b / n) as u8)
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color::rgb(
            self.r().saturating_add(rhs.r()),
            self.g().saturating_add(rhs.g()),
            self.b().saturating_add(rhs.b()),
        )
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, factor: f32) -> Self::Output {
        Color::rgb(
            (self.r() as f32 * factor).min(255.0) as u8,
            (self.g() as f32 * factor).min(255.0) as u8,
            (self.b() as f32 * factor).min(255.0) as u8,
        )
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color::rgb(
            ((self.r() as u16 * rhs.r() as u16) / 255) as u8,
            ((self.g() as u16 * rhs.g() as u16) / 255) as u8,
            ((self.b() as u16 * rhs.b() as u16) / 255) as u8,
        )
    }
}
