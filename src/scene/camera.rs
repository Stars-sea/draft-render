use crate::scene::Projection;
use glam::{Mat4, Quat, Vec3A, Vec4};

pub struct Camera {
    position: Vec3A,
    rotation: Quat,
    proj: Projection,

    vp_cache: Option<Mat4>,
}

impl Camera {
    pub fn new(position: Vec3A, rotation: Quat, proj: Projection) -> Self {
        Self {
            position,
            rotation,
            proj,
            vp_cache: None,
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
        self.vp_cache = None;
    }
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.vp_cache = None;
    }
    pub fn set_projection(&mut self, projection: Projection) {
        self.proj = projection;
        self.vp_cache = None;
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

    pub fn vp_matrix(&mut self, aspect: f32) -> Mat4 {
        let vp = self.proj.projection_matrix(aspect) * self.view_matrix();
        self.vp_cache = Some(vp);
        vp
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3A::ZERO, Quat::IDENTITY, Projection::default())
    }
}
