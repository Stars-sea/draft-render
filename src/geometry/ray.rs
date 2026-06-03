use glam::Vec3A;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
    pub inv_direction: Vec3A,
}

impl Ray {
    pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
        let dir = direction.normalize();
        Self {
            origin,
            direction: dir,
            inv_direction: Vec3A::ONE / dir,
        }
    }

    pub fn at(&self, t: f32) -> Vec3A {
        self.origin + self.direction * t
    }
}
