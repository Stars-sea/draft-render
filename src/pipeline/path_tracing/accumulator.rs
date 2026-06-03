use crate::color::Color;
use crate::pipeline::path_tracing::scene::{ObjTransform, TraceScene};
use crate::scene::{Camera, Scene};
use rayon::prelude::*;

/// Progressive accumulation buffer: accumulates per-pixel HDR samples.
pub struct Accumulator {
    data: Vec<Color>,
    count: Vec<u32>,
    width: usize,
}

impl Accumulator {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width * height;
        Self {
            data: vec![Color::BLACK; n],
            count: vec![0; n],
            width,
        }
    }

    /// Trace `spp` samples per pixel, accumulating into the HDR buffer.
    pub fn accumulate(&mut self, ts: &TraceScene, camera: &Camera, scene: &Scene, spp: u32) {
        let transforms: Vec<ObjTransform> = scene
            .objects
            .iter()
            .map(|obj| ObjTransform {
                model: obj.transform.transform_matrix(),
                normal_mat: obj.transform.normal_matrix(),
            })
            .collect();

        for si in 0..spp {
            let w = self.width;
            self.data.par_iter_mut().enumerate().for_each(|(i, d)| {
                *d += ts.trace_pixel(camera, &transforms, i % w, i / w, si);
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

    pub fn sample_count(&self) -> u32 {
        self.count[0]
    }

    pub fn reset(&mut self) {
        self.data.fill(Color::BLACK);
        self.count.fill(0);
    }
}
