//! Minimal glTF 2.0 loader — extracts triangle meshes + PBR materials.
//! Only handles the subset produced by convert_cornell_gltf.py.

use crate::color::Color;
use crate::scene::{Material, MeshBuilder, SceneObject, SubMesh, Transform};
use anyhow::{Context, Result};
use glam::Vec3A;
use gltf::mesh::util::ReadIndices;
use gltf::{Document, Glb, json};
use std::sync::Arc;

/// Load a binary glTF file and return SceneObjects grouped by material.
pub fn load_glb(bytes: &[u8]) -> Result<Vec<SceneObject>> {
    let glb = Glb::from_slice(bytes).context("invalid GLB")?;
    let json = json::Root::from_slice(&glb.json).context("invalid glTF JSON")?;
    let doc = Document::from_json(json)?;
    let data = glb.bin.as_deref().context("GLB missing binary buffer")?;

    let mut objects = Vec::new();

    for mesh in doc.meshes() {
        for prim in mesh.primitives() {
            let reader = prim.reader(|buf| if buf.index() == 0 { Some(data) } else { None });

            // Vertex positions
            let positions = reader
                .read_positions()
                .context("mesh missing POSITION attribute")?;
            let verts: Vec<Vec3A> = positions.map(|p| Vec3A::new(p[0], p[1], p[2])).collect();

            // Indices (triangles)
            let indices: Vec<[usize; 3]> = match reader.read_indices() {
                Some(ReadIndices::U16(iter)) => {
                    let mut tris = Vec::new();
                    let idx: Vec<u16> = iter.collect();
                    for chunk in idx.chunks(3) {
                        if chunk.len() == 3 {
                            tris.push([chunk[0] as usize, chunk[1] as usize, chunk[2] as usize]);
                        }
                    }
                    tris
                }
                Some(ReadIndices::U32(iter)) => {
                    let mut tris = Vec::new();
                    let idx: Vec<u32> = iter.collect();
                    for chunk in idx.chunks(3) {
                        if chunk.len() == 3 {
                            tris.push([chunk[0] as usize, chunk[1] as usize, chunk[2] as usize]);
                        }
                    }
                    tris
                }
                _ => anyhow::bail!("mesh missing indices"),
            };

            // Material
            let mat = prim.material();
            let pbr = mat.pbr_metallic_roughness();
            let base = pbr.base_color_factor();
            let albedo = Color::from_linear_rgb(base[0], base[1], base[2]);
            let mut material = Material::solid(albedo)
                .with_roughness(pbr.roughness_factor())
                .with_metallic(pbr.metallic_factor());
            if mat.double_sided() {
                material = material.with_double_sided();
            }

            // Build mesh
            let mut builder = MeshBuilder::new();
            for v in &verts {
                builder = builder.vertex(*v);
            }
            for t in &indices {
                builder = builder.triangle(t[0], t[1], t[2]);
            }

            objects.push(SceneObject::single(
                SubMesh::new(Arc::new(builder.build()), material),
                Transform::default(),
            ));
        }
    }

    Ok(objects)
}
