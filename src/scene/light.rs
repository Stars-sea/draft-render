use crate::color::Color;
use fastrand::Rng;
use glam::Vec3A;

/// Result of sampling a direction toward a light from a shading point.
pub struct LightSample {
    /// Normalized direction from shading point toward the light.
    pub wi: Vec3A,
    /// Incident radiance arriving at the shading point (already includes attenuation).
    pub radiance: Color,
    /// Distance to the sampled point (for shadow-ray t_max).
    pub dist: f32,
    /// Solid-angle PDF. Delta lights use 1.0 so the Monte Carlo estimator
    /// `albedo * radiance * cosθ / pdf` naturally degenerates to the delta form.
    pub pdf: f32,
}

/// GAMES101-style light trait: every light is sampled through a single `sample()` method.
pub trait Light: Send + Sync {
    /// Sample a direction from `point` toward the light.
    /// Delta lights return a deterministic sample; area lights pick a random surface point.
    fn sample(&self, point: Vec3A, rng: &mut Rng) -> Option<LightSample>;

    /// Solid-angle PDF of `wi` from `point` via `sample()`.
    /// Returns 0 for delta lights (BSDF cannot hit them).
    fn pdf(&self, point: Vec3A, wi: Vec3A) -> f32;

    /// Whether this light is a delta distribution (point / directional).
    fn is_delta(&self) -> bool;
}

// ── DirectionalLight ──────────────────────────────────────────────

pub struct DirectionalLight {
    pub direction: Vec3A,
    pub color: Color,
    pub intensity: f32,
}

impl DirectionalLight {
    pub fn new(direction: Vec3A, color: Color, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
        }
    }
}

impl Light for DirectionalLight {
    fn sample(&self, _point: Vec3A, _rng: &mut Rng) -> Option<LightSample> {
        Some(LightSample {
            wi: self.direction,
            radiance: self.color * self.intensity,
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
    pub position: Vec3A,
    pub color: Color,
    pub intensity: f32,
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
        let attenuation = 1.0 / (1.0 + dist * dist);
        Some(LightSample {
            wi,
            radiance: self.color * self.intensity * attenuation,
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

// ── QuadLight ─────────────────────────────────────────────────────

/// Rectangular area light defined by a corner and two edge vectors.
#[allow(dead_code)]
pub struct QuadLight {
    pub position: Vec3A,
    pub edge_u: Vec3A,
    pub edge_v: Vec3A,
    pub color: Color,
    pub intensity: f32,
    normal: Vec3A,
    inv_area: f32,
}

#[allow(dead_code)]
impl QuadLight {
    pub fn new(
        position: Vec3A,
        edge_u: Vec3A,
        edge_v: Vec3A,
        color: Color,
        intensity: f32,
    ) -> Self {
        let normal = edge_u.cross(edge_v);
        let area = normal.length();
        Self {
            position,
            edge_u,
            edge_v,
            color,
            intensity,
            normal: normal / area,
            inv_area: 1.0 / area,
        }
    }
}

impl Light for QuadLight {
    fn sample(&self, point: Vec3A, rng: &mut Rng) -> Option<LightSample> {
        let p = self.position + rng.f32() * self.edge_u + rng.f32() * self.edge_v;

        let to = p - point;
        let dist_sq = to.length_squared();
        let dist = dist_sq.sqrt();
        let wi = to / dist;

        let cos_light = (-wi).dot(self.normal);
        if cos_light <= 0.0 {
            return None;
        }
        let pdf = self.inv_area * dist_sq / cos_light;
        Some(LightSample {
            wi,
            radiance: self.color * self.intensity,
            dist,
            pdf,
        })
    }

    fn pdf(&self, point: Vec3A, wi: Vec3A) -> f32 {
        let denom = wi.dot(self.normal);
        if denom.abs() < 1e-6 {
            return 0.0;
        }
        let t = (self.position - point).dot(self.normal) / denom;
        if t <= 0.0 {
            return 0.0;
        }
        let p = point + wi * t;
        let local = p - self.position;
        let u = local.cross(self.edge_v).dot(self.normal) * self.inv_area;
        let v = self.edge_u.cross(local).dot(self.normal) * self.inv_area;
        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return 0.0;
        }
        let cos_light = (-wi).dot(self.normal);
        self.inv_area * t * t / cos_light
    }

    fn is_delta(&self) -> bool {
        false
    }
}
