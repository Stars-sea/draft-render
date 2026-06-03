use crate::color::Color;
use crate::pipeline::path_tracing::scene::TraceScene;
use crate::scene::Camera;
use rayon::prelude::*;

/// Progressive accumulation buffer: accumulates per-pixel samples across frames.
pub struct Accumulator {
    data: Vec<Color>,
    count: Vec<u32>,
    width: usize,
    #[allow(dead_code)]
    height: usize,
}

impl Accumulator {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width * height;
        Self {
            data: vec![Color::BLACK; n],
            count: vec![0; n],
            width,
            height,
        }
    }

    /// Trace and accumulate one sample per pixel (parallel).
    pub fn accumulate(&mut self, ts: &TraceScene, camera: &Camera) {
        let w = self.width;
        self.data
            .par_iter_mut()
            .zip(self.count.par_iter_mut())
            .enumerate()
            .for_each(|(i, (d, c))| {
                let x = i % w;
                let y = i / w;
                *d += ts.trace_pixel(camera, x, y, *c);
                *c += 1;
            });
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

    pub fn sample_count(&self) -> u32 {
        self.count[0]
    }

    pub fn reset(&mut self) {
        self.data.fill(Color::BLACK);
        self.count.fill(0);
    }
}
