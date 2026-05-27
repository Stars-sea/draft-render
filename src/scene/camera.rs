use crate::linalg::{Mat4f, Quaternion, Rotator, Vec3f, transform};
use crate::scene::Projection;
use num_traits::{ConstZero, Zero};

pub struct Camera {
    position: Vec3f,
    rotator: Rotator<f32>,
    proj: Projection,

    vp_cache: Option<Mat4f>,
}

impl Camera {
    pub fn new(position: Vec3f, rotator: Rotator<f32>, proj: Projection) -> Self {
        Self {
            position,
            rotator,
            proj,
            vp_cache: None,
        }
    }

    pub fn position(&self) -> Vec3f {
        self.position
    }
    pub fn rotator(&self) -> Rotator<f32> {
        self.rotator
    }
    pub fn projection(&self) -> &Projection {
        &self.proj
    }

    pub fn set_position(&mut self, position: Vec3f) {
        self.position = position;
        self.vp_cache = None;
    }
    pub fn set_rotator(&mut self, rotator: Rotator<f32>) {
        self.rotator = rotator;
        self.vp_cache = None;
    }
    pub fn set_projection(&mut self, projection: Projection) {
        self.proj = projection;
        self.vp_cache = None;
    }

    pub fn with_position(mut self, position: Vec3f) -> Self {
        self.set_position(position);
        self
    }
    pub fn with_rotation(mut self, rotator: impl Fn(Rotator<f32>) -> Rotator<f32>) -> Self {
        self.set_rotator(rotator(self.rotator));
        self
    }
    pub fn with_projection(mut self, projection: Projection) -> Self {
        self.set_projection(projection);
        self
    }

    fn view_matrix(&self) -> Mat4f {
        let rotation: Quaternion<f32> = self.rotator.into();
        let forward = rotation.rotate_vector(Vec3f::unit_z());
        let up = rotation.rotate_vector(Vec3f::unit_y());
        let right = up.cross(&forward).normalize();
        let corrected_up = forward.cross(&right);

        let r = Mat4f::from([
            [right.x(), corrected_up.x(), forward.x(), 0.0],
            [right.y(), corrected_up.y(), forward.y(), 0.0],
            [right.z(), corrected_up.z(), forward.z(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let t = transform::translate(-self.position);
        r.inverse().unwrap() * t
    }

    pub fn vp_matrix(&mut self, aspect: f32) -> Mat4f {
        let vp = self.proj.projection_matrix(aspect) * self.view_matrix();
        self.vp_cache = Some(vp);
        vp
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3f::ZERO, Rotator::identity(), Projection::default())
    }
}
