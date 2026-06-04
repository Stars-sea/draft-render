use super::intersection;
use super::math::power_heuristic;
use super::sampling::stratify_jitter;
use super::scene::TraceScene;
use super::shading::ShadingPoint;
use crate::color::Color;
use crate::geometry::Ray;
use crate::scene::ObjTransform;
use fastrand::Rng;
use glam::Vec3A;

const MAX_DEPTH: u32 = 8;
const RR_START: u32 = 3;

pub(super) struct PathTracer<'a> {
    ts: &'a TraceScene,
    transforms: &'a [ObjTransform],
}

impl<'a> PathTracer<'a> {
    pub(super) fn new(ts: &'a TraceScene, transforms: &'a [ObjTransform]) -> Self {
        Self { ts, transforms }
    }

    pub(super) fn trace_pixel(&self, x: usize, y: usize, sample_index: u32) -> Color {
        let seed = ((y * self.ts.width + x) as u64).wrapping_mul(2654435761) ^ sample_index as u64;
        let jitter = stratify_jitter(seed);
        let ray = self
            .ts
            .camera
            .primary_ray(x, y, self.ts.width, self.ts.height, jitter);
        self.path_trace(ray, seed)
    }

    fn path_trace(&self, mut ray: Ray, seed: u64) -> Color {
        let mut radiance = Color::BLACK;
        let mut throughput = Color::WHITE;
        let mut rng = Rng::with_seed(seed);
        let mut prev_bsdf_pdf = 1.0;
        let mut prev_point = Vec3A::ZERO;

        for depth in 0..MAX_DEPTH {
            let Some(sh) = intersection::intersect_scene(
                self.ts,
                &ray,
                intersection::RAY_EPS,
                self.transforms,
            ) else {
                if depth == 0 {
                    radiance = sky_color(ray.direction);
                }
                break;
            };

            let sp = intersection::resolve_hit(self.ts, &sh, &ray, self.transforms);

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
            radiance += throughput * self.direct_light(&sp, wo, &mut rng);

            if russian_roulette(depth, &mut throughput, &mut rng) {
                break;
            }

            let s = sp.bsdf.sample(wo, sp.normal, &mut rng);
            let cos_theta = sp.normal.dot(s.wi).max(0.0);
            prev_bsdf_pdf = sp.bsdf.pdf(wo, s.wi, sp.normal);
            prev_point = sp.point;

            throughput *= sp.bsdf.evaluate(wo, s.wi, sp.normal) * (cos_theta / s.pdf);
            let origin = sp.point
                + intersection::offset_along_normal(sp.geom_normal, s.wi, intersection::RAY_EPS);
            ray = Ray::new(origin, s.wi);
        }

        radiance
    }

    #[inline]
    fn direct_light(&self, sp: &ShadingPoint, wo: Vec3A, rng: &mut Rng) -> Color {
        let mut contrib = Color::BLACK;
        for light in &self.ts.lights {
            let Some(ls) = light.sample(sp.point, rng) else {
                continue;
            };
            let cos_theta = sp.normal.dot(ls.wi);
            if cos_theta <= 0.0 {
                continue;
            }
            if intersection::shadowed(
                self.ts,
                sp.point,
                sp.geom_normal,
                ls.wi,
                ls.dist,
                self.transforms,
            ) {
                continue;
            }
            let bsdf_val = sp.bsdf.evaluate(wo, ls.wi, sp.normal);
            let mis_w = if light.is_delta() {
                1.0
            } else {
                power_heuristic(ls.pdf, sp.bsdf.pdf(wo, ls.wi, sp.normal))
            };
            contrib += bsdf_val * ls.radiance * (cos_theta * mis_w / ls.pdf);
        }
        contrib
    }

    #[inline]
    fn pdf_lights(&self, point: Vec3A, wi: Vec3A) -> f32 {
        self.ts
            .lights
            .iter()
            .map(|light| light.pdf(point, wi))
            .sum()
    }
}

#[inline]
fn russian_roulette(depth: u32, throughput: &mut Color, rng: &mut Rng) -> bool {
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

fn sky_color(dir: Vec3A) -> Color {
    let t = 0.5 * (dir.y + 1.0);
    Color(Vec3A::new(0.5, 0.7, 0.9)).lerp(&Color::WHITE, t)
}
