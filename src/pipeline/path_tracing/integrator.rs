use crate::color::Color;
use crate::geometry::Ray;
use crate::pipeline::bvh::Hit;
use crate::pipeline::path_tracing::bsdf;
use crate::pipeline::path_tracing::sampling::{power_heuristic, stratify_jitter};
use crate::pipeline::path_tracing::scene::TraceScene;
use crate::scene::Camera;

use fastrand::Rng;
use glam::Vec3A;

const RAY_EPS: f32 = 1e-3;
const MAX_DEPTH: u32 = 8;
const RR_START: u32 = 3;

struct ShadingPoint {
    point: Vec3A,
    normal: Vec3A,
    albedo: Color,
    emission: Color,
    roughness: f32,
    metallic: f32,
}

impl ShadingPoint {
    fn eval_bsdf(&self, wo: Vec3A, wi: Vec3A) -> Color {
        bsdf::evaluate(
            wo,
            wi,
            self.normal,
            self.albedo,
            self.roughness,
            self.metallic,
        )
    }

    fn sample_bsdf(&self, wo: Vec3A, rng: &mut Rng) -> bsdf::BsdfSample {
        bsdf::sample(wo, self.normal, self.roughness, self.metallic, rng)
    }

    fn pdf_bsdf(&self, wo: Vec3A, wi: Vec3A) -> f32 {
        bsdf::pdf(wo, wi, self.normal, self.roughness, self.metallic)
    }
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
        let mut prev_bsdf_pdf = 1.0;
        let mut prev_point = Vec3A::ZERO;

        for depth in 0..MAX_DEPTH {
            let Some(hit) = self.bvh.intersect(&ray, RAY_EPS, f32::INFINITY) else {
                if depth == 0 {
                    radiance = sky_color(ray.direction);
                }
                break;
            };

            let sp = self.resolve_hit(&hit, &ray);

            if sp.emission.0.max_element() > 0.0 {
                if depth == 0 {
                    radiance += throughput * sp.emission;
                } else {
                    let light_pdf = self.pdf_lights(prev_point, ray.direction);
                    radiance +=
                        throughput * sp.emission * power_heuristic(prev_bsdf_pdf, light_pdf);
                }
                break;
            }

            let wo = -ray.direction;
            radiance += throughput * self.direct_light(&sp, wo, &mut rng);

            if self.russian_roulette(depth, &mut throughput, &mut rng) {
                break;
            }

            let s = sp.sample_bsdf(wo, &mut rng);
            let cos_theta = sp.normal.dot(s.wi).max(0.0);
            prev_bsdf_pdf = sp.pdf_bsdf(wo, s.wi);
            prev_point = sp.point;

            throughput *= sp.eval_bsdf(wo, s.wi) * (cos_theta / s.pdf);
            ray = Ray::new(sp.point + s.wi * RAY_EPS, s.wi);
        }

        radiance
    }

    #[inline]
    fn resolve_hit(&self, hit: &Hit, ray: &Ray) -> ShadingPoint {
        let point = hit.point(ray);
        let uv = hit.interpolate_uv(&self.bvh.uvs);
        let mat = &self.materials[self.bvh.material_ids[hit.tri_idx] as usize];
        let (albedo, _) = mat.albedo_alpha_at(uv);

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
            emission: mat.emission,
            roughness: mat.roughness,
            metallic: mat.metallic,
        }
    }

    #[inline]
    fn direct_light(&self, sp: &ShadingPoint, wo: Vec3A, rng: &mut Rng) -> Color {
        let mut contrib = Color::BLACK;
        for light in &self.lights {
            let Some(ls) = light.sample(sp.point, rng) else {
                continue;
            };
            let cos_theta = sp.normal.dot(ls.wi);
            if cos_theta <= 0.0 {
                continue;
            }
            if self.shadowed(sp.point, ls.wi, ls.dist) {
                continue;
            }

            let bsdf_val = sp.eval_bsdf(wo, ls.wi);
            let mis_w = if light.is_delta() {
                1.0
            } else {
                power_heuristic(ls.pdf, sp.pdf_bsdf(wo, ls.wi))
            };
            contrib += bsdf_val * ls.radiance * (cos_theta * mis_w / ls.pdf);
        }
        contrib
    }

    #[inline]
    fn shadowed(&self, point: Vec3A, dir: Vec3A, t_max: f32) -> bool {
        let ray = Ray::new(point + dir * RAY_EPS, dir);
        self.bvh.intersect_any(&ray, RAY_EPS, t_max - RAY_EPS)
    }

    #[inline]
    fn pdf_lights(&self, point: Vec3A, wi: Vec3A) -> f32 {
        self.lights.iter().map(|light| light.pdf(point, wi)).sum()
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
