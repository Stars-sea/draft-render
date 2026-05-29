use crate::color::Color;
use crate::pmx::reader::Reader;
use crate::scene::{Material, Texture};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub(crate) struct PmxMaterial {
    pub diffuse: [f32; 4],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub texture_index: Option<usize>,
    pub num_face_vertices: i32,
}

impl PmxMaterial {
    /// Converts this PMX material to a scene Material, loading the referenced
    /// texture file if present. Falls back to the diffuse colour on failure.
    pub fn to_material(&self, textures: &[String], pmx_dir: &Path) -> Material {
        let spec = Color::rgb(
            f32_to_u8(self.specular[0]),
            f32_to_u8(self.specular[1]),
            f32_to_u8(self.specular[2]),
        );
        let tex = self
            .texture_index
            .and_then(|i| textures.get(i))
            .and_then(|p| load_texture(pmx_dir, p));

        let base = match tex {
            Some(t) => Material::textured(Arc::new(t)),
            None => Material::solid(Color::argb(
                f32_to_u8(self.diffuse[3]),
                f32_to_u8(self.diffuse[0]),
                f32_to_u8(self.diffuse[1]),
                f32_to_u8(self.diffuse[2]),
            )),
        };
        base.with_specular(spec).with_shininess(self.shininess)
    }
}

fn f32_to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0) as u8
}

fn load_texture(pmx_dir: &Path, tex_path: &str) -> Option<Texture> {
    let path = pmx_dir.join(tex_path);
    let img = image::open(&path).ok()?.to_rgba8();
    let data = img
        .pixels()
        .map(|p| Color::argb(p[3], p[0], p[1], p[2]))
        .collect();
    Some(Texture::new(img.width() as usize, img.height() as usize, data))
}

pub(crate) fn read_materials(r: &mut Reader) -> Result<Vec<PmxMaterial>> {
    let count = r.read_i32()? as usize;
    let mut mats = Vec::with_capacity(count);
    for _ in 0..count {
        let _name = r.read_string()?;
        let _name_en = r.read_string()?;
        let diffuse = [r.read_f32()?, r.read_f32()?, r.read_f32()?, r.read_f32()?];
        let specular = [r.read_f32()?, r.read_f32()?, r.read_f32()?];
        let shininess = r.read_f32()?;
        let _ambient = [r.read_f32()?, r.read_f32()?, r.read_f32()?];
        let _flags = r.read_u8()?;
        let _edge_color = [r.read_f32()?, r.read_f32()?, r.read_f32()?, r.read_f32()?];
        let _edge_size = r.read_f32()?;
        let texture_index = r.read_texture_index()?;
        let _sphere_texture_index = r.read_index(r.texture_index_size)?;
        let _sphere_mode = r.read_u8()?;
        let toon_flag = r.read_u8()?;
        if toon_flag == 0 {
            let _ = r.read_index(r.texture_index_size)?;
        } else {
            let _ = r.read_u8()?;
        }
        let _memo = r.read_string()?;
        let num_face_vertices = r.read_i32()?;
        mats.push(PmxMaterial {
            diffuse,
            specular,
            shininess,
            texture_index,
            num_face_vertices,
        });
    }
    Ok(mats)
}
