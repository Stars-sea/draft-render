use glam::Vec3A;

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
