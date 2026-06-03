use fastrand::Rng;
use glam::{Vec2, Vec3A};

pub(super) fn stratify_jitter(seed: u64) -> Vec2 {
    const GRID: u32 = 8;
    let mut rng = Rng::with_seed(seed);
    let idx = (seed as u32) % (GRID * GRID);
    let x = idx % GRID;
    let y = idx / GRID;
    Vec2::new(
        (x as f32 + rng.f32()) / GRID as f32,
        (y as f32 + rng.f32()) / GRID as f32,
    )
}

pub(super) fn cosine_sample_hemisphere(normal: Vec3A, rng: &mut Rng) -> Vec3A {
    let u1 = rng.f32();
    let u2 = rng.f32();

    let phi = 2.0 * std::f32::consts::PI * u2;
    let cos_theta = u1.sqrt();
    let sin_theta = (1.0 - u1).sqrt();

    let (t, b) = orthonormal_basis(normal);
    t * (phi.cos() * sin_theta) + b * (phi.sin() * sin_theta) + normal * cos_theta
}

pub(super) fn orthonormal_basis(n: Vec3A) -> (Vec3A, Vec3A) {
    let t = if n.x.abs() > 0.9 {
        Vec3A::Y.cross(n).normalize()
    } else {
        Vec3A::X.cross(n).normalize()
    };
    let b = n.cross(t);
    (t, b)
}

/// Power heuristic (β=2) for MIS: `pdf_a² / (pdf_a² + pdf_b²)`.
pub(super) fn power_heuristic(pdf_a: f32, pdf_b: f32) -> f32 {
    let a2 = pdf_a * pdf_a;
    let b2 = pdf_b * pdf_b;
    a2 / (a2 + b2)
}
