use crate::color::Color;
use crate::pipeline::path_tracing::sampling::{cosine_sample_hemisphere, orthonormal_basis};
use fastrand::Rng;
use glam::Vec3A;
use std::f32::consts::FRAC_1_PI;

pub struct BsdfSample {
    pub wi: Vec3A,
    pub pdf: f32,
}

/// Evaluate Cook-Torrance GGX + diffuse BSDF.
pub fn evaluate(
    wo: Vec3A,
    wi: Vec3A,
    n: Vec3A,
    albedo: Color,
    roughness: f32,
    metallic: f32,
) -> Color {
    let cos_wo = n.dot(wo).max(0.0);
    let cos_wi = n.dot(wi).max(0.0);
    if cos_wo <= 0.0 || cos_wi <= 0.0 {
        return Color::BLACK;
    }

    let f0 = Color::WHITE * 0.04 * (1.0 - metallic) + albedo * metallic;

    let diffuse = albedo * FRAC_1_PI * (1.0 - metallic);

    let wh = (wo + wi).normalize();
    let cos_wh = n.dot(wh).max(1e-6);
    let cos_wo_wh = wo.dot(wh).max(1e-6);

    let alpha = roughness * roughness;
    let d = ggx_d(cos_wh, alpha);
    let g = ggx_g(cos_wo, cos_wi, alpha);
    let f = fresnel_schlick(cos_wo_wh, f0);

    let denom = 4.0 * cos_wo * cos_wi;
    let specular = if denom > 1e-6 { f * (d * g / denom) } else { Color::BLACK };

    diffuse + specular
}

/// Sample a BSDF direction. Returns the sampled direction and its PDF.
pub fn sample(
    wo: Vec3A,
    n: Vec3A,
    roughness: f32,
    metallic: f32,
    rng: &mut Rng,
) -> BsdfSample {
    let cos_wo = n.dot(wo).max(0.0);
    let f0_avg = 0.04 * (1.0 - metallic) + metallic;
    let f = fresnel_schlick_f32(cos_wo, f0_avg);
    let spec_prob = (0.05 + 0.45 * f).min(0.95);

    if rng.f32() < spec_prob {
        let (wi, pdf) = ggx_sample(wo, n, roughness, rng);
        BsdfSample { wi, pdf: pdf * spec_prob }
    } else {
        let wi = cosine_sample_hemisphere(n, rng);
        let cos_theta = n.dot(wi).max(0.0);
        let diff_pdf = cos_theta * FRAC_1_PI * (1.0 - spec_prob);
        BsdfSample { wi, pdf: diff_pdf }
    }
}

/// GGX (Trowbridge-Reitz) normal distribution.
fn ggx_d(cos_theta_h: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let denom = cos_theta_h * cos_theta_h * (a2 - 1.0) + 1.0;
    a2 / (std::f32::consts::PI * denom * denom)
}

/// Smith geometry shadowing-masking, separable form: G(wo,wi) = G1(wo) * G1(wi).
fn ggx_g(cos_wo: f32, cos_wi: f32, alpha: f32) -> f32 {
    ggx_g1(cos_wo, alpha) * ggx_g1(cos_wi, alpha)
}

fn ggx_g1(cos_v: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let tan_sq = (1.0 - cos_v * cos_v) / (cos_v * cos_v).max(1e-12);
    2.0 / (1.0 + (1.0 + a2 * tan_sq).sqrt())
}

/// Schlick Fresnel (RGB).
fn fresnel_schlick(cos_theta: f32, f0: Color) -> Color {
    let p = (1.0 - cos_theta).powi(5);
    Color(f0.0 + (Vec3A::ONE - f0.0) * p)
}

/// Schlick Fresnel (scalar, for sampling probability).
fn fresnel_schlick_f32(cos_theta: f32, f0: f32) -> f32 {
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

/// Sample half-vector from GGX distribution, then reflect wo about wh.
fn ggx_sample(wo: Vec3A, n: Vec3A, roughness: f32, rng: &mut Rng) -> (Vec3A, f32) {
    let alpha = roughness * roughness;
    let a2 = alpha * alpha;

    let u1 = rng.f32();
    let u2 = rng.f32();
    let phi = 2.0 * std::f32::consts::PI * u1;
    let cos_theta = ((1.0 - u2) / (1.0 + (a2 - 1.0) * u2)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let (t, b) = orthonormal_basis(n);
    let wh = t * (phi.cos() * sin_theta) + b * (phi.sin() * sin_theta) + n * cos_theta;

    let wi = 2.0 * wo.dot(wh) * wh - wo;
    if n.dot(wi) <= 0.0 {
        return (wi, 0.0); // invalid, will be rejected
    }

    let cos_theta_h = n.dot(wh).max(1e-6);
    let d = ggx_d(cos_theta_h, alpha);
    let pdf = d * cos_theta_h / (4.0 * wo.dot(wh).max(1e-6));
    (wi, pdf)
}

/// Combined BSDF PDF for direction `wi` given `wo`, matching `sample()`.
pub fn pdf(wo: Vec3A, wi: Vec3A, n: Vec3A, roughness: f32, metallic: f32) -> f32 {
    let cos_wi = n.dot(wi).max(0.0);
    if cos_wi <= 0.0 {
        return 0.0;
    }
    let cos_wo = n.dot(wo).max(0.0);
    let f0_avg = 0.04 * (1.0 - metallic) + metallic;
    let spec_prob = (0.05 + 0.45 * fresnel_schlick_f32(cos_wo, f0_avg)).min(0.95);

    let diff_pdf = cos_wi * FRAC_1_PI;

    let wh = (wo + wi).normalize();
    let cos_theta_h = n.dot(wh).max(1e-6);
    let alpha = roughness * roughness;
    let d = ggx_d(cos_theta_h, alpha);
    let spec_pdf = d * cos_theta_h / (4.0 * wo.dot(wh).max(1e-6));

    diff_pdf * (1.0 - spec_prob) + spec_pdf * spec_prob
}
