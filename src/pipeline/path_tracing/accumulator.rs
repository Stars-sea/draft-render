use super::tracer::{PathTracer, PathTracerConfig};
use crate::color::Color;
use crate::pipeline::path_tracing::scene::TraceScene;
use crate::scene::Scene;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

/// Progressive accumulation buffer: accumulates per-pixel HDR samples.
pub struct Accumulator {
    data: Vec<Color>,
    count: Vec<u32>,
    /// Per-depth path termination counters.  Index = depth (0..max_depth+1).
    pub depth_histogram: Vec<AtomicU32>,
    width: usize,
}

impl Accumulator {
    pub fn new(width: usize, height: usize, max_depth: u32) -> Self {
        let n = width * height;
        let depth_histogram = (0..=max_depth).map(|_| AtomicU32::new(0)).collect();
        Self {
            data: vec![Color::BLACK; n],
            count: vec![0; n],
            depth_histogram,
            width,
        }
    }

    /// Trace `spp` samples per pixel, accumulating into the HDR buffer.
    pub fn accumulate(&mut self, ts: &TraceScene, scene: &Scene, spp: u32) {
        self.accumulate_with_config(ts, scene, spp, PathTracerConfig::default());
    }

    /// Trace `spp` samples per pixel with the given path-tracer configuration.
    pub fn accumulate_with_config(
        &mut self,
        ts: &TraceScene,
        _scene: &Scene,
        spp: u32,
        config: PathTracerConfig,
    ) {
        let pt = PathTracer::new(ts, &ts.transforms, config);
        let hist = &self.depth_histogram;

        let offset = self.count[0]; // running total so far
        for si in 0..spp {
            let sample_index = offset + si;
            let w = self.width;
            self.data.par_iter_mut().enumerate().for_each(|(i, d)| {
                let (color, depth) = pt.trace_pixel(i % w, i / w, sample_index);
                *d += color;
                let d = depth.min(hist.len() as u32 - 1) as usize;
                hist[d].fetch_add(1, Ordering::Relaxed);
            });
        }

        for c in &mut self.count {
            *c += spp;
        }
    }

    /// Current averaged HDR image (linear RGB).
    pub fn as_image(&self) -> Vec<Color> {
        self.data
            .iter()
            .zip(&self.count)
            .map(|(&d, &c)| {
                if c > 0 {
                    d * (1.0 / c as f32)
                } else {
                    Color::BLACK
                }
            })
            .collect()
    }

    /// Raw HDR data as flat `[f32; H*W*3]` (R, G, B interleaved per pixel,
    /// row-major). Suitable for writing to a `.bin` file for offline analysis.
    pub fn as_float_buffer(&self) -> Vec<f32> {
        let mut buf = Vec::with_capacity(self.data.len() * 3);
        for (&d, &c) in self.data.iter().zip(&self.count) {
            let inv = if c > 0 { 1.0 / c as f32 } else { 1.0 };
            buf.push(d.0.x * inv);
            buf.push(d.0.y * inv);
            buf.push(d.0.z * inv);
        }
        buf
    }

    pub fn sample_count(&self) -> u32 {
        self.count[0]
    }

    pub fn reset(&mut self) {
        self.data.fill(Color::BLACK);
        self.count.fill(0);
        for a in &self.depth_histogram {
            a.store(0, Ordering::Relaxed);
        }
    }
}
