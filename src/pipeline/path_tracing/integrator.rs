use crate::color::Color;
use crate::geometry::Ray;
use crate::pipeline::bvh::Hit;
use crate::pipeline::path_tracing::sampling::{cosine_sample_hemisphere, stratify_jitter};
use crate::pipeline::path_tracing::scene::TraceScene;
use crate::scene::Camera;
use fastrand::Rng;
use glam::Vec3A;

const RAY_EPS: f32 = 1e-4;
const MAX_DEPTH: u32 = 8;
const RR_START: u32 = 3;

struct ShadingPoint {
    point: Vec3A,
    normal: Vec3A,
    albedo: Color,
}

impl TraceScene {
    pub fn trace_pixel(&self, camera: &Camera, x: usize, y: usize, sample_index: u32) -> Color {
        let jitter = stratify_jitter(sample_index);
        let ray = camera.primary_ray(x, y, self.width, self.height, jitter);
        let seed = ((y * self.width + x) as u64).wrapping_mul(2654435761) ^ sample_index as u64;
        self.path_trace(ray, seed)
    }

    fn path_trace(&self, mut ray: Ray, seed: u64) -> Color {
        let mut radiance = Color::BLACK;
        let mut throughput = Color::WHITE;
        let mut rng = Rng::with_seed(seed);

        for depth in 0..MAX_DEPTH {
            let Some(hit) = self.bvh.intersect(&ray, RAY_EPS, f32::INFINITY) else {
                if radiance.0 == Vec3A::ZERO {
                    radiance = sky_color(ray.direction);
                }
                break;
            };

            let sp = self.resolve_hit(&hit, &ray);
            radiance += throughput * (sp.albedo * 0.03);
            radiance += throughput * self.direct_light(&sp);

            if self.russian_roulette(depth, &mut throughput, &mut rng) {
                break;
            }

            let wi = cosine_sample_hemisphere(sp.normal, &mut rng);
            throughput *= sp.albedo;
            ray = Ray::new(sp.point + wi * RAY_EPS, wi);
        }

        radiance
    }

    #[inline]
    fn resolve_hit(&self, hit: &Hit, ray: &Ray) -> ShadingPoint {
        let point = hit.point(ray);
        let uv = hit.interpolate_uv(&self.bvh.uvs);
        let mat = &self.materials[self.bvh.material_ids[hit.tri_idx] as usize];
        let (albedo, _alpha) = mat.albedo_alpha_at(uv);

        let tri = &self.bvh.triangles[hit.tri_idx];
        let mut normal = tri.normal();
        let wo = -ray.direction;
        if normal.dot(wo) < 0.0 {
            normal = -normal;
        }

        ShadingPoint {
            point,
            normal,
            albedo,
        }
    }

    #[inline]
    fn direct_light(&self, sp: &ShadingPoint) -> Color {
        let mut contrib = Color::BLACK;
        for light in &self.lights {
            let dir = light.direction(sp.point);
            let n_dot_l = sp.normal.dot(dir);
            if n_dot_l <= 0.0 {
                continue;
            }
            let dist = light.distance(sp.point);
            let shadow_ray = Ray::new(sp.point + dir * RAY_EPS, dir);
            if self.bvh.intersect_any(&shadow_ray, RAY_EPS, dist - RAY_EPS) {
                continue;
            }
            let li = light.color() * (light.intensity() * light.attenuation(sp.point));
            contrib += sp.albedo * li * n_dot_l;
        }
        contrib
    }

    #[inline]
    fn russian_roulette(&self, depth: u32, throughput: &mut Color, rng: &mut Rng) -> bool {
        if depth < RR_START {
            return false;
        }
        let p = throughput.0.max_element().min(0.9);
        if rng.f32() > p {
            return true;
        }
        *throughput *= 1.0 / p;
        false
    }
}

fn sky_color(dir: Vec3A) -> Color {
    let t = 0.5 * (dir.y + 1.0);
    Color(Vec3A::new(0.5, 0.7, 0.9)).lerp(&Color::WHITE, t)
}
