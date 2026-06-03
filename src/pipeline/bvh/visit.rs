use crate::geometry::{Ray, Triangle, intersect};
use crate::pipeline::bvh::node::BvhNode;

pub(super) const NULL_NODE: u32 = u32::MAX;

const TRAVERSAL_STACK: usize = 64;

pub(super) struct TraversalStack {
    data: [u32; TRAVERSAL_STACK],
    sp: usize,
}

impl TraversalStack {
    pub(super) fn new(root: u32) -> Self {
        let mut data = [0u32; TRAVERSAL_STACK];
        data[0] = root;
        Self { data, sp: 1 }
    }

    pub(super) fn pop(&mut self) -> Option<u32> {
        if self.sp == 0 {
            return None;
        }
        self.sp -= 1;
        Some(self.data[self.sp])
    }

    #[inline]
    pub(super) fn push_children(
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

        let pass = |h: Option<(f32, f32)>| h.is_some_and(|(tn, tf)| tn.max(t_min) <= tf.min(t_max));

        match (hit_l.filter(|_| pass(hit_l)), hit_r.filter(|_| pass(hit_r))) {
            (Some((tn_l, _)), Some((tn_r, _))) => {
                if tn_l <= tn_r {
                    self.data[self.sp] = right;
                    self.data[self.sp + 1] = left;
                } else {
                    self.data[self.sp] = left;
                    self.data[self.sp + 1] = right;
                }
                self.sp += 2;
            }
            (Some(_), None) => {
                self.data[self.sp] = left;
                self.sp += 1;
            }
            (None, Some(_)) => {
                self.data[self.sp] = right;
                self.sp += 1;
            }
            (None, None) => {}
        }
    }
}

// ---- Visitor ----

pub(super) trait Visitor: Sized {
    type Output;
    fn t_bound(&self) -> f32;
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

pub(super) struct ClosestHitVisitor<'a> {
    pub(super) t_closest: f32,
    pub(super) best: Option<(usize, f32, f32, f32)>,
    pub(super) cull_backface: &'a [bool],
}

impl<'a> Visitor for ClosestHitVisitor<'a> {
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
                if self.cull_backface[start + i] && tri.is_backface_to(ray.direction) {
                    continue;
                }
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

pub(super) struct AnyHitVisitor<'a> {
    pub(super) found: bool,
    pub(super) t_max: f32,
    pub(super) cull_backface: &'a [bool],
}

impl<'a> Visitor for AnyHitVisitor<'a> {
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
        for (i, tri) in tris[start..(start + n)].iter().enumerate() {
            if let Some((t, _, _)) = intersect(ray, tri)
                && (t_min..self.t_max).contains(&t)
            {
                if self.cull_backface[start + i] && tri.is_backface_to(ray.direction) {
                    continue;
                }
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
