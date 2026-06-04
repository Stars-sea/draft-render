use crate::color::Color;
use super::bsdf::Bsdf;
use glam::Vec3A;

pub(super) struct ShadingPoint {
    pub(super) point: Vec3A,
    pub(super) normal: Vec3A,
    pub(super) geom_normal: Vec3A,
    pub(super) emission: Color,
    pub(super) bsdf: Bsdf,
}
