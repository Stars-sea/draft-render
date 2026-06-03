use crate::color::Color;
use fastrand::Rng;
use glam::Vec3A;

pub struct LightSample {
    pub wi: Vec3A,
    pub radiance: Color,
    pub dist: f32,
    pub pdf: f32,
}

pub trait Light: Send + Sync {
    fn sample(&self, point: Vec3A, rng: &mut Rng) -> Option<LightSample>;
    fn pdf(&self, point: Vec3A, wi: Vec3A) -> f32;
    fn is_delta(&self) -> bool;
}

// ── DirectionalLight ──────────────────────────────────────────────

pub struct DirectionalLight {
    direction: Vec3A,
    radiance: Color,
}

impl DirectionalLight {
    pub fn new(direction: Vec3A, color: Color, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            radiance: color * intensity,
        }
    }
}

impl Light for DirectionalLight {
    fn sample(&self, _point: Vec3A, _rng: &mut Rng) -> Option<LightSample> {
        Some(LightSample {
            wi: self.direction,
            radiance: self.radiance,
            dist: f32::INFINITY,
            pdf: 1.0,
        })
    }

    fn pdf(&self, _point: Vec3A, _wi: Vec3A) -> f32 {
        0.0
    }
    fn is_delta(&self) -> bool {
        true
    }
}

// ── PointLight ────────────────────────────────────────────────────

pub struct PointLight {
    position: Vec3A,
    color: Color,
    intensity: f32,
}

impl PointLight {
    pub fn new(position: Vec3A, color: Color, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
        }
    }
}

impl Light for PointLight {
    fn sample(&self, point: Vec3A, _rng: &mut Rng) -> Option<LightSample> {
        let to = self.position - point;
        let dist = to.length();
        let wi = to / dist;
        Some(LightSample {
            wi,
            radiance: self.color * self.intensity * (1.0 / (1.0 + dist * dist)),
            dist,
            pdf: 1.0,
        })
    }

    fn pdf(&self, _point: Vec3A, _wi: Vec3A) -> f32 {
        0.0
    }
    fn is_delta(&self) -> bool {
        true
    }
}
