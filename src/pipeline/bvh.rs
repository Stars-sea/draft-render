use crate::pipeline::bounding_box::BoundingBox;
use crate::pipeline::geometry::{Ray, Triangle, intersect};
use std::cmp::Ordering;

// ---- BVH ----

const LEAF_SIZE: usize = 4;
const MAX_DEPTH: u32 = 32;
const NULL_NODE: u32 = u32::MAX;
const SAH_BINS: usize = 12;
const SAH_TRAVERSAL: f32 = 1.0;
const SAH_INTERSECT: f32 = 1.0;
const TRAVERSAL_STACK: usize = 64;

/// Bounding Volume Hierarchy.
///
/// All nodes live in a single contiguous `Vec` for cache-friendly traversal.
/// Triangles are reordered during construction so that leaves reference
/// contiguous ranges.  Built with binned SAH for higher-quality splits.
#[allow(clippy::upper_case_acronyms)]
pub struct BVH {
    nodes: Vec<BVHNode>,
    root: u32,
    pub triangles: Vec<Triangle>,
}

enum BVHNode {
    Leaf {
        bbox: BoundingBox,
        first: u32,
        count: u32,
    },
    Node {
        bbox: BoundingBox,
        left: u32,
        right: u32,
    },
}

impl BVHNode {
    fn bbox(&self) -> &BoundingBox {
        match self {
            BVHNode::Leaf { bbox, .. } | BVHNode::Node { bbox, .. } => bbox,
        }
    }
}

impl BVH {
    pub fn build(mut triangles: Vec<Triangle>) -> Self {
        let len = triangles.len();
        // pre-allocate optimization based on previous context
        let mut nodes = Vec::with_capacity(len * 2);
        let root = build_range(&mut nodes, &mut triangles, 0, len, 0);
        nodes.shrink_to_fit();
        BVH {
            nodes,
            root,
            triangles,
        }
    }

    /// Find the closest triangle intersection.
    /// Returns `(triangle_index, t, u, v)` — `u`, `v` are barycentric coords.
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(usize, f32, f32, f32)> {
        intersect_node(ray, &self.triangles, &self.nodes, self.root, t_min, t_max)
    }

    /// Test whether the ray hits *any* triangle (used for shadow rays).
    pub fn intersect_any(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        intersect_any_node(ray, &self.triangles, &self.nodes, self.root, t_min, t_max)
    }
}

trait SortByCentroid {
    fn sort_by_centroid(&mut self, axis: usize);
}

impl SortByCentroid for [Triangle] {
    fn sort_by_centroid(&mut self, axis: usize) {
        self.sort_by(|a, b| {
            a.centroid()[axis]
                .partial_cmp(&b.centroid()[axis])
                .unwrap_or(Ordering::Equal)
        });
    }
}

// ---- Build ----

fn build_range(
    nodes: &mut Vec<BVHNode>,
    triangles: &mut [Triangle],
    start: usize,
    end: usize,
    depth: u32,
) -> u32 {
    let count = end - start;
    if count == 0 {
        return NULL_NODE;
    }

    let slice = &triangles[start..end];
    let mut bbox = BoundingBox::empty();
    for tri in slice {
        let (v1, v2) = (tri.v1(), tri.v2());
        bbox.min = bbox.min.min(tri.v0).min(v1).min(v2);
        bbox.max = bbox.max.max(tri.v0).max(v1).max(v2);
    }

    if count <= LEAF_SIZE || depth >= MAX_DEPTH {
        let idx = nodes.len() as u32;
        nodes.push(BVHNode::Leaf {
            bbox,
            first: start as u32,
            count: count as u32,
        });
        return idx;
    }

    let (axis, mid) = sah_or_median_split(&triangles[start..end], &bbox, count);
    triangles[start..end].sort_by_centroid(axis);

    let left = build_range(nodes, triangles, start, mid, depth + 1);
    let right = build_range(nodes, triangles, mid, end, depth + 1);

    let idx = nodes.len() as u32;
    nodes.push(BVHNode::Node { bbox, left, right });
    idx
}

// ---- SAH ----

struct SahSplit {
    axis: usize,
    left_count: usize,
}

fn sah_or_median_split(triangles: &[Triangle], bbox: &BoundingBox, count: usize) -> (usize, usize) {
    let sah = find_sah_split(triangles, bbox);
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

fn find_sah_split(triangles: &[Triangle], parent_bbox: &BoundingBox) -> SahSplit {
    let ext = parent_bbox.extents();
    let parent_sa = parent_bbox.surface_area();

    let mut best = SahSplit {
        axis: 0,
        left_count: triangles.len() / 2,
    };
    let mut best_cost = SAH_TRAVERSAL + parent_sa * triangles.len() as f32 * SAH_INTERSECT;

    for axis in 0..3 {
        if ext[axis] <= 0.0 {
            continue;
        }
        sah_evaluate_axis(
            triangles,
            parent_bbox,
            axis,
            parent_sa,
            &mut best,
            &mut best_cost,
        );
    }

    best
}

/// Evaluate one axis for SAH: bin centroids, sweep for lowest cost split.
fn sah_evaluate_axis(
    triangles: &[Triangle],
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

    for tri in triangles {
        let c = tri.centroid()[axis];
        let bin = ((c - axis_min) * inv_ext * SAH_BINS as f32) as usize;
        let bin = bin.min(SAH_BINS - 1);
        bins[bin].bbox = bins[bin].bbox.merge(&tri.bounding_box());
        bins[bin].count += 1;
    }

    // Precompute right-side cumulative bboxes for O(N) sweep
    let (mut right_bbox, mut right_count) = ([BoundingBox::empty(); SAH_BINS], [0usize; SAH_BINS]);
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

// ---- Traversal ----

#[inline]
#[allow(clippy::too_many_arguments)]
fn push_nodes(
    ray: &Ray,
    nodes: &[BVHNode],
    left: u32,
    right: u32,
    t_min: f32,
    t_max: f32,
    stack: &mut [u32],
    sp: &mut usize,
) {
    let hit_l = nodes[left as usize].bbox().intersect_range(ray);
    let hit_r = nodes[right as usize].bbox().intersect_range(ray);

    let pass_l = hit_l.is_some_and(|(tn, tf)| tn.max(t_min) <= tf.min(t_max));
    let pass_r = hit_r.is_some_and(|(tn, tf)| tn.max(t_min) <= tf.min(t_max));

    let tn_l = hit_l.unwrap_or((f32::INFINITY, 0.0)).0;
    let tn_r = hit_r.unwrap_or((f32::INFINITY, 0.0)).0;

    debug_assert!(*sp + 2 <= TRAVERSAL_STACK, "BVH traversal stack overflow");

    if pass_l && pass_r {
        if tn_l <= tn_r {
            stack[*sp] = right;
            stack[*sp + 1] = left;
        } else {
            stack[*sp] = left;
            stack[*sp + 1] = right;
        }
        *sp += 2;
    } else if pass_l {
        stack[*sp] = left;
        *sp += 1;
    } else if pass_r {
        stack[*sp] = right;
        *sp += 1;
    }
}

fn intersect_node(
    ray: &Ray,
    triangles: &[Triangle],
    nodes: &[BVHNode],
    node_idx: u32,
    t_min: f32,
    t_max: f32,
) -> Option<(usize, f32, f32, f32)> {
    if node_idx == NULL_NODE {
        return None;
    }

    let mut stack = [0u32; TRAVERSAL_STACK];
    let mut sp = 1;
    stack[0] = node_idx;

    let mut t_closest = t_max;
    let mut best = None;

    while sp > 0 {
        sp -= 1;
        let node = &nodes[stack[sp] as usize];
        if !node.bbox().intersect(ray, t_min, t_closest) {
            continue;
        }

        match node {
            BVHNode::Leaf { first, count, .. } => {
                let (start, n) = (*first as usize, *count as usize);
                for (i, tri) in triangles[start..(start + n)].iter().enumerate() {
                    if let Some((t, u, v)) = intersect(ray, tri)
                        && (t_min..t_closest).contains(&t)
                    {
                        t_closest = t;
                        best = Some((start + i, t, u, v));
                    }
                }
            }
            BVHNode::Node { left, right, .. } => {
                push_nodes(
                    ray, nodes, *left, *right, t_min, t_closest, &mut stack, &mut sp,
                );
            }
        }
    }

    best
}

fn intersect_any_node(
    ray: &Ray,
    triangles: &[Triangle],
    nodes: &[BVHNode],
    node_idx: u32,
    t_min: f32,
    t_max: f32,
) -> bool {
    if node_idx == NULL_NODE {
        return false;
    }

    let mut stack = [0u32; TRAVERSAL_STACK];
    let mut sp = 1;
    stack[0] = node_idx;

    while sp > 0 {
        sp -= 1;
        let node = &nodes[stack[sp] as usize];
        if !node.bbox().intersect(ray, t_min, t_max) {
            continue;
        }

        match node {
            BVHNode::Leaf { first, count, .. } => {
                for tri in &triangles[*first as usize..(*first + *count) as usize] {
                    if let Some((t, _, _)) = intersect(ray, tri)
                        && (t_min..t_max).contains(&t)
                    {
                        return true;
                    }
                }
            }
            BVHNode::Node { left, right, .. } => {
                push_nodes(ray, nodes, *left, *right, t_min, t_max, &mut stack, &mut sp);
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;

    #[test]
    fn bvh_build_empty() {
        let bvh = BVH::build(vec![]);
        assert!(
            bvh.intersect(&Ray::new(Vec3A::ZERO, Vec3A::Z), 0.0, 100.0)
                .is_none()
        );
    }

    #[test]
    fn bvh_intersect_single_triangle() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
        );
        let bvh = BVH::build(vec![tri]);
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, 100.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().0, 0);
    }

    #[test]
    fn bvh_intersect_any_shadow() {
        let tri = Triangle::new(
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
        );
        let bvh = BVH::build(vec![tri]);
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));
        let miss = Ray::new(Vec3A::new(2.0, 2.0, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(!bvh.intersect_any(&miss, 0.0, 100.0));
    }
}
