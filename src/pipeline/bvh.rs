use crate::pipeline::bounding_box::BoundingBox;
use crate::pipeline::geometry::{Ray, Triangle, intersect};
use std::cmp::Ordering;

// ---- BVH ----

const NULL_NODE: u32 = u32::MAX;

/// Bounding Volume Hierarchy.
///
/// All nodes live in a single contiguous `Vec` for cache-friendly traversal.
/// Triangles are reordered during construction so that leaves reference
/// contiguous ranges.  Built with binned SAH for higher-quality splits.
pub struct Bvh {
    nodes: Vec<BvhNode>,
    root: u32,
    pub triangles: Vec<Triangle>,
}

enum BvhNode {
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

impl BvhNode {
    #[inline]
    fn bbox(&self) -> &BoundingBox {
        match self {
            BvhNode::Leaf { bbox, .. } | BvhNode::Node { bbox, .. } => bbox,
        }
    }
}

impl Bvh {
    pub fn build(mut triangles: Vec<Triangle>) -> Self {
        let len = triangles.len();
        let mut nodes = Vec::with_capacity(len * 2);
        let root = BvhBuilder {
            nodes: &mut nodes,
            triangles: &mut triangles,
        }
        .build();
        nodes.shrink_to_fit();
        Bvh {
            nodes,
            root,
            triangles,
        }
    }

    /// Find the closest triangle intersection.
    /// Returns `(triangle_index, t, u, v)` — `u`, `v` are barycentric coords.
    #[inline]
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(usize, f32, f32, f32)> {
        ClosestHitVisitor {
            t_closest: t_max,
            best: None,
        }
        .traverse(ray, &self.triangles, &self.nodes, self.root, t_min)
    }

    /// Test whether the ray hits *any* triangle (used for shadow rays).
    #[inline]
    pub fn intersect_any(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        AnyHitVisitor {
            found: false,
            t_max,
        }
        .traverse(ray, &self.triangles, &self.nodes, self.root, t_min)
    }
}

// ---- Build ----

const LEAF_SIZE: usize = 4;
const MAX_DEPTH: u32 = 32;
const SAH_BINS: usize = 12;
const SAH_TRAVERSAL: f32 = 1.0;
const SAH_INTERSECT: f32 = 1.0;

struct BvhBuilder<'a> {
    nodes: &'a mut Vec<BvhNode>,
    triangles: &'a mut [Triangle],
}

impl<'a> BvhBuilder<'a> {
    fn build(&mut self) -> u32 {
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
        self.triangles[start..end].sort_by(|a, b| {
            a.centroid()[axis]
                .partial_cmp(&b.centroid()[axis])
                .unwrap_or(Ordering::Equal)
        });

        let left = self.build_range(start, mid, depth + 1);
        let right = self.build_range(mid, end, depth + 1);

        let idx = self.nodes.len() as u32;
        self.nodes.push(BvhNode::Node { bbox, left, right });
        idx
    }

    // ---- SAH ----

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
        let ext = parent_bbox.extents();
        let parent_sa = parent_bbox.surface_area();

        let n = end - start;
        let mut best = SahSplit {
            axis: 0,
            left_count: n / 2,
        };
        let mut best_cost = n as f32 * SAH_INTERSECT;

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

// ---- Traversal ----

const TRAVERSAL_STACK: usize = 64;

/// Fixed-size stack for iterative BVH traversal.  Lives entirely on the call
/// stack (zero heap allocation) and tests child AABBs before pushing so only
/// nodes that the ray actually hits enter the work queue.
struct TraversalStack {
    data: [u32; TRAVERSAL_STACK],
    sp: usize,
}

impl TraversalStack {
    fn new(root: u32) -> Self {
        let mut data = [0u32; TRAVERSAL_STACK];
        data[0] = root;
        Self { data, sp: 1 }
    }

    fn pop(&mut self) -> Option<u32> {
        if self.sp == 0 {
            return None;
        }
        self.sp -= 1;
        Some(self.data[self.sp])
    }

    /// Test both children's AABBs against the ray and push the ones that
    /// pass, closer child last so it is popped first.
    #[inline]
    fn push_children(
        &mut self,
        ray: &Ray,
        nodes: &[BvhNode],
        left: u32,
        right: u32,
        t_min: f32,
        t_max: f32,
    ) {
        debug_assert!(
            self.sp + 2 <= TRAVERSAL_STACK,
            "BVH traversal stack overflow"
        );

        let hit_l = nodes[left as usize].bbox().intersect_range(ray);
        let hit_r = nodes[right as usize].bbox().intersect_range(ray);

        let pass_l = hit_l.is_some_and(|(tn, tf)| tn.max(t_min) <= tf.min(t_max));
        let pass_r = hit_r.is_some_and(|(tn, tf)| tn.max(t_min) <= tf.min(t_max));

        if !pass_l && !pass_r {
            return;
        }

        // Both pass — push closer child last so it is popped first
        if pass_l && pass_r {
            let (tn_l, _) = hit_l.unwrap();
            let (tn_r, _) = hit_r.unwrap();
            if tn_l <= tn_r {
                self.data[self.sp] = right;
                self.data[self.sp + 1] = left;
            } else {
                self.data[self.sp] = left;
                self.data[self.sp + 1] = right;
            }
            self.sp += 2;
            return;
        }

        // Exactly one child passes
        self.data[self.sp] = if pass_l { left } else { right };
        self.sp += 1;
    }
}

// ---- Visitor ----

trait Visitor: Sized {
    type Output;
    /// The current far bound — AABB checks beyond this distance are skipped.
    fn t_bound(&self) -> f32;
    /// Visit all triangles in `tris[start..][..n]`.  Return `false` to abort
    /// the traversal early (shadow ray found a hit), or `true` to continue.
    fn visit_leaf(
        &mut self,
        ray: &Ray,
        tris: &[Triangle],
        start: usize,
        n: usize,
        t_min: f32,
    ) -> bool;
    fn finish(self) -> Self::Output;

    fn traverse(
        mut self,
        ray: &Ray,
        triangles: &[Triangle],
        nodes: &[BvhNode],
        root: u32,
        t_min: f32,
    ) -> Self::Output {
        if root == NULL_NODE {
            return self.finish();
        }

        let mut stack = TraversalStack::new(root);

        while let Some(idx) = stack.pop() {
            let node = &nodes[idx as usize];
            if !node.bbox().intersect(ray, t_min, self.t_bound()) {
                continue;
            }

            match node {
                BvhNode::Leaf { first, count, .. } => {
                    if !self.visit_leaf(ray, triangles, *first as usize, *count as usize, t_min) {
                        break;
                    }
                }
                BvhNode::Node { left, right, .. } => {
                    stack.push_children(ray, nodes, *left, *right, t_min, self.t_bound());
                }
            }
        }

        self.finish()
    }
}

struct ClosestHitVisitor {
    t_closest: f32,
    best: Option<(usize, f32, f32, f32)>,
}

impl Visitor for ClosestHitVisitor {
    type Output = Option<(usize, f32, f32, f32)>;

    fn t_bound(&self) -> f32 {
        self.t_closest
    }

    fn visit_leaf(
        &mut self,
        ray: &Ray,
        tris: &[Triangle],
        start: usize,
        n: usize,
        t_min: f32,
    ) -> bool {
        for (i, tri) in tris[start..(start + n)].iter().enumerate() {
            if let Some((t, u, v)) = intersect(ray, tri)
                && (t_min..self.t_closest).contains(&t)
            {
                self.t_closest = t;
                self.best = Some((start + i, t, u, v));
            }
        }
        true
    }

    fn finish(self) -> Self::Output {
        self.best
    }
}

struct AnyHitVisitor {
    found: bool,
    t_max: f32,
}

impl Visitor for AnyHitVisitor {
    type Output = bool;

    fn t_bound(&self) -> f32 {
        self.t_max
    }

    fn visit_leaf(
        &mut self,
        ray: &Ray,
        tris: &[Triangle],
        start: usize,
        n: usize,
        t_min: f32,
    ) -> bool {
        for tri in &tris[start..(start + n)] {
            if let Some((t, _, _)) = intersect(ray, tri)
                && (t_min..self.t_max).contains(&t)
            {
                self.found = true;
                return false;
            }
        }
        true
    }

    fn finish(self) -> Self::Output {
        self.found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;

    #[test]
    fn bvh_build_empty() {
        let bvh = Bvh::build(vec![]);
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
        let bvh = Bvh::build(vec![tri]);
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
        let bvh = Bvh::build(vec![tri]);
        let ray = Ray::new(Vec3A::new(0.25, 0.25, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));
        let miss = Ray::new(Vec3A::new(2.0, 2.0, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(!bvh.intersect_any(&miss, 0.0, 100.0));
    }

    #[test]
    fn bvh_many_triangles_closest_hit() {
        // 25 triangles spread across a 5x5 grid at varying depths (z=1..26).
        // Exercises the SAH / median split path (count > LEAF_SIZE).
        let mut tris = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                let x = col as f32;
                let y = row as f32;
                let z = 1.0 + (row * 5 + col) as f32;
                tris.push(Triangle::new(
                    Vec3A::new(x, y, z),
                    Vec3A::new(x + 0.8, y, z),
                    Vec3A::new(x, y + 0.8, z),
                ));
            }
        }
        let bvh = Bvh::build(tris);

        // Ray aimed at center of triangle (col=2, row=3), index = 3*5+2 = 17
        let ray = Ray::new(Vec3A::new(2.2, 3.2, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, 100.0);
        assert!(hit.is_some());
        let (idx, t, _, _) = hit.unwrap();
        assert_eq!(idx, 17);
        let expected_z = 1.0 + 17.0;
        assert!((t - expected_z).abs() < 0.01);

        // Shadow ray should also find it
        assert!(bvh.intersect_any(&ray, 0.0, 100.0));

        // Ray that misses all triangles (aimed at empty space between grid cells)
        let miss = Ray::new(Vec3A::new(-0.5, -0.5, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        assert!(bvh.intersect(&miss, 0.0, 100.0).is_none());
    }
}
