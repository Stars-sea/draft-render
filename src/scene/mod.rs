mod camera;
mod light;
mod material;
mod mesh;
mod object;
mod projection;
mod transform;

use std::sync::Arc;

pub use camera::Camera;
pub use light::{DirectionalLight, Light, PointLight};
pub use material::{Material, Texture};
pub use mesh::{Mesh, MeshBuilder, SubMesh};
pub use object::SceneObject;
pub use projection::Projection;
pub use transform::Transform;

pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn add_light(&mut self, light: Arc<dyn Light + Send + Sync>) {
        self.lights.push(light);
    }

    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }
}
