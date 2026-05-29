use crate::scene::{Mesh, SubMesh};
use anyhow::Result;
use glam::{Vec2, Vec3A};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

mod geometry;
mod material;
mod reader;

use geometry::{PmxVertex, read_faces, read_vertices};
use material::{PmxMaterial, read_materials};
use reader::Reader;

/// Parses a PMX file and returns one SubMesh per material slot.
/// Texture references are resolved relative to the PMX file's directory.
pub fn load_pmx(path: impl AsRef<Path>) -> Result<Vec<SubMesh>> {
    let path = path.as_ref();
    let pmx_dir = path.parent().unwrap_or(Path::new("."));
    let data = std::fs::read(path)?;
    let mut r = Reader::new(data);

    reader::read_header(&mut r)?;
    reader::skip_model_info(&mut r)?;

    let verts = read_vertices(&mut r)?;
    let faces = read_faces(&mut r)?;

    let texture_count = r.read_i32()? as usize;
    let mut textures = Vec::with_capacity(texture_count);
    for _ in 0..texture_count {
        textures.push(r.read_string()?);
    }

    let materials = read_materials(&mut r)?;
    Ok(build_submeshes(&verts, &faces, &textures, &materials, pmx_dir))
}

fn build_submeshes(
    verts: &[PmxVertex],
    faces: &[[usize; 3]],
    textures: &[String],
    materials: &[PmxMaterial],
    pmx_dir: &Path,
) -> Vec<SubMesh> {
    let mut submeshes = Vec::new();
    let mut face_offset = 0usize;

    for mat in materials {
        let tri_count = (mat.num_face_vertices as usize) / 3;
        if tri_count == 0 || face_offset + tri_count > faces.len() {
            face_offset += tri_count;
            continue;
        }

        let mut remap: HashMap<usize, usize> = HashMap::new();
        let mut out_verts: Vec<Vec3A> = Vec::new();
        let mut out_uvs: Vec<Vec2> = Vec::new();
        let mut out_normals: Vec<Vec3A> = Vec::new();
        let mut out_indices: Vec<[usize; 3]> = Vec::new();

        for tri in &faces[face_offset..face_offset + tri_count] {
            let mut new_tri = [0usize; 3];
            for (k, &vi) in tri.iter().enumerate() {
                let next_idx = out_verts.len();
                let new_idx = *remap.entry(vi).or_insert_with(|| {
                    out_verts.push(verts[vi].position);
                    out_uvs.push(verts[vi].uv);
                    out_normals.push(verts[vi].normal);
                    next_idx
                });
                new_tri[k] = new_idx;
            }
            out_indices.push(new_tri);
        }
        face_offset += tri_count;

        submeshes.push(SubMesh::new(
            Arc::new(Mesh {
                vertices: out_verts,
                indices: out_indices,
                uvs: out_uvs,
                normals: out_normals,
            }),
            mat.to_material(textures, pmx_dir),
        ));
    }

    submeshes
}
