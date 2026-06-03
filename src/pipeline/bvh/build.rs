use crate::geometry::{BoundingBox, Triangle};
use crate::pipeline::bvh::node::BvhNode;
use crate::pipeline::bvh::visit::NULL_NODE;

use glam::{Vec2, Vec3A};
use std::cmp::Ordering;

const LEAF_SIZE: usize = 4;
const MAX_DEPTH: u32 = 32;
const SAH_BINS: usize = 12;
const SAH_TRAVERSAL: f32 = 1.0;
const SAH_INTERSECT: f32 = 1.0;

pub(super) struct BvhBuilder<'a> {
    pub(super) nodes: &'a mut Vec<BvhNode>,
    pub(super) triangles: &'a mut [Triangle],
    pub(super) material_ids: &'a mut [u32],
    pub(super) uvs: &'a mut [[Vec2; 3]],
    pub(super) normals: &'a mut [[Vec3A; 3]],
}

impl<'a> BvhBuilder<'a> {
    pub(super) fn build(&mut self) -> u32 {
        let len = self.triangles.len();
        self.build_range(0, len, 0)
    }

    fn build_range(&mut self, start: usize, end: usize, depth: u32) -> u32 {
        let count = end - start;
        if count == 0 {
            return NULL_NODE;
        }

        let bbox = self.triangles[start..end]
            .iter()
            .fold(BoundingBox::empty(), |bbox, tri| {
                bbox.merge(&tri.bounding_box())
            });

        if count <= LEAF_SIZE || depth >= MAX_DEPTH {
            let idx = self.nodes.len() as u32;
            self.nodes.push(BvhNode::Leaf {
                bbox,
                first: start as u32,
                count: count as u32,
            });
            return idx;
        }

        let (axis, left_count) = self.sah_or_median(start, end, &bbox);
        let mid = start + left_count;

        {
            let tris = &self.triangles[start..end];
            let mut indices: Vec<usize> = (0..tris.len()).collect();
            indices.sort_unstable_by(|&a, &b| {
                tris[a].centroid()[axis]
                    .partial_cmp(&tris[b].centroid()[axis])
                    .unwrap_or(Ordering::Equal)
            });
            let tmp_tris: Vec<_> = indices.iter().map(|&i| tris[i]).collect();
            let tmp_ids: Vec<_> = indices.iter().map(|&i| self.material_ids[start + i]).collect();
            let tmp_uvs: Vec<_> = indices.iter().map(|&i| self.uvs[start + i]).collect();
            let tmp_normals: Vec<_> = indices.iter().map(|&i| self.normals[start + i]).collect();
            self.triangles[start..end].copy_from_slice(&tmp_tris);
            self.material_ids[start..end].copy_from_slice(&tmp_ids);
            self.uvs[start..end].copy_from_slice(&tmp_uvs);
            self.normals[start..end].copy_from_slice(&tmp_normals);
        }

        let left = self.build_range(start, mid, depth + 1);
        let right = self.build_range(mid, end, depth + 1);

        let idx = self.nodes.len() as u32;
        self.nodes.push(BvhNode::Node { bbox, left, right });
        idx
    }

    fn sah_or_median(&self, start: usize, end: usize, bbox: &BoundingBox) -> (usize, usize) {
        let count = end - start;
        let sah = self.find_sah(start, end, bbox);
        if sah.left_count > 0 && sah.left_count < count {
            return (sah.axis, sah.left_count);
        }
        let ext = bbox.extents();
        let axis = if ext.x >= ext.y && ext.x >= ext.z {
            0
        } else if ext.y >= ext.z {
            1
        } else {
            2
        };
        (axis, count / 2)
    }

    fn find_sah(&self, start: usize, end: usize, parent_bbox: &BoundingBox) -> SahSplit {
        let parent_sa = parent_bbox.surface_area();
        let n = end - start;
        let mut best = SahSplit {
            axis: 0,
            left_count: n / 2,
        };
        let mut best_cost = n as f32 * SAH_INTERSECT;

        let ext = parent_bbox.extents();
        for axis in 0..3 {
            if ext[axis] <= 0.0 {
                continue;
            }
            self.evaluate_sah_axis(
                start,
                end,
                parent_bbox,
                axis,
                parent_sa,
                &mut best,
                &mut best_cost,
            );
        }

        best
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_sah_axis(
        &self,
        start: usize,
        end: usize,
        parent_bbox: &BoundingBox,
        axis: usize,
        parent_sa: f32,
        best: &mut SahSplit,
        best_cost: &mut f32,
    ) {
        struct Bin {
            bbox: BoundingBox,
            count: usize,
        }

        let mut bins: [Bin; SAH_BINS] = std::array::from_fn(|_| Bin {
            bbox: BoundingBox::empty(),
            count: 0,
        });

        let inv_ext = 1.0 / parent_bbox.extents()[axis];
        let axis_min = parent_bbox.min[axis];

        for tri in &self.triangles[start..end] {
            let c = tri.centroid()[axis];
            let bin = ((c - axis_min) * inv_ext * SAH_BINS as f32) as usize;
            let bin = bin.min(SAH_BINS - 1);
            bins[bin].bbox = bins[bin].bbox.merge(&tri.bounding_box());
            bins[bin].count += 1;
        }

        let mut right_bbox = [BoundingBox::empty(); SAH_BINS];
        let mut right_count = [0usize; SAH_BINS];
        let (mut acc_bbox, mut acc_count) = (BoundingBox::empty(), 0usize);
        for i in (0..SAH_BINS).rev() {
            acc_bbox = acc_bbox.merge(&bins[i].bbox);
            acc_count += bins[i].count;
            right_bbox[i] = acc_bbox;
            right_count[i] = acc_count;
        }

        let (mut left_bbox, mut left_count) = (BoundingBox::empty(), 0usize);
        for i in 0..(SAH_BINS - 1) {
            left_bbox = left_bbox.merge(&bins[i].bbox);
            left_count += bins[i].count;
            let (r_bbox, r_count) = (right_bbox[i + 1], right_count[i + 1]);

            if left_count == 0 || r_count == 0 {
                continue;
            }

            let cost = SAH_TRAVERSAL
                + (left_bbox.surface_area() / parent_sa) * left_count as f32 * SAH_INTERSECT
                + (r_bbox.surface_area() / parent_sa) * r_count as f32 * SAH_INTERSECT;

            if cost < *best_cost {
                *best_cost = cost;
                *best = SahSplit { axis, left_count };
            }
        }
    }
}

struct SahSplit {
    axis: usize,
    left_count: usize,
}
