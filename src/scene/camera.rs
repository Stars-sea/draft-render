use crate::scene::Projection;
use glam::{Mat4, Quat, Vec3A, Vec4};

pub struct Camera {
    position: Vec3A,
    rotation: Quat,
    proj: Projection,
}

#[allow(unused)]
impl Camera {
    pub fn new(position: Vec3A, rotation: Quat, proj: Projection) -> Self {
        Self {
            position,
            rotation,
            proj,
        }
    }

    pub fn position(&self) -> Vec3A {
        self.position
    }
    pub fn rotation(&self) -> Quat {
        self.rotation
    }
    pub fn projection(&self) -> &Projection {
        &self.proj
    }

    pub fn set_position(&mut self, position: Vec3A) {
        self.position = position;
    }
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }
    pub fn set_projection(&mut self, projection: Projection) {
        self.proj = projection;
    }

    pub fn with_position(mut self, position: Vec3A) -> Self {
        self.set_position(position);
        self
    }
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.set_rotation(rotation);
        self
    }
    pub fn with_projection(mut self, projection: Projection) -> Self {
        self.set_projection(projection);
        self
    }

    fn view_matrix(&self) -> Mat4 {
        let fwd: Vec3A = self.rotation * Vec3A::Z;
        let up: Vec3A = self.rotation * Vec3A::Y;
        let right = up.cross(fwd).normalize();
        let corrected_up = fwd.cross(right);

        let r = Mat4::from_cols(
            Vec4::new(right.x, right.y, right.z, 0.0),
            Vec4::new(corrected_up.x, corrected_up.y, corrected_up.z, 0.0),
            Vec4::new(fwd.x, fwd.y, fwd.z, 0.0),
            Vec4::W,
        );

        let t = Mat4::from_translation((-self.position).into());
        r.inverse() * t
    }

    pub fn vp_matrix(&self, aspect: f32) -> Mat4 {
        self.proj.projection_matrix(aspect) * self.view_matrix()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3A::ZERO, Quat::IDENTITY, Projection::default())
    }
}
