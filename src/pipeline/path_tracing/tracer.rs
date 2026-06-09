use super::intersection;
use super::math::power_heuristic;
use super::sampling::halton_2d;
use super::scene::TraceScene;
use super::shading::ShadingPoint;
use crate::color::Color;
use crate::geometry::Ray;
use crate::scene::ObjTransform;
use fastrand::Rng;
use glam::{Vec2, Vec3A};

/// Sampling strategy for pixel AA / lens sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sampler {
    /// 2D Halton (bases 2,3) with Cranley-Patterson rotation — deterministic QMC.
    Halton,
    /// Pure pseudo-random (jitter) — standard MC white-noise baseline.
    Jitter,
}

/// Tunable parameters for the path tracer, exposed so the essay can sweep
/// each factor independently.
#[derive(Debug, Clone, Copy)]
pub struct PathTracerConfig {
    pub max_depth: u32,
    /// Russian-roulette start depth (0 = disabled).
    pub rr_start: u32,
    /// Indirect-clamp threshold (≤ 0 = disabled).
    pub indirect_clamp: f32,
    pub sampler: Sampler,
    /// Global seed mixed into per-pixel seeds for independent repetitions.
    pub seed: u64,
}

impl Default for PathTracerConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            rr_start: 3,
            indirect_clamp: 10.0,
            sampler: Sampler::Halton,
            seed: 0,
        }
    }
}

pub(super) struct PathTracer<'a> {
    ts: &'a TraceScene,
    transforms: &'a [ObjTransform],
    config: PathTracerConfig,
}

impl<'a> PathTracer<'a> {
    pub(super) fn new(
        ts: &'a TraceScene,
        transforms: &'a [ObjTransform],
        config: PathTracerConfig,
    ) -> Self {
        Self {
            ts,
            transforms,
            config,
        }
    }

    /// Trace only the primary ray (center of pixel) and return first-hit
    /// world-space position and geometric normal.  Used to generate a
    /// per-pixel geometry buffer for region-mask classification.
    pub(super) fn first_hit(&self, x: usize, y: usize) -> Option<(Vec3A, Vec3A)> {
        let ray = self.ts.camera.primary_ray(
            x,
            y,
            self.ts.width,
            self.ts.height,
            Vec2::new(0.5, 0.5), // pixel center
        );
        let sh = intersection::intersect_scene(self.ts, &ray, intersection::RAY_EPS, self.transforms)?;
        let sp = intersection::resolve_hit(self.ts, &sh, &ray, self.transforms);
        Some((sp.point, sp.geom_normal))
    }

    pub(super) fn trace_pixel(&self, x: usize, y: usize, sample_index: u32) -> (Color, u32) {
        let seed = ((y * self.ts.width + x) as u64)
            .wrapping_mul(2654435761)
            .wrapping_add(self.config.seed)
            ^ sample_index as u64;
        let uv = match self.config.sampler {
            Sampler::Halton => halton_2d(sample_index, seed),
            Sampler::Jitter => {
                let mut rng = Rng::with_seed(
                    seed.wrapping_mul(6364136223846793005)
                        .wrapping_add(sample_index as u64),
                );
                Vec2::new(rng.f32(), rng.f32())
            }
        };
        let ray = self
            .ts
            .camera
            .primary_ray(x, y, self.ts.width, self.ts.height, uv);
        self.path_trace(ray, seed)
    }

    fn path_trace(&self, mut ray: Ray, seed: u64) -> (Color, u32) {
        let mut radiance = Color::BLACK;
        let mut throughput = Color::WHITE;
        let mut rng = Rng::with_seed(seed);
        let mut prev_bsdf_pdf = 1.0;
        let mut prev_point = Vec3A::ZERO;

        for depth in 0..self.config.max_depth {
            let Some(sh) = intersection::intersect_scene(
                self.ts,
                &ray,
                intersection::RAY_EPS,
                self.transforms,
            ) else {
                if depth == 0 {
                    radiance = sky_color(ray.direction);
                }
                return (radiance, depth);
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
                return (radiance, depth);
            }

            let wo = -ray.direction;
            radiance += throughput * self.direct_light(&sp, wo, &mut rng);

            if self.russian_roulette(depth, &mut throughput, &mut rng) {
                return (radiance, depth);
            }

            let s = sp.bsdf.sample(wo, sp.normal, &mut rng);
            let cos_theta = sp.normal.dot(s.wi).max(0.0);
            prev_bsdf_pdf = sp.bsdf.pdf(wo, s.wi, sp.normal);
            prev_point = sp.point;

            let mut contrib = sp.bsdf.evaluate(wo, s.wi, sp.normal) * (cos_theta / s.pdf);
            if self.config.indirect_clamp > 0.0 {
                let m = contrib.max_channel();
                if m > self.config.indirect_clamp {
                    contrib *= self.config.indirect_clamp / m;
                }
            }
            throughput *= contrib;
            let origin = sp.point
                + intersection::offset_along_normal(sp.geom_normal, s.wi, intersection::RAY_EPS);
            ray = Ray::new(origin, s.wi);
        }

        (radiance, self.config.max_depth)
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

    #[inline]
    fn russian_roulette(&self, depth: u32, throughput: &mut Color, rng: &mut Rng) -> bool {
        if self.config.rr_start == 0 || depth < self.config.rr_start {
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

fn sky_color(dir: Vec3A) -> Color {
    let t = 0.5 * (dir.y + 1.0);
    Color(Vec3A::new(0.5, 0.7, 0.9)).lerp(&Color::WHITE, t)
}
