use super::microfacet;
use super::sampling::cosine_sample_hemisphere;
use crate::color::Color;
use fastrand::Rng;
use glam::Vec3A;
use std::f32::consts::FRAC_1_PI;

pub struct BsdfSample {
    pub wi: Vec3A,
    pub pdf: f32,
}

pub struct Bsdf {
    albedo: Color,
    roughness: f32,
    metallic: f32,
}

impl Bsdf {
    pub fn new(albedo: Color, roughness: f32, metallic: f32) -> Self {
        Self {
            albedo,
            roughness,
            metallic,
        }
    }

    /// Evaluate Cook-Torrance GGX + diffuse BSDF.
    pub fn evaluate(&self, wo: Vec3A, wi: Vec3A, n: Vec3A) -> Color {
        let cos_wo = n.dot(wo).max(0.0);
        let cos_wi = n.dot(wi).max(0.0);
        if cos_wo <= 0.0 || cos_wi <= 0.0 {
            return Color::BLACK;
        }

        let f0 = Color::WHITE * 0.04 * (1.0 - self.metallic) + self.albedo * self.metallic;

        let diffuse = self.albedo * FRAC_1_PI * (1.0 - self.metallic);

        let wh = (wo + wi).normalize();
        let cos_wh = n.dot(wh).max(1e-6);
        let cos_wo_wh = wo.dot(wh).max(1e-6);

        let alpha = self.roughness * self.roughness;
        let d = microfacet::ggx_d(cos_wh, alpha);
        let g = microfacet::ggx_g(cos_wo, cos_wi, alpha);
        let f = microfacet::fresnel_schlick(cos_wo_wh, f0);

        let denom = 4.0 * cos_wo * cos_wi;
        let specular = if denom > 1e-6 {
            f * (d * g / denom)
        } else {
            Color::BLACK
        };

        diffuse + specular
    }

    /// Sample a BSDF direction. GGX is retried until a valid (above-surface)
    /// direction is produced, so the MIS PDF from `pdf()` always matches.
    pub fn sample(&self, wo: Vec3A, n: Vec3A, rng: &mut Rng) -> BsdfSample {
        let prob = microfacet::spec_prob(n.dot(wo).max(0.0), self.metallic);

        let wi = if rng.f32() < prob {
            loop {
                let (ggx_wi, ggx_pdf) = microfacet::ggx_sample(wo, n, self.roughness, rng);
                if ggx_pdf > 0.0 && n.dot(ggx_wi) > 0.0 {
                    break ggx_wi;
                }
            }
        } else {
            cosine_sample_hemisphere(n, rng)
        };

        BsdfSample {
            wi,
            pdf: self.pdf(wo, wi, n),
        }
    }

    /// Combined BSDF PDF for direction `wi` given `wo`, matching `sample()`.
    pub fn pdf(&self, wo: Vec3A, wi: Vec3A, n: Vec3A) -> f32 {
        let cos_wi = n.dot(wi).max(0.0);
        if cos_wi <= 0.0 {
            return 0.0;
        }
        let prob = microfacet::spec_prob(n.dot(wo).max(0.0), self.metallic);
        let diff_pdf = cos_wi * FRAC_1_PI;

        let wh = (wo + wi).normalize();
        let alpha = self.roughness * self.roughness;
        let d = microfacet::ggx_d(n.dot(wh).max(1e-6), alpha);
        let spec_pdf = d * n.dot(wh).max(1e-6) / (4.0 * wo.dot(wh).max(1e-6));

        diff_pdf * (1.0 - prob) + spec_pdf * prob
    }
}
