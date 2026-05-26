use crate::linalg::{Mat4f, Vec3f, transform};
use crate::projection::{Perspective, Projection};
use num_traits::Zero;
use std::sync::Arc;

pub struct Camera {
    position: Vec3f,
    direction: Vec3f,
    up: Vec3f,
    proj: Arc<dyn Projection>,
}

impl Camera {
    pub fn new(position: Vec3f, direction: Vec3f, up: Vec3f, proj: Arc<dyn Projection>) -> Self {
        Self {
            position,
            direction,
            up,
            proj,
        }
    }

    pub fn view_matrix(&self) -> Mat4f {
        let camera_x_axis = self.up.cross(&self.direction);
        let camera_y_axis = self.direction.cross(&camera_x_axis);
        let camera_z_axis = -self.direction;

        let inverse_rotate = Mat4f::from([
            [camera_x_axis[0], camera_y_axis[0], camera_z_axis[0], 0.0],
            [camera_x_axis[1], camera_y_axis[1], camera_z_axis[1], 0.0],
            [camera_x_axis[2], camera_y_axis[2], camera_z_axis[2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let translate = transform::translate(-self.position);

        inverse_rotate.inverse().unwrap() * translate
    }

    pub fn vp_matrix(&self) -> Mat4f {
        let view_matrix = self.view_matrix();
        let proj_matrix = self.proj.projection_matrix();

        proj_matrix * view_matrix
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Vec3f::zero(),
            -Vec3f::UNIT_Z,
            Vec3f::UNIT_Y,
            Arc::new(Perspective::default()),
        )
    }
}
