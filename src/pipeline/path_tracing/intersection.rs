use crate::geometry::Ray;
use crate::pipeline::bvh::Hit;
use crate::scene::ObjTransform;
use glam::Vec3A;

use super::bsdf::Bsdf;
use super::scene::TraceScene;
use super::shading::ShadingPoint;

pub(super) struct SceneHit {
    pub(super) hit: Hit,
    pub(super) obj_idx: usize,
    pub(super) local_ray: Ray,
}

pub(super) const RAY_EPS: f32 = 1e-3;

pub(super) fn offset_along_normal(normal: Vec3A, dir: Vec3A, eps: f32) -> Vec3A {
    if normal.dot(dir) > 0.0 {
        normal * eps
    } else {
        -normal * eps
    }
}

/// Find the closest hit across all objects.
pub(super) fn intersect_scene(
    ts: &TraceScene,
    ray: &Ray,
    t_min: f32,
    transforms: &[ObjTransform],
) -> Option<SceneHit> {
    let mut best: Option<SceneHit> = None;
    let mut closest_t = f32::INFINITY;

    for (i, obj) in ts.objects.iter().enumerate() {
        let (local_ray, s) = transforms[i].ray_to_local(ray);
        if s < 1e-12 {
            continue;
        }
        let Some(hit) = obj.intersect(&local_ray, t_min, f32::INFINITY) else {
            continue;
        };
        let t = hit.t / s;
        if t < closest_t {
            closest_t = t;
            best = Some(SceneHit { hit, obj_idx: i, local_ray });
        }
    }

    best
}

pub(super) fn resolve_hit(
    ts: &TraceScene,
    sh: &SceneHit,
    world_ray: &Ray,
    transforms: &[ObjTransform],
) -> ShadingPoint {
    let txf = &transforms[sh.obj_idx];
    let bvh = &ts.objects[sh.obj_idx];
    let (tri, mat_id, _, norms) = bvh.tri_data(sh.hit.tri_idx);

    let local_point = sh.hit.point(&sh.local_ray);
    let world_point = txf.model.transform_point3a(local_point);

    let uv = sh.hit.interpolate_uv(&bvh.uvs);
    let mat = &ts.materials[mat_id as usize];
    let (albedo, _) = mat.albedo_alpha_at(uv);

    let mut shading_normal = if norms[0] != Vec3A::ZERO {
        sh.hit.interpolate_normal(&bvh.normals)
    } else {
        tri.normal()
    };
    shading_normal = (txf.normal_mat * shading_normal).normalize();

    let mut geom_normal = tri.normal();
    geom_normal = (txf.normal_mat * geom_normal).normalize();

    if shading_normal.dot(-world_ray.direction) < 0.0 {
        shading_normal = -shading_normal;
        geom_normal = -geom_normal;
    }

    ShadingPoint {
        point: world_point,
        normal: shading_normal,
        geom_normal,
        emission: mat.emission,
        bsdf: Bsdf::new(albedo, mat.roughness, mat.metallic),
    }
}

#[inline]
pub(super) fn shadowed(
    ts: &TraceScene,
    point: Vec3A,
    normal: Vec3A,
    dir: Vec3A,
    t_max: f32,
    transforms: &[ObjTransform],
) -> bool {
    let ray = Ray::new(point + offset_along_normal(normal, dir, RAY_EPS), dir);
    let t_clamped = t_max - RAY_EPS;
    for (i, obj) in ts.objects.iter().enumerate() {
        let (local_ray, dir_scale) = transforms[i].ray_to_local(&ray);
        if obj.intersect_any(&local_ray, RAY_EPS, t_clamped * dir_scale) {
            return true;
        }
    }
    false
}
