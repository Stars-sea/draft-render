use super::math::orthonormal_basis;
use fastrand::Rng;
use glam::{Vec2, Vec3A};
use std::f32::consts::PI;

/// Van der Corput / radical inverse in base `base`.
fn radical_inverse(mut n: u32, base: u32) -> f32 {
    let mut result = 0.0;
    let mut inv_base = 1.0 / base as f32;
    while n > 0 {
        result += (n % base) as f32 * inv_base;
        n /= base;
        inv_base /= base as f32;
    }
    result
}

/// 2D Halton sequence (bases 2, 3) with Cranley-Patterson rotation.
/// Each pixel gets a different scramble via `seed` so samples are
/// decorrelated across neighbouring pixels.
pub(super) fn halton_2d(index: u32, seed: u64) -> Vec2 {
    let mut rng = Rng::with_seed(seed);
    Vec2::new(
        (radical_inverse(index, 2) + rng.f32()).fract(),
        (radical_inverse(index, 3) + rng.f32()).fract(),
    )
}

pub(super) fn cosine_sample_hemisphere(normal: Vec3A, rng: &mut Rng) -> Vec3A {
    let u1 = rng.f32();
    let u2 = rng.f32();

    let phi = 2.0 * PI * u2;
    let cos_theta = u1.sqrt();
    let sin_theta = (1.0 - u1).sqrt();

    let (t, b) = orthonormal_basis(normal);
    t * (phi.cos() * sin_theta) + b * (phi.sin() * sin_theta) + normal * cos_theta
}
