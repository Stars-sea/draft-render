use crate::geometry::BoundingBox;

pub(super) enum BvhNode {
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
    pub(super) fn bbox(&self) -> &BoundingBox {
        match self {
            BvhNode::Leaf { bbox, .. } | BvhNode::Node { bbox, .. } => bbox,
        }
    }
}
