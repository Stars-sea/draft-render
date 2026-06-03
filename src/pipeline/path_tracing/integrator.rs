use crate::color::Color;
use crate::geometry::Ray;
use crate::pipeline::bvh::Hit;
use crate::pipeline::path_tracing::bsdf;
use crate::pipeline::path_tracing::sampling::{power_heuristic, stratify_jitter};
use crate::pipeline::path_tracing::scene::{ObjTransform, TraceScene};
use crate::scene::Camera;

use fastrand::Rng;
use glam::{Mat4, Vec3A};

const RAY_EPS: f32 = 1e-3;
const MAX_DEPTH: u32 = 8;
const RR_START: u32 = 3;

struct ShadingPoint {
    point: Vec3A,
    normal: Vec3A,
    geom_normal: Vec3A,
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

struct SceneHit {
    hit: Hit,
    obj_idx: usize,
    local_ray: Ray,
}

impl TraceScene {
    pub(super) fn trace_pixel(
        &self,
        camera: &Camera,
        transforms: &[ObjTransform],
        x: usize,
        y: usize,
        sample_index: u32,
    ) -> Color {
        let seed = ((y * self.width + x) as u64).wrapping_mul(2654435761) ^ sample_index as u64;
        let jitter = stratify_jitter(seed);
        let ray = camera.primary_ray(x, y, self.width, self.height, jitter);
        self.path_trace(ray, transforms, seed)
    }

    fn path_trace(&self, mut ray: Ray, transforms: &[ObjTransform], seed: u64) -> Color {
        let mut radiance = Color::BLACK;
        let mut throughput = Color::WHITE;
        let mut rng = Rng::with_seed(seed);
        let mut prev_bsdf_pdf = 1.0;
        let mut prev_point = Vec3A::ZERO;

        for depth in 0..MAX_DEPTH {
            let Some(sh) = self.intersect_scene(&ray, RAY_EPS, transforms) else {
                if depth == 0 {
                    radiance = sky_color(ray.direction);
                }
                break;
            };

            let sp = self.resolve_hit(&sh, &ray, transforms);

            if sp.emission.max_channel() > 0.0 {
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
            radiance += throughput * self.direct_light(&sp, wo, transforms, &mut rng);

            if self.russian_roulette(depth, &mut throughput, &mut rng) {
                break;
            }

            let s = sp.sample_bsdf(wo, &mut rng);
            let cos_theta = sp.normal.dot(s.wi).max(0.0);
            prev_bsdf_pdf = sp.pdf_bsdf(wo, s.wi);
            prev_point = sp.point;

            throughput *= sp.eval_bsdf(wo, s.wi) * (cos_theta / s.pdf);
            ray = Ray::new(
                sp.point + offset_along_normal(sp.geom_normal, s.wi, RAY_EPS),
                s.wi,
            );
        }

        radiance
    }

    /// Transform a world-space ray to an object's local space.
    fn ray_to_local(ray: &Ray, inv_model: &Mat4) -> (Ray, f32) {
        let origin = inv_model.transform_point3a(ray.origin);
        let dir_raw = inv_model.transform_vector3a(ray.direction);
        let s = dir_raw.length();
        (Ray::new(origin, dir_raw / s), s)
    }

    /// Find the closest hit across all objects.
    fn intersect_scene(
        &self,
        ray: &Ray,
        t_min: f32,
        transforms: &[ObjTransform],
    ) -> Option<SceneHit> {
        let mut best: Option<SceneHit> = None;
        let mut closest_t = f32::INFINITY;

        for (i, obj) in self.objects.iter().enumerate() {
            let inv = transforms[i].model.inverse();
            let (local_ray, s) = Self::ray_to_local(ray, &inv);
            if s < 1e-12 {
                continue;
            }
            let Some(hit) = obj.intersect(&local_ray, t_min, f32::INFINITY) else {
                continue;
            };
            let t = hit.t / s;
            if t < closest_t {
                closest_t = t;
                best = Some(SceneHit { hit, obj_idx: i, local_ray });
            }
        }

        best
    }

    fn resolve_hit(
        &self,
        sh: &SceneHit,
        world_ray: &Ray,
        transforms: &[ObjTransform],
    ) -> ShadingPoint {
        let txf = &transforms[sh.obj_idx];
        let bvh = &self.objects[sh.obj_idx];

        let local_point = sh.hit.point(&sh.local_ray);
        let world_point = txf.model.transform_point3a(local_point);

        let uv = sh.hit.interpolate_uv(&bvh.uvs);
        let mat = &self.materials[bvh.material_ids[sh.hit.tri_idx] as usize];
        let (albedo, _) = mat.albedo_alpha_at(uv);

        let mut shading_normal = if bvh.normals[sh.hit.tri_idx][0] != Vec3A::ZERO {
            sh.hit.interpolate_normal(&bvh.normals)
        } else {
            bvh.triangles[sh.hit.tri_idx].normal()
        };
        shading_normal = (txf.normal_mat * shading_normal).normalize();

        let mut geom_normal = bvh.triangles[sh.hit.tri_idx].normal();
        geom_normal = (txf.normal_mat * geom_normal).normalize();

        if shading_normal.dot(-world_ray.direction) < 0.0 {
            shading_normal = -shading_normal;
            geom_normal = -geom_normal;
        }

        ShadingPoint {
            point: world_point,
            normal: shading_normal,
            geom_normal,
            albedo,
            emission: mat.emission,
            roughness: mat.roughness,
            metallic: mat.metallic,
        }
    }

    #[inline]
    fn direct_light(
        &self,
        sp: &ShadingPoint,
        wo: Vec3A,
        transforms: &[ObjTransform],
        rng: &mut Rng,
    ) -> Color {
        let mut contrib = Color::BLACK;
        for light in &self.lights {
            let Some(ls) = light.sample(sp.point, rng) else {
                continue;
            };
            let cos_theta = sp.normal.dot(ls.wi);
            if cos_theta <= 0.0 {
                continue;
            }
            if self.shadowed(sp.point, sp.geom_normal, ls.wi, ls.dist, transforms) {
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
    fn shadowed(
        &self,
        point: Vec3A,
        normal: Vec3A,
        dir: Vec3A,
        t_max: f32,
        transforms: &[ObjTransform],
    ) -> bool {
        let ray = Ray::new(point + offset_along_normal(normal, dir, RAY_EPS), dir);
        let t_clamped = t_max - RAY_EPS;
        for (i, obj) in self.objects.iter().enumerate() {
            let inv = transforms[i].model.inverse();
            let (local_ray, dir_scale) = Self::ray_to_local(&ray, &inv);
            if obj.intersect_any(&local_ray, RAY_EPS, t_clamped * dir_scale) {
                return true;
            }
        }
        false
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
        let p = throughput.max_channel().min(0.9);
        if rng.f32() > p {
            return true;
        }
        *throughput *= 1.0 / p;
        false
    }
}

fn offset_along_normal(normal: Vec3A, dir: Vec3A, eps: f32) -> Vec3A {
    if normal.dot(dir) > 0.0 {
        normal * eps
    } else {
        -normal * eps
    }
}

fn sky_color(dir: Vec3A) -> Color {
    let t = 0.5 * (dir.y + 1.0);
    Color(Vec3A::new(0.5, 0.7, 0.9)).lerp(&Color::WHITE, t)
}
