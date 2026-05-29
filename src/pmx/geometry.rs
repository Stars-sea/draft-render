use crate::pmx::reader::Reader;
use anyhow::{bail, Result};
use glam::{Vec2, Vec3A};

pub(crate) struct PmxVertex {
    pub position: Vec3A,
    pub normal: Vec3A,
    pub uv: Vec2,
}

fn skip_bone_weights(r: &mut Reader) -> Result<()> {
    let weight_type = r.read_u8()?;
    match weight_type {
        0 => {
            r.read_index(r.vertex_index_size)?;
        }
        1 => {
            r.read_index(r.vertex_index_size)?;
            r.read_index(r.vertex_index_size)?;
            r.read_f32()?;
        }
        2 | 4 => {
            for _ in 0..4 {
                r.read_index(r.vertex_index_size)?;
            }
            for _ in 0..4 {
                r.read_f32()?;
            }
        }
        3 => {
            r.read_index(r.vertex_index_size)?;
            r.read_index(r.vertex_index_size)?;
            r.read_f32()?;
            r.read_vec3()?;
            r.read_vec3()?;
            r.read_vec3()?;
        }
        _ => bail!("unknown vertex weight type: {weight_type}"),
    }
    Ok(())
}

pub(crate) fn read_vertices(r: &mut Reader) -> Result<Vec<PmxVertex>> {
    let count = r.read_i32()? as usize;
    let mut verts = Vec::with_capacity(count);
    for _ in 0..count {
        let position = r.read_vec3()?;
        let normal = r.read_vec3()?;
        let uv = r.read_vec2()?;
        for _ in 0..r.additional_uv {
            let _ = r.read_f32()?;
            let _ = r.read_f32()?;
            let _ = r.read_f32()?;
            let _ = r.read_f32()?;
        }
        skip_bone_weights(r)?;
        let _edge_scale = r.read_f32()?;
        verts.push(PmxVertex { position, normal, uv });
    }
    Ok(verts)
}

pub(crate) fn read_faces(r: &mut Reader) -> Result<Vec<[usize; 3]>> {
    let count = r.read_i32()? as usize;
    let mut raw = Vec::with_capacity(count);
    for _ in 0..count {
        raw.push(r.read_raw_vertex_index()?);
    }
    // PMX spec says 1-based indexing, but some 2.1 exporters use 0-based.
    // Detect by the minimum index value and use it as the base offset.
    let offset = *raw.iter().min().unwrap_or(&1);
    let mut faces = Vec::with_capacity(count / 3);
    for chunk in raw.chunks_exact(3) {
        faces.push([
            (chunk[0] - offset) as usize,
            (chunk[1] - offset) as usize,
            (chunk[2] - offset) as usize,
        ]);
    }
    Ok(faces)
}
