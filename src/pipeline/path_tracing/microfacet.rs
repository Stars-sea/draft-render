use crate::color::Color;
use super::math::orthonormal_basis;
use fastrand::Rng;
use glam::Vec3A;
use std::f32::consts::PI;

/// Probability of selecting the specular lobe, used by both `sample()` and `pdf()`.
pub(super) fn spec_prob(cos_wo: f32, metallic: f32) -> f32 {
    let f0 = 0.04 * (1.0 - metallic) + metallic;
    (0.05 + 0.45 * fresnel_schlick_f32(cos_wo, f0)).min(0.95)
}

/// GGX (Trowbridge-Reitz) normal distribution.
pub(super) fn ggx_d(cos_theta_h: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let denom = cos_theta_h * cos_theta_h * (a2 - 1.0) + 1.0;
    a2 / (PI * denom * denom)
}

/// Smith geometry shadowing-masking, separable form: G(wo,wi) = G1(wo) * G1(wi).
pub(super) fn ggx_g(cos_wo: f32, cos_wi: f32, alpha: f32) -> f32 {
    ggx_g1(cos_wo, alpha) * ggx_g1(cos_wi, alpha)
}

fn ggx_g1(cos_v: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let tan_sq = (1.0 - cos_v * cos_v) / (cos_v * cos_v).max(1e-12);
    2.0 / (1.0 + (1.0 + a2 * tan_sq).sqrt())
}

/// Schlick Fresnel (RGB).
pub(super) fn fresnel_schlick(cos_theta: f32, f0: Color) -> Color {
    let p = (1.0 - cos_theta).powi(5);
    Color(f0.0 + (Vec3A::ONE - f0.0) * p)
}

/// Schlick Fresnel (scalar, for sampling probability).
fn fresnel_schlick_f32(cos_theta: f32, f0: f32) -> f32 {
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

/// Sample half-vector from GGX distribution, then reflect wo about wh.
pub(super) fn ggx_sample(
    wo: Vec3A,
    n: Vec3A,
    roughness: f32,
    rng: &mut Rng,
) -> (Vec3A, f32) {
    let alpha = roughness * roughness;
    let a2 = alpha * alpha;

    let u1 = rng.f32();
    let u2 = rng.f32();
    let phi = 2.0 * PI * u1;
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
