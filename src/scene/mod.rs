mod camera;
mod light;
mod material;
mod mesh;
mod object;
mod transform;

use std::sync::Arc;

pub use camera::Camera;
pub use light::{DirectionalLight, Light, LightSample, PointLight, QuadLight};
pub use material::{Material, Texture};
pub use mesh::{Mesh, MeshBuilder, SubMesh};
pub use object::SceneObject;
pub use transform::Transform;

pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Arc<dyn Light>>,
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

    pub fn add_light(&mut self, light: Arc<dyn Light>) {
        self.lights.push(light);
    }

    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }
}
